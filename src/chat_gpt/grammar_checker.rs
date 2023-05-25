use openai_chatgpt_api::ChatGptChatFormat;

use crate::chat_gpt::{ask_chat_gpt, split_options_and_body};

pub async fn check_grammar(open_api_token: &str, user_input: String) -> anyhow::Result<String> {
    let (lang, text) = split_options_and_body(user_input, "Chinese".to_string(), 'l');
    let conversation_history: Vec<ChatGptChatFormat> = vec![
        ChatGptChatFormat::new_system(&format!("You are a language teacher, diagnose grammar problems for me and explain them to me in {lang}.

")),
        ChatGptChatFormat::new_user(&text),
    ];

    ask_chat_gpt(open_api_token, conversation_history).await
}
