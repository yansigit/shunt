---
title: "Command Code: API キーとサブスクリプション"
description: "認証情報とプロトコルを分離した二つの Command Code 製品。"
---

名前は意図的に異なります。どちらも順序付き upstream のプリセットであり、レガシープロバイダーテーブルは自動作成されません。

| プリセット | kind / auth | 接続先 |
| --- | --- | --- |
| `commandcode` | `openai_chat` / `api_key` | `https://api.commandcode.ai/provider/v1/chat/completions` |
| `command-code` | `command_code` / `command_code_oauth` | `https://api.commandcode.ai/alpha/generate` |

## 設定
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

API アカウントで確認したモデル ID を使い、`cc-api` のマッピングを別途追加してください。下の表は API キー製品には適用されません。未マッピングのモデル用に Anthropic upstream を残します。既存設定は変更されません。

## 認証情報

API キーは `SHUNT_COMMANDCODE_API_KEY`、サブスクリプションは `SHUNT_COMMAND_CODE_TOKEN` を使用します。後者の変数が存在しない場合のみ、既存の `~/.commandcode/auth.json`（文字列 `apiKey`、任意の `userId`）を読み取り専用で読みます。空または不正な明示トークンは、ファイルへ切り替えず失敗します。ファイル上限は 16 KiB、認証情報取得の待機上限は 5 秒です。トークンは空でないヘッダー安全な ASCII が必要です。

Shunt はこのファイルのログイン、whoami、更新、コピー、修復、書き込みを行いません。アカウント切り替えや認証パス・バージョン上書きもありません。`command_code_oauth` は `command_code` 専用です。接続先は正規 HTTPS ホストの 443 番のみで、ユーザー情報・クエリ・フラグメントは禁止です。パスは空、`/`、`/alpha/generate` のいずれかです。取得前と bearer ヘッダー生成前に検証し、リダイレクトは追跡しません。

## 正確なサブスクリプションモデル / effort

| モデル ID（大文字小文字を区別） | 明示可能な effort |
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

effort の省略は可能です。リクエストの明示値はルート既定値より優先されます。`none`、`ultra` など未対応の値は補正せず拒否します。不明な ID と報告のみの Luna、Gemini 3.7 Flash、vision-exp は許可しません。

## 変換と上限

対応するテキスト、平文推論、画像、実際のツール呼び出し・結果履歴を、ワークスペース情報なしで送ります。平文サブエージェント結果と継続履歴は対応しますが、不透明な状態は拒否します。記録のないツール結果は成功を捏造せず、実行状態不明と表示します。ツール一覧と選択は明示的です。生のプロジェクトパスや会話 ID は送信しません。明示した会話には認証情報にスコープされた不透明セッション、それ以外にはリクエスト単位のランダムセッションを使います。

両製品は単一応答と Anthropic ストリーミング出力に対応します。サブスクリプション NDJSON は常に逐次読み取りで、JSON レコード 1 MiB（LF/CRLF 除外）、転送全体 32 MiB、意味データ 8 MiB、ツール引数 512 KiB/件、ツール 128 件、コンテンツブロック 4,096 件が上限です。受信リクエストの既定上限は 32 MiB で、`server.limits.max_request_bytes` で設定します。トークン計数はローカル推定です。

成功には対応する確定終了レコードと完全にフレーム化された EOF が必要です。不正・重複・終了後・不明・途中切断のレコードは修復せず失敗し、EOF だけから成功を合成しません。使用量は検証済み整数演算を使い、キャッシュトークンを包括入力から分離して二重計上を防ぎます。プロバイダーのエラー終了は使用量を保持した失敗です。接続前と証明できる失敗だけが同じ認証情報とセッションで再試行可能です。送信後の失敗や出力・ツール活動後の再送は行いません。キャンセルは upstream を閉じ、受付枠を解放します。

読み取りアイドル上限と完全レコードの進捗期限は、それぞれ固定 120 秒です。断片的なバイトでは進捗期限をリセットしません。ターン全体の期限ではありません。ゲートウェイエラーは受信プロトコル形式です。API キー製品は[汎用 Chat 契約](/ja/providers/openai-chat/)に従います。

## サブスクリプションの互換性の詳細

受け付ける最上位フィールドは `model`、`messages`、`max_tokens`、`stream`、`system`、`output_config`、`temperature`、`tools`、`tool_choice` です。`top_p`、`top_k`、`stop_sequences`、最上位の `thinking`、`metadata` を含む他のフィールドは 400 を返し、黙って無視したり転送したりしません。`output_config.effort` には正確に対応している値を指定してください。

ツール一覧は名前順に並べ替えられます。`tool_choice: any` は少なくとも一度のツール呼び出しを求めるシステム指示を追加します。`tool_choice: tool` は一覧を指定ツールに絞り、そのツール名を含む指示を追加します。これらはモデルに見える指示であり、ツール呼び出しを保証するネイティブプロトコル機能ではありません。`auto` は指示を追加せず、`none` は一覧を削除します。

## 検証範囲

この表とクライアント版 `0.52.1` は、2026-09-08 に確認した固定 OpenCodex `055c3ecf` のソース由来です。隔離 TLS と CLI/モックテストは実サービスの可用性証明ではありません。最小リクエストの受理とバージョンの現行性は Phase 16 の任意ライブ検証項目です。公開の接続先バイパスはありません。[設定リファレンス](/ja/reference/configuration/)も参照してください。
