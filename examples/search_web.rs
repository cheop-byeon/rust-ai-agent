use ai_agent::{
    knowledge_base::{chunk::fixed_length_chunking, search::vector_search},
    tools::web_search::execute::{WebSearchArgs, search_web},
};
use anyhow::Ok;
use tiktoken_rs::cl100k_base;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env, including API keys.
    dotenvy::dotenv()?;

    // Initialize logging to observe requests made by web_search.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Step 1: prepare a broad web search. The topic of the World Cup's top
    // scorer can return related but less relevant content such as schedules,
    // teams, and highlight reels.
    let web_search_args = WebSearchArgs {
        query: "top scorer of the 2026 World Cup in Canada, Mexico, and the United States".to_string(),
        max_results: 10,
        topic: "general".to_string(),
        time_range: Some("year".to_string()),
    };

    // Execute the search and collect web results.
    let output = search_web(web_search_args).await?;

    // Join every result's title and content into one long text to simulate
    // passing raw search results directly to the model.
    let full_text = output
        .results
        .iter()
        .map(|r| format!("Title: {}\n{}", r.title, r.content))
        .collect::<Vec<_>>()
        .join("\n\n");

    // Language models process text as tokens rather than characters. API
    // pricing and context length are both measured in tokens.
    //
    // cl100k_base is the tokenizer used by the GPT-4/3.5 generation. Use it
    // locally to estimate token counts without calling an API.
    //
    // The actual gpt-4o-mini model uses the newer o200k_base tokenizer, so
    // counts may differ slightly, but they are sufficient for comparing savings.
    let enc = cl100k_base()?;
    
    // Step 2: measure how many tokens the raw text would consume.
    let total_tokens = enc.encode_with_special_tokens(&full_text).len();

    println!("Total characters: {}", full_text.len());
    println!("Total tokens: {}", total_tokens);

    // Step 3: split each search result into 500-character chunks with 50
    // characters of overlap, preserving context across chunk boundaries.
    let mut all_chunks = Vec::new();
    for result in &output.results {
        let text = format!("Title: {}\n{}", result.title, result.content);
        for chunk in fixed_length_chunking(&text, 500, 50) {
            all_chunks.push(chunk);
        }
    }

    println!("Total chunks: {}", all_chunks.len());

    // Step 4: vector search. Use the same query, but search the chunks created
    // above instead of the entire internet. vector_search embeds the query and
    // each chunk, then ranks them by cosine similarity.
    let query = "top scorer of the 2026 World Cup in Canada, Mexico, and the United States";
    let hits = vector_search(query, &all_chunks, 3).await?;

    println!("\nQuery: '{query}'");
    println!("{}", "=".repeat(60));
    for (i, hit) in hits.iter().enumerate() {
        // Use chars() instead of byte slicing so multibyte characters are not
        // cut in the middle.
        let preview: String = hit.text.chars().take(300).collect();
        println!("\n[{}] Similarity: {:.3}", i + 1, hit.similarity);
        println!("{preview}");
    }

    // Step 5: keep only the three most relevant chunks and measure the token
    // reduction after compression.
    let selected_text = hits
        .iter()
        .map(|hit| hit.text.clone())
        .collect::<Vec<_>>()
        .join("\n\n");
    let selected_tokens = enc.encode_with_special_tokens(&selected_text).len();

    println!("\n{}", "=".repeat(60));
    println!("Total tokens: {total_tokens}");
    println!("Selected tokens: {selected_tokens}");
    // Savings from sending all results to sending only the most relevant chunks.
    println!(
        "Savings rate: {:.1}%",
        (1.0 - selected_tokens as f64 / total_tokens as f64) * 100.0
    );

    Ok(())
}
