use crate::file_io::{FileIOMessage, Message};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

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

#[derive(Serialize, Deserialize)]
struct Model {
    name: String,
    version: String,
    #[serde(rename = "displayName")]
    display_name: String,
    description: String,
    #[serde(rename = "inputTokenLimit")]
    input_token_limit: i32,
    #[serde(rename = "outputTokenLimit")]
    output_token_limit: i32,
    #[serde(rename = "supportedGenerationMethods")]
    supported_generation_methods: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Models {
    models: Vec<Model>,
}

pub struct Genai {
    api_key: String,
    model_name: String,
    end_point: String,
    message_thread: Vec<Content>,
    client: Client,
    sender: Sender<FileIOMessage>,
}

impl Genai {
    pub fn new(api_key: String, model_name: &str, sender: Sender<FileIOMessage>) -> Self {
        Genai {
            api_key,
            model_name: model_name.to_string(),
            end_point: "https://generativelanguage.googleapis.com/v1beta/models".to_string(),
            message_thread: Vec::new(),
            client: Client::new(),
            sender,
        }
    }

    pub async fn single_query(&self, prompt: String) -> Result<(), Box<dyn std::error::Error>> {
        let file_name = prompt
            .clone()
            .chars()
            .filter(|&c| !c.is_whitespace())
            .take(20)
            .collect();

        let part: Part = Part { text: prompt };

        let content: Content = Content {
            role: "user".to_string(),
            parts: vec![part],
        };

        let request_body = RequestBody {
            contents: vec![content],
        };

        let url = format!(
            "{}/{}:generateContent?key={}",
            self.end_point, self.model_name, self.api_key
        );

        let res = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if let Err(e) = self.parse_stream(res, file_name).await {
            println!("Error parsing stream: {}", e);
        }

        Ok(())
    }

    pub async fn message_thread(
        &mut self,
        prompt: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file_name = prompt
            .clone()
            .chars()
            .filter(|&c| !c.is_whitespace())
            .take(20)
            .collect();
        let part = Part { text: prompt };
        self.message_thread.push(Content {
            role: "user".to_string(),
            parts: vec![part],
        });

        let url = format!(
            "{}/{}:streamGenerateContent?alt=sse&key={}",
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

        if let Ok(response_parts) = self.parse_stream(res, file_name).await {
            self.message_thread.push(Content {
                role: "model".to_string(),
                parts: response_parts,
            });
        }

        Ok(())
    }

    pub async fn list_models(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}?key={}", self.end_point, self.api_key);

        let res = self.client.get(&url).send().await?.text().await?;
        let models: Models = serde_json::from_str(&res)?;
        for model in models.models {
            println!("{}, \n{}\n", model.display_name, model.description);
        }
        Ok(())
    }

    async fn parse_stream(
        &self,
        mut stream: Response,
        file_name: String,
    ) -> Result<Vec<Part>, Box<dyn std::error::Error>> {
        let mut response_parts: Vec<Part> = Vec::new();

        while let Some(chunk) = stream.chunk().await? {
            let json_string = std::str::from_utf8(&chunk)?;
            if json_string.starts_with("data: ") {
                let json_data = json_string.trim_start_matches("data: ");
                let data: Data = serde_json::from_str(json_data)?;
                if let Some(content) = &data.candidates[0].content {
                    let text = &content.parts[0].text;
                    response_parts.push(content.parts[0].clone());
                    let _ = self
                        .sender
                        .send(FileIOMessage {
                            text: text.clone(),
                            message: Message::Write,
                            file_name: file_name.clone(),
                        })
                        .await;
                    println!("{}", text);
                }
            } else {
            }
        }
        Ok(response_parts)
    }
}
