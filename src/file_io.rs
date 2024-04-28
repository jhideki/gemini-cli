use phf::phf_map;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWrite, AsyncWriteExt, Result};
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

#[derive(Clone)]
pub struct FileWriteMessage {
    pub file_name: Option<String>,
    pub text: String,
}

pub async fn write(mut receiver: mpsc::Receiver<FileWriteMessage>) -> Result<()> {
    while let Some(message) = receiver.recv().await {
        if let Some(file_name) = message.file_name {
            if let Some(extension) = EXTENSIONS.get(&file_name) {
                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(format!("response.{}", extension))
                    .await?;
                file.write_all(message.text.as_bytes()).await?;
            }
        }
    }
    Ok(())
}
