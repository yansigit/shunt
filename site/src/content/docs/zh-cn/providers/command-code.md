---
title: "Command Code：API 密钥与订阅"
description: "使用独立凭据与协议的两个 Command Code 产品。"
---

两个名称有意区分。它们是有序 upstream 预设，不会自动创建旧式提供方表。

| 预设 | kind / auth | 目标 |
| --- | --- | --- |
| `commandcode` | `openai_chat` / `api_key` | `https://api.commandcode.ai/provider/v1/chat/completions` |
| `command-code` | `command_code` / `command_code_oauth` | `https://api.commandcode.ai/alpha/generate` |

## 配置
```toml
[[upstreams]]
name = "anthropic"
provider = "anthropic"

[[upstreams]]
name = "cc-api"
provider = "commandcode"

[[upstreams]]
name = "cc-sub"
provider = "command-code"
effort = "high"

[[models]]
id = "cc-sub-glm"
[models.upstream_model]
cc-sub = "zai-org/GLM-5.3"
```

请使用已在 API 账户中验证的模型 ID，另行添加 `cc-api` 映射。下表不适用于 API 密钥产品。保留 Anthropic upstream 以处理未映射的模型。现有提供方设置不变。

## 凭据

API 密钥预设使用 `SHUNT_COMMANDCODE_API_KEY`；订阅使用 `SHUNT_COMMAND_CODE_TOKEN`。仅当后者完全不存在时，才只读访问现有的 `~/.commandcode/auth.json`（字符串 `apiKey`，可选 `userId`）。显式空值或无效令牌直接失败，不回退到文件。文件上限为 16 KiB，凭据查询最多等待 5 秒。令牌必须是非空、可安全放入标头的 ASCII。

Shunt 不会对此订阅文件执行登录、whoami、刷新、复制、修复或写入，也不进行账户轮换，不提供凭据路径或版本覆盖。`command_code_oauth` 仅用于 `command_code`。订阅目标仅允许规范 HTTPS 主机和 443 端口，禁止用户信息、查询及片段；路径可为空、`/` 或 `/alpha/generate`。查询前和构造 bearer 标头前均验证，不跟随重定向。

## 精确的订阅模型 / effort

| 模型 ID（区分大小写） | 可显式指定的 effort |
| --- | --- |
| `deepseek/deepseek-v4-pro` | `high`, `max` |
| `deepseek/deepseek-v4-flash` | `high`, `max` |
| `zai-org/GLM-5` | `high`, `max` |
| `zai-org/GLM-5.1` | `high`, `max` |
| `zai-org/GLM-5.2` | `high`, `max` |
| `zai-org/GLM-5.2-Fast` | `high`, `max` |
| `zai-org/GLM-5.3` | `low`, `high`, `max` |
| `meta/muse-spark-1.2` | `low`, `medium`, `high`, `xhigh`, `max` |
| `meta/muse-spark-1.2-contributor` | `low`, `medium`, `high`, `xhigh`, `max` |
| `meta/muse-spark-1.1` | `low`, `medium`, `high`, `xhigh`, `max` |

可以省略 effort。请求中的显式值优先于路由默认值；不支持的值（包括 `none`、`ultra`）直接拒绝，不做修正。不接纳未知 ID，也不接纳仅有报告的 Luna、Gemini 3.7 Flash 和 vision-exp 条目。

## 转换与限制

订阅将支持的文本、明文推理、图像及真实工具调用/结果历史转换为不含工作区信息的请求。支持明文子代理结果和续接历史，但拒绝不透明状态。缺少记录的工具结果标记为执行状态未知，不伪造成功。工具目录及选择是显式的。不发送原始项目路径或会话标识；显式会话使用凭据隔离的不透明 session，其余使用请求级随机 session。

两个产品都支持单次响应及 Anthropic 流式输出。订阅始终增量读取 NDJSON：每条 JSON 记录 1 MiB（不含 LF/CRLF），总传输 32 MiB，语义数据 8 MiB，每个工具参数 512 KiB，工具 128 个，内容块 4,096 个。入站请求默认上限为 32 MiB，可通过 `server.limits.max_request_bytes` 配置。令牌计数使用本地估算。

成功需要受支持的权威结束记录以及完整分帧的 EOF。格式错误、重复、结束后的、未知或截断记录均失败，不修复，也不会仅凭 EOF 合成成功。用量采用经过检查的整数运算，从包含缓存的输入中拆分缓存令牌，避免重复计数。提供方错误结束仍为失败，但保留已报告用量。仅能证明发生在连接前的失败可以保留相同凭据和 session 重试；发送后失败或输出/工具活动后不得重放。取消会关闭 upstream 并释放请求名额。

订阅读取空闲上限与完整记录进度期限分别固定为 120 秒；零散字节不会重置进度期限。这并非整个回合的时限。网关错误使用入站协议的格式。API 密钥产品保留[通用 Chat 契约](/zh-cn/providers/openai-chat/)。

## 订阅兼容性细节

接受的顶层字段为 `model`、`messages`、`max_tokens`、`stream`、`system`、`output_config`、`temperature`、`tools` 和 `tool_choice`。其他字段（包括 `top_p`、`top_k`、`stop_sequences`、顶层 `thinking` 和 `metadata`）返回 400，不会被静默忽略或转发。请在 `output_config.effort` 中使用明确支持的值。

工具目录按名称排序。`tool_choice: any` 会添加要求至少调用一个工具的系统指令；`tool_choice: tool` 会将目录缩小为指定工具，并添加点名该工具的指令。这些是模型可见的指令，不是保证工具调用的原生协议功能。`auto` 不添加指令，`none` 则移除工具目录。

## 验证边界

此矩阵与客户端版本 `0.52.1` 来自 2026-09-08 检查的固定 OpenCodex `055c3ecf` 源码。隔离 TLS 和 CLI/模拟测试不证明实际提供方可用性。最小请求格式是否被接受、版本是否仍适用，仍属于 Phase 16 的可选实时验证。不存在公开的目标绕过配置。参见[配置参考](/zh-cn/reference/configuration/)。
