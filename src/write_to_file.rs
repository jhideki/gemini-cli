use phf::phf_map;
use tokio::fs::File;
use tokio::io;
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

pub struct FileWriteMessage {
    file: File,
    text: String,
}

pub async fn write(mut receiver: mpsc::Receiver<FileWriteMessage>) -> Result<()> {
    while let Some(message) = receiver.recv().await {
        message.file.write_all(message.text.as_bytes()).await?;
    }
    Ok(())
}

pub fn create(file_type: String) -> Result<File, io::Error> {
    if let Some(extension) = EXTENSIONS.get(&file_type) {
        return Ok(File::create(format!("response.{}", extension)));
    }
    Ok(File::create("response.md"))
}
