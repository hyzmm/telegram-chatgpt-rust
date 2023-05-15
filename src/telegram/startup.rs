use std::sync::Arc;

use openai_chatgpt_api::ChatGptChatFormat;
use teloxide::types::ParseMode;
use teloxide::utils::command::ParseError;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::sync::Mutex;

use crate::chat_gpt::ask_chat_gpt;
use crate::storages::{self};
use crate::storages::{Role, Roles};
use crate::utils::telegram_utils::escape_markdown_v2_reversed_chars;

fn split_role_name_and_system(input: String) -> Result<(String, String), ParseError> {
    let parts = input.splitn(2, ':').collect::<Vec<&str>>();
    if parts.len() < 2 {
        return Err(ParseError::TooFewArguments {
            expected: 2,
            found: parts.len(),
            message: "Expected format: <role_name>:<system>".to_string(),
        });
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}
#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "just for test")]
    Test,
    #[command(description = "clear conversation history and start a new session")]
    Clear,
    #[command(description = "list all roles")]
    ListRoles,
    #[command(parse_with = split_role_name_and_system, description = "add a role")]
    NewRole { role_name: String, system: String },
    #[command(description = "delete a role")]
    DeleteRole { role_name: String },
    #[command(description = "switch role")]
    SwitchRole { role_name: String },
}

type ConversationHistoryRef = Arc<Mutex<Vec<ChatGptChatFormat>>>;
type RolesRef = Arc<Mutex<Roles>>;

#[derive(Clone)]
struct Settings {
    open_ai_api_key: String,
}

impl Settings {
    fn from_env() -> Settings {
        Settings {
            open_ai_api_key: std::env::var("OPEN_AI_API_KEY").expect("OPEN_AI_API_KEY must be set"),
        }
    }
}

pub fn get_default_role(roles: &Roles) -> (&str, &str) {
    let system;
    let _role;
    if let Some((role_name, role)) = roles.iter().next() {
        system = role.system.as_str();
        _role = role_name.as_str();
    } else {
        system = "you are a helpful assistant.";
        _role = "assistant";
    }
    (_role, system)
}

pub async fn startup() -> Result<(), anyhow::Error> {
    let settings = Settings::from_env();
    let bot = Bot::from_env();

    let saved_roles = storages::get_roles()?;

    let ignore_update = |_upd| Box::pin(async {});

    let (current_role, default_system) = get_default_role(&saved_roles);

    let chat_gpt_system = ChatGptChatFormat::new_system(default_system);
    let conversation_history = Arc::new(Mutex::new(vec![chat_gpt_system]));
    let current_role = Arc::new(Mutex::new(current_role.to_string()));

    let saved_roles_ref = Arc::new(Mutex::new(saved_roles));

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(answer_command),
        )
        .endpoint(handle_message);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            conversation_history,
            settings,
            saved_roles_ref,
            current_role
        ])
        .default_handler(ignore_update)
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

async fn answer_command(
    bot: Bot,
    msg: Message,
    conversation_history: ConversationHistoryRef,
    command: Command,
    roles: RolesRef,
    current_role: Arc<Mutex<String>>,
) -> ResponseResult<()> {
    match command {
        Command::Test => {
            bot.send_message(msg.chat.id, r#"
*bold \*text*
_italic \*text_
__underline__
~strikethrough~
||spoiler||
*bold _italic bold ~italic bold strikethrough ||italic bold strikethrough spoiler||~ __underline italic bold___ bold*
[inline URL](http://www.example.com/)
[inline mention of a user](tg://user?id=123456789)
![👍](tg://emoji?id=5368324170671202286)
`inline fixed-width code`
```
pre-formatted fixed-width code block
```
```python
pre-formatted fixed-width code block written in the Python programming language
```"#).parse_mode(ParseMode::MarkdownV2).await?;
        }
        Command::Clear => {
            let mut conversation_history = conversation_history.lock().await;
            conversation_history.truncate(1);
            bot.send_message(
                msg.chat.id,
                "Conversation history cleared, new session started.",
            )
            .await?;
        }
        Command::ListRoles => {
            let roles = roles.lock().await;
            let current_role = current_role.lock().await;

            let mut output = String::new();
            for (index, (name, role)) in roles.iter().enumerate() {
                let current_sign = if &*current_role == name { "__" } else { "" };
                output.push_str(&escape_markdown_v2_reversed_chars(&format!(
                    "{}. {current_sign}*{}*: {}{current_sign}\n",
                    index + 1,
                    name,
                    role.system,
                )));
            }
            bot.send_message(msg.chat.id, output)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
        }
        Command::NewRole { role_name, system } => {
            new_role(&bot, &msg, &roles, &role_name, system)
                .await
                .expect("Failed to create new role");
        }
        Command::DeleteRole { role_name } => {
            delete_role(bot, msg, roles, &role_name)
                .await
                .expect("Failed to delete role");
        }
        Command::SwitchRole { role_name } => {
            switch_role(
                bot,
                msg,
                conversation_history,
                roles,
                current_role,
                &role_name,
            )
            .await
            .expect("Failed to switch role");
        }
    }
    Ok(())
}

async fn switch_role(
    bot: Bot,
    msg: Message,
    conversation_history: ConversationHistoryRef,
    roles: RolesRef,
    current_role: Arc<Mutex<String>>,
    role_name: &String,
) -> Result<(), anyhow::Error> {
    let mut conversation_history = conversation_history.lock().await;
    conversation_history.clear();

    let roles = roles.lock().await;
    if let Some(role) = roles.get(role_name) {
        let mut current_role = current_role.lock().await;
        *current_role = role_name.clone();

        conversation_history.push(ChatGptChatFormat::new_system(&role.system));
        bot.send_message(
            msg.chat.id,
            escape_markdown_v2_reversed_chars(&format!("Switched to role *{role_name}*.")),
        )
        .parse_mode(ParseMode::MarkdownV2)
        .await?;
    } else {
        bot.send_message(
            msg.chat.id,
            escape_markdown_v2_reversed_chars(&format!("Role *{role_name}* not found.")),
        )
        .parse_mode(ParseMode::MarkdownV2)
        .await?;
    }
    Ok(())
}

async fn delete_role(
    bot: Bot,
    msg: Message,
    roles: RolesRef,
    role_name: &String,
) -> Result<(), anyhow::Error> {
    let mut roles = roles.lock().await;
    if roles.remove(role_name).is_some() {
        storages::rewrite_file(&roles).expect("Failed to write roles to file");
        bot.send_message(
            msg.chat.id,
            escape_markdown_v2_reversed_chars(&format!("Role *{role_name}* deleted successfully.")),
        )
    } else {
        bot.send_message(msg.chat.id, format!("Role *{role_name}* not found."))
    }
    .parse_mode(ParseMode::MarkdownV2)
    .await?;
    Ok(())
}

async fn new_role(
    bot: &Bot,
    msg: &Message,
    roles: &RolesRef,
    role_name: &String,
    system: String,
) -> Result<(), anyhow::Error> {
    let mut roles = roles.lock().await;
    roles.insert(role_name.clone(), Role { system });
    storages::rewrite_file(&roles).expect("Failed to write roles to file");
    bot.send_message(
        msg.chat.id,
        escape_markdown_v2_reversed_chars(&format!(
            "Role *{role_name}* added successfully. And now I'm {role_name}"
        )),
    )
    .parse_mode(ParseMode::MarkdownV2)
    .await?;
    Ok(())
}

async fn handle_message(
    bot: Bot,
    msg: Message,
    conversation_history: ConversationHistoryRef,
    settings: Settings,
    // cmd: Command,
) -> ResponseResult<()> {
    if let Some(question) = msg.text() {
        let mut conversation_history = conversation_history.lock().await;
        conversation_history.push(ChatGptChatFormat::new_user(question));
        // println!("{:?}", conversation_history);
        bot.send_chat_action(msg.chat.id, teloxide::types::ChatAction::Typing)
            .await?;
        if let Ok(answer) = ask_chat_gpt(
            settings.open_ai_api_key.as_str(),
            conversation_history.clone(),
        )
        .await
        {
            conversation_history.push(ChatGptChatFormat::new_assistant(&answer));
            bot.send_message(msg.chat.id, escape_markdown_v2_reversed_chars(&answer))
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
        }
    }
    Ok(())
}
