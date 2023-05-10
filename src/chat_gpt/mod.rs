use anyhow::Context;
use openai_chatgpt_api::{
    ChatGpt, ChatGptChatFormat, ChatGptRequestChatCompletions, ChatGptResponse,
};

pub async fn ask_chat_gpt(conversation_history: Vec<ChatGptChatFormat>) -> anyhow::Result<String> {
    let chat_gpt = ChatGpt::new("OPEN_AI_TOKEN");

    let request = ChatGptRequestChatCompletions::new("gpt-3.5-turbo", conversation_history);

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
    Ok(content.to_string())
}