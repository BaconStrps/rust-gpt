//! # Chat API
//! 
//! The chat API is used to have a conversation with the GPT-3.5 model which runs ChatGPT.  
//! 
//! The main structs used in here are [`ChatResponse`] and [`ChatMessage`].
use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
/// Represents one of the messages sent to or received from the chat API.
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
/// Represents the usage information returned by the chat API.
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize, Serialize)]
/// Represents the choice object returned by the chat API.
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Deserialize, Serialize)]
/// Represents a response from the chat API.
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub choices: Vec<ChatChoice>,
    pub usage: Usage,
}
#[derive(Debug)]
/// Represents one of the roles that can be used in the chat API.
pub enum Role {
    User,
    Assistant,
    System,
}

impl Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Role {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;

        Role::try_from(s.as_str()).map_err(serde::de::Error::custom)
    }
}

impl ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::System => "system",
        }.to_string()
    }
}

impl TryFrom<&str> for Role {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "user" => Ok(Role::User),
            "assistant" => Ok(Role::Assistant),
            "system" => Ok(Role::System),
            _ => Err("Invalid Role".into()),
        }
    }
}