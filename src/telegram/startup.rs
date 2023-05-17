use std::sync::Arc;

use log::info;
use openai_chatgpt_api::ChatGptChatFormat;
use teloxide::types::ParseMode;
use teloxide::utils::command::ParseError;
use teloxide::{payloads::SendMessageSetters, prelude::*, utils::command::BotCommands};
use tokio::sync::Mutex;

use crate::chat_gpt::ask_chat_gpt;
use crate::storages;
use crate::storages::{Role, Roles};
use crate::telegram::message_helper::send_roles_using_inline_keyboard;
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
    DeleteRole,
    #[command(description = "switch role")]
    SwitchRole,
}

type ConversationHistoryRef = Arc<Mutex<Vec<ChatGptChatFormat>>>;
pub type RolesRef = Arc<Mutex<Roles>>;

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

    let handler = dptree::entry()
        .branch(
            Update::filter_message().branch(
                dptree::entry()
                    .filter_command::<Command>()
                    .endpoint(command_handler),
            ),
        )
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

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

async fn switch_role(bot: Bot, msg: Message, roles: RolesRef) -> Result<(), anyhow::Error> {
    send_roles_using_inline_keyboard(bot, msg, roles, "Choose a role from the list below:").await?;
    Ok(())
}
async fn do_switch_role(
    bot: Bot,
    msg: Message,
    conversation_history: ConversationHistoryRef,
    roles: RolesRef,
    current_role: Arc<Mutex<String>>,
    role_name: &String,
) -> Result<(), anyhow::Error> {
    let roles = roles.lock().await;
    if let Some(role) = roles.get(role_name) {
        let mut conversation_history = conversation_history.lock().await;
        conversation_history.clear();

        let mut current_role = current_role.lock().await;
        if &*current_role == role_name {
            bot.edit_message_text(msg.chat.id, msg.id, "I'm already this role.")
                .await?;
        } else {
            *current_role = role_name.clone();

            conversation_history.push(ChatGptChatFormat::new_system(&role.system));
            bot.edit_message_text(
                msg.chat.id,
                msg.id,
                escape_markdown_v2_reversed_chars(&format!("Switched to role *{role_name}*.")),
            )
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        }
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
    send_roles_using_inline_keyboard(bot, msg, roles, "Choose a role to delete:").await?;
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

async fn command_handler(
    bot: Bot,
    msg: Message,
    conversation_history: ConversationHistoryRef,
    current_role: Arc<Mutex<String>>,
    roles: RolesRef, // cmd: Command,
    command: Command,
) -> ResponseResult<()> {
    match command {
        Command::NewRole { role_name, system } => {
            new_role(&bot, &msg, &roles, &role_name, system)
                .await
                .unwrap();
        }
        Command::DeleteRole => {
            delete_role(bot, msg, roles).await.unwrap();
        }
        Command::SwitchRole => {
            switch_role(bot, msg, roles).await.unwrap();
        }
        Command::Test => {
            just_for_test(&bot, &msg).await.unwrap();
        }
        Command::ListRoles => {
            list_roles(&bot, &msg, roles, current_role).await.unwrap();
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
    }

    Ok(())
}
async fn message_handler(
    bot: Bot,
    msg: Message,
    conversation_history: ConversationHistoryRef,
    settings: Settings,
) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        let mut conversation_history = conversation_history.lock().await;
        conversation_history.push(ChatGptChatFormat::new_user(text));
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

async fn list_roles(
    bot: &Bot,
    msg: &Message,
    roles: RolesRef,
    current_role: Arc<Mutex<String>>,
) -> Result<(), anyhow::Error> {
    let roles = roles.lock().await;
    let current_role = current_role.lock().await;
    let roles_list = roles
        .iter()
        .enumerate()
        .map(|(index, (name, role))| {
            format!(
                "{}. {underline}*{name}*: {}{underline}",
                index + 1,
                role.system,
                underline = if name == &*current_role { "__" } else { "" }
            )
        })
        .collect::<Vec<String>>();
    bot.send_message(
        msg.chat.id,
        escape_markdown_v2_reversed_chars(&format!("Roles:\n{}", roles_list.join("\n"))),
    )
    .parse_mode(ParseMode::MarkdownV2)
    .await?;
    Ok(())
}

async fn just_for_test(bot: &Bot, msg: &Message) -> Result<(), anyhow::Error> {
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
    Ok(())
}

async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    conversation_history: ConversationHistoryRef,
    roles: RolesRef,
    current_role: Arc<Mutex<String>>,
) -> ResponseResult<()> {
    if let Some(new_role) = q.data {
        bot.answer_callback_query(q.id).await?;
        if q.message.is_none() {
            info!("No message in callback query");
            return Ok(());
        }
        do_switch_role(
            bot,
            q.message.unwrap(),
            conversation_history,
            roles,
            current_role,
            &new_role,
        )
        .await
        .unwrap();
    }

    Ok(())
}
