use std::sync::Arc;

use ai_agent::{agent::Agent, constant::GPT_4O_MINI_MODEL, tools::build_toolbox};
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

    let toolbox = Arc::new(build_toolbox().await?);

    let now = Local::now();
    let current_time = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let instructions = format!(
        r#"You are a professional, reliable, and helpful AI assistant.

    Current local time: {}

    Always interpret relative time expressions such as "today", "yesterday", "tomorrow",
    "this week", "this month", and "last month" relative to the current time above.

    Tool usage rules:
    1. Answer directly when the question can be answered without a tool.
    2. Use Web Search for current information such as news, weather, exchange rates, stocks, or web searches.
    3. Use Calculator for precise calculations.
    4. Use the Expense MCP tools to query, summarize, create, modify, or delete expense records.
    5. Do not guess data that a tool can provide. Call the tool instead of answering "I don't know".
    6. After a tool returns, answer naturally, concisely, and accurately without describing the tool-call process."#,
        current_time
    );

    let agent = Agent::new(GPT_4O_MINI_MODEL, Some(&instructions), toolbox).with_max_steps(8);

    println!("\n=== Agent::run test ===");
    let result = agent
        .run(
                r"I want to buy a Mac Mini M4.

Please help me analyze the purchase:
1. Use the search tool to find the current price of the Mac Mini M4.
2. Query my Software expenses from the past three months.
3. Calculate how many times the Mac Mini M4 price is greater than those expenses,
    and how many months I would need to save 500 yuan per month to afford it.
4. Based on my spending, recommend whether I should buy it.

All prices and spending data must come from tools. Do not guess any data.",
        )
        .await?;

    println!("Answer: {}", result.output);
    println!(
        "\nThis run took {} steps and recorded {} events (execution_id = {})",
        result.context.current_step,
        result.context.events.len(),
        result.context.execution_id
    );

    println!(
        "Token usage: prompt={} completion={} total={}",
        result.context.usage.prompt_tokens,
        result.context.usage.completion_tokens,
        result.context.usage.total_tokens
    );

    println!("{:#?}", result.context);

    Ok(())
}
