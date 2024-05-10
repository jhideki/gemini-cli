use phf::phf_map;
use std::env;
use std::fs;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, Result};
use tokio::sync::mpsc;

static EXTENSIONS: phf::Map<&'static str, &'static str> = phf_map! (
    "python" => "py",
    "javascript" => "js",
    "java" => "java",
    "jsx" => "jsx",
    "tsx" => "tsx",
    "c" => "c",
    "c++" => "cpp",
    "c#" => "cs",
    "html" => "html",
    "css" => "css",
    "typescript" => "ts",
    "go" => "go",
    "rust" => "rs",
    "php" => "php",
    "ruby" => "rb",
    "swift" => "swift",
    "kotlin" => "kt",
);

pub enum Message {
    Write,
    Remove,
}

pub struct FileIOMessage {
    pub text: String,
    pub message: Message,
    pub file_name: String,
}

pub struct FileIO {
    is_writing: bool,
    file_path: PathBuf,
    files: Vec<PathBuf>,
    file_name: String,
}

impl FileIO {
    pub fn new() -> Self {
        let mut path = match env::current_dir() {
            Ok(path) => path,
            Err(e) => {
                println!("Error getting file path: {}", e);
                std::process::exit(0);
            }
        };
        if !fs::metadata("responses/").is_ok() {
            fs::create_dir("responses").expect("Error creating responses dir");
        }
        path.push("responses/");
        FileIO {
            is_writing: false,
            file_path: path,
            files: Vec::new(),
            file_name: String::new(),
        }
    }

    pub async fn start(&mut self, mut receiver: mpsc::Receiver<FileIOMessage>) -> Result<()> {
        while let Some(io_message) = receiver.recv().await {
            match io_message.message {
                Message::Write => {
                    self.file_name = io_message.file_name;
                    self.write(io_message.text).await?;
                }
                Message::Remove => self.remove(),
            }
        }
        Ok(())
    }

    pub async fn write(&mut self, message: String) -> Result<()> {
        let lines: Vec<&str> = message.lines().collect();
        for i in 0..lines.len() {
            if self.check_for_code(lines[i]) {
                continue;
            }
            if self.is_writing {
                //check if it is the last line
                let line_buff = if i == lines.len() - 1 {
                    lines[i].to_string()
                } else {
                    format!("{}{}", lines[i], "\n")
                };
                self.write_to_file(&line_buff).await?;
            }
        }

        Ok(())
    }

    async fn write_to_file(&mut self, line: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.file_path)
            .await?;
        file.write_all(line.as_bytes()).await?;
        Ok(())
    }

    //Returns true if current line contains the MD: ```
    fn check_for_code(&mut self, line: &str) -> bool {
        if let Some(index) = line.find("```") {
            if self.is_writing {
                //reset file path
                self.file_path.pop();
                self.is_writing = false;
            } else {
                let remaining_line = &line[index + 3..];
                self.file_path.push(self.file_name.clone());
                if let Some(extension) = EXTENSIONS.get(&remaining_line) {
                    self.file_path.set_extension(extension);
                } else {
                    self.file_path.set_extension("md");
                }

                self.files.push(self.file_path.clone());
                self.is_writing = true;
            }
            return true;
        }
        false
    }

    pub fn remove(&mut self) {
        while let Some(file) = self.files.pop() {
            match fs::remove_file(&file) {
                Ok(_) => println!("File: {} has been deleted", file.to_string_lossy()),
                Err(e) => eprintln!("Error deleting file: {}", e),
            }
        }
        fs::remove_dir("responses").expect("error deleting dir");
    }
}

pub fn read(path: &str) -> Result<String> {
    fs::read_to_string(path)
}
