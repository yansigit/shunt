---
title: "OpenCode Go: 証拠ゲート、サポートなし"
description: "正確で hermetic、かつ資格情報安全なタプルが証明されるまで OpenCode Go は許可されません。"
---

# OpenCode Go は現在サポートされていません

許可された OpenCode Go 集合は空です。shunt は資格情報取得前ゲートで明示的な Go 選択をすべて、資格情報の参照やネットワーク送信より前に拒否するため、現在は資格情報も `x-opencode-session` ヘッダーも送信されません。

オプトイン構成 ID は `kind = "opencode_go"` で、`SHUNT_OPENCODE_GO_API_KEY` と正規の宛先 `https://opencode.ai/zen/go/v1` を使います。現在のソースのみの候補は次のとおりです。

- `glm-5.3-flash`
- `omen-alpha`
- `muse-spark-1.3-contributor`
- `deepseek-v4-flash`

これらは候補であり、サポート済みでもライブ検証済みでもありません。不明なフィールド、誤った wire、ファミリー推論、未サポートの effort 別名、失敗した証拠は拒否されます。ゲートウェイは厳格な権威ある終端を要求し、寛容な EOF から成功を合成したり、不完全なターンを修復したりしません。

## 将来の昇格契約

将来の昇格には、正確なモデル・宛先・wire・effort・capability の証拠、hermetic 適合性、資格情報安全なキャプチャまたはライブ検証が必要です。一致する Chat 契約を再利用する場合に限り、`https://opencode.ai/zen/go/v1` へ安定した不透明な conversation-scoped `x-opencode-session` を送信できます。現在の空の許可集合にはセッション生成器がありません。

設定の詳細は[設定リファレンス](/ja/reference/configuration/)を参照してください。
