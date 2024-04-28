use crate::file_io::{create, write, FileWriteMessage};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::fs::File;
use tokio::sync::mpsc;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Part {
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestBody {
    contents: Vec<Content>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SafetyRating {
    category: String,
    probability: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Candidate {
    content: Option<Content>,
    #[serde(rename = "finishReason")]
    finish_reason: String,
    index: u32,
    #[serde(rename = "safetyRatings")]
    safety_ratings: Vec<SafetyRating>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    candidates: Vec<Candidate>,
}

pub struct Genai {
    api_key: String,
    model_name: String,
    end_point: String,
    message_thread: Vec<Content>,
    client: Client,
}
impl Genai {
    pub fn new(api_key: String, model_name: &str) -> Self {
        Genai {
            api_key,
            model_name: model_name.to_string(),
            end_point: "https://generativelanguage.googleapis.com/v1beta/models/".to_string(),
            message_thread: Vec::new(),
            client: Client::new(),
        }
    }

    pub async fn single_query(&self, prompt: String) -> Result<(), Box<dyn std::error::Error>> {
        let part: Part = Part { text: prompt };
        let content: Content = Content {
            role: "user".to_string(),
            parts: vec![part],
        };
        let request_body = RequestBody {
            contents: vec![content],
        };
        let url = format!(
            "{}{}:generateContent?key={}",
            self.end_point, self.model_name, self.api_key
        );
        println!("{}", url);
        println!("{}", serde_json::to_string(&request_body).unwrap());
        let client = Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let json_response: Value = response.json().await?;
        println!("{}", json_response.to_string());
        Ok(())
    }

    pub async fn message_thread(
        &mut self,
        prompt: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let part = Part { text: prompt };
        self.message_thread.push(Content {
            role: "user".to_string(),
            parts: vec![part],
        });

        let url = format!(
            "{}{}:streamGenerateContent?alt=sse&key={}",
            self.end_point, self.model_name, self.api_key
        );

        let request_body = RequestBody {
            contents: self.message_thread.clone(),
        };

        let res = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if let Ok(response_parts) = Self::parse_stream(res).await {
            self.message_thread.push(Content {
                role: "model".to_string(),
                parts: response_parts,
            });
        }

        Ok(())
    }

    async fn parse_stream(mut stream: Response) -> Result<Vec<Part>, Box<dyn std::error::Error>> {
        let mut response_parts: Vec<Part> = Vec::new();
        let (sender, receiver) = mpsc::channel(32);
        let mut message = FileWriteMessage {
            file_name: None,
            text: String::from(""),
        };

        tokio::spawn(async move {
            if let Err(e) = write(receiver).await {
                println!("Error writing to file {}", e);
            }
        });

        while let Some(chunk) = stream.chunk().await? {
            let json_string = std::str::from_utf8(&chunk)?;
            println!("{}", json_string);
            if json_string.starts_with("data: ") {
                let json_data = json_string.trim_start_matches("data: ");
                let data: Data = serde_json::from_str(json_data)?;
                if let Some(content) = &data.candidates[0].content {
                    let text = &content.parts[0].text;
                    response_parts.push(content.parts[0].clone());
                    message.file_name = Self::check_for_code(&text);
                    message.text = text.clone();
                    sender.send(message.clone());
                    println!("{}", text);
                }
            } else {
            }
        }
        Ok(response_parts)
    }
    fn check_for_code(line: &String) -> Option<String> {
        let mut file_type = String::new();
        if let Some(index) = line.find("```") {
            while let Some(c) = line[index..].chars().next() {
                if c == '\\' {
                    return Some(file_type);
                } else {
                    file_type.push(c);
                }
            }
        }
        None
    }
}
