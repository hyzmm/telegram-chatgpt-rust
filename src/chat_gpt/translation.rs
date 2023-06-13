use llm_chain::{executor, parameters, prompt};

pub async fn translate(user_input: String) -> anyhow::Result<String> {
    let (lang, text) = get_lang_and_text(user_input);
    let exec = executor!()?;
    let res = prompt!("translate following text to {lang}", "{{text}}")
        .run(&parameters! {"lang"=>lang, "text"=>text}, &exec)
        .await?;

    Ok(res.to_string())
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
