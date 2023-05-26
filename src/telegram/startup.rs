use std::sync::Arc;

use log::info;
use openai_chatgpt_api::ChatGptChatFormat;
use serde::{Deserialize, Serialize};
use teloxide::dispatching::dialogue;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dptree::case;
use teloxide::types::ParseMode;
use teloxide::{payloads::SendMessageSetters, prelude::*, utils::command::BotCommands};
use tokio::sync::Mutex;

use crate::chat_gpt::ask_chat_gpt;
use crate::storages::{Role, Roles};
use crate::telegram::message_helper::send_roles_using_inline_keyboard;
use crate::utils::telegram_utils::escape_markdown_v2_reversed_chars;
use crate::{chat_gpt, storages};

type NewRoleDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum State {
    #[default]
    None,
    ReceiveNewRoleName,
    ReceiveNewRoleSystem {
        role_name: String,
    },
}

#[derive(BotCommands, Clone, Serialize, Deserialize)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "Just for test")]
    Test,
    #[command(description = "Clear conversation history and start a new session")]
    Clear,
    #[command(description = "List all roles")]
    ListRoles,
    #[command(description = "Add a role")]
    NewRole,
    #[command(description = "Delete a role")]
    DeleteRole,
    #[command(description = "Switch to another role")]
    SwitchRole,
    #[command(
        rename = "trans",
        description = "Translate given text to specify language"
    )]
    Translate(String),
    #[command(
        rename = "naming",
        description = "Generate variable names based on the scene you described"
    )]
    VariableNamer(String),
    #[command(
        rename = "gramcheck",
        description = "Check the grammar of the sentence and provide suggestions for improvement."
    )]
    CheckGrammar(String),
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
    bot.set_my_commands(Command::bot_commands()).await?;

    let saved_roles = storages::get_roles()?;

    let ignore_update = |_upd| Box::pin(async {});

    let (current_role, default_system) = get_default_role(&saved_roles);

    let chat_gpt_system = ChatGptChatFormat::new_system(default_system);
    let conversation_history = Arc::new(Mutex::new(vec![chat_gpt_system]));
    let current_role = Arc::new(Mutex::new(current_role.to_string()));

    let saved_roles_ref = Arc::new(Mutex::new(saved_roles));

    let handler = dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(
            Update::filter_message()
                .branch(case![State::ReceiveNewRoleName].endpoint(receive_new_role_name))
                .branch(
                    case![State::ReceiveNewRoleSystem { role_name }]
                        .endpoint(receive_new_role_system),
                )
                .branch(
                    dptree::entry()
                        .filter_command::<Command>()
                        .branch(dptree::endpoint(command_handler)),
                )
                .branch(dptree::endpoint(message_handler)),
        )
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            conversation_history,
            settings,
            saved_roles_ref,
            current_role,
            InMemStorage::<State>::new()
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
    send_roles_using_inline_keyboard(
        bot,
        msg,
        roles,
        "Choose a role from the list below:",
        Command::SwitchRole,
    )
    .await?;
    Ok(())
}

async fn do_switch_role(
    bot: Bot,
    msg: Message,
    conversation_history: ConversationHistoryRef,
    roles: RolesRef,
    current_role: Arc<Mutex<String>>,
    role_name: &str,
) -> Result<(), anyhow::Error> {
    let roles = roles.lock().await;
    if let Some(role) = roles.get(role_name) {
        let mut conversation_history = conversation_history.lock().await;
        conversation_history.clear();

        let mut current_role = current_role.lock().await;
        if *current_role == role_name {
            bot.edit_message_text(msg.chat.id, msg.id, "I'm already this role.")
                .await?;
        } else {
            *current_role = role_name.to_string();

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

async fn delete_role(bot: Bot, msg: Message, roles: RolesRef) -> Result<(), anyhow::Error> {
    send_roles_using_inline_keyboard(
        bot,
        msg,
        roles,
        "Choose a role to delete:",
        Command::DeleteRole,
    )
    .await?;
    Ok(())
}

async fn do_delete_role(
    bot: Bot,
    msg: Message,
    roles: RolesRef,
    role_name: &str,
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

async fn start_new_role_dialogue(
    bot: Bot,
    msg: Message,
    dialogue: NewRoleDialogue,
) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Let's start creating a role. Please tell me what is the name of the role?",
    )
    .await?;
    dialogue.update(State::ReceiveNewRoleName).await?;
    Ok(())
}

async fn receive_new_role_name(bot: Bot, msg: Message, dialogue: NewRoleDialogue) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(role_name) => {
            bot.send_message(
                msg.chat.id,
                "Next, enter the description of the role. It will be used as a system for ChatGPT.",
            )
            .await?;
            dialogue
                .update(State::ReceiveNewRoleSystem { role_name })
                .await?
        }
        None => {
            bot.send_message(msg.chat.id, "Please enter a valid role name.")
                .await?;
        }
    }
    Ok(())
}

async fn receive_new_role_system(
    bot: Bot,
    msg: Message,
    roles: RolesRef,
    role_name: String,
    dialogue: NewRoleDialogue,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(role_system) => {
            create_role(&roles, &role_name, role_system).await?;
            dialogue.update(State::None).await?;
            bot.send_message(
                msg.chat.id,
                escape_markdown_v2_reversed_chars(&format!(
                    "Role *{role_name}* added successfully. And now I'm {role_name}"
                )),
            )
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please enter a valid role system.")
                .await?;
        }
    }
    Ok(())
}

async fn create_role(
    roles: &RolesRef,
    role_name: &str,
    system: String,
) -> Result<(), anyhow::Error> {
    let mut roles = roles.lock().await;
    roles.insert(role_name.to_string(), Role { system });
    storages::rewrite_file(&roles).expect("Failed to write roles to file");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn command_handler(
    bot: Bot,
    msg: Message,
    conversation_history: ConversationHistoryRef,
    current_role: Arc<Mutex<String>>,
    roles: RolesRef, // cmd: Command,
    command: Command,
    dialogue: NewRoleDialogue,
    settings: Settings,
) -> HandlerResult {
    match command {
        Command::NewRole => start_new_role_dialogue(bot, msg, dialogue).await?,
        Command::DeleteRole => delete_role(bot, msg, roles).await?,
        Command::SwitchRole => switch_role(bot, msg, roles).await?,
        Command::Test => just_for_test(&bot, &msg).await?,
        Command::ListRoles => list_roles(&bot, &msg, roles, current_role).await?,
        Command::Clear => clear_conversation(&bot, &msg, conversation_history).await?,
        Command::Translate(user_input) => translate(bot, msg, settings, user_input).await?,
        Command::VariableNamer(scene) => naming_variable(bot, msg, settings, scene).await?,
        Command::CheckGrammar(sentance) => check_grammar(bot, msg, settings, sentance).await?,
    }

    Ok(())
}

async fn message_handler(
    bot: Bot,
    msg: Message,
    conversation_history: ConversationHistoryRef,
    settings: Settings,
) -> HandlerResult {
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
) -> HandlerResult {
    if let Some(callback_data) = q.data {
        bot.answer_callback_query(q.id).await?;
        if q.message.is_none() {
            info!("No message in callback query");
            return Ok(());
        }

        let parts = callback_data.splitn(2, ' ').collect::<Vec<_>>();
        let command = parts[0];
        let callback_data = parts[1];

        if let Ok(command) = serde_json::from_str::<Command>(command) {
            match command {
                Command::DeleteRole => {
                    do_delete_role(bot, q.message.unwrap(), roles, callback_data).await?;
                }
                Command::SwitchRole => {
                    do_switch_role(
                        bot,
                        q.message.unwrap(),
                        conversation_history,
                        roles,
                        current_role,
                        callback_data,
                    )
                    .await?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

async fn clear_conversation(
    bot: &Bot,
    msg: &Message,
    conversation_history: ConversationHistoryRef,
) -> HandlerResult {
    let mut conversation_history = conversation_history.lock().await;
    conversation_history.truncate(1);
    bot.send_message(
        msg.chat.id,
        "Conversation history cleared, new session started.",
    )
    .await?;
    Ok(())
}

async fn translate(
    bot: Bot,
    msg: Message,
    settings: Settings,
    user_input: String,
) -> HandlerResult {
    bot.send_chat_action(msg.chat.id, teloxide::types::ChatAction::Typing)
        .await?;
    let output = chat_gpt::translate(settings.open_ai_api_key.as_str(), user_input).await?;
    bot.send_message(msg.chat.id, output).await?;
    Ok(())
}

async fn naming_variable(
    bot: Bot,
    msg: Message,
    settings: Settings,
    scene: String,
) -> HandlerResult {
    bot.send_chat_action(msg.chat.id, teloxide::types::ChatAction::Typing)
        .await?;
    let output = chat_gpt::naming_variable(settings.open_ai_api_key.as_str(), scene).await?;
    bot.send_message(msg.chat.id, output).await?;
    Ok(())
}

async fn check_grammar(bot: Bot, msg: Message, settings: Settings, scene: String) -> HandlerResult {
    bot.send_chat_action(msg.chat.id, teloxide::types::ChatAction::Typing)
        .await?;
    let output = chat_gpt::check_grammar(settings.open_ai_api_key.as_str(), scene).await?;
    bot.send_message(msg.chat.id, output).await?;
    Ok(())
}
