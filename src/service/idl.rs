use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(default)]
pub struct ErrorResponse {
    pub code: i32,
    pub message: &'static str
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct RegisterResponse {
    pub success: bool
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct LoginRequest {
    pub username: String,
    pub password: String
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct LoginResponse {
    pub success: bool,
    pub token: String
}

/// openai platform chat completion message struct
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct GptMessage {
    pub role: String,
    pub content: String
}

/// OpenAI Platform Chat Completion Request
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<GptMessage>,
    pub stream: bool,
    pub temperature: f32
}

impl ChatCompletionRequest {
    /// Create a chat completion request
    pub fn new(model: &str, messages: Vec<GptMessage>) -> Self {
        ChatCompletionRequest { model: model.to_string(), 
            messages,
            stream: true, 
            temperature: 0.5
        }
    }
}
