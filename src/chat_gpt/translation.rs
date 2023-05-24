use openai_chatgpt_api::ChatGptChatFormat;

use crate::chat_gpt::ask_chat_gpt;

pub async fn translate(open_api_token: &str, user_input: String) -> anyhow::Result<String> {
    let (lang, text) = get_lang_and_text(user_input);
    let conversation_history: Vec<ChatGptChatFormat> = vec![
        ChatGptChatFormat::new_system(&format!("translate input text to {lang}")),
        ChatGptChatFormat::new_user(&text),
    ];

    ask_chat_gpt(open_api_token, conversation_history).await
}

fn get_lang_and_text(user_input: String) -> (String, String) {
    let default_lang = "english".to_string();
    let parts = user_input.splitn(2, ' ').collect::<Vec<&str>>();
    if parts.len() == 1 {
        return (default_lang, parts[0].to_string());
    }

    let lang = parts[0];
    if let Some(lang) = lang.strip_prefix("-l") {
        if lang.is_empty() {
            return (default_lang, parts[1].to_string());
        }
        return (lang.to_string(), parts[1].to_string());
    }

    (default_lang, user_input)
}
