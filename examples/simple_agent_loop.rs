use ai_agent::{constant::GPT_4O_MINI_MODEL, llm::complete::chat_complete, tools::build_toolbox};
use chrono::Local;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

   // build_toolbox is async: it starts expense_mcp_server, performs the
   // handshake, discovers its tools, and adds them to the calculator and
   // web_search tools in the same toolbox.
    let toolbox = build_toolbox().await?;

   // Current time
    let now = Local::now();
    let current_time = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let system_prompt = format!(
        r#"You are a professional, reliable, and helpful AI assistant.

Current local time: {}

Always interpret relative time expressions such as "today", "yesterday", "tomorrow",
"this week", "this month", and "last month" relative to the current time above.

You can use multiple tools to complete a task.

Tool usage rules:

1. Answer directly when the question can be answered without a tool.

2. Use Web Search when the user needs current information, such as:
   - news
   - weather
   - exchange rates
   - stocks
   - web searches

3. Use Calculator for mathematical, monetary, percentage, or any other precise calculations.

4. Use the Expense MCP tools when the user needs to query, summarize, create, modify, or delete expense records, such as:
   - create_expense
   - list_expenses
   - get_summary
   and others.

5. Do not guess data that a tool can provide.

6. If a tool can provide the answer, call it instead of answering "I don't know".

7. After a tool returns, answer naturally, concisely, and accurately without describing the tool-call process.

Always prioritize completing the user's task over calling tools unnecessarily."#,
        current_time
    );

   println!("\n=== Test 0: Agent loop ===");
    let result = chat_complete(
        GPT_4O_MINI_MODEL,
        Some(&system_prompt),
      r"I want to buy a Mac Mini M4.

   Please help me analyze the purchase:

   1. Use the search tool to find the current price of the Mac Mini M4.
   2. Query my Software expenses from the past three months.
   3. Calculate:
      - how many times the Mac Mini M4 price is greater than those expenses;
      - how many months I would need to save 500 yuan per month to afford it.
   4. Based on my spending, recommend whether I should buy it.

   All prices and spending data must come from tools.
   Do not guess any data.",
        &toolbox,
    )
    .await?;
   println!("Answer: {result}");

   // // Test 1: query spending for a category in a month (should trigger
   // // get_summary or list_expenses).
   // println!("\n=== Test 1: query July Food spending ===");
    // let result = chat_complete(
    //     GPT_4O_MINI_MODEL,
    //     Some(&system_prompt),
   //     "How much did I spend on Food in July?",
    //     &toolbox,
    // )
    // .await?;
   // println!("Answer: {result}");

   // // Test 2: add an expense (should trigger create_expense).
   // println!("\n=== Test 2: add an expense ===");
    // let result = chat_complete(
    //     GPT_4O_MINI_MODEL,
    //     Some(&system_prompt),
   //     "Record an expense: I spent 28 yuan on coffee at Starbucks today in the Food category. Then summarize my July Food spending.",
    //     &toolbox,
    // )
    // .await?;
   // println!("Answer: {result}");

   // // Test 3: view the overall July expense summary (should trigger
   // // get_summary and include the expense added in Test 2 because
   // // expense-tracker-api shares data in process memory rather than
   // // resetting it for every request).
   // println!("\n=== Test 3: overall July expense summary ===");
    // let result = chat_complete(
    //     GPT_4O_MINI_MODEL,
    //     Some(&system_prompt),
   //     "Summarize my July expenses by category.",
    //     &toolbox,
    // )
    // .await?;
   // println!("Answer: {result}");

    Ok(())
}
