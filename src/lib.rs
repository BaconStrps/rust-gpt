#![allow(dead_code)]
use std::error::Error;

// this library is super unfinished
// and incredibly unpolished
// but it'll work for borf for now
use serde_json::json;
use async_trait::async_trait;
use serde::Serialize;
use once_cell::sync::OnceCell;

static RQCLIENT: OnceCell<reqwest::Client> = OnceCell::new();
static COMPLETION_URL: &'static str = "https://api.openai.com/v1/completions";
static CHAT_URL: &'static str = "https://api.openai.com/v1/chat/completions";

#[async_trait]
pub trait SendRequest {
    type Response;
    type Error;
    async fn send(self) -> Result<Self::Response, Self::Error>;
}

pub trait CompletionLike {}
pub struct Completion;
pub struct Chat;
#[derive(Debug)]
pub enum CompletionModel{
    TextDavinci003,
    TextDavinci002,
    CodeDavinci002,
}

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

#[derive(Debug)]
pub struct CompletionChoice {
    pub text: String,
    pub index: u32,
    pub logprobs: u32,
    pub finish_reason: String,
}

impl From<serde_json::Value> for CompletionChoice {
    fn from(value: serde_json::Value) -> Self {
        Self {
            text: value["text"].as_str().unwrap().to_string(),
            index: value["index"].as_u64().unwrap() as u32,
            logprobs: value["logprobs"].as_u64().unwrap_or_default() as u32,
            finish_reason: value["finish_reason"].as_str().unwrap_or_default().to_string(),
        }
    }
}

#[derive(Debug)]
pub struct CompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<CompletionChoice>,
}

#[derive(Debug)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub usage: (u32, u32, u32),
    pub messages: Vec<ChatMessage>,
}



#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl Into<serde_json::Value> for ChatMessage {
    fn into(self) -> serde_json::Value {
        json!({
            "role": self.role,
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
pub struct Request<T> {
    to_send: String,
    api_key: String,
    state: std::marker::PhantomData<T>,
}

#[async_trait]
impl SendRequest for Request<Completion> {
    type Response = CompletionResponse;
    type Error = reqwest::Error;
    async fn send(self) -> Result<Self::Response, reqwest::Error> {
        let client = RQCLIENT.get_or_init(|| reqwest::Client::new());

        let resp = client.post(COMPLETION_URL)
            .header("Content-Type", "application/json")
            .header("Authorization", self.api_key)
            .body(self.to_send)
            .send()
            .await?;

        let body = resp.text().await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        Ok(CompletionResponse {
            id: json["id"].as_str().unwrap().to_string(),
            object: json["object"].as_str().unwrap().to_string(),
            created: json["created"].as_u64().unwrap(),
            model: json["model"].as_str().unwrap().to_string(),
            choices: json["choices"].as_array().unwrap().iter().map(|choice| CompletionChoice::from(choice.clone())).collect()
        })
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
        Ok(ChatResponse {
            id: json["id"].as_str().unwrap().to_string(),
            object: json["object"].as_str().unwrap().to_string(),
            created: json["created"].as_u64().unwrap(),
            model: json["model"].as_str().unwrap().to_string(),
            usage: (
                json["usage"]["prompt_tokens"].as_u64().unwrap() as u32,
                json["usage"]["completion_tokens"].as_u64().unwrap() as u32,
                json["usage"]["total_tokens"].as_u64().unwrap() as u32,
            ),
            messages: json["choices"].as_array().unwrap().iter().map(|message| ChatMessage {
                role: message["message"]["role"].as_str().unwrap().to_string(),
                content: message["message"]["content"].as_str().unwrap().to_string(),
            }).collect()
        })
    }
}

// impl Request<Chat> {
//     pub async fn send_chat(self) -> Result<ChatResponse, reqwest::Error> {
//         let client = RQCLIENT.get_or_init(|| reqwest::Client::new());

//         let resp = client.post(CHAT_URL)
//             .header("Content-Type", "application/json")
//             .header("Authorization", self.api_key)
//             .body(self.to_send)
//             .send()
//             .await?;

//         let body = resp.text().await.unwrap();
//         let json: serde_json::Value = serde_json::from_str(&body).unwrap();
//         Ok(ChatResponse {
//             id: json["id"].as_str().unwrap().to_string(),
//             object: json["object"].as_str().unwrap().to_string(),
//             created: json["created"].as_u64().unwrap(),
//             model: json["model"].as_str().unwrap().to_string(),
//             usage: (
//                 json["usage"]["prompt_tokens"].as_u64().unwrap() as u32,
//                 json["usage"]["completion_tokens"].as_u64().unwrap() as u32,
//                 json["usage"]["total_tokens"].as_u64().unwrap() as u32,
//             ),
//             messages: json["choices"].as_array().unwrap().iter().map(|choice| ChatMessage {
//                 role: choice["message"]["role"].as_str().unwrap().to_string(),
//                 content: choice["message"]["content"].as_str().unwrap().to_string(),
//             }).collect()
//         })
//     }
// }


#[derive(Debug)]
pub struct RequestBuilder<T> {
    req: serde_json::Value,
    api_key: String,
    state: std::marker::PhantomData<T>,
}

impl<C: CompletionLike> RequestBuilder<C> {
    pub fn new<T: ToString>(model: T, mut api_key: String) -> Self {

        api_key.insert_str(0, "Bearer ");

        let req = json!({
            "model": model.to_string(),
        });

        Self {
            req,
            api_key,
            state: std::marker::PhantomData,
        }
    }

    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.req["max_tokens"] = json!(max_tokens);
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.req["temperature"] = json!(temperature);
        self
    }

    pub fn top_p(mut self, top_p: f32) -> Self {
        self.req["top_p"] = json!(top_p);
        self
    }

    pub fn frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.req["frequency_penalty"] = json!(frequency_penalty);
        self
    }

    pub fn presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.req["presence_penalty"] = json!(presence_penalty);
        self
    }

    pub fn stop<T: ToString>(mut self, stop: T) -> Self {
        self.req["stop"] = json!(stop.to_string());
        self
    }

    pub fn n(mut self, n: u32) -> Self {
        self.req["n"] = json!(n);
        self
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.req["stream"] = json!(stream);
        self
    }

    pub fn logprobs(mut self, logprobs: u32) -> Self {
        self.req["logprobs"] = json!(logprobs);
        self
    }

    pub fn echo(mut self, echo: bool) -> Self {
        self.req["echo"] = json!(echo);
        self
    }

    pub fn best_of(mut self, best_of: u32) -> Self {
        self.req["best_of"] = json!(best_of);
        self
    }

    pub fn logit_bias(mut self, logit_bias: serde_json::Value) -> Self {
        self.req["logit_bias"] = logit_bias;
        self
    }
}

impl RequestBuilder<Completion> {
    pub fn prompt<T: ToString>(mut self, prompt: T) -> Self {
        self.req["prompt"] = json!(prompt.to_string());
        self
    }

    pub fn build_completion(self) -> Request<Completion> {
        Request {
            api_key: self.api_key,
            to_send: self.req.to_string(),
            state: std::marker::PhantomData
        }
    }
}

impl RequestBuilder<Chat> {

    pub fn messages(mut self, messages: Vec<ChatMessage>) -> Self {
        self.req["messages"] = json!(messages);
        self
    }

    pub fn build_chat(self) -> Request<Chat> {
        Request {
            api_key: self.api_key,
            to_send: self.req.to_string(),
            state: std::marker::PhantomData
        }
    }
}
