mod file_io;
mod genai;
use dotenv::dotenv;
use file_io::FileIO;
use file_io::{FileIOMessage, Message};
use genai::Genai;
use std::env;
use std::io;
use std::io::Write;
use tokio::sync::mpsc;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = env::var("GEMINI_API_KEY").expect("error loading env");
    let (sender, receiver) = mpsc::channel(32);
    let mut client = Genai::new(api_key, "gemini-pro", sender.clone());

    tokio::spawn(async move {
        let mut file_io = FileIO::new();
        if let Err(e) = file_io.start(receiver).await {
            println!("Error writing to file {}", e);
        }
    });
    loop {
        println!("Enter a prompt:");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let trimmed = input.trim();
        if trimmed == "exit" {
            sender
                .send(FileIOMessage {
                    text: String::new(),
                    message: Message::Remove,
                })
                .await?;
            break;
        }
        client.message_thread(input).await?;
    }
    println!("Ending session");
    Ok(())
}
