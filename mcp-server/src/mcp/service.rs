use super::transport::{lock, Connection, Shared};
use super::{handler, schema};
use crate::runner::AttemptClock;
use rmcp::model::*;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};
use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

static SUPPORTED_VERSIONS: &[ProtocolVersion] = &[ProtocolVersion::V_2025_11_25];

pub(super) struct ToolOwner {
    pub id: RequestId,
    pub clock: AttemptClock,
    pub budget: std::time::Duration,
    pub cancelled: Arc<AtomicBool>,
    pub handle: std::thread::JoinHandle<()>,
}
pub(super) struct McpService {
    pub shared: Shared,
}
impl ServerHandler for McpService {
    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(SUPPORTED_VERSIONS)
    }
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2025_11_25)
            .with_server_info(Implementation::new(
                "godot-agent-kit-mcp",
                env!("CARGO_PKG_VERSION"),
            ))
    }
    async fn list_tools(
        &self,
        _: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(schema::tools()))
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let operation = handler::Operation::from_name(&request.name)
            .ok_or_else(|| super::transport::error(-32602))?;
        let domain_id = handler::new_request_id().map_err(|_| super::transport::error(-32603))?;
        let invocation = match handler::prepare(operation, request.arguments, domain_id.clone()) {
            Ok(invocation) => invocation,
            Err(result) => return Ok(result.into()),
        };
        let (sender, receiver) = tokio::sync::oneshot::channel();
        {
            let mut state = lock(&self.shared);
            if state.phase != Connection::Ready {
                return Ok(handler::failure(
                    operation,
                    &domain_id,
                    handler::Failure::HostBeforeDispatch,
                )
                .into());
            }
            if state.active.is_some() {
                return Ok(handler::failure(operation, &domain_id, handler::Failure::Busy).into());
            }
            let Some(owned) = state.ids.get(&context.id) else {
                return Err(super::transport::error(-32600));
            };
            let clock = owned.clock;
            let cancelled = owned.cancelled.clone();
            let registry = state.registry.clone();
            let worker_cancelled = cancelled.clone();
            let fallback_id = domain_id.clone();
            let handle = match std::thread::Builder::new()
                .name("mcp-supervisor".into())
                .spawn(move || {
                    // A panic cannot recover operation certainty after dispatch.
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        handler::execute(invocation, &registry, clock, &worker_cancelled)
                    }))
                    .unwrap_or_else(|_| {
                        handler::failure(
                            operation,
                            &fallback_id,
                            handler::Failure::HostAfterDispatch,
                        )
                    });
                    let _ = sender.send(result);
                }) {
                Ok(handle) => handle,
                Err(_) => {
                    return Ok(handler::failure(
                        operation,
                        &domain_id,
                        handler::Failure::HostBeforeDispatch,
                    )
                    .into())
                }
            };
            state.active = Some(ToolOwner {
                id: context.id.clone(),
                clock,
                budget: operation.budget(),
                cancelled,
                handle,
            });
        }
        // The transport also translates cancellation before forwarding the SDK
        // notification. The owner is retained even if the SDK drops this future.
        let mut receiver = receiver;
        tokio::select! {
            result = &mut receiver => result.map(Into::into).map_err(|_| super::transport::error(-32603)),
            _ = context.ct.cancelled() => {
                if let Some(owned) = lock(&self.shared).ids.get(&context.id) {
                    owned.cancelled.store(true, Ordering::Relaxed);
                }
                receiver.await.map(Into::into).map_err(|_| super::transport::error(-32603))
            }
        }
    }
}
