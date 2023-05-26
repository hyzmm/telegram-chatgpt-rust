# Telegram ChatGPT Bot

This is a one-on-one Telegram ChatGPT bot designed for personal use.

## Features

1. Allows ChatGPT to play specific roles, with the ability to create and switch between roles.
2. Translates text.
3. Provides variable naming suggestions based on scene descriptions.
4. Helps diagnose syntax issues in statements and provides suggestions for correction.

## Getting Started

Download the repository:

```shell
git clone https://github.com/hyzmm/telegram-chatgpt-rust
cd telegram-chatgpt-rust
```

Before running, make sure to have your OpenAI API key and Telegram bot token ready:

```shell
export OPEN_AI_API_KEY=sk-...
export TELOXIDE_TOKEN=YOUR_TELEGRAM_BOT_TOKEN
cargo run
```

## Command List

After running the code, you can see the list of commands supported by the bot in the chat window:

<img width="626" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/5be2be51-24eb-4627-9275-a07e6b044abe" alt="">

## Usage

In addition to the supported commands, you can also chat directly with the bot. Use `/listroles` to view all roles, with a default role named **assistant** set as a programming assistant.

<img width="626" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/fa3c6973-5331-4a7c-a5fc-cfb6e557f21c" alt="">

### Roles

You can create a role using `/newrole`, which can then be switched between.

<img width="626" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/ad28c0a2-a211-4ba5-8f10-1d4bcb37d54c" alt="">

After creating a role, it will be set as the default. You can also delete a role using `/deleterole`, or switch to another role using `/switchrole`.

## Clearing Sessions

The chat context with the bot is sent to the ChatGPT service. If a conversation does not depend on the historical context, you can use `/clear` to start a new session.

<img width="626" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/737103f2-f6e6-438f-8393-125f19b03321" alt="">

## Other Commands
For convenience, some commonly used features are provided as bot commands, without the need to create or switch roles. The following are these commands:

### Translation `/trans`

<img width="626" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/c9e82f2e-f76e-4296-bf05-0d74b26487e8" alt="">

By default, the input content is translated into English. You can specify the target language using `-l`:

<img width="626" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/1ff8b67a-5276-4cba-8ab5-9fbae0ba8eb6" alt="">

### Variable Naming Suggestions `/naming`

This is an auxiliary tool used by programmers. By describing a scene, the bot provides naming suggestions:

<img width="626" alt="image" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/2fe856a0-357b-4bda-8a92-4eb689544d67">

### Syntax Checker `/gramcheck`

<img width="610" alt="image-20230526181219118" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/5bc719ba-104d-4af3-8a90-5d0cfa7a6788">

By default, the bot responds to you in Chinese, if you want the bot to respond in another language, you can add the -l parameter:

<img width="626" alt="image" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/eb2e7181-960d-410e-ab32-559ee6547775">

