use anyhow::Result;
use rmcp::model::{CallToolRequestParams, Tool};
use rmcp::service::{RoleClient, RunningService};
use rmcp::{
    ServiceExt,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

/// Wrapper for an MCP server connection. It starts the expense_mcp_server
/// child process internally and communicates with it over stdio.
pub struct McpClient {
    service: RunningService<RoleClient, ()>,
}

impl McpClient {
    /// Start expense_mcp_server as a child process and establish a connection.
    pub async fn connect() -> Result<Self> {
        let service = ()
            .serve(TokioChildProcess::new(Command::new("cargo").configure(
                |cmd| {
                    cmd.args(["run", "--quiet", "--bin", "expense_mcp_server"]);
                },
            ))?)
            .await?;

        Ok(Self { service })
    }

    /// Retrieve all tools exposed by the server.
    pub async fn list_tools(&self) -> Result<Vec<Tool>> {
        let result = self.service.list_tools(Default::default()).await?;
        Ok(result.tools)
    }

    /// Call a tool by name with a JSON object of arguments.
    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<String> {
        let params = CallToolRequestParams::new(name.to_string())
            .with_arguments(arguments.as_object().cloned().unwrap_or_default());

        let result = self.service.call_tool(params).await?;

        let text = result
            .content
            .iter()
            .filter_map(|block| block.as_text().map(|t| t.text.clone()))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(text)
    }
}
