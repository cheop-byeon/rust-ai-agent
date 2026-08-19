use ai_agent::{constant::GPT_4O_MINI_MODEL, llm::complete::chat_complete, tools::build_toolbox};
use anyhow::Ok;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let toolbox = build_toolbox().await?;

    let result = chat_complete(
        GPT_4O_MINI_MODEL,
        Some(
                r#"You are a versatile assistant. Today's date is July 22, 2026.
        You can use tools to search for current information.
        Important: when a tool returns search results, use those results directly in your answer. Do not say "the information has not been published" or "I don't know".
        Your training data has a cutoff date and may be outdated, so always prioritize information returned by tools."#,
            ),
            "What was the score of the 2026 World Cup final?",
        &toolbox,
    )
    .await?;

    tracing::info!("Response: {result:#?}");

    Ok(())
}
