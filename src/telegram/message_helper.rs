use teloxide::prelude::*;
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, Message, ReplyMarkup,
};
use teloxide::Bot;

use crate::telegram::startup::RolesRef;

pub async fn send_roles_using_inline_keyboard(
    bot: Bot,
    msg: Message,
    roles: RolesRef,
    text: &str,
) -> Result<(), anyhow::Error> {
    let roles = roles.lock().await;

    let buttons: Vec<Vec<InlineKeyboardButton>> = roles
        .keys()
        .collect::<Vec<&String>>()
        .chunks(2)
        .map(|roles| {
            roles
                .iter()
                .map(|role| {
                    let role = *role;
                    InlineKeyboardButton::new(
                        role,
                        InlineKeyboardButtonKind::CallbackData(role.clone()),
                    )
                })
                .collect::<Vec<InlineKeyboardButton>>()
        })
        .collect();

    bot.send_message(msg.chat.id, text)
        .reply_markup(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup {
            inline_keyboard: buttons,
        }))
        .send()
        .await?;

    Ok(())
}
