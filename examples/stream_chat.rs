use ai_agent::{
    constant::GPT_4O_MINI_MODEL,
    llm::{
        semaphore::get_semaphore,
        stream::{chat_stream_with_retry},
    },
};

use tokio::task::JoinSet;
use tracing::{Instrument, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let prompts = vec![
        "Explain Rust ownership in three sentences",
        "What is asynchronous programming, and how does it differ from multithreading?",
        "Explain the TCP three-way handshake",
        "Explain what a large language model is in simple terms",
        "What is the difference between Arc and Rc in Rust?",
        "What is RAG, and why is it common in AI applications?",
        "Explain the difference between HTTP and HTTPS",
        "What is a deadlock, and how can it be prevented?",
        "Explain recursion using an everyday analogy",
        "Why is Rust memory-safe even though it has no GC?",
    ];

    let mut set = JoinSet::new();
    for prompt in prompts {
        let span = tracing::info_span!("Chat", prompt = prompt);
        set.spawn(
            async move {
                tracing::info!("\n\n{prompt}");
                let permit = get_semaphore().acquire().await?;
                let output =
                    chat_stream_with_retry(GPT_4O_MINI_MODEL, Some("You are a versatile assistant."), prompt)
                        .await?;
                drop(permit);
                Ok::<_, anyhow::Error>((prompt, output))
            }
            .instrument(span),
        );
    }

    while let Some(result) = set.join_next().await {
        match result {
            Ok(Ok((prompt, result)))=> tracing::info!("\n{prompt}\n{result}"),
            Ok(Err(err)) => tracing::error!("Task panicked: {err}"),
            Err(err)=> tracing::error!("Task panicked: {err}"),
        }
    }

    Ok(())
}
