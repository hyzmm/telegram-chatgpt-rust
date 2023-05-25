use anyhow::Context;
use log::info;
use openai_chatgpt_api::{
    ChatGpt, ChatGptChatFormat, ChatGptRequestChatCompletions, ChatGptResponse,
};

pub use grammar_checker::check_grammar;
pub use translation::translate;
pub use variable_namer::naming_variable;

mod grammar_checker;
mod translation;
mod variable_namer;

pub async fn ask_chat_gpt(
    open_api_token: &str,
    conversation_history: Vec<ChatGptChatFormat>,
) -> anyhow::Result<String> {
    let chat_gpt = ChatGpt::new(open_api_token);

    let request = ChatGptRequestChatCompletions::new("gpt-4", conversation_history);

    let res = chat_gpt
        .chat_completions(&request)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let choices = res.to_value().get("choices").context("No choices")?;
    let first_choice = choices.get(0).context("No first choice")?;
    let message = first_choice.get("message").context("No message")?;
    let content = message
        .get("content")
        .and_then(|e| e.as_str())
        .context("No content")?;
    info!("ChatGPT response: {}", content);
    Ok(content.to_string())
}

fn split_options_and_body(
    user_input: String,
    default: String,
    option_flag: char,
) -> (String, String) {
    let parts = user_input.splitn(2, ' ').collect::<Vec<&str>>();
    if parts.len() == 1 {
        return (default, parts[0].to_string());
    }

    let lang = parts[0];
    if let Some(lang) = lang.strip_prefix(&format!("-{option_flag}")) {
        if lang.is_empty() {
            return (default, parts[1].to_string());
        }
        return (lang.to_string(), parts[1].to_string());
    }

    (default, user_input)
}
