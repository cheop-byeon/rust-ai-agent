use ai_agent::{constant::GPT_4O_MINI_MODEL, llm::{structured::chat_complete_structured}};
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

    let plan = chat_complete_structured(
        GPT_4O_MINI_MODEL,
        Some("You are a versatile assistant."),
        "I want to attend matches at the World Cup in Canada, Mexico, and the United States. How should I plan the trip?",
    )
    .await?;

    println!("Response: {plan:#?}");

    Ok(())
}
