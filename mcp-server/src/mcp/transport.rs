use super::framing::{self, ByteCount, CappedBytes, Input, FRAME_LIMIT, RESPONSE_LIMIT};
use super::service::{McpService, ToolOwner};
use crate::runner::AttemptClock;
use rmcp::model::*;
use rmcp::transport::Transport;
use rmcp::{ErrorData, RoleServer};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};

const CONTROL_BUDGET: Duration = Duration::from_secs(10);
pub(super) type Shared = Arc<Mutex<State>>;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Connection {
    AwaitingInitialize,
    Ready,
    Closing,
    Closed,
}
pub(super) struct OwnedId {
    pub clock: AttemptClock,
    budget: Duration,
    pub cancelled: Arc<AtomicBool>,
}
pub(super) struct State {
    pub phase: Connection,
    pub registry: PathBuf,
    pub ids: HashMap<RequestId, OwnedId>,
    pub active: Option<ToolOwner>,
    initialize: Option<RequestId>,
    initialized_response: bool,
}
impl State {
    fn stop(&mut self) {
        self.phase = Connection::Closing;
        for owned in self.ids.values() {
            owned.cancelled.store(true, Ordering::Relaxed);
        }
        if let Some(owner) = &self.active {
            owner.cancelled.store(true, Ordering::Relaxed);
        }
    }
    fn reap_cancelled(&mut self) {
        if self.active.as_ref().is_some_and(|owner| {
            (owner.cancelled.load(Ordering::Relaxed) || !self.ids.contains_key(&owner.id))
                && owner.handle.is_finished()
        }) {
            let owner = self.active.take().expect("finished owner");
            let _ = owner.handle.join();
            self.ids.remove(&owner.id);
        }
    }
}

pub(super) fn lock(shared: &Shared) -> std::sync::MutexGuard<'_, State> {
    match shared.lock() {
        Ok(state) => state,
        Err(poisoned) => {
            let mut state = poisoned.into_inner();
            state.stop();
            state
        }
    }
}

pub(super) fn error(code: i32) -> ErrorData {
    let text = match code {
        -32700 => "Parse error",
        -32600 => "Invalid request",
        -32601 => "Method not found",
        -32602 => "Invalid params",
        _ => "Internal error",
    };
    ErrorData::new(ErrorCode(code), text, None)
}
fn checked_id(value: &Value) -> Option<RequestId> {
    match value {
        Value::String(text) if text.len() <= 128 => {
            Some(NumberOrString::String(text.clone().into()))
        }
        Value::Number(number) => number.as_i64().map(NumberOrString::Number),
        _ => None,
    }
}

// Selected-revision errors require an explicit null id; the modern SDK omits it.
#[derive(Serialize)]
struct LegacyError {
    jsonrpc: JsonRpcVersion2_0,
    id: Option<RequestId>,
    error: ErrorData,
}
struct WriteJob {
    output: ServerJsonRpcMessage,
    complete: oneshot::Sender<io::Result<()>>,
}
fn writer() -> io::Result<mpsc::Sender<WriteJob>> {
    let (tx, mut rx) = mpsc::channel::<WriteJob>(1);
    std::thread::Builder::new()
        .name("mcp-output".into())
        .spawn(move || {
            let mut stdout = io::stdout().lock();
            while let Some(job) = rx.blocking_recv() {
                let mut bytes = CappedBytes::new(RESPONSE_LIMIT - 1);
                let serialized = match job.output {
                    JsonRpcMessage::Error(message) => serde_json::to_writer(
                        &mut bytes,
                        &LegacyError {
                            jsonrpc: message.jsonrpc,
                            id: message.id,
                            error: message.error,
                        },
                    )
                    .map_err(|_| io::Error::other("serialization failure")),
                    message => {
                        // Check the authoritative object separately from wire escaping.
                        let object_ok = match &message {
                            JsonRpcMessage::Response(response) => match &response.result {
                                ServerResult::CallToolResult(result) => {
                                    result.structured_content.as_ref().is_none_or(|object| {
                                        serde_json::to_writer(ByteCount::new(FRAME_LIMIT), object)
                                            .is_ok()
                                    })
                                }
                                _ => true,
                            },
                            _ => true,
                        };
                        if object_ok {
                            serde_json::to_writer(&mut bytes, &message)
                                .map_err(|_| io::Error::other("serialization failure"))
                        } else {
                            Err(io::Error::other("tool object limit"))
                        }
                    }
                };
                let result = serialized
                    .and_then(|_| stdout.write_all(&bytes.bytes))
                    .and_then(|_| stdout.write_all(b"\n"))
                    .and_then(|_| stdout.flush());
                let failed = result.is_err();
                let _ = job.complete.send(result);
                if failed {
                    return;
                }
            }
        })?;
    Ok(tx)
}
async fn deliver(
    writer: &mpsc::Sender<WriteJob>,
    output: ServerJsonRpcMessage,
    remaining: Duration,
) -> io::Result<()> {
    let (complete, finished) = oneshot::channel();
    tokio::time::timeout(remaining, async {
        writer
            .send(WriteJob { output, complete })
            .await
            .map_err(|_| io::Error::other("delivery failure"))?;
        finished
            .await
            .map_err(|_| io::Error::other("delivery failure"))?
    })
    .await
    .map_err(|_| io::Error::other("delivery deadline"))?
}
fn remaining(clock: AttemptClock, budget: Duration) -> Duration {
    budget.saturating_sub(Duration::from_micros(clock.elapsed_us()))
}

struct BoundedTransport {
    shared: Shared,
    input: mpsc::Receiver<Input>,
    writer: mpsc::Sender<WriteJob>,
    partial: Arc<Mutex<Option<Instant>>>,
    shutdown: &'static AtomicBool,
    started: Instant,
}
impl BoundedTransport {
    async fn reject(&mut self, id: Option<RequestId>, code: i32) -> bool {
        let left = {
            let state = lock(&self.shared);
            state
                .ids
                .values()
                .map(|owned| remaining(owned.clock, owned.budget))
                .min()
                .unwrap_or(CONTROL_BUDGET)
        };
        let result = deliver(
            &self.writer,
            ServerJsonRpcMessage::error(error(code), id),
            left,
        )
        .await;
        if result.is_err() {
            self.fail();
            return false;
        }
        true
    }
    fn fail(&self) {
        lock(&self.shared).stop();
        eprintln!("mcp connection closed");
    }
    fn timed_out(&self) -> bool {
        let state = lock(&self.shared);
        (state.phase == Connection::AwaitingInitialize && self.started.elapsed() >= CONTROL_BUDGET)
            || match self.partial.lock() {
                Ok(partial) => partial.is_some_and(|started| started.elapsed() >= CONTROL_BUDGET),
                Err(_) => true,
            }
            || state
                .ids
                .values()
                .any(|owned| remaining(owned.clock, owned.budget).is_zero())
    }
}
impl Transport<RoleServer> for BoundedTransport {
    type Error = io::Error;
    fn send(
        &mut self,
        mut message: ServerJsonRpcMessage,
    ) -> impl Future<Output = io::Result<()>> + Send + 'static {
        let shared = self.shared.clone();
        let writer = self.writer.clone();
        async move {
            let id = match &message {
                JsonRpcMessage::Response(response) => Some(response.id.clone()),
                JsonRpcMessage::Error(error) => error.id.clone(),
                _ => None,
            };
            let left = {
                let state = lock(&shared);
                if matches!(state.phase, Connection::Closing | Connection::Closed) {
                    return Err(io::Error::other("connection closing"));
                }
                id.as_ref()
                    .and_then(|id| state.ids.get(id))
                    .map_or(CONTROL_BUDGET, |owned| remaining(owned.clock, owned.budget))
            };
            if let JsonRpcMessage::Response(response) = &mut message {
                response.result.strip_result_type_for_legacy_peer();
            }
            let output = match message {
                JsonRpcMessage::Error(message) => {
                    ServerJsonRpcMessage::error(error(message.error.code.0), message.id)
                }
                message => message,
            };
            let delivered = deliver(&writer, output, left).await;
            let mut state = lock(&shared);
            if delivered.is_err() {
                state.stop();
                eprintln!("mcp delivery failed");
            } else if let Some(id) = id {
                if state.initialize.as_ref() == Some(&id) {
                    state.initialized_response = true;
                }
                state.ids.remove(&id);
                if state
                    .active
                    .as_ref()
                    .is_some_and(|owner| owner.id == id && owner.handle.is_finished())
                {
                    let owner = state.active.take().expect("finished owner");
                    let _ = owner.handle.join();
                }
            }
            delivered
        }
    }
    async fn receive(&mut self) -> Option<ClientJsonRpcMessage> {
        loop {
            {
                let mut state = lock(&self.shared);
                state.reap_cancelled();
                if matches!(state.phase, Connection::Closing | Connection::Closed) {
                    return None;
                }
            }
            if self.shutdown.load(Ordering::Relaxed) || self.timed_out() {
                self.fail();
                return None;
            }
            let input =
                match tokio::time::timeout(Duration::from_millis(5), self.input.recv()).await {
                    Ok(Some(input)) => input,
                    Ok(None) => {
                        lock(&self.shared).stop();
                        return None;
                    }
                    Err(_) => continue,
                };
            let (bytes, clock) = match input {
                Input::Frame(bytes, clock) => (bytes, clock),
                Input::End => {
                    lock(&self.shared).stop();
                    return None;
                }
                Input::Failure => {
                    self.fail();
                    return None;
                }
            };
            if std::str::from_utf8(&bytes).is_err() {
                self.fail();
                return None;
            }
            let value = match framing::parse(&bytes) {
                Ok(value) => value,
                Err(_) => {
                    if !self.reject(None, -32700).await {
                        return None;
                    }
                    continue;
                }
            };
            let Some(object) = value.as_object() else {
                if !self.reject(None, -32600).await {
                    return None;
                }
                continue;
            };
            let id = object.get("id").and_then(checked_id);
            let valid = object.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
                && object.get("method").and_then(Value::as_str).is_some()
                && object
                    .keys()
                    .all(|key| matches!(key.as_str(), "jsonrpc" | "id" | "method" | "params"))
                && (!object.contains_key("id") || id.is_some());
            if !valid {
                if !self.reject(None, -32600).await {
                    return None;
                }
                continue;
            }
            let method = object["method"].as_str().expect("checked method");
            let Some(id) = id else {
                match method {
                    "notifications/initialized" => {
                        let mut state = lock(&self.shared);
                        if state.phase == Connection::AwaitingInitialize
                            && state.initialized_response
                            && object.get("params").is_none_or(|p| {
                                p.as_object().is_some_and(|p| {
                                    p.keys().all(|key| key == "_meta")
                                        && p.get("_meta").is_none_or(Value::is_object)
                                })
                            })
                        {
                            state.phase = Connection::Ready;
                        }
                        continue;
                    }
                    "notifications/cancelled" => {
                        let valid =
                            object
                                .get("params")
                                .and_then(Value::as_object)
                                .is_some_and(|params| {
                                    params.keys().all(|key| {
                                        matches!(key.as_str(), "requestId" | "reason" | "_meta")
                                    }) && params.get("requestId").and_then(checked_id).is_some()
                                        && params.get("reason").is_none_or(Value::is_string)
                                        && params.get("_meta").is_none_or(Value::is_object)
                                });
                        if !valid {
                            continue;
                        }
                        let known = object
                            .get("params")
                            .and_then(|p| p.get("requestId"))
                            .and_then(checked_id)
                            .is_some_and(|cancel_id| {
                                let state = lock(&self.shared);
                                if let Some(owned) = state.ids.get(&cancel_id) {
                                    owned.cancelled.store(true, Ordering::Relaxed);
                                    true
                                } else {
                                    false
                                }
                            });
                        // Delivery cancellation does not destroy the supervisor owner.
                        if known {
                            if let Ok(message) = serde_json::from_value(value) {
                                return Some(message);
                            }
                        }
                        continue;
                    }
                    _ => continue,
                }
            };
            let code = {
                let state = lock(&self.shared);
                if state.ids.contains_key(&id) || state.ids.len() >= 8 {
                    Some(-32600)
                } else if !matches!(
                    method,
                    "initialize" | "ping" | "server/discover" | "tools/list" | "tools/call"
                ) {
                    Some(-32601)
                } else if (method == "initialize"
                    && (state.phase != Connection::AwaitingInitialize
                        || state.initialize.is_some()))
                    || (matches!(method, "tools/list" | "tools/call")
                        && state.phase != Connection::Ready)
                {
                    Some(-32600)
                } else {
                    None
                }
            };
            if let Some(code) = code {
                if !self.reject(Some(id), code).await {
                    return None;
                }
                continue;
            }
            let params = object.get("params");
            let params_valid = match method {
                "tools/call" => params.and_then(Value::as_object).is_some_and(|p| {
                    p.keys()
                        .all(|key| matches!(key.as_str(), "name" | "arguments" | "_meta"))
                        && p.get("name")
                            .and_then(Value::as_str)
                            .and_then(super::handler::Operation::from_name)
                            .is_some()
                        && p.get("arguments").is_none_or(Value::is_object)
                        && p.get("_meta").is_none_or(Value::is_object)
                }),
                "tools/list" | "ping" | "server/discover" => params.is_none_or(|p| {
                    p.as_object().is_some_and(|p| {
                        p.keys().all(|key| key == "_meta")
                            && p.get("_meta").is_none_or(Value::is_object)
                    })
                }),
                "initialize" => params.and_then(Value::as_object).is_some_and(|p| {
                    p.keys().all(|key| {
                        matches!(
                            key.as_str(),
                            "protocolVersion" | "capabilities" | "clientInfo" | "_meta"
                        )
                    }) && p.get("protocolVersion").is_some_and(Value::is_string)
                        && p.get("capabilities").is_some_and(Value::is_object)
                        && p.get("clientInfo").is_some_and(Value::is_object)
                        && p.get("_meta").is_none_or(Value::is_object)
                }),
                _ => false,
            };
            if !params_valid {
                if !self.reject(Some(id), -32602).await {
                    return None;
                }
                continue;
            }
            if method == "server/discover" {
                // This source-free compatibility probe does not start the SDK's
                // newer inline lifecycle or authorize any editor operation.
                use rmcp::ServerHandler;
                let config = McpService {
                    shared: self.shared.clone(),
                }
                .get_info();
                let result =
                    DiscoverResult::from_server_info(vec![ProtocolVersion::V_2025_11_25], config);
                let response =
                    ServerJsonRpcMessage::response(ServerResult::DiscoverResult(result), id);
                if deliver(&self.writer, response, remaining(clock, CONTROL_BUDGET))
                    .await
                    .is_err()
                {
                    self.fail();
                    return None;
                }
                continue;
            }
            let budget = if method == "tools/call" {
                params
                    .and_then(|p| p.get("name"))
                    .and_then(Value::as_str)
                    .and_then(super::handler::Operation::from_name)
                    .expect("checked tool")
                    .budget()
            } else {
                CONTROL_BUDGET
            };
            let initializing = method == "initialize";
            let expected = match method {
                "initialize" => 0,
                "ping" => 1,
                "tools/list" => 3,
                "tools/call" => 4,
                _ => unreachable!("checked method"),
            };
            let message: ClientJsonRpcMessage = match serde_json::from_value(value) {
                Ok(message @ JsonRpcMessage::Request(_)) => message,
                _ => {
                    if !self.reject(Some(id), -32602).await {
                        return None;
                    }
                    continue;
                }
            };
            let typed = match &message {
                JsonRpcMessage::Request(request) => matches!(
                    (expected, &request.request),
                    (0, ClientRequest::InitializeRequest(_))
                        | (1, ClientRequest::PingRequest(_))
                        | (3, ClientRequest::ListToolsRequest(_))
                        | (4, ClientRequest::CallToolRequest(_))
                ),
                _ => false,
            };
            if !typed {
                if !self.reject(Some(id), -32602).await {
                    return None;
                }
                continue;
            }
            let mut state = lock(&self.shared);
            if initializing {
                state.initialize = Some(id.clone());
            }
            state.ids.insert(
                id,
                OwnedId {
                    clock,
                    budget,
                    cancelled: Arc::new(AtomicBool::new(false)),
                },
            );
            return Some(message);
        }
    }
    async fn close(&mut self) -> io::Result<()> {
        lock(&self.shared).stop();
        Ok(())
    }
}

/// Runs the selected local service. Does not join a potentially blocked stdin or
/// stdout thread; the owning executable bounds runtime teardown and exits.
///
/// # Errors
/// Returns a source-free I/O error if the runtime, transport or protocol service
/// cannot run. EOF and cancellation are not evidence of mutation rollback.
pub fn run(registry: PathBuf, shutdown: &'static AtomicBool) -> io::Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()?;
    let result = runtime.block_on(async {
        let partial = Arc::new(Mutex::new(None));
        let input = framing::reader(partial.clone())?;
        let writer = writer()?;
        let shared = Arc::new(Mutex::new(State {
            phase: Connection::AwaitingInitialize,
            registry,
            ids: HashMap::new(),
            active: None,
            initialize: None,
            initialized_response: false,
        }));
        let transport = BoundedTransport {
            shared: shared.clone(),
            input,
            writer,
            partial,
            shutdown,
            started: Instant::now(),
        };
        let service = rmcp::service::serve_directly(
            McpService {
                shared: shared.clone(),
            },
            transport,
            None,
        );
        let waiting = service.waiting();
        tokio::pin!(waiting);
        let mut service_result = None;
        loop {
            tokio::select! {
                result = &mut waiting => { service_result = Some(result); break; }
                _ = tokio::time::sleep(Duration::from_millis(5)) => {
                    if shutdown.load(Ordering::Relaxed) { lock(&shared).stop(); }
                    if lock(&shared).phase == Connection::Closing { break; }
                }
            }
        }
        lock(&shared).stop();
        loop {
            let done = {
                let mut state = lock(&shared);
                match state.active.as_ref() {
                    None => true,
                    Some(owner) if owner.handle.is_finished() => {
                        let owner = state.active.take().expect("finished owner");
                        let _ = owner.handle.join();
                        true
                    }
                    Some(owner) => remaining(
                        owner.clock,
                        owner.budget.saturating_sub(Duration::from_millis(500)),
                    )
                    .is_zero(),
                }
            };
            if done {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        lock(&shared).phase = Connection::Closed;
        // Do not inherit the SDK's five-second EOF drain after owned work has
        // already reached its original cutoff.
        match service_result {
            Some(Err(_)) => Err(io::Error::other("service failure")),
            _ => Ok(()),
        }
    });
    runtime.shutdown_timeout(Duration::from_secs(1));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    static RUNNING: AtomicBool = AtomicBool::new(false);
    fn state() -> Shared {
        Arc::new(Mutex::new(State {
            phase: Connection::Ready,
            registry: PathBuf::from("/nonexistent/mcp-regression-registry"),
            ids: HashMap::new(),
            active: None,
            initialize: None,
            initialized_response: true,
        }))
    }
    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap()
    }
    fn frame(value: Value) -> Input {
        Input::Frame(serde_json::to_vec(&value).unwrap(), AttemptClock::start())
    }

    #[test]
    fn duplicate_id_preserves_original_cancel_owner() {
        runtime().block_on(async {
            let shared = state();
            let id = NumberOrString::Number(1);
            let original = Arc::new(AtomicBool::new(false));
            lock(&shared).ids.insert(
                id.clone(),
                OwnedId {
                    clock: AttemptClock::start(),
                    budget: CONTROL_BUDGET,
                    cancelled: original.clone(),
                },
            );
            let (send_input, input) = mpsc::channel(4);
            let (writer, mut output) = mpsc::channel::<WriteJob>(1);
            send_input
                .send(frame(
                    serde_json::json!({"jsonrpc":"2.0","id":1,"method":"ping"}),
                ))
                .await
                .unwrap();
            send_input
                .send(frame(
                    serde_json::json!({"jsonrpc":"2.0","id":2,"method":"ping"}),
                ))
                .await
                .unwrap();
            let acknowledge = tokio::spawn(async move {
                let job = output.recv().await.unwrap();
                let JsonRpcMessage::Error(error) = job.output else {
                    panic!("duplicate must be rejected");
                };
                assert_eq!(error.id, Some(NumberOrString::Number(1)));
                assert_eq!(error.error.code, ErrorCode(-32600));
                job.complete.send(Ok(())).unwrap();
            });
            let mut transport = BoundedTransport {
                shared: shared.clone(),
                input,
                writer,
                partial: Arc::new(Mutex::new(None)),
                shutdown: &RUNNING,
                started: Instant::now(),
            };
            assert!(transport.receive().await.is_some());
            acknowledge.await.unwrap();
            let state = lock(&shared);
            assert!(Arc::ptr_eq(&state.ids[&id].cancelled, &original));
            assert!(!original.load(Ordering::Relaxed));
            assert_eq!(state.ids.len(), 2);
        });
    }

    #[test]
    fn unknown_cancel_is_ignored_known_cancel_retains_handle() {
        runtime().block_on(async {
            let shared = state();
            let id = NumberOrString::Number(1);
            let cancelled = Arc::new(AtomicBool::new(false));
            let (release, blocked) = std::sync::mpsc::channel();
            let handle = std::thread::spawn(move || { blocked.recv().unwrap(); });
            let clock = AttemptClock::start();
            {
                let mut state = lock(&shared);
                state.ids.insert(id.clone(), OwnedId { clock, budget: CONTROL_BUDGET, cancelled: cancelled.clone() });
                state.ids.insert(NumberOrString::Number(2), OwnedId { clock, budget: CONTROL_BUDGET, cancelled: Arc::new(AtomicBool::new(false)) });
                state.active = Some(ToolOwner { id: id.clone(), clock, budget: Duration::from_secs(5), cancelled: cancelled.clone(), handle });
            }
            let (send_input, input) = mpsc::channel(4);
            let (writer, _output) = mpsc::channel(1);
            send_input.send(frame(serde_json::json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":3}}))).await.unwrap();
            send_input.send(frame(serde_json::json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":1}}))).await.unwrap();
            let mut transport = BoundedTransport { shared: shared.clone(), input, writer, partial: Arc::new(Mutex::new(None)), shutdown: &RUNNING, started: Instant::now() };
            let received = transport.receive().await.unwrap();
            assert!(matches!(received, JsonRpcMessage::Notification(_)));
            assert!(cancelled.load(Ordering::Relaxed));
            assert!(!lock(&shared).ids[&NumberOrString::Number(2)].cancelled.load(Ordering::Relaxed));
            assert!(lock(&shared).active.is_some());
            release.send(()).unwrap();
            let owner = lock(&shared).active.take().unwrap();
            owner.handle.join().unwrap();
        });
    }

    #[test]
    fn outstanding_ids_are_bounded_at_eight() {
        runtime().block_on(async {
            let shared = state();
            let (send_input, input) = mpsc::channel(16);
            let (writer, mut output) = mpsc::channel::<WriteJob>(1);
            for id in 0..9 {
                send_input
                    .send(frame(
                        serde_json::json!({"jsonrpc":"2.0","id":id,"method":"ping"}),
                    ))
                    .await
                    .unwrap();
            }
            send_input.send(Input::End).await.unwrap();
            let acknowledge = tokio::spawn(async move {
                let job = output.recv().await.unwrap();
                let JsonRpcMessage::Error(error) = job.output else {
                    panic!("capacity must be rejected");
                };
                assert_eq!(error.error.code, ErrorCode(-32600));
                job.complete.send(Ok(())).unwrap();
            });
            let mut transport = BoundedTransport {
                shared: shared.clone(),
                input,
                writer,
                partial: Arc::new(Mutex::new(None)),
                shutdown: &RUNNING,
                started: Instant::now(),
            };
            for _ in 0..8 {
                assert!(transport.receive().await.is_some());
            }
            assert!(transport.receive().await.is_none());
            acknowledge.await.unwrap();
            assert_eq!(lock(&shared).ids.len(), 8);
        });
    }

    #[test]
    fn delivery_backpressure_is_finite() {
        runtime().block_on(async {
            let (writer, mut output) = mpsc::channel::<WriteJob>(1);
            let blocked = tokio::spawn(async move {
                let job = output.recv().await.unwrap();
                tokio::time::sleep(Duration::from_secs(1)).await;
                drop(job);
            });
            let started = Instant::now();
            let result = deliver(
                &writer,
                ServerJsonRpcMessage::error(error(-32700), None),
                Duration::from_millis(10),
            )
            .await;
            assert!(result.is_err());
            assert!(started.elapsed() < Duration::from_secs(1));
            blocked.abort();
        });
    }

    #[test]
    fn handshake_and_partial_deadlines_do_not_expire_ready_idle() {
        let shared = state();
        let (_send_input, input) = mpsc::channel(1);
        let (writer, _output) = mpsc::channel(1);
        let partial = Arc::new(Mutex::new(None));
        let transport = BoundedTransport {
            shared: shared.clone(),
            input,
            writer,
            partial: partial.clone(),
            shutdown: &RUNNING,
            started: Instant::now() - Duration::from_secs(11),
        };
        assert!(!transport.timed_out());
        lock(&shared).phase = Connection::AwaitingInitialize;
        assert!(transport.timed_out());
        lock(&shared).phase = Connection::Ready;
        if let Ok(mut partial) = partial.lock() {
            *partial = Some(Instant::now() - Duration::from_secs(11));
        } else {
            panic!("poisoned test fixture");
        }
        assert!(transport.timed_out());
    }
}

#[cfg(test)]
mod service_tests {
    use super::*;
    static RUNNING: AtomicBool = AtomicBool::new(false);
    #[test]
    fn busy_call_is_refused_without_queuing_or_replacing_owner() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        runtime.block_on(async {
            let clock = AttemptClock::start();
            let cancelled = Arc::new(AtomicBool::new(false));
            let (release, blocked) = std::sync::mpsc::channel();
            let handle = std::thread::spawn(move || { blocked.recv().unwrap(); });
            let active_id = NumberOrString::String("owned-call".into());
            let shared = Arc::new(Mutex::new(State {
                phase: Connection::Ready, registry: PathBuf::from("/nonexistent/mcp-regression-registry"),
                ids: HashMap::new(), active: Some(ToolOwner { id: active_id.clone(), clock, budget: Duration::from_secs(5), cancelled: cancelled.clone(), handle }),
                initialize: None, initialized_response: true,
            }));
            let (send_input, input) = mpsc::channel(1);
            let (writer, mut output) = mpsc::channel::<WriteJob>(1);
            let transport = BoundedTransport { shared: shared.clone(), input, writer, partial: Arc::new(Mutex::new(None)), shutdown: &RUNNING, started: Instant::now() };
            let service = rmcp::service::serve_directly(McpService { shared: shared.clone() }, transport, None);
            send_input.send(Input::Frame(serde_json::to_vec(&serde_json::json!({
                "jsonrpc":"2.0","id":"busy-call","method":"tools/call",
                "params":{"name":"read_script","arguments":{"project_root":"/fixture/project","script_path":"res://test.gd"}}
            })).unwrap(), AttemptClock::start())).await.unwrap();
            let job = tokio::time::timeout(Duration::from_secs(1), output.recv()).await.unwrap().unwrap();
            let JsonRpcMessage::Response(response) = job.output else { panic!("busy must be a tool result"); };
            let ServerResult::CallToolResult(result) = response.result else { panic!("busy must be a tool result"); };
            assert_eq!(result.structured_content.unwrap()["error"]["code"], "server_busy");
            assert_eq!(result.is_error, Some(true));
            job.complete.send(Ok(())).unwrap();
            assert_eq!(lock(&shared).active.as_ref().unwrap().id, active_id);
            assert!(!cancelled.load(Ordering::Relaxed));
            release.send(()).unwrap();
            lock(&shared).active.take().unwrap().handle.join().unwrap();
            drop(send_input);
            tokio::time::timeout(Duration::from_secs(1), service.waiting()).await.unwrap().unwrap();
        });
    }
}
