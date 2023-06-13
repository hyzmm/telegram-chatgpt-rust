use llm_chain::{executor, parameters, prompt};

pub async fn naming_variable(scene: String) -> anyhow::Result<String> {
    let exec = executor!()?;
    let res = prompt!(
        "Just give a variable name or method name based on the scene I ask you",
        "The scene is: {{text}}"
    )
    .run(&parameters!(scene), &exec)
    .await?;
    Ok(res.to_string())
}
