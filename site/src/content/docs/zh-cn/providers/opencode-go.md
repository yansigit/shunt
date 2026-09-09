---
title: "OpenCode Go：证据门控，不支持"
description: "在证明精确、hermetic 且凭据安全的元组之前，不准入 OpenCode Go。"
---

# OpenCode Go 目前不受支持

已准入的 OpenCode Go 集合为空（空准入集合）。shunt 在凭据前门控中、凭据查找或网络分发之前拒绝所有显式 Go 选择，因此当前不会发送凭据或 `x-opencode-session` header。

可选配置标识为 `kind = "opencode_go"`，使用 `SHUNT_OPENCODE_GO_API_KEY` 和规范目标 `https://opencode.ai/zen/go/v1`。当前仅有源码证据的候选为：

- `glm-5.3-flash`
- `omen-alpha`
- `muse-spark-1.3-contributor`
- `deepseek-v4-flash`

这些只是候选，并非受支持或已实时验证的模型。未知字段、错误 wire、按系列推断、不支持的 effort 别名和失败证据都会被拒绝。网关要求严格的权威终止，不会从宽松 EOF 合成成功，也不会修复不完整的 turn。

## 未来准入合约

未来准入必须同时具备精确的模型、目标、wire、effort 和 capability 证据、hermetic 一致性以及凭据安全的捕获或实时验证。只有复用匹配的 Chat 合约时，才能向 `https://opencode.ai/zen/go/v1` 发送稳定、opaque、conversation-scoped 的 `x-opencode-session`；当前空准入集合不生成此 header。

配置详情请参阅[配置参考](/zh-cn/reference/configuration/)。
