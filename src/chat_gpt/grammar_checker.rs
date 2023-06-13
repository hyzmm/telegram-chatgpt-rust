use llm_chain::{executor, parameters, prompt};

use crate::chat_gpt::split_options_and_body;

pub async fn check_grammar(user_input: String) -> anyhow::Result<String> {
    let (lang, text) = split_options_and_body(user_input, "Chinese".to_string(), 'l');
    let exec = executor!()?;
    let res = prompt!(
        "You are a language teacher, diagnose grammar problems for me and explain them to me in {{lang}}.",
        "My input is: {{text}}"
    )
        .run(&parameters!{"lang" => lang, "text" => text}, &exec) // ...and run it
        .await?;
    Ok(res.to_string())
}
