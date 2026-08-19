use std::sync::Arc;

use serde_json::Value;

use crate::{agent::ExecutionContext, tools::{mcp::client::McpClient, tool::Tool}};

/// Wraps an MCP tool in the Tool trait understood by the Agent.
/// Each McpTool represents one entry from an MCP server's list_tools() result.
/// Execution is forwarded to McpClient::call_tool, so the Agent loop treats it
/// exactly like calculator or web_search.
pub struct McpTool {
    client: Arc<McpClient>,
    name: String,
    description: String,
    parameters: Value,
}

impl McpTool {
    /// Convert rmcp's raw protocol tool description into our McpTool.
    /// name and description become owned Strings because Tool requires
    /// name(&self) -> &str to return a reference tied to self's lifetime;
    /// borrowing the 'static Cow<str> from rmcp::model::Tool is not sufficient.
    pub fn new(client: Arc<McpClient>, tool: rmcp::model::Tool) -> Self {
        let parameters = Value::Object((*tool.input_schema).clone());

        Self {
            client,
            name: tool.name.to_string(),
            description: tool
                .description
                .map(|d| d.to_string())
                .unwrap_or_default(),
            parameters,
        }
    }
}

#[async_trait::async_trait]
impl Tool for McpTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters(&self) -> Value {
        self.parameters.clone()
    }

    async fn execute(&self, args_json: &str, _context: &ExecutionContext) -> anyhow::Result<String> {
        // The model provides arguments as a JSON string. Parse them into a
        // Value and forward them to McpClient::call_tool. The downstream
        // request to the MCP server and expense-tracker-api is handled there.
        let args: Value = serde_json::from_str(args_json)?;
        self.client.call_tool(&self.name, args).await
    }
}
