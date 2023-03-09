//! # OpenAI Completion/Chat Rust API
//! Provides a neat and rusty way of interacting with the OpenAI Completion/Chat API.
//! You can find the documentation for the API [here](https://platform.openai.com/docs/api-reference/completions).
//! ## Example
//! ```rust no_run
//! use rust_gpt::RequestBuilder;
//! use rust_gpt::CompletionModel;
//! use rust_gpt::SendRequest;
//! 
//! #[tokio::main]
//! async fn main() {
//!     let req = RequestBuilder::new(CompletionModel::TextDavinci003, "YOUR_API_KEY")
//!         .prompt("Write a sonnet about a crab named Ferris in the style of Shakespeare.")
//!         .build_completion();
//!     let response = req.send().await.unwrap();
//!     println!("My bot replied with: \"{:?}\"", response);
//! }
//!```
//! 
//! ## General Usage
//! You will most likely just use the [`RequestBuilder`] to create a request. You can then use the [`SendRequest`] trait to send the request.
//! Right now only the completion and chat endpoints are supported.
//! These two endpoints require different parameters, so you will need to use the [`build_completion`] and [`build_chat`] methods respectively.  
//! 
//! [`RequestBuilder`] can take any type that implements [`ToString`] as the model input and any type that implements [`Display`] as the API key.
//! 
//! [`build_completion`]: ./struct.RequestBuilder.html#method.build_completion
//! [`build_chat`]: ./struct.RequestBuilder.html#method.build_chat
//! 
//! ## Completion
//! The completion endpoint requires a [`prompt`] parameter. You can set this with the [`prompt`] method which takes any type that implements [`ToString`].
//! 
//! [`prompt`]: ./struct.RequestBuilder.html#method.prompt
//! 
//! ## Chat
//! The chat endpoint is a little more complicated. It requires a [`messages`] parameter which is a list of messages.
//! These messages are represented by the [`ChatMessage`] struct. You can create a [`ChatMessage`] with the [`new`] method.
//! 
//! [`messages`]: ./struct.RequestBuilder.html#method.messages
//! [`new`]: ./struct.ChatMessage.html#method.new
//! 
//! 
//! 
//! ## Additional Notes
//! The API is still in development, so there may be some breaking changes in the future.  
//! The API is also not fully tested, so there may be some bugs.  
//! There is a little bit of error handling, but it is not very robust.  
//! [serde_json](https://docs.rs/serde_json/latest/serde_json/) is used to seralize and deserialize the responses and messages. Although since many are derived they may not match up with the exact API json responses.
//! 

#![allow(dead_code)]
use std::{error::Error, fmt::Display};

use serde_json::json;
use async_trait::async_trait;
use serde::{Serialize, Deserialize, ser::SerializeStruct};
use once_cell::sync::OnceCell;

static RQCLIENT: OnceCell<reqwest::Client> = OnceCell::new();
static COMPLETION_URL: &'static str = "https://api.openai.com/v1/completions";
static CHAT_URL: &'static str = "https://api.openai.com/v1/chat/completions";

#[async_trait]
/// A trait for abstracting sending requests between APIs.
pub trait SendRequest {
    /// The type of the response.
    type Response;
    /// The type of the error.
    type Error;
    /// Sends the request, returning whether or not there was an error with the response.
    async fn send(self) -> Result<Self::Response, Self::Error>;
}
#[doc(hidden)]
pub trait CompletionLike {}
#[doc(hidden)]
pub struct Completion;
#[doc(hidden)]
pub struct Chat;
#[derive(Debug)]
/// The current completion models.
pub enum CompletionModel{
    TextDavinci003,
    TextDavinci002,
    CodeDavinci002,
}
/// The current chat models.
pub enum ChatModel {
    Gpt35Turbo,
    GPT35Turbo0301,
}

impl CompletionLike for Completion {}
impl CompletionLike for Chat {}

impl ToString for CompletionModel {
    fn to_string(&self) -> String {
        match self {
            CompletionModel::TextDavinci003 => "text-davinci-003",
            CompletionModel::TextDavinci002 => "text-davinci-002",
            CompletionModel::CodeDavinci002 => "code-davinci-002",
        }.to_string()
    }
}

impl ToString for ChatModel {
    fn to_string(&self) -> String {
        match self {
            ChatModel::Gpt35Turbo => "gpt-3.5-turbo",
            ChatModel::GPT35Turbo0301 => "gpt-3.5-turbo-0301",
        }.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
/// Represents one of the choices returned by the completion API.
pub struct CompletionChoice {
    pub text: String,
    pub index: u32,
    pub logprobs: Option<u64>,
    pub finish_reason: String,
}

#[derive(Debug, Deserialize)]
/// Represents a response from the completion API.
pub struct CompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<CompletionChoice>,
}

impl Serialize for CompletionResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("CompletionResponse", 5)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("object", &self.object)?;
        state.serialize_field("created", &self.created)?;
        state.serialize_field("model", &self.model)?;
        state.serialize_field("choices", &self.choices)?;
        state.end()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatChoice {
    index: u32,
    message: ChatMessage,
    finish_reason: String,
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
#[derive(Debug, Serialize, Deserialize)]
/// Represents one of the messages sent to or received from the chat API.
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl Into<serde_json::Value> for ChatMessage {
    fn into(self) -> serde_json::Value {
        json!({
            "role": self.role.to_string(),
            "content": self.content
        })
    }
}

impl ToString for ChatMessage {
    fn to_string(&self) -> String {
        json!({
            "role": self.role,
            "content": self.content
        }).to_string()
    }
}

#[derive(Debug)]
/// A generic request which can be used to send requests to the OpenAI API.
pub struct Request<T> {
    to_send: String,
    api_key: String,
    state: std::marker::PhantomData<T>,
}

#[async_trait]
impl SendRequest for Request<Completion> {
    type Response = CompletionResponse;
    type Error = Box<dyn Error>;
    async fn send(self) -> Result<Self::Response, Box<dyn Error>> {
        let client = RQCLIENT.get_or_init(|| reqwest::Client::new());

        let resp = client.post(COMPLETION_URL)
            .header("Content-Type", "application/json")
            .header("Authorization", self.api_key)
            .body(self.to_send)
            .send()
            .await?;

        let body = resp.text().await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();

        if let Some(error) = json["error"].as_str() {
            return Err(error.into());
        }

        Ok(CompletionResponse::deserialize(json)?)
    }
    
}

#[async_trait]
impl SendRequest for Request<Chat> {
    type Response = ChatResponse;
    type Error = Box<dyn Error>;
    async fn send(self) -> Result<Self::Response, Box<dyn Error>> {

        if self.to_send.find("messages").is_none() {
            return Err("No messages in request.".into());
        }

        let client = RQCLIENT.get_or_init(|| reqwest::Client::new());

        let resp = client.post(CHAT_URL)
            .header("Content-Type", "application/json")
            .header("Authorization", self.api_key)
            .body(self.to_send)
            .send()
            .await?;

        let body = resp.text().await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();

        if let Some(error) = json["error"]["message"].as_str() {
            return Err(error.into());
        }

        Ok(ChatResponse::deserialize(json)?)

        // Ok(ChatResponse {
        //     id: json["id"].as_str().unwrap().to_string(),
        //     object: json["object"].as_str().unwrap().to_string(),
        //     created: json["created"].as_u64().unwrap(),
        //     model: json["model"].as_str().unwrap().to_string(),
        //     usage: (
        //         json["usage"]["prompt_tokens"].as_u64().unwrap() as u32,
        //         json["usage"]["completion_tokens"].as_u64().unwrap() as u32,
        //         json["usage"]["total_tokens"].as_u64().unwrap() as u32,
        //     ),
        //     choices: json["choices"].as_array().unwrap().iter().map(|message| ChatMessage {
        //         role: message["message"]["role"].as_str().unwrap().try_into().unwrap(),
        //         content: message["message"]["content"].as_str().unwrap().to_string(),
        //     }).collect()
        // })
    }
}

#[derive(Debug)]
/// A builder for creating requests to the OpenAI API.
pub struct RequestBuilder<T> {
    req: serde_json::Value,
    api_key: String,
    state: std::marker::PhantomData<T>,
}

impl<C: CompletionLike> RequestBuilder<C> {
    /// Create a new request builder.
    pub fn new<T: ToString, S: Display>(model: T, api_key: S) -> Self {

        let api_key = format!("Bearer {}", api_key);

        let req = json!({
            "model": model.to_string(),
        });

        Self {
            req,
            api_key,
            state: std::marker::PhantomData,
        }
    }
    /// Set the max_tokens parameter.
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.req["max_tokens"] = json!(max_tokens);
        self
    }
    /// Set the temperature parameter.
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.req["temperature"] = json!(temperature);
        self
    }
    /// Set the top_p parameter.
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.req["top_p"] = json!(top_p);
        self
    }
    /// Set the frequency_penalty parameter.
    pub fn frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.req["frequency_penalty"] = json!(frequency_penalty);
        self
    }
    /// Set the presence_penalty parameter.
    pub fn presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.req["presence_penalty"] = json!(presence_penalty);
        self
    }
    /// Set the stop parameter.
    pub fn stop<T: ToString>(mut self, stop: T) -> Self {
        self.req["stop"] = json!(stop.to_string());
        self
    }
    /// Set the n parameter.
    pub fn n(mut self, n: u32) -> Self {
        self.req["n"] = json!(n);
        self
    }
}

impl RequestBuilder<Completion> {
    /// Set the prompt parameter.
    pub fn prompt<T: ToString>(mut self, prompt: T) -> Self {
        self.req["prompt"] = json!(prompt.to_string());
        self
    }
    /// Builds a completion request.
    pub fn build_completion(self) -> Request<Completion> {
        Request {
            api_key: self.api_key,
            to_send: self.req.to_string(),
            state: std::marker::PhantomData
        }
    }
}

impl RequestBuilder<Chat> {
    /// Set the messages parameter.
    pub fn messages(mut self, messages: Vec<ChatMessage>) -> Self {
        self.req["messages"] = json!(messages);
        self
    }
    /// Builds a chat request.
    pub fn build_chat(self) -> Request<Chat> {
        Request {
            api_key: self.api_key,
            to_send: self.req.to_string(),
            state: std::marker::PhantomData
        }
    }
}
