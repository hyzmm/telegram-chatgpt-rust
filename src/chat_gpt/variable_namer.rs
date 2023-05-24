use openai_chatgpt_api::ChatGptChatFormat;

use crate::chat_gpt::ask_chat_gpt;

pub async fn naming_variable(open_api_token: &str, scene: String) -> anyhow::Result<String> {
    let conversation_history: Vec<ChatGptChatFormat> = vec![
        ChatGptChatFormat::new_system(
            "Just give a variable name or method name based on the scene I ask you",
        ),
        ChatGptChatFormat::new_user(&scene),
    ];

    ask_chat_gpt(open_api_token, conversation_history).await
}
