use super::framing::{self, ByteCount, CappedBytes, Input, FRAME_LIMIT, RESPONSE_LIMIT};
use super::service::{McpService, ToolOwner};
use crate::runner::AttemptClock;
use rmcp::model::*;
use rmcp::transport::Transport;
use rmcp::{ErrorData, RoleServer};
use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
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
#[derive(Clone, Copy, PartialEq, Eq)]
enum InitializeDelivery {
    NotStarted,
    Sending,
    NotifiedWhileSending,
    Delivered,
}
pub(super) struct State {
    pub phase: Connection,
    pub registry: PathBuf,
    pub ids: HashMap<RequestId, OwnedId>,
    pub active: Option<ToolOwner>,
    initialize: Option<RequestId>,
    initialize_delivery: InitializeDelivery,
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
            !self
                .ids
                .get(&owner.id)
                .is_some_and(|owned| Arc::ptr_eq(&owned.cancelled, &owner.cancelled))
                && owner.handle.is_finished()
        }) {
            let owner = self.active.take().expect("finished owner");
            let _ = owner.handle.join();
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
struct LegacyError<'a> {
    jsonrpc: JsonRpcVersion2_0,
    id: &'a Option<RequestId>,
    error: &'a ErrorData,
}
struct WriteJob {
    output: ServerJsonRpcMessage,
    complete: oneshot::Sender<io::Result<()>>,
    initialization: Option<Shared>,
}
impl WriteJob {
    fn publish_delivery(&self, succeeded: bool) {
        if let Some(shared) = &self.initialization {
            let mut state = lock(shared);
            if !succeeded {
                state.stop();
            } else if state.phase == Connection::AwaitingInitialize {
                if state.initialize_delivery == InitializeDelivery::NotifiedWhileSending {
                    state.phase = Connection::Ready;
                }
                state.initialize_delivery = InitializeDelivery::Delivered;
            }
        }
    }
}
fn writer() -> io::Result<mpsc::Sender<WriteJob>> {
    let (tx, mut rx) = mpsc::channel::<WriteJob>(1);
    std::thread::Builder::new()
        .name("mcp-output".into())
        .spawn(move || {
            let mut stdout = io::stdout().lock();
            while let Some(job) = rx.blocking_recv() {
                let mut bytes = CappedBytes::new(RESPONSE_LIMIT - 1);
                let serialized = match &job.output {
                    JsonRpcMessage::Error(message) => serde_json::to_writer(
                        &mut bytes,
                        &LegacyError {
                            jsonrpc: JsonRpcVersion2_0,
                            id: &message.id,
                            error: &message.error,
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
                // Publish successful flush before waking the independently
                // polled SDK send future; initialized cannot race that task.
                job.publish_delivery(!failed);
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
    initialization: Option<Shared>,
) -> io::Result<()> {
    let (complete, finished) = oneshot::channel();
    tokio::time::timeout(remaining, async {
        writer
            .send(WriteJob {
                output,
                complete,
                initialization,
            })
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

// Receive-side responses remain bounded while cancellation/EOF is consumed.
// reserve() and polling borrowed receivers are cancellation-safe: dropping
// receive() never drops replies or renews their original deadlines.
struct PendingReply {
    id: Option<RequestId>,
    job: Option<WriteJob>,
    finished: oneshot::Receiver<io::Result<()>>,
    deadline: tokio::time::Instant,
}
struct BoundedTransport {
    shared: Shared,
    input: mpsc::Receiver<Input>,
    writer: mpsc::Sender<WriteJob>,
    partial: Arc<Mutex<Option<Instant>>>,
    shutdown: &'static AtomicBool,
    started: Instant,
    pending: VecDeque<PendingReply>,
}
impl BoundedTransport {
    fn queue_reply(&mut self, output: ServerJsonRpcMessage, left: Duration) {
        // Delivery bookkeeping shares the approved eight-ID capacity with SDK
        // handlers; even a rejection must not retain a ninth owned identity.
        let at_capacity = lock(&self.shared).ids.len() + self.pending.len() >= 8;
        if at_capacity {
            self.fail();
            return;
        }
        let (complete, finished) = oneshot::channel();
        let id = match &output {
            JsonRpcMessage::Response(response) => Some(response.id.clone()),
            JsonRpcMessage::Error(error) => error.id.clone(),
            _ => None,
        };
        self.pending.push_back(PendingReply {
            id,
            job: Some(WriteJob {
                output,
                complete,
                initialization: None,
            }),
            finished,
            deadline: tokio::time::Instant::now() + left,
        });
    }
    fn reject(&mut self, id: Option<RequestId>, code: i32, clock: AttemptClock) {
        let left = lock(&self.shared)
            .ids
            .values()
            .map(|owned| remaining(owned.clock, owned.budget))
            .fold(remaining(clock, CONTROL_BUDGET), Duration::min);
        self.queue_reply(ServerJsonRpcMessage::error(error(code), id), left);
    }
    async fn finish_reply(
        writer: &mpsc::Sender<WriteJob>,
        pending: &mut PendingReply,
    ) -> io::Result<()> {
        tokio::time::timeout_at(pending.deadline, async {
            if pending.job.is_some() {
                let permit = writer
                    .reserve()
                    .await
                    .map_err(|_| io::Error::other("delivery failure"))?;
                permit.send(pending.job.take().expect("owned reply"));
            }
            (&mut pending.finished)
                .await
                .map_err(|_| io::Error::other("delivery failure"))?
        })
        .await
        .map_err(|_| io::Error::other("delivery deadline"))?
    }
    fn fail(&self) {
        lock(&self.shared).stop();
        eprintln!("mcp connection closed");
    }
    fn timed_out(&self) -> bool {
        if self
            .pending
            .iter()
            .any(|reply| tokio::time::Instant::now() >= reply.deadline)
        {
            return true;
        }
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
        let id = match &message {
            JsonRpcMessage::Response(response) => Some(response.id.clone()),
            JsonRpcMessage::Error(error) => error.id.clone(),
            _ => None,
        };
        // Snapshot the owner before returning the independently polled send
        // future. Its completion must never retire a later use of this ID.
        let owned = id.as_ref().and_then(|id| {
            lock(&shared)
                .ids
                .get(id)
                .map(|owned| (owned.clock, owned.budget, owned.cancelled.clone()))
        });
        async move {
            let left = {
                let state = lock(&shared);
                if matches!(state.phase, Connection::Closing | Connection::Closed) {
                    return Err(io::Error::other("connection closing"));
                }
                owned.as_ref().map_or(CONTROL_BUDGET, |(clock, budget, _)| {
                    remaining(*clock, *budget)
                })
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
            let suppressed = owned
                .as_ref()
                .is_some_and(|(_, _, cancelled)| cancelled.load(Ordering::Relaxed));
            let delivered = if suppressed {
                Ok(())
            } else {
                let initialization = if matches!(
                    &output,
                    JsonRpcMessage::Response(response)
                        if matches!(response.result, ServerResult::InitializeResult(_))
                ) {
                    let mut state = lock(&shared);
                    if state.phase == Connection::AwaitingInitialize
                        && state.initialize.as_ref() == id.as_ref()
                    {
                        state.initialize_delivery = InitializeDelivery::Sending;
                        Some(shared.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };
                deliver(&writer, output, left, initialization).await
            };
            let mut state = lock(&shared);
            if delivered.is_err() {
                state.stop();
                eprintln!("mcp delivery failed");
            } else if let Some(id) = id {
                let same_owner = owned.as_ref().is_some_and(|(_, _, cancelled)| {
                    state
                        .ids
                        .get(&id)
                        .is_some_and(|current| Arc::ptr_eq(&current.cancelled, cancelled))
                });
                if same_owner {
                    state.ids.remove(&id);
                }
                state.reap_cancelled();
            }
            delivered
        }
    }
    async fn receive(&mut self) -> Option<ClientJsonRpcMessage> {
        loop {
            {
                let mut state = lock(&self.shared);
                // Cancellations are translated locally, not forwarded to rmcp:
                // every handler completion still reaches send(), including
                // validation/admission refusals with no supervisor.
                state.reap_cancelled();
                if matches!(state.phase, Connection::Closing | Connection::Closed) {
                    return None;
                }
            }
            if self.shutdown.load(Ordering::Relaxed) || self.timed_out() {
                self.fail();
                return None;
            }
            let waiting_for_initialize_delivery = {
                let state = lock(&self.shared);
                state.phase == Connection::AwaitingInitialize
                    && state.initialize_delivery == InitializeDelivery::NotifiedWhileSending
            };
            if waiting_for_initialize_delivery {
                // Bytes can be visible before flush returns. Retain initialized
                // but leave the next frame in the bounded reader channel until
                // successful writer publication, not SDK send-task resumption.
                // No tool exists yet; the original handshake deadline applies.
                tokio::time::sleep(Duration::from_millis(5)).await;
                continue;
            }
            let input = if let Some(pending) = self.pending.front_mut() {
                // Both borrowed futures are cancellation-safe. Only delivery
                // completion releases this reply; input can still stop owned
                // work while stdout is blocked.
                tokio::select! {
                    biased;
                    result = Self::finish_reply(&self.writer, pending) => {
                        if result.is_err() {
                            self.fail();
                            return None;
                        }
                        self.pending.pop_front();
                        continue;
                    }
                    input = self.input.recv() => input,
                    _ = tokio::time::sleep(Duration::from_millis(5)) => continue,
                }
            } else {
                match tokio::time::timeout(Duration::from_millis(5), self.input.recv()).await {
                    Ok(input) => input,
                    Err(_) => continue,
                }
            };
            let Some(input) = input else {
                lock(&self.shared).stop();
                return None;
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
            let mut value = match framing::parse(&bytes) {
                Ok(value) => value,
                Err(_) => {
                    self.reject(None, -32700, clock);
                    continue;
                }
            };
            let Some(object) = value.as_object() else {
                self.reject(None, -32600, clock);
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
                self.reject(None, -32600, clock);
                continue;
            }
            let method = object["method"].as_str().expect("checked method");
            let Some(id) = id else {
                match method {
                    "notifications/initialized" => {
                        let mut state = lock(&self.shared);
                        if state.phase == Connection::AwaitingInitialize
                            && object.get("params").is_none_or(|p| {
                                p.as_object().is_some_and(|p| {
                                    p.keys().all(|key| key == "_meta")
                                        && p.get("_meta").is_none_or(Value::is_object)
                                })
                            })
                        {
                            match state.initialize_delivery {
                                InitializeDelivery::Sending => {
                                    state.initialize_delivery =
                                        InitializeDelivery::NotifiedWhileSending;
                                }
                                InitializeDelivery::Delivered => state.phase = Connection::Ready,
                                _ => {}
                            }
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
                        if let Some(cancel_id) = object
                            .get("params")
                            .and_then(|p| p.get("requestId"))
                            .and_then(checked_id)
                        {
                            if let Some(owned) = lock(&self.shared).ids.get(&cancel_id) {
                                owned.cancelled.store(true, Ordering::Relaxed);
                            }
                        }
                        continue;
                    }
                    _ => continue,
                }
            };
            let code = {
                let state = lock(&self.shared);
                if state.ids.contains_key(&id)
                    || self
                        .pending
                        .iter()
                        .any(|reply| reply.id.as_ref() == Some(&id))
                    || state.ids.len() + self.pending.len() >= 8
                {
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
                self.reject(Some(id), code, clock);
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
                self.reject(Some(id), -32602, clock);
                continue;
            }
            if method == "tools/call" && !self.pending.is_empty() {
                // Backpressure owns delivery capacity, not another execution.
                // Refuse without invoking or queueing the synchronous runner.
                let operation = params
                    .and_then(|p| p.get("name"))
                    .and_then(Value::as_str)
                    .and_then(super::handler::Operation::from_name)
                    .expect("checked tool");
                match super::handler::new_request_id() {
                    Ok(domain_id) => {
                        let arguments = value
                            .get_mut("params")
                            .and_then(Value::as_object_mut)
                            .and_then(|params| params.remove("arguments"))
                            .and_then(|arguments| match arguments {
                                Value::Object(arguments) => Some(arguments),
                                _ => None,
                            });
                        // Keep the handler's input-before-capacity precedence,
                        // moving arguments rather than copying source text.
                        let result = match super::handler::prepare(
                            operation,
                            arguments,
                            domain_id.clone(),
                        ) {
                            Ok(_) => super::handler::failure(
                                operation,
                                &domain_id,
                                super::handler::Failure::Busy,
                            ),
                            Err(result) => result,
                        };
                        let mut result = ServerResult::CallToolResult(result);
                        result.strip_result_type_for_legacy_peer();
                        self.queue_reply(
                            ServerJsonRpcMessage::response(result, id),
                            remaining(clock, operation.budget()),
                        );
                    }
                    Err(()) => self.reject(Some(id), -32603, clock),
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
                self.queue_reply(response, remaining(clock, CONTROL_BUDGET));
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
                    self.reject(Some(id), -32602, clock);
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
                self.reject(Some(id), -32602, clock);
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

/// Prepares native documents when configured, then runs the selected local
/// service. The preparation owner stays alive through transport shutdown. Does
/// not join a potentially blocked stdin or stdout thread; the owning executable
/// bounds runtime teardown and exits.
///
/// # Errors
/// Returns a source-free I/O error if the runtime, transport or protocol service
/// cannot run. EOF and cancellation are not evidence of mutation rollback.
pub fn run(
    registry: PathBuf,
    validator_engine: Option<PathBuf>,
    shutdown: &'static AtomicBool,
) -> io::Result<()> {
    let prepared_documents = validator_engine
        .as_deref()
        .map(|binary| crate::runner::stock_validation::native_docs::prepare(binary, shutdown))
        .transpose()
        .map_err(io::Error::other)?;
    if let Some(prepared) = &prepared_documents {
        eprintln!("mcp validator ready {} us", prepared.elapsed_us());
    }
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
            initialize_delivery: InitializeDelivery::NotStarted,
        }));
        let transport = BoundedTransport {
            shared: shared.clone(),
            input,
            writer,
            partial,
            shutdown,
            started: Instant::now(),
            pending: VecDeque::new(),
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
    drop(prepared_documents);
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
            initialize_delivery: InitializeDelivery::Delivered,
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
    fn initialized_and_catalog_do_not_depend_on_sdk_write_acknowledgment() {
        runtime().block_on(async {
            let shared = state();
            {
                let mut state = lock(&shared);
                state.phase = Connection::AwaitingInitialize;
                state.initialize_delivery = InitializeDelivery::NotStarted;
            }
            let (send_input, input) = mpsc::channel(4);
            let (writer, mut output) = mpsc::channel::<WriteJob>(1);
            let mut transport = BoundedTransport {
                shared: shared.clone(),
                input,
                writer,
                partial: Arc::new(Mutex::new(None)),
                shutdown: &RUNNING,
                started: Instant::now(),
                pending: VecDeque::new(),
            };
            // A genuinely premature notification cannot enable the catalog.
            send_input
                .send(frame(serde_json::json!({
                    "jsonrpc":"2.0","method":"notifications/initialized"
                })))
                .await
                .unwrap();
            send_input
                .send(frame(serde_json::json!({
                    "jsonrpc":"2.0","id":1,"method":"initialize",
                    "params":{"protocolVersion":"2025-11-25","capabilities":{},
                        "clientInfo":{"name":"regression","version":"1"}}
                })))
                .await
                .unwrap();
            assert!(transport.receive().await.is_some());
            assert!(lock(&shared).phase == Connection::AwaitingInitialize);
            let result = {
                use rmcp::ServerHandler;
                McpService {
                    shared: shared.clone(),
                }
                .get_info()
            };
            let send = transport.send(ServerJsonRpcMessage::response(
                ServerResult::InitializeResult(result),
                NumberOrString::Number(1),
            ));
            let mut send = std::pin::pin!(send);
            let mut context = std::task::Context::from_waker(std::task::Waker::noop());
            assert!(send.as_mut().poll(&mut context).is_pending());
            let job = output.recv().await.unwrap();
            // Model response bytes already visible, with flush and SDK ack
            // independently controlled.
            send_input
                .send(frame(serde_json::json!({
                    "jsonrpc":"2.0","method":"notifications/initialized"
                })))
                .await
                .unwrap();
            send_input
                .send(frame(serde_json::json!({
                    "jsonrpc":"2.0","id":2,"method":"tools/list"
                })))
                .await
                .unwrap();
            {
                let mut receive = std::pin::pin!(transport.receive());
                assert!(receive.as_mut().poll(&mut context).is_pending());
            }
            assert!(lock(&shared).phase == Connection::AwaitingInitialize);
            assert!(lock(&shared).initialize_delivery == InitializeDelivery::NotifiedWhileSending);
            job.publish_delivery(true);
            // Catalog request is admitted before the SDK send future is even
            // acknowledged, and survives its dropped concurrent receive.
            let request = transport.receive().await.unwrap();
            assert!(matches!(request, JsonRpcMessage::Request(request)
                if matches!(request.request, ClientRequest::ListToolsRequest(_))));
            assert!(lock(&shared).phase == Connection::Ready);
            job.complete.send(Ok(())).unwrap();
            send.await.unwrap();
        });
    }

    #[test]
    fn sdk_returns_catalog_while_initialization_send_task_waits_for_ack() {
        runtime().block_on(async {
            let shared = state();
            {
                let mut state = lock(&shared);
                state.phase = Connection::AwaitingInitialize;
                state.initialize_delivery = InitializeDelivery::NotStarted;
            }
            let (send_input, input) = mpsc::channel(4);
            let (writer, mut output) = mpsc::channel::<WriteJob>(1);
            let transport = BoundedTransport {
                shared: shared.clone(),
                input,
                writer,
                partial: Arc::new(Mutex::new(None)),
                shutdown: &RUNNING,
                started: Instant::now(),
                pending: VecDeque::new(),
            };
            let service = rmcp::service::serve_directly(
                McpService {
                    shared: shared.clone(),
                },
                transport,
                None,
            );
            send_input
                .send(frame(serde_json::json!({
                    "jsonrpc":"2.0","id":1,"method":"initialize",
                    "params":{"protocolVersion":"2025-11-25","capabilities":{},
                        "clientInfo":{"name":"regression","version":"1"}}
                })))
                .await
                .unwrap();
            let initialization = tokio::time::timeout(Duration::from_secs(1), output.recv())
                .await
                .unwrap()
                .unwrap();
            assert!(
                matches!(&initialization.output, JsonRpcMessage::Response(response)
                if matches!(response.result, ServerResult::InitializeResult(_)))
            );
            initialization.publish_delivery(true);
            send_input
                .send(frame(serde_json::json!({
                    "jsonrpc":"2.0","method":"notifications/initialized"
                })))
                .await
                .unwrap();
            send_input
                .send(frame(serde_json::json!({
                    "jsonrpc":"2.0","id":2,"method":"tools/list"
                })))
                .await
                .unwrap();
            // The SDK independently polls receive and response-send tasks. The
            // initialized notification and following list must work even when
            // initialization's send task has not received its acknowledgment.
            let catalog = tokio::time::timeout(Duration::from_secs(1), output.recv())
                .await
                .unwrap()
                .unwrap();
            let public = serde_json::to_value(&catalog.output).unwrap();
            assert_eq!(public["id"], 2);
            assert_eq!(
                public["result"]["tools"],
                serde_json::to_value(super::super::schema::tools()).unwrap()
            );
            assert!(lock(&shared).phase == Connection::Ready);
            initialization.complete.send(Ok(())).unwrap();
            catalog.complete.send(Ok(())).unwrap();
            drop(send_input);
            tokio::time::timeout(Duration::from_secs(1), service.waiting())
                .await
                .unwrap()
                .unwrap();
        });
    }

    #[test]
    fn blocked_receive_reply_keeps_cancellation_and_eof_observable() {
        runtime().block_on(async {
            for enqueued in [false, true] {
                for cancel in [false, true] {
                    let shared = state();
                    let id = NumberOrString::Number(1);
                    let cancelled = Arc::new(AtomicBool::new(false));
                    let (release, blocked) = std::sync::mpsc::channel();
                    let clock = AttemptClock::start();
                    {
                        let mut state = lock(&shared);
                        state.ids.insert(
                            id.clone(),
                            OwnedId {
                                clock,
                                budget: CONTROL_BUDGET,
                                cancelled: cancelled.clone(),
                            },
                        );
                        state.active = Some(ToolOwner {
                            id: id.clone(),
                            clock,
                            budget: CONTROL_BUDGET,
                            cancelled: cancelled.clone(),
                            handle: std::thread::spawn(move || {
                                blocked.recv().unwrap();
                            }),
                        });
                    }
                    let (send_input, input) = mpsc::channel(4);
                    let (writer, mut output) = mpsc::channel::<WriteJob>(1);
                    let mut blocked_finished = None;
                    if !enqueued {
                        let (complete, finished) = oneshot::channel();
                        blocked_finished = Some(finished);
                        writer
                            .send(WriteJob {
                                output: ServerJsonRpcMessage::error(error(-32700), None),
                                complete,
                                initialization: None,
                            })
                            .await
                            .unwrap();
                    }
                    let mut transport = BoundedTransport {
                        shared: shared.clone(),
                        input,
                        writer,
                        partial: Arc::new(Mutex::new(None)),
                        shutdown: &RUNNING,
                        started: Instant::now(),
                        pending: VecDeque::new(),
                    };
                    send_input
                        .send(frame(serde_json::json!({
                            "jsonrpc":"2.0","id":2,"method":"server/discover"
                        })))
                        .await
                        .unwrap();
                    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
                    {
                        let mut receive = std::pin::pin!(transport.receive());
                        assert!(receive.as_mut().poll(&mut context).is_pending());
                    }
                    let deadline = transport.pending.front().unwrap().deadline;
                    assert_eq!(transport.pending.front().unwrap().job.is_none(), enqueued);
                    let held = output.recv().await.unwrap();
                    send_input
                        .send(frame(serde_json::json!({
                            "jsonrpc":"2.0","id":3,"method":"tools/call",
                            "params":{"name":"read_script","arguments":{
                                "project_root":"/fixture/project","script_path":"res://test.gd"
                            }}
                        })))
                        .await
                        .unwrap();
                    send_input
                        .send(frame(serde_json::json!({
                            "jsonrpc":"2.0","id":4,"method":"unsupported"
                        })))
                        .await
                        .unwrap();
                    send_input
                        .send(frame(serde_json::json!({
                            "jsonrpc":"2.0","id":5,"method":"tools/call",
                            "params":{"name":"read_script","arguments":{"project_root":42}}
                        })))
                        .await
                        .unwrap();
                    {
                        let mut receive = std::pin::pin!(transport.receive());
                        assert!(receive.as_mut().poll(&mut context).is_pending());
                    }
                    assert_eq!(transport.pending.len(), 4);
                    let busy = transport.pending[1].job.as_ref().unwrap();
                    let public = serde_json::to_value(&busy.output).unwrap();
                    assert_eq!(
                        public["result"]["structuredContent"]["error"]["code"],
                        "server_busy"
                    );
                    let invalid = transport.pending[3].job.as_ref().unwrap();
                    let public = serde_json::to_value(&invalid.output).unwrap();
                    assert_eq!(
                        public["result"]["structuredContent"]["error"]["category"],
                        "input"
                    );
                    assert_eq!(lock(&shared).ids.len(), 1, "another tool was not admitted");
                    assert!(!cancelled.load(Ordering::Relaxed));
                    if cancel {
                        send_input
                            .send(frame(serde_json::json!({
                                "jsonrpc":"2.0","method":"notifications/cancelled",
                                "params":{"requestId":1}
                            })))
                            .await
                            .unwrap();
                        {
                            let mut receive = std::pin::pin!(transport.receive());
                            assert!(receive.as_mut().poll(&mut context).is_pending());
                        }
                        assert!(cancelled.load(Ordering::Relaxed));
                        assert_eq!(transport.pending.front().unwrap().deadline, deadline);
                        assert!(lock(&shared).phase == Connection::Ready);
                    }
                    send_input.send(Input::End).await.unwrap();
                    assert!(transport.receive().await.is_none());
                    assert!(cancelled.load(Ordering::Relaxed));
                    assert!(lock(&shared).phase == Connection::Closing);
                    assert!(Arc::ptr_eq(&lock(&shared).ids[&id].cancelled, &cancelled));
                    assert!(lock(&shared).active.is_some());
                    // Neither interruption depended on releasing stdout.
                    held.complete.send(Ok(())).unwrap();
                    if let Some(finished) = blocked_finished {
                        finished.await.unwrap().unwrap();
                    }
                    release.send(()).unwrap();
                    lock(&shared).active.take().unwrap().handle.join().unwrap();
                }
            }
        });
    }

    #[test]
    fn receive_replies_survive_dropped_futures_before_and_after_enqueue() {
        runtime().block_on(async {
            for request in [
                serde_json::json!({"jsonrpc":"2.0","id":"reply","method":"server/discover"}),
                serde_json::json!({"jsonrpc":"2.0","id":"reply","method":"unsupported"}),
                serde_json::json!({"jsonrpc":"2.0","id":"reply","method":"ping","params":{"private_source":"must not echo"}}),
            ] {
                let (send_input, input) = mpsc::channel(2);
                let (writer, mut output) = mpsc::channel::<WriteJob>(1);
                let (complete, _finished) = oneshot::channel();
                writer.send(WriteJob {
                    output: ServerJsonRpcMessage::error(error(-32700), None),
                    complete,
                    initialization: None,
                }).await.unwrap();
                send_input.send(frame(request.clone())).await.unwrap();
                let mut transport = BoundedTransport {
                    shared: state(), input, writer,
                    partial: Arc::new(Mutex::new(None)), shutdown: &RUNNING,
                    started: Instant::now(), pending: VecDeque::new(),
                };
                {
                    let mut receive = std::pin::pin!(transport.receive());
                    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
                    assert!(receive.as_mut().poll(&mut context).is_pending());
                }
                let deadline = transport.pending.front().unwrap().deadline;
                assert!(transport.pending.front().unwrap().job.is_some());
                output.recv().await.unwrap().complete.send(Ok(())).unwrap();
                {
                    let mut receive = std::pin::pin!(transport.receive());
                    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
                    assert!(receive.as_mut().poll(&mut context).is_pending());
                }
                assert!(transport.pending.front().unwrap().job.is_none());
                assert_eq!(transport.pending.front().unwrap().deadline, deadline);
                let reply = output.recv().await.unwrap();
                let public = serde_json::to_value(&reply.output).unwrap();
                assert_eq!(public["id"], "reply");
                if request["method"] == "server/discover" {
                    assert_eq!(public["result"]["supportedVersions"], serde_json::json!(["2025-11-25"]));
                } else {
                    assert!(public.get("error").is_some());
                }
                assert!(!public.to_string().contains("private_source"));
                reply.complete.send(Ok(())).unwrap();
                send_input.send(frame(serde_json::json!({"jsonrpc":"2.0","id":"next","method":"ping"}))).await.unwrap();
                let next = transport.receive().await.unwrap();
                let JsonRpcMessage::Request(next) = next else { panic!("next request lost"); };
                assert_eq!(next.id, NumberOrString::String("next".into()));
                assert!(transport.pending.is_empty());
                assert!(output.try_recv().is_err(), "reply must not be duplicated");
            }
        });
    }

    #[test]
    fn dropped_receive_does_not_renew_reply_deadline() {
        runtime().block_on(async {
            let (_send_input, input) = mpsc::channel(1);
            let (writer, _output) = mpsc::channel::<WriteJob>(1);
            let mut transport = BoundedTransport {
                shared: state(),
                input,
                writer,
                partial: Arc::new(Mutex::new(None)),
                shutdown: &RUNNING,
                started: Instant::now(),
                pending: VecDeque::new(),
            };
            // A later reply cannot inherit the first reply's longer deadline.
            transport.queue_reply(
                ServerJsonRpcMessage::error(error(-32600), Some(NumberOrString::Number(1))),
                CONTROL_BUDGET,
            );
            transport.queue_reply(
                ServerJsonRpcMessage::error(error(-32600), None),
                Duration::ZERO,
            );
            assert!(transport.receive().await.is_none());
            assert!(lock(&transport.shared).phase == Connection::Closing);
        });
    }

    #[test]
    fn backpressured_receive_reply_capacity_is_bounded() {
        runtime().block_on(async {
            let shared = state();
            let (send_input, input) = mpsc::channel(16);
            let (writer, mut output) = mpsc::channel::<WriteJob>(1);
            let (complete, _finished) = oneshot::channel();
            writer
                .send(WriteJob {
                    output: ServerJsonRpcMessage::error(error(-32700), None),
                    complete,
                    initialization: None,
                })
                .await
                .unwrap();
            for id in 1..=9 {
                send_input
                    .send(frame(serde_json::json!({
                        "jsonrpc":"2.0","id":id,"method":"unsupported"
                    })))
                    .await
                    .unwrap();
            }
            let mut transport = BoundedTransport {
                shared: shared.clone(),
                input,
                writer,
                partial: Arc::new(Mutex::new(None)),
                shutdown: &RUNNING,
                started: Instant::now(),
                pending: VecDeque::new(),
            };
            assert!(transport.receive().await.is_none());
            assert!(lock(&shared).phase == Connection::Closing);
            assert_eq!(transport.pending.len(), 8);
            assert!(lock(&shared).ids.is_empty());
            assert!(transport.pending.iter().all(|reply| reply.job.is_some()));
            assert!(matches!(
                output.recv().await.unwrap().output,
                JsonRpcMessage::Error(_)
            ));
            assert!(output.try_recv().is_err());
        });
    }

    #[test]
    fn cancelled_non_dispatched_completions_retire_only_their_owner() {
        runtime().block_on(async {
            for request in [
                serde_json::json!({"jsonrpc":"2.0","id":1,"method":"ping"}),
                serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
                serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"read_script","arguments":{"project_root":42}}}),
            ] {
                let shared = state();
                let (send_input, input) = mpsc::channel(4);
                let (writer, mut output) = mpsc::channel::<WriteJob>(1);
                let mut transport = BoundedTransport {
                    shared: shared.clone(), input, writer,
                    partial: Arc::new(Mutex::new(None)), shutdown: &RUNNING,
                    started: Instant::now(), pending: VecDeque::new(),
                };
                send_input.send(frame(request)).await.unwrap();
                assert!(transport.receive().await.is_some());
                let old = lock(&shared).ids[&NumberOrString::Number(1)].cancelled.clone();
                send_input.send(frame(serde_json::json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":1}}))).await.unwrap();
                send_input.send(frame(serde_json::json!({"jsonrpc":"2.0","id":2,"method":"ping"}))).await.unwrap();
                assert!(transport.receive().await.is_some());
                assert!(old.load(Ordering::Relaxed));
                let completion = transport.send(ServerJsonRpcMessage::error(error(-32602), Some(NumberOrString::Number(1))));
                completion.await.unwrap();
                assert!(!lock(&shared).ids.contains_key(&NumberOrString::Number(1)));
                assert!(output.try_recv().is_err(), "cancelled completion was delivered");
                assert!(!transport.timed_out());

                // A delayed old send future owns an Arc, never just the reused ID.
                lock(&shared).ids.insert(NumberOrString::Number(1), OwnedId {
                    clock: AttemptClock::start(), budget: CONTROL_BUDGET, cancelled: old,
                });
                let delayed = transport.send(ServerJsonRpcMessage::error(error(-32602), Some(NumberOrString::Number(1))));
                let new = Arc::new(AtomicBool::new(false));
                lock(&shared).ids.insert(NumberOrString::Number(1), OwnedId {
                    clock: AttemptClock::start(), budget: CONTROL_BUDGET, cancelled: new.clone(),
                });
                delayed.await.unwrap();
                assert!(Arc::ptr_eq(&lock(&shared).ids[&NumberOrString::Number(1)].cancelled, &new));
                assert!(output.try_recv().is_err());
            }
        });
    }

    #[test]
    fn sdk_cancelled_controls_validation_and_busy_calls_leave_connection_usable() {
        runtime().block_on(async {
            for request in [
                serde_json::json!({"jsonrpc":"2.0","id":1,"method":"ping"}),
                serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}),
                serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"read_script","arguments":{"project_root":42}}}),
                serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"read_script","arguments":{"project_root":"/fixture/project","script_path":"res://test.gd"}}}),
            ] {
                let shared = state();
                let active_cancelled = Arc::new(AtomicBool::new(false));
                let (release, blocked) = std::sync::mpsc::channel();
                lock(&shared).active = Some(ToolOwner {
                    id: NumberOrString::String("supervised".into()),
                    clock: AttemptClock::start(), budget: CONTROL_BUDGET,
                    cancelled: active_cancelled.clone(),
                    handle: std::thread::spawn(move || { blocked.recv().unwrap(); }),
                });
                let (send_input, input) = mpsc::channel(4);
                let (writer, mut output) = mpsc::channel::<WriteJob>(1);
                // Queue before starting the SDK so cancellation is processed
                // before its independently spawned handler can complete.
                send_input.try_send(frame(request)).unwrap();
                send_input.try_send(frame(serde_json::json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":1}}))).unwrap();
                send_input.try_send(frame(serde_json::json!({"jsonrpc":"2.0","id":2,"method":"ping"}))).unwrap();
                let transport = BoundedTransport {
                    shared: shared.clone(), input, writer,
                    partial: Arc::new(Mutex::new(None)), shutdown: &RUNNING,
                    started: Instant::now(), pending: VecDeque::new(),
                };
                let service = rmcp::service::serve_directly(
                    McpService { shared: shared.clone() }, transport, None,
                );
                let reply = tokio::time::timeout(Duration::from_secs(1), output.recv()).await.unwrap().unwrap();
                let public = serde_json::to_value(&reply.output).unwrap();
                assert_eq!(public["id"], 2);
                assert_eq!(public["result"], serde_json::json!({}));
                reply.complete.send(Ok(())).unwrap();
                tokio::time::timeout(Duration::from_secs(1), async {
                    while !lock(&shared).ids.is_empty() {
                        tokio::task::yield_now().await;
                    }
                }).await.unwrap();
                assert!(output.try_recv().is_err());
                assert!(lock(&shared).active.is_some());
                assert!(!active_cancelled.load(Ordering::Relaxed));
                // Reuse the cancelled ID only after its old SDK completion.
                send_input.send(frame(serde_json::json!({"jsonrpc":"2.0","id":1,"method":"ping"}))).await.unwrap();
                let reply = tokio::time::timeout(Duration::from_secs(1), output.recv()).await.unwrap().unwrap();
                let public = serde_json::to_value(&reply.output).unwrap();
                assert_eq!(public["id"], 1);
                assert_eq!(public["result"], serde_json::json!({}));
                reply.complete.send(Ok(())).unwrap();
                release.send(()).unwrap();
                lock(&shared).active.take().unwrap().handle.join().unwrap();
                drop(send_input);
                tokio::time::timeout(Duration::from_secs(1), service.waiting()).await.unwrap().unwrap();
            }
        });
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
                pending: VecDeque::new(),
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
            send_input.send(frame(serde_json::json!({"jsonrpc":"2.0","id":3,"method":"ping"}))).await.unwrap();
            let mut transport = BoundedTransport { shared: shared.clone(), input, writer, partial: Arc::new(Mutex::new(None)), shutdown: &RUNNING, started: Instant::now(), pending: VecDeque::new() };
            let received = transport.receive().await.unwrap();
            assert!(matches!(received, JsonRpcMessage::Request(_)));
            assert!(cancelled.load(Ordering::Relaxed));
            assert!(!lock(&shared).ids[&NumberOrString::Number(2)].cancelled.load(Ordering::Relaxed));
            assert!(lock(&shared).active.is_some());
            release.send(()).unwrap();
            let owner = lock(&shared).active.take().unwrap();
            owner.handle.join().unwrap();
        });
    }

    #[test]
    fn finished_supervisor_waits_for_completion_and_cannot_erase_reused_id() {
        runtime().block_on(async {
            let shared = state();
            let id = NumberOrString::Number(1);
            let old = Arc::new(AtomicBool::new(true));
            let clock = AttemptClock::start();
            {
                let mut state = lock(&shared);
                state.ids.insert(
                    id.clone(),
                    OwnedId {
                        clock,
                        budget: CONTROL_BUDGET,
                        cancelled: old.clone(),
                    },
                );
                state.active = Some(ToolOwner {
                    id: id.clone(),
                    clock,
                    budget: CONTROL_BUDGET,
                    cancelled: old,
                    handle: std::thread::spawn(|| {}),
                });
            }
            tokio::time::timeout(Duration::from_secs(1), async {
                while !lock(&shared).active.as_ref().unwrap().handle.is_finished() {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
            {
                let mut state = lock(&shared);
                state.reap_cancelled();
                assert!(
                    state.active.is_some(),
                    "SDK completion still owns the old ID"
                );
                assert!(state.ids.contains_key(&id));
                let new = Arc::new(AtomicBool::new(false));
                state.ids.insert(
                    id.clone(),
                    OwnedId {
                        clock,
                        budget: CONTROL_BUDGET,
                        cancelled: new.clone(),
                    },
                );
                state.reap_cancelled();
                assert!(state.active.is_none());
                assert!(Arc::ptr_eq(&state.ids[&id].cancelled, &new));
            }
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
            let mut transport = BoundedTransport {
                shared: shared.clone(),
                input,
                writer,
                partial: Arc::new(Mutex::new(None)),
                shutdown: &RUNNING,
                started: Instant::now(),
                pending: VecDeque::new(),
            };
            for _ in 0..8 {
                assert!(transport.receive().await.is_some());
            }
            assert!(transport.receive().await.is_none());
            assert!(
                transport.pending.is_empty(),
                "ninth ID must not be retained for rejection"
            );
            assert!(
                output.try_recv().is_err(),
                "capacity failure cannot queue another reply"
            );
            let state = lock(&shared);
            assert!(state.phase == Connection::Closing);
            assert_eq!(state.ids.len(), 8);
            assert!(state
                .ids
                .values()
                .all(|owned| owned.cancelled.load(Ordering::Relaxed)));
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
                None,
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
            pending: VecDeque::new(),
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
                initialize: None, initialize_delivery: InitializeDelivery::Delivered,
            }));
            let (send_input, input) = mpsc::channel(1);
            let (writer, mut output) = mpsc::channel::<WriteJob>(1);
            let transport = BoundedTransport { shared: shared.clone(), input, writer, partial: Arc::new(Mutex::new(None)), shutdown: &RUNNING, started: Instant::now(), pending: VecDeque::new() };
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
