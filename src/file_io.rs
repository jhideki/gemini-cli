use phf::phf_map;
use std::fs;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, Result};
use tokio::sync::mpsc;

static EXTENSIONS: phf::Map<&'static str, &'static str> = phf_map! (
    "python" => "py",
    "js" => "js",
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
}

pub struct FileIO {
    is_writing: bool,
    file_name: Option<String>,
    file_path: String,
}

impl FileIO {
    pub fn new() -> Self {
        FileIO {
            is_writing: false,
            file_name: None,
            file_path: String::from(""),
        }
    }
    pub async fn start(&mut self, mut receiver: mpsc::Receiver<FileIOMessage>) -> Result<()> {
        while let Some(io_message) = receiver.recv().await {
            match io_message.message {
                Message::Write => self.write(io_message.text).await?,
                Message::Remove => self.remove(),
            }
        }
        Ok(())
    }
    pub async fn write(&mut self, message: String) -> Result<()> {
        let (skip_line, is_writing) =
            check_for_code(&message, self.is_writing, &mut self.file_name);
        self.is_writing = is_writing;
        if skip_line {
            return Ok(());
        }
        if let Some(ref file) = self.file_name {
            if let Some(extension) = EXTENSIONS.get(&file) {
                self.file_path = String::from(format!("response.{}", extension));
                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(&self.file_path)
                    .await?;
                file.write_all(message.as_bytes()).await?;
            }
        }
        Ok(())
    }

    pub fn remove(&self) {
        match fs::remove_file(&self.file_path) {
            Ok(_) => println!("File: {} has been deleted", self.file_path),
            Err(e) => eprintln!("Error deleting file: {}", e),
        }
    }
}

fn check_for_code(line: &String, is_writing: bool, file_name: &mut Option<String>) -> (bool, bool) {
    let mut file_type = String::new();
    if let Some(index) = line.find("```") {
        if is_writing {
            return (true, false);
        } else {
            let remaining_line = &line[index + 3..];
            for byte in remaining_line.bytes() {
                let c = byte as char;
                if c == '\n' {
                    *file_name = Some(file_type);
                    return (true, true);
                } else {
                    file_type.push(c);
                }
            }
        }
    }
    (false, is_writing)
}
