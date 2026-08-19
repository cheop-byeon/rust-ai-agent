use std::sync::Arc;

use ai_agent::{agent::Agent, callback::{approval::ApprovalCallback, search_compressor::SearchCompressorCallback}, constant::{GPT_4O_MINI_MODEL, VISION_MODEL}, tools::build_file_explorer_toolbox};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let toolbox = Arc::new(build_file_explorer_toolbox(VISION_MODEL));

    let instructions = r#"You are an assistant skilled at exploring files.
When you receive an archive, extract it first, then use list_files to inspect the directory structure.
Use filenames to identify potentially relevant files, and confirm them one by one with read_file or read_image.
Only then provide a conclusion. Do not skip the exploration steps and guess."#;

    let agent = Agent::new(GPT_4O_MINI_MODEL, Some(instructions), toolbox)
        .with_max_steps(20)
        // Dangerous operations require human approval. delete_file is on the
        // list; the other file tools do not require approval.
        .with_before_tool_callback(Arc::new(ApprovalCallback::new(["delete_file"])))
        // Compress long web_search results automatically instead of handling
        // them manually each time.
        .with_after_tool_callback(Arc::new(SearchCompressorCallback));

    let result = agent
        .run(r#"Read the candidate information and job requirements in this archive and identify the best candidate.
After confirming the result, delete job_description.txt because it is no longer needed.
Archive path: /Users/dave/Desktop/example/candidates.zip"#)
        .await?;

    println!("Answer: {}", result.output);
    println!(
        "\nThis run took {} steps and recorded {} events",
        result.context.current_step,
        result.context.events.len()
    );

    Ok(())
}
