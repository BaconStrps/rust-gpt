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

use async_trait::async_trait;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use serde_json::json;

pub mod chat;
pub mod completion;

static RQCLIENT: OnceCell<reqwest::Client> = OnceCell::new();
static COMPLETION_URL: &str = "https://api.openai.com/v1/completions";
static CHAT_URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Debug, Clone)]
pub struct JsonParseError {
    json_string: String,
}

#[derive(Debug)]
pub enum SendRequestError {
    ReqwestError(reqwest::Error),
    OpenAiError(String),
    JsonError(JsonParseError),
}

impl Display for SendRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SendRequestError::ReqwestError(e) => write!(f, "Reqwest error: {}", e),
            SendRequestError::OpenAiError(e) => write!(f, "OpenAI error: {}", e),
            SendRequestError::JsonError(e) => write!(f, "Json error: {}", e),
        }
    }
}

impl Display for JsonParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not parse json: {}", self.json_string)
    }
}

impl Error for SendRequestError {}

impl From<reqwest::Error> for SendRequestError {
    fn from(e: reqwest::Error) -> Self {
        SendRequestError::ReqwestError(e)
    }
}

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
pub struct CompletionState;
#[doc(hidden)]
pub struct ChatState;
#[derive(Debug, Clone)]
/// The current completion models.
pub enum CompletionModel {
    TextDavinci003,
    TextDavinci002,
    CodeDavinci002,
}
#[derive(Debug, Clone)]
/// The current chat models.
pub enum ChatModel {
    Gpt35Turbo,
    GPT35Turbo0301,
}

impl CompletionLike for CompletionState {}
impl CompletionLike for ChatState {}

impl ToString for CompletionModel {
    fn to_string(&self) -> String {
        match self {
            CompletionModel::TextDavinci003 => "text-davinci-003",
            CompletionModel::TextDavinci002 => "text-davinci-002",
            CompletionModel::CodeDavinci002 => "code-davinci-002",
        }
        .to_string()
    }
}

impl ToString for ChatModel {
    fn to_string(&self) -> String {
        match self {
            ChatModel::Gpt35Turbo => "gpt-3.5-turbo",
            ChatModel::GPT35Turbo0301 => "gpt-3.5-turbo-0301",
        }
        .to_string()
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
impl SendRequest for Request<CompletionState> {
    type Response = completion::CompletionResponse;
    type Error = SendRequestError;
    async fn send(self) -> Result<Self::Response, Self::Error> {
        use SendRequestError::*;
        let client = RQCLIENT.get_or_init(reqwest::Client::new);

        let resp = client
            .post(COMPLETION_URL)
            .header("Content-Type", "application/json")
            .header("Authorization", self.api_key)
            .body(self.to_send)
            .send()
            .await?;

        let body = resp.text().await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();

        let response = match completion::CompletionResponse::deserialize(json.clone()) {
            Ok(r) => r,
            Err(_) => return Err(JsonError(JsonParseError { json_string: serde_json::to_string_pretty(&json).unwrap() })),
        };

        Ok(response)
    }
}

#[async_trait]
impl SendRequest for Request<ChatState> {
    type Response = chat::ChatResponse;
    type Error = SendRequestError;

    
    async fn send(self) -> Result<Self::Response, SendRequestError> {
        use SendRequestError::*;

        if !self.to_send.contains("messages") {
            return Err(OpenAiError("No messages in request.".into()));
        }

        let client = RQCLIENT.get_or_init(reqwest::Client::new);

        let resp = client
            .post(CHAT_URL)
            .header("Content-Type", "application/json")
            .header("Authorization", self.api_key)
            .body(self.to_send)
            .send()
            .await?;

        let body = resp.text().await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();

        if !json["error"].is_null() {
            return Err(OpenAiError(serde_json::to_string_pretty(&json).unwrap().into()));
        }

        let response = match chat::ChatResponse::deserialize(json.clone()) {
            Ok(r) => r,
            Err(_) => return Err(JsonError(JsonParseError { json_string: serde_json::to_string_pretty(&json).unwrap() })),
        };

        Ok(response)

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
        let api_key = format!("Bearer {api_key}");

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

    pub fn user(mut self, user: String) -> Self {
        self.req["user"] = json!(user);
        self
    }
}

impl RequestBuilder<CompletionState> {
    /// Set the prompt parameter.
    pub fn prompt<T: ToString>(mut self, prompt: T) -> Self {
        self.req["prompt"] = json!(prompt.to_string());
        self
    }
    /// Builds a completion request.
    pub fn build_completion(self) -> Request<CompletionState> {
        Request {
            api_key: self.api_key,
            to_send: self.req.to_string(),
            state: std::marker::PhantomData,
        }
    }
}

impl RequestBuilder<ChatState> {
    /// Set the messages parameter.
    pub fn messages(mut self, messages: Vec<chat::ChatMessage>) -> Self {
        self.req["messages"] = json!(messages);
        self
    }

    fn chat_parameters(mut self, chat_parameters: chat::ChatParameters) -> Self {
        let mut params = json!(chat_parameters);
        params["messages"] = self.req.get("messages").unwrap().clone();
        params["model"] = self.req.get("model").unwrap().clone();
        self.req = params;
        self
    }

    /// Builds a chat request.
    pub fn build_chat(self) -> Request<ChatState> {
        Request {
            api_key: self.api_key,
            to_send: self.req.to_string(),
            state: std::marker::PhantomData,
        }
    }
}
