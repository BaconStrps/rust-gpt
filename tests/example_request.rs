use rust_gpt::{chat::*, *};

#[test]
fn example_chat_request() {
    let req = RequestBuilder::new(
        ChatModel::Gpt35Turbo,
        std::env::var("OPENAI_API_KEY").unwrap(),
    )
    .messages(vec![
        ChatMessage {
            role: Role::System,
            content: "You are a helpful assistant.".to_string().into(),
        },
        ChatMessage {
            role: Role::User,
            content: "Who started World War 2?".to_string().into(),
        },
    ])
    .max_tokens(128)
    .build_chat();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let _resphandle = rt.block_on(req.send()).unwrap();

    // println!("Chat: {:?}", resphandle);
}

#[test]
fn example_completion_request() {
    let req = RequestBuilder::new(
        CompletionModel::TextDavinci003,
        std::env::var("OPENAI_API_KEY").unwrap(),
    )
    .prompt("Once upon a time, there was a")
    .max_tokens(128)
    .build_completion();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let _resphandle = rt.block_on(req.send()).unwrap();

    // println!("Completion: {:?}", resphandle);
}

#[test]
fn chat_experimental_test() {
    let chat = ChatBuilder::new(
        ChatModel::Gpt35Turbo,
        std::env::var("OPENAI_API_KEY").unwrap(),
    )
    .max_tokens(128)
    .system(ChatMessage {
        role: Role::System,
        content: "You are a dog with an incredible amount of trivia knowledge".to_string().into(),
    })
    .build();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(chat.ask("Give me some crab facts")).unwrap();

    let _response = rt.block_on(chat.get_response(None)).unwrap();
    let messages = rt.block_on(chat.get_messages());

    println!("Messages 1: \n{messages:?}");

    rt.block_on(chat.ask("Can you rephrase what you just told me?"))
        .unwrap();

    let _response = rt.block_on(chat.get_response(None)).unwrap();
    let messages = rt.block_on(chat.get_messages());

    println!("Messages 2: \n{messages:?}");
}
