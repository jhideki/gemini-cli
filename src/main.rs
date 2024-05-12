mod errors;
mod file_io;
mod genai;

use errors::{EnvVarError, InvalidArgument};
use file_io::FileIO;
use file_io::{read, FileIOMessage, Message};
use genai::Genai;

use phf::phf_map;
use std::io;
use std::io::Write;
use std::{env, process};
use tokio::sync::mpsc;

static ARGS: phf::Map<&'static str, usize> = phf_map! (
    "-h" => 1,
    "-v" => 2,
    "-a" => 3,
    "-m" => 4,
    "-l" => 5,
    "-p" => 6,
    "-f" => 7,
);

fn display_help() {
    println!("'-h': Display help information about the program");
    println!("'-v': Display version information");
    println!("'-f': Pass a file to Gemini");
    println!("'-p': Prompt to send to Gemini");
    println!("'-a': Set api key \n E.g., gemini-cli -a <YOUR API KEY>. This will set a global environment variable");
    println!("'-m': Set gemini model.");
    println!("'-l': List gemini models.");
}

async fn run_commands(
    args: Vec<Arguments>,
    client: Genai,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut prompt: Option<String> = None;
    for arg in args {
        match arg.cmd.as_str() {
            "-v" => println!("Version 0.1.0. Drop a star and follow me on github pls. https://github.com/jhideki"),
            "-a" => {
                if let Some(value) = arg.value {
                    env::set_var("GEMINI_API_KEY", value);
                } else {
                    return Err(Box::new(InvalidArgument {}));
                }
            }
            "-p" => {
                if let Some(value) = arg.value {
                    prompt = Some(value);
                }
            }
            "-h" => {
                display_help();
            }
            "-f" => {
                if let Some(value) = arg.value {
                    if let Some(ref mut prompt) = &mut prompt {
                        if let Ok(data) = read(&value) {
                            prompt.push_str(&data);
                        }
                    } else {
                        if let Ok(data) = read(&value) {
                            prompt = Some(String::from(&data));
                        }
                    }
                }
            }
            _ => {
                return Err(Box::new(InvalidArgument {}));
            }
        }
    }
    if let Some(prompt) = prompt {
        let _ = client.single_query(prompt).await;
    }
    Ok(())
}

struct Arguments {
    cmd: String,
    value: Option<String>,
    order: usize,
}

fn process_arguments(args: Vec<String>) -> Result<Vec<Arguments>, Box<dyn std::error::Error>> {
    let mut arg_commands: Vec<Arguments> = Vec::new();
    for mut i in 1..args.len() {
        match args[i].chars().nth(0).unwrap() {
            '-' => {
                if let Some(order) = ARGS.get(&args[i][1..]) {
                    arg_commands.push(Arguments {
                        cmd: args[i].clone(),
                        value: None,
                        order: *order,
                    })
                }
            }
            '"' => {
                let mut prompt = String::new();
                while i < args.len() && !args[i].ends_with("\"") {
                    prompt.push_str(&args[i].clone());
                    prompt.push(' ');
                    i += 1;
                }
                if let Some(mut arg) = arg_commands.pop() {
                    arg.value = Some(prompt);
                    arg_commands.push(arg);
                } else {
                    return Err(Box::new(InvalidArgument {}));
                }
            }
            _ => {
                return Err(Box::new(InvalidArgument {}));
            }
        }
    }
    arg_commands.sort_by_key(|k| k.order);
    Ok(arg_commands)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const EXIT: &str = "exit";
    const YES: &str = "yY";

    let api_key = match env::var("GEMINI_API_KEY") {
        Ok(val) => val,
        Err(e) => {
            println!("Failed to load env variable {}", e);
            process::exit(0);
        }
    };

    let (sender, receiver) = mpsc::channel(32);
    let mut client = Genai::new(api_key, "gemini-pro", sender.clone());

    tokio::spawn(async move {
        let mut file_io = FileIO::new();
        if let Err(e) = file_io.start(receiver).await {
            println!("Error writing to file {}", e);
        }
    });

    //CLI arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        if let Ok(arguments) = process_arguments(args) {
            if let Err(e) = run_commands(arguments, client).await {
                println!("Error: {}", e);
            }
        }
        std::process::exit(0);
    }

    //Main runtime loop
    loop {
        println!("Enter a prompt:");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let trimmed = input.trim();
        if trimmed == EXIT {
            print!("Would you like to delete the './responses' directory? (y/n): ");
            io::stdout().flush().unwrap();
            let _ = io::stdin().read_line(&mut input);
            let trimmed = input.trim();
            if YES.contains(trimmed) {
                sender
                    .send(FileIOMessage {
                        text: String::new(),
                        message: Message::Remove,
                        file_name: String::new(),
                    })
                    .await?;
            }
            println!("The 'repsonses' directory has been deleted. Exiting the program...");
            break;
        }

        if let (Some(start), Some(end)) = (trimmed.find("<"), trimmed.find(">")) {
            if let Some(file_name) = trimmed.get(start + 1..end) {
                if let Ok(data) = read(file_name) {
                    input.push_str(&data);
                }
            }
        }
        client.message_thread(input).await?;
    }
    println!("Ending session");
    Ok(())
}
