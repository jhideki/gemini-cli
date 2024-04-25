use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Part {
    text: String,
}

#[derive(Serialize, Deserialize)]
struct RequestBody {
    contents: Vec<Content>,
}

#[derive(Serialize, Deserialize)]
struct SafetyRating {
    category: String,
    probability: String,
}

#[derive(Serialize, Deserialize)]
struct Candidate {
    content: Content,
    #[serde(rename = "finishedReason")]
    finished_reason: String,
    index: u32,
    #[serde(rename = "SafetyRatings")]
    safety_ratings: Vec<SafetyRating>,
}

pub struct Genai {
    api_key: String,
    model_name: String,
    end_point: String,
    message_thread: Vec<Content>,
}
impl Genai {
    pub fn new(api_key: String, model_name: &str) -> Self {
        Genai {
            api_key,
            model_name: model_name.to_string(),
            end_point: "https://generativelanguage.googleapis.com/v1beta/models/".to_string(),
            message_thread: Vec::new(),
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
        let client = Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;
        let json_response: Vec<Candidate> = response.json().await?;
        println!("{}", json_response[0].content.parts[0].text);
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
            "{}{}:generateContent?key={}",
            self.end_point, self.model_name, self.api_key
        );
        let request_body = RequestBody {
            contents: self.message_thread.clone(),
        };
        let client = Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;
        let json_response: Vec<Candidate> = response.json().await?;
        Ok(())
    }
}
