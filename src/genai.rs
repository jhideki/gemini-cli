use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    content: Content,
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
        let mut parts = Vec::new();
        parts.push(part);
        self.message_thread.push(Content {
            role: "user".to_string(),
            parts,
        });

        let url = format!(
            "{}{}:streamGenerateContent?alt=sse&key={}",
            self.end_point, self.model_name, self.api_key
        );

        let request_body = RequestBody {
            contents: self.message_thread.clone(),
        };

        let mut res = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        while let Some(chunk) = res.chunk().await? {
            let chunk = serde_json::Deserializer::from_slice(&chunk).into_iter::<Value>();
            for val in chunk {
                println!("{}", val.unwrap());
            }
        }
        Ok(())
    }
}
