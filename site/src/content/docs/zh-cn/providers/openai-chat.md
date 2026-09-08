---
title: OpenAI 兼容 (Chat Completions)
description: 仅凭 API 密钥将映射的模型路由到任意 OpenAI Chat Completions 端点。
---

**OpenAI 兼容 (Chat Completions)** 是一种通用 provider 类型,而非具名 provider:
`kind = "openai_chat"` 将 shunt 指向任何提供 OpenAI Chat Completions API
(`POST /chat/completions`) 的后端,shunt 会把 Claude Code 的 Anthropic Messages 请求翻译成该
形态 — 包括流式传输。它没有内置 preset,因此 upstream 必须显式声明 `kind`、`base_url` 和
API 密钥凭据。

## 配置 upstream

```toml
[[upstreams]]
name = "anthropic"
provider = "anthropic"   # keep Anthropic as the default for unrouted models (e.g. claude-*)

[[upstreams]]
name = "chat"
kind = "openai_chat"
base_url = "https://api.example.com/v1"
auth = { mode = "api_key", env = "CHAT_API_KEY" }

[[routes]]
model = "gpt-5.4"
provider = "chat"
```

有序 `[[upstreams]]` 会替换 shunt 的内置 provider,因此配置必须同时声明仍作为回退的
`anthropic` 默认值(`server.default_provider` 默认为 `anthropic`)。

旧式 `[providers.chat]` 表形式仍然受支持:改用 `kind`、`base_url`、`auth = "api_key"` 和
`api_key_env = "CHAT_API_KEY"` 代替 auth 映射。不要在同一文件中混用 `[[upstreams]]` 和
`[providers.*]`。

`kind = "openai_chat"` 只接受 `auth = "api_key"`。启动时会拒绝任何其他凭据模式 —
适配器按请求注入配置的密钥,没有其他凭据路径。

## 凭据

```bash
export CHAT_API_KEY='...'
```

不要把密钥写进配置文件。`shunt check` 只校验配置结构,不读取密钥值 — 如果 `CHAT_API_KEY`
未设置,第一条路由到 `chat` 的请求会返回认证错误。

## base URL 语法

`base_url` 必须是纯 `http://` 或 `https://` 根路径:不允许查询字符串、片段、userinfo
(`user:pass@`)、点路径段、空白或反斜杠。shunt 会追加恰好一个 `/chat/completions` 路径 —
已经以 `/chat/completions` 结尾的根路径保持不变 — 因此 `https://api.example.com/v1` 与
`https://api.example.com/v1/chat/completions` 等价。`shunt check` 在启动时校验同样的语法。

## 适配器承载的内容

翻译采用默认拒绝的白名单,因此不受支持的请求会以带类型的 400 失败关闭(fail closed),而不会静默降级:

- **文本轮次**(system、user、assistant)以及 `max_tokens`、`temperature`、`top_p` 和
  `stop_sequences`。不受支持的顶层请求字段会被拒绝。单个文本负载以 UTF-8 计不超过 8 MiB。
- **图像**:user 消息中的 base64 数据或 URL。
- **工具**:声明、`tool_choice` 以及成对的 `tool_use`/`tool_result` 轮次;使用请求局部的 id
  注册表,并发请求之间不会共享状态。
- **流式**:翻译为 Anthropic SSE 并保留用量:流式请求会强制上游的
  `stream_options.include_usage` 约定。聚合的 unary 响应上限为 32 MiB,流式状态机每轮最多
  保留 8 MiB 语义字节。

## 失败与取消语义

一旦请求可能已经到达上游,生成 POST 就不会被重新派发,并且重定向会被直接拒绝,从而注入的 bearer
不会被带离配置的源站。上游的非成功状态码会以网关自有错误的形式呈现;读取空闲超过 120 秒会使该
轮次失败。取消请求会关闭上游连接并释放准入槽位。

这些保证由仓库测试套件中的合成一致性夹具覆盖;此处不主张也未验证任何真实 provider 行为。

## 协议限制

工具参数按请求缓冲，在结束边界仅解析一次。工具块在 `message_start` 之后按首次到达
顺序输出。最多允许128个调用，每个调用的参数上限为1 MiB。SSE事件负载和未完成的残余
负载各有独立的8 MiB上限，不计帧分隔符。累计8 MiB语义预算计算文本、推理、工具标识
和参数，不包括残余缓冲区。

成功必须同时有受支持的结束原因和 `[DONE]`，不能仅凭EOF。`stop`、`length` 和
`tool_calls` 分别映射为 `end_turn`、`max_tokens` 和 `tool_use`，其他原因会被拒绝。
已经解码的后续帧或残余字节会触发错误；结束后不会等待未来字节或HTTP EOF，也无法撤回
已发送的文本。令牌计数必须是 `0..=i64::MAX` 范围内的整数。

支持纯文本thinking。响应字段 `reasoning` 和 `reasoning_content` 是提供方扩展，
并非通用OpenAI约定。冲突、带签名或已删节的推理会被拒绝。读取空闲限制固定为120秒，
没有新增配置键。此适配器的 `count_tokens` 始终采用本地估算，即使配置了其他策略。
此适配器不会写入凭据文件。

## 验证

```bash
shunt check    # -> config ok
shunt run
curl -sS http://127.0.0.1:3001/v1/messages \
  -H 'anthropic-version: 2023-06-01' \
  -H 'content-type: application/json' \
  -d '{"model":"gpt-5.4","max_tokens":16,"messages":[{"role":"user","content":"Reply with OK."}]}'
```
