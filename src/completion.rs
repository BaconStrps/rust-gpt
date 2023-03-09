//! # Completion API.
//! 
//! Includes the structs that represent a response from the Completion API.

use serde::{Deserialize, Serialize, ser::SerializeStruct};


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