# Telegram ChatGPT 机器人

这是一个一对一的 Telegram ChatGPT 机器人，专为个人使用而设计。

## 功能

1. 允许 ChatGPT 扮演特定角色，并能够创建角色和在角色之间切换。
2. 翻译文本。
3. 基于场景描述提供变量命名建议。
4. 帮助诊断语法问题并提供纠正建议。

## 开始

下载代码库：

```shell
git clone https://github.com/hyzmm/telegram-chatgpt-rust
cd telegram-chatgpt-rust
```

在运行之前，请确保已准备好你的 OpenAI API 密钥和 Telegram 机器人令牌：

```shell
export OPEN_AI_API_KEY=sk-...
export TELOXIDE_TOKEN=YOUR_TELEGRAM_BOT_TOKEN
cargo run
```

## 命令列表

运行代码后，你可以在聊天窗口中看到机器人支持的命令列表：

<img width="626" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/5be2be51-24eb-4627-9275-a07e6b044abe" alt="">

## 用法

除了支持的命令外，你可以直接与机器人聊天。使用 `/listroles` 查看所有角色，其中默认角色名为 **assistant**，它是个编程助手。

<img width="626" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/fa3c6973-5331-4a7c-a5fc-cfb6e557f21c" alt="">

### 角色

你可以使用 `/newrole` 创建角色，后面可以在角色之间切换。

<img width="626" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/ad28c0a2-a211-4ba5-8f10-1d4bcb37d54c" alt="">

创建角色后，它将被设置为默认角色。您还可以使用 `/deleterole` 删除角色，或使用 `/switchrole` 切换到另一个角色。

## 清除会话

与机器人的聊天上下文将被发送到 ChatGPT 服务。如果对话不依赖于历史上下文，则可以使用 `/clear` 开始新会话。

<img width="626" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/737103f2-f6e6-438f-8393-125f19b03321" alt="">

## 其他命令

为了方便起见，一些常用功能作为机器人命令提供，无需创建或切换角色。以下是这些命令：

### 翻译 `/trans`

<img width="626" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/c9e82f2e-f76e-4296-bf05-0d74b26487e8" alt="">

默认情况下，输入内容将被翻译为英语。您可以使用 `-l` 指定目标语言：

<img width="626" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/1ff8b67a-5276-4cba-8ab5-9fbae0ba8eb6" alt="">

### 变量命名建议 `/naming`

这是程序员使用的辅助工具。通过描述场景，机器人提供命名建议：

<img width="637" alt="image" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/5e89507f-537c-4261-982c-145afaa7daf4">

### 语法检查器 `/gramcheck`

<img width="610" alt="image-20230526181219118" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/5bc719ba-104d-4af3-8a90-5d0cfa7a6788">

默认情况下，机器人会用中文回复您，如果您希望机器人用另一种语言回复您，可以添加 `-l` 参数：

<img width="626" alt="image" src="https://github.com/hyzmm/telegram-chatgpt-rust/assets/48704743/eb2e7181-960d-410e-ab32-559ee6547775">

