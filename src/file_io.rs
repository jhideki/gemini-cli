use phf::phf_map;
use std::fs;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, Result};
use tokio::sync::mpsc;

static EXTENSIONS: phf::Map<&'static str, &'static str> = phf_map! (
    "python" => "py",
    "javascript" => "js",
    "java" => "java",
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
    file_path: String,
    file_name: String,
    files: Vec<String>,
}

impl FileIO {
    pub fn new() -> Self {
        FileIO {
            is_writing: false,
            file_path: String::from(""),
            file_name: String::from(""),
            files: Vec::new(),
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
                self.is_writing = false;
            } else {
                let remaining_line = &line[index + 3..];
                if let Some(extension) = EXTENSIONS.get(&remaining_line) {
                    let file_path = String::from(format!("{}.{}", self.file_name, extension));
                    self.file_path = file_path.clone();
                    self.files.push(file_path.clone());
                    self.is_writing = true;
                }
            }
            return true;
        }
        false
    }

    pub fn remove(&mut self) {
        while let Some(file) = self.files.pop() {
            match fs::remove_file(&file) {
                Ok(_) => println!("File: {} has been deleted", self.file_path),
                Err(e) => eprintln!("Error deleting file: {}", e),
            }
        }
    }
}

pub fn read(path: &str) -> Result<String> {
    fs::read_to_string(path)
}
