
use rust_gpt::*;

#[test]
fn example_chat_request() {
    let req = RequestBuilder::new(ChatModel::Gpt35Turbo, std::env::var("OPENAI_API_KEY").unwrap())
        .messages(vec![ChatMessage{
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string()
        }, ChatMessage{
            role: "user".to_string(),
            content: "Who started World War 2?".to_string()
        }])
        .max_tokens(128)
        .build_chat();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let resphandle = rt.block_on(req.send()).unwrap();

    println!("Chat: {:?}", resphandle);
}


#[test]
fn example_completion_request() {
    let req = RequestBuilder::new(CompletionModel::TextDavinci003, std::env::var("OPENAI_API_KEY").unwrap())
        .prompt("Once upon a time, there was a")
        .max_tokens(128)
        .build_completion();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let resphandle = rt.block_on(req.send()).unwrap();

    println!("Completion: {:?}", resphandle);
}