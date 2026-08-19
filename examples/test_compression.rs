use ai_agent::{
    agent::Agent, callback::search_compressor::SearchCompressorCallback, constant::GPT_4O_MINI_MODEL, tools::build_toolbox,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let toolbox = Arc::new(build_toolbox().await?);

    let agent = Agent::new(
        GPT_4O_MINI_MODEL,
        Some("You are an assistant who makes good use of web search.".to_string()),
        toolbox,
    )
    .with_max_steps(5)
    .with_after_tool_callback(Arc::new(SearchCompressorCallback));

    let result = agent.run("What are the highlights of the 2026 World Artificial Intelligence Conference (WAIC)?").await?;
    println!("\nFinal answer: {}", result.output);

    Ok(())
}
