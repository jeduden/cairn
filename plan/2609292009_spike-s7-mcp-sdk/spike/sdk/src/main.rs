//! Spike S7, SDK side: `event_search` as a stub tool on rmcp over stdio.
use std::{sync::Arc, time::Duration};

use rmcp::{
    ErrorData, RoleServer, ServerHandler, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ListToolsResult,
        PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
    },
    service::RequestContext,
};
use serde_json::{Value, json};

struct Stub;

fn input_schema() -> Arc<serde_json::Map<String, Value>> {
    let Value::Object(map) = json!({
        "type": "object",
        "properties": {"query": {"type": "string"}, "k": {"type": "integer"}},
        "required": ["query"],
    }) else {
        unreachable!()
    };
    Arc::new(map)
}

impl ServerHandler for Stub {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let tool = Tool::new("event_search", "Search the record (stub)", input_schema());
        Ok(ListToolsResult { tools: vec![tool], ..Default::default() })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let query = request
            .arguments
            .as_ref()
            .and_then(|a| a.get("query"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        // "slow" stands in for a long query: it waits until cancelled.
        if query == "slow" {
            tokio::select! {
                _ = context.ct.cancelled() => {}
                _ = tokio::time::sleep(Duration::from_secs(30)) => {}
            }
        }
        let envelope = json!({"trust": "untrusted", "hits": [{"address": "w1:1", "query": query}]});
        let mut result = CallToolResult::structured(envelope.clone());
        result.content = vec![ContentBlock::text(envelope.to_string())];
        Ok(result.into())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = Stub.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
