mod genai;
use dotenv::dotenv;
use genai::Genai;
use reqwest::Error;
use std::env;
use std::io;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("GEMINI_API_KEY").expect("error loading env");
    let client = Genai::new(api_key, "gemini-pro");
    loop {
        println!("Enter a prompt:");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input);
        let trimmed = input.trim();
        if trimmed == "exit" {
            break;
        }
        client.single_query(input).await?;
    }
    println!("Ending session");
    Ok(())
}
