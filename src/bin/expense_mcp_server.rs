use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{ErrorData as McpError, ServiceExt, schemars, tool, tool_router, transport::stdio};
use serde::{Deserialize, Serialize};

// ---------- Parameter and request-body types ----------

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ListExpensesParams {
    /// Filter by category, such as "Food". Case-insensitive; omit to return all categories.
    category: Option<String>,
    /// Filter by month in "YYYY-MM" format; omit to include all months.
    month: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GetSummaryParams {
    /// Summarize one month in "YYYY-MM" format; omit to summarize all historical data.
    month: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GetExpenseParams {
    /// Expense record ID from the result of list_expenses or create_expense.
    id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DeleteExpenseParams {
    /// Expense record ID to delete.
    id: String,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
struct CreateExpenseParams {
    description: String,
    amount: f64,
    category: String,
    /// ISO date format: "YYYY-MM-DD".
    date: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct UpdateExpenseParams {
    /// Expense record ID to update.
    id: String,
    description: Option<String>,
    amount: Option<f64>,
    category: Option<String>,
    /// ISO date format: "YYYY-MM-DD".
    date: Option<String>,
}

/// Sends only fields supplied by the user to the backend API, matching the
/// expense-tracker-api partial-update behavior. Unspecified fields (`None`) are
/// skipped by `skip_serializing_if` instead of being serialized as null and
/// accidentally cleared.
#[derive(Debug, Serialize)]
struct UpdateExpenseBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<String>,
}

// ---------- MCP server ----------

// This server does not connect to a database. It is an adapter for the
// expense-tracker-api Axum web service: each MCP tool sends an HTTP request
// and wraps the result in the format required by MCP.
#[derive(Clone)]
struct ExpenseServer {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl ExpenseServer {
    fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            // Prefer environment variables and fall back to local defaults so
            // the code does not need to change when deployed elsewhere.
            base_url: std::env::var("EXPENSE_API_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            api_key: std::env::var("EXPENSE_API_KEY")
                .unwrap_or_else(|_| "dev-secret-key".to_string()),
        }
    }

    /// Shared response handling for all six tools. Network failures and
    /// non-2xx API responses become `McpError`; only 2xx responses return the
    /// response body as a `CallToolResult`.
    async fn respond(
        result: Result<reqwest::Response, reqwest::Error>,
    ) -> Result<CallToolResult, McpError> {
        match result {
            Ok(resp) => {
                let status = resp.status();
                let body = resp
                    .text()
                    .await
                    .unwrap_or_else(|e| format!("Failed to read response body: {e}"));

                if status.is_success() {
                    Ok(CallToolResult::success(vec![ContentBlock::text(body)]))
                } else {
                    Err(McpError::internal_error(
                        format!("expense-tracker-api returned {status}: {body}"),
                        None,
                    ))
                }
            }
            Err(e) => Err(McpError::internal_error(
                format!("Request to expense-tracker-api failed: {e}"),
                None,
            )),
        }
    }
}

// #[tool_router(server_handler)] collects the methods marked with #[tool]
// into a tool list, so no separate ServerHandler implementation is needed.
#[tool_router(server_handler)]
impl ExpenseServer {
    #[tool(description = "List expenses, optionally filtered by category and/or month (YYYY-MM)")]
    async fn list_expenses(
        &self,
        Parameters(p): Parameters<ListExpensesParams>,
    ) -> Result<CallToolResult, McpError> {
        // Add only the category and month values supplied by the user.
        let mut query = vec![];
        if let Some(category) = &p.category {
            query.push(("category".to_string(), category.clone()));
        }
        if let Some(month) = &p.month {
            query.push(("month".to_string(), month.clone()));
        }

        let result = self
            .http
            .get(format!("{}/expenses", self.base_url))
            .header("x-api-key", &self.api_key)
            .query(&query)
            .send()
            .await;

        Self::respond(result).await
    }

    #[tool(description = "Get a single expense by its id")]
    async fn get_expense(
        &self,
        Parameters(p): Parameters<GetExpenseParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .http
            .get(format!("{}/expenses/{}", self.base_url, p.id))
            .header("x-api-key", &self.api_key)
            .send()
            .await;

        Self::respond(result).await
    }

    #[tool(description = "Create a new expense")]
    async fn create_expense(
        &self,
        Parameters(p): Parameters<CreateExpenseParams>,
    ) -> Result<CallToolResult, McpError> {
        // p is already the JSON body expected by the API, so it can be sent
        // directly with .json(&p).
        let result = self
            .http
            .post(format!("{}/expenses", self.base_url))
            .header("x-api-key", &self.api_key)
            .json(&p)
            .send()
            .await;

        Self::respond(result).await
    }

    #[tool(
        description = "Partially update an existing expense — only send the fields that should change"
    )]
    async fn update_expense(
        &self,
        Parameters(p): Parameters<UpdateExpenseParams>,
    ) -> Result<CallToolResult, McpError> {
        // Convert to a separate body so optional fields can be skipped. The
        // id belongs in the URL path and must not be included in the body.
        let body = UpdateExpenseBody {
            description: p.description,
            amount: p.amount,
            category: p.category,
            date: p.date,
        };

        let result = self
            .http
            .put(format!("{}/expenses/{}", self.base_url, p.id))
            .header("x-api-key", &self.api_key)
            .json(&body)
            .send()
            .await;

        Self::respond(result).await
    }

    #[tool(description = "Delete an expense by its id")]
    async fn delete_expense(
        &self,
        Parameters(p): Parameters<DeleteExpenseParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .http
            .delete(format!("{}/expenses/{}", self.base_url, p.id))
            .header("x-api-key", &self.api_key)
            .send()
            .await;

        Self::respond(result).await
    }

    #[tool(
        description = "Get total spending and a per-category breakdown, optionally for one month (YYYY-MM)"
    )]
    async fn get_summary(
        &self,
        Parameters(p): Parameters<GetSummaryParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut query = vec![];
        if let Some(month) = &p.month {
            query.push(("month".to_string(), month.clone()));
        }

        let result = self
            .http
            .get(format!("{}/expenses/summary", self.base_url))
            .header("x-api-key", &self.api_key)
            .query(&query)
            .send()
            .await;

        Self::respond(result).await
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Write logs to stderr, not stdout. With stdio transport, stdout is
    // reserved for MCP protocol messages and regular logs would corrupt it.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let server = ExpenseServer::new();
    // Start with stdio transport so the client can launch this process as a
    // child and exchange messages over standard input and output.
    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}