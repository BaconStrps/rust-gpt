//! # Chat API
//! 
//! The chat API is used to have a conversation with the GPT-3.5 model which runs ChatGPT.  
//! 
//! The main structs used in here are [`ChatResponse`] and [`ChatMessage`].
use std::{error::Error, collections::VecDeque};
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize, ser::SerializeStruct};

use crate::SendRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents one of the messages sent to or received from the chat API.
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl Default for ChatMessage {
    fn default() -> Self {
        Self {
            role: Role::User,
            content: String::new(),
        }
    }
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
#[derive(Debug, Clone)]
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



// ----------------------------------------------------
// new unstable chat thing

pub struct ChatBuilder {
    system: ChatMessage,
    chat_parameters: ChatParameters,
    api_key: String,
    model: crate::ChatModel,
    len: usize,
}

impl ChatBuilder {

    pub fn new(model: crate::ChatModel, api_key: String) -> Self {

        let mut default_msg = ChatMessage::default();
        default_msg.role = Role::System;

        ChatBuilder {
            model,
            api_key,
            system: default_msg,
            chat_parameters: ChatParameters::default(),
            len: 5,
        }
    }

    pub fn len(mut self, len: usize) -> Self {
        self.len = len;
        self
    }

    pub fn system(mut self, system: ChatMessage) -> Self {
        self.system = system;
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.chat_parameters.temperature = temperature;
        self
    }

    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.chat_parameters.max_tokens = max_tokens;
        self
    }

    pub fn top_p(mut self, top_p: f32) -> Self {
        self.chat_parameters.top_p = top_p;
        self
    }

    pub fn presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.chat_parameters.presence_penalty = presence_penalty;
        self
    }

    pub fn frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.chat_parameters.frequency_penalty = frequency_penalty;
        self
    }

    pub fn user(mut self, user: String) -> Self {
        self.chat_parameters.user = Some(user);
        self
    }

    pub fn build(self) -> Chat {
        let chat = Chat::new(self.system, self.model, self.len, self.api_key, self.chat_parameters);
        chat
    }

}


#[derive(Debug, Clone, Deserialize)]
pub struct ChatParameters {
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: f32,
    pub presence_penalty: f32,
    pub frequency_penalty: f32,
    pub user: Option<String>,
}

impl Default for ChatParameters {
    fn default() -> Self {
        Self {
            temperature: 1.0,
            max_tokens: 4096,
            top_p: 1.0,
            presence_penalty: 0.0,
            frequency_penalty: 0.0,
            user: None,
        }
    }
}

impl Serialize for ChatParameters {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ChatParameters", 6)?;
        state.serialize_field("temperature", &self.temperature)?;
        state.serialize_field("max_tokens", &self.max_tokens)?;
        state.serialize_field("top_p", &self.top_p)?;
        state.serialize_field("presence_penalty", &self.presence_penalty)?;
        state.serialize_field("frequency_penalty", &self.frequency_penalty)?;
        state.serialize_field("user", &self.user)?;
        state.end()
    }
}


pub struct Chat {
    system: ChatMessage,
    chat_parameters: ChatParameters,
    api_key: String,
    model: crate::ChatModel,
    len: usize,
    messages: Mutex<VecDeque<ChatMessage>>,
    message_queue: Mutex<VecDeque<ChatMessage>>,
}


impl Chat {

    fn new<T: ToString>(system: ChatMessage, model: crate::ChatModel, len: usize, api_key: T, chat_parameters: ChatParameters) -> Self {
        Self {
            system,
            chat_parameters,
            api_key: api_key.to_string(),
            model,
            len,
            messages: Mutex::new(VecDeque::new()),
            message_queue: Mutex::new(VecDeque::new()),
        }
    }


    pub async fn get_messages(&self) -> Vec<ChatMessage> {

        let mut messages = self.messages.lock().await.clone();

        messages.push_front(self.system.clone());

        messages.into()
    }

    pub async fn ask(&self, message: &str) -> Result<(), Box<dyn Error>> {

        let msg = ChatMessage {
            role: Role::User,
            content: message.to_string(),
        };

        self.message_queue.lock().await.push_back(msg);
        Ok(())
    }

    pub async fn get_response(&self, user: Option<String>) -> Result<ChatMessage, Box<dyn Error>> {
        let msg = if let Some(message) = self.message_queue.lock().await.pop_front() {
            message
        } else {
            return Err("No message to send".into());
        };

        let mut messages = self.messages.lock().await;

        // * 2 because we don't count assistant messages
        // + 2 because we don't count the system message and the message we're about to send
        if (messages.len() + 2) * 2 >= self.len {
            messages.pop_front();
        }

        messages.push_back(msg.clone());

        let mut to_send = messages.clone();
        to_send.push_front(self.system.clone());

        let builder = crate::RequestBuilder::new(self.model.clone(), self.api_key.clone())
            .chat_parameters(self.chat_parameters.clone())
            .messages(to_send.into());

        let builder = if let Some(user) = user {
            builder.user(user)
        } else {
            builder
        };

        let req = builder.build_chat();

        let resp = req.send().await?;

        let message = resp.choices[0].message.clone();

        messages.push_back(message.clone());

        Ok(message)
    }
}


