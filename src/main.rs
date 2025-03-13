#[tokio::main]
async fn main() {
    if let Err(e) = auto_pr::run().await {
        eprintln!("Error generating PR description: {}", e);
        std::process::exit(1);
    }
    println!("Successfully generated PR description in pr_prompt.md");
}