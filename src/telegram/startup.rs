use std::sync::Arc;

use openai_chatgpt_api::ChatGptChatFormat;
use teloxide::types::ParseMode;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::sync::Mutex;

use crate::chat_gpt::ask_chat_gpt;
use crate::storages;
use crate::storages::Roles;
use crate::utils::telegram_utils::escape_markdown_v2_reversed_chars;

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
}

type ConversationHistory = Arc<Mutex<Vec<ChatGptChatFormat>>>;

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

pub async fn startup() -> Result<(), anyhow::Error> {
    let settings = Settings::from_env();
    let bot = Bot::from_env();

    let storage_roles = storages::get_roles()?;

    let ignore_update = |_upd| Box::pin(async {});

    let chat_gpt_system = ChatGptChatFormat::new_system(
        "You are my personal assistant. Most of questions I asked are related to programming. Your reply to me can be in Markdown format.",
    );
    let conversation_history = Arc::new(Mutex::new(vec![chat_gpt_system]));
    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(answer_command),
        )
        .endpoint(handle_message);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![conversation_history, settings, storage_roles])
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
    conversation_history: ConversationHistory,
    command: Command,
    roles: Roles,
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
            // iterate `roles` to get all roles, store in a string
            let mut output = String::new();
            for (index, (name, role)) in roles.iter().enumerate() {
                output.push_str(&escape_markdown_v2_reversed_chars(&format!(
                    "{}. *{}*: {}\n",
                    index + 1,
                    name,
                    role.system
                )));
            }
            bot.send_message(msg.chat.id, output)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;
        }
    }
    Ok(())
}

async fn handle_message(
    bot: Bot,
    msg: Message,
    conversation_history: ConversationHistory,
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
