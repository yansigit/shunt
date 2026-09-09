---
title: インバウンド Codex エンドポイント
description: OpenAI の Codex CLI 自身を shunt へ向け、ChatGPT/Codex OAuth アカウントプールで負荷分散する。
---

このサイトの他のガイドはすべて **Claude Code** を別のバックエンドへルーティングします。shunt は逆方向にも動けます。オプトインの生の OpenAI Responses パススルーによって、**Codex CLI** が自身の `base_url` を shunt へ向け、ChatGPT/Codex OAuth アカウントプールで負荷分散されるようにするものです。これはオプトインです。`[server.codex_endpoint]` がない場合、それらのルートはいずれも登録されず、shunt のデフォルトの HTTP サーフェスは変わりません。

これは [Codex マルチアカウント](/ja/guides/codex-multi-account/)と同じアカウントプールの上に構築されます — 選択、クールダウン、リフレッシュはそのまま共有されます。正確なフェイルオーバーの表とリロードのセマンティクスを含む完全な仕様は、[M11 の挙動仕様](https://github.com/pleaseai/shunt/blob/main/docs/m11-inbound-codex-endpoint.md)を参照してください。

エンドツーエンドのセットアップ — エンドポイントの有効化、Codex CLI を shunt へ向ける、クライアント認証、アカウントのプロビジョニング、entitle されたモデルの選択 — については [Codex CLI の接続](/ja/guides/connect-codex-cli/)に従ってください。このページは*エンドポイントが何をするか*に焦点を当てており、あちらのガイドが*どう接続するか*のチェックリストです。

## エンドポイントを有効にする

```toml
[server.codex_endpoint]   # all keys optional; default shown
provider = "codex"        # must be a chatgpt_oauth provider
collaboration = false     # translated V2 collaboration bridge is opt-in
```

```bash
shunt check
shunt run
```

起動時の検証は、未知の `provider` や `auth = "chatgpt_oauth"` を使わないプロバイダーを拒否します — このエンドポイントはオペレーターの Codex ベアラーを注入するため、`chatgpt_oauth` プロバイダーだけが要件を満たします。すべてのキーとデフォルトは[設定リファレンス](/ja/reference/configuration/)を、登録されるルートは [HTTP エンドポイント](/ja/reference/endpoints/)を参照してください。

このオプトインにより、Codex CLI のモデル検出も解析可能になります。`GET /models` と `GET /backend-api/codex/models` は有効なフォールバック `{"models":[]}` を返します。共有の `GET /v1/models` パスでは、`client_version` クエリフィールドが Anthropic 風のヘッダーより優先され、Codex 形式を選択します。このフィールドがなければ、既存の Anthropic 検出レスポンスは変わりません。これらのリクエストは通常のモデル検出認証ゲートを通り、shunt は不完全な Codex モデル行を生成しません。

## クライアント analytics のシンク

Codex CLI は base URL へプロダクト analytics も POST します。shunt は CLI が生成しうる両方のパスを受け付けます。

- `POST /backend-api/codex/analytics-events/events`
- `POST /codex/analytics-events/events`

これらのルートは Responses ルートと同じ `[server.auth]` ポリシーを使いますが、テレメトリーを上流へ転送することは決してありません。プールされたアカウントを 1 つ選ぶと、クライアントのイベントがそのアカウントへ誤って帰属されてしまうためです。認証後は、不正な形式・読み取り不能・サイズ超過のボディも含めて、常に `200 {}` を返します。

payload とイベントのプロパティは、ログにも記録されずエクスポートもされません。shunt が記録するのは、オプトインの `shunt.codex_client_events` カウンターの `event` 属性としてサニタイズ済みの `event_type` だけです。名前に使えるのは小文字の ASCII 英字、数字、`.`、`_`、`-` で、最大 64 バイトです。不正な名前は `other` に、認識できないバッチは `unparsed` になります。Sentry も OpenTelemetry のメトリクスも有効でなければ、これは純粋な破棄シンクです。

## Codex CLI を shunt へ向ける

Codex CLI は、使用する base URL が何であれ常に `/responses` を末尾に付けるため、`~/.codex/config.toml` はどちらの形状でも動作します。

**ChatGPT バックエンドの base URL をミラーする:**

```toml
chatgpt_base_url = "http://127.0.0.1:3001/backend-api/codex"
```

**またはカスタムモデルプロバイダー**（トップレベルの `model_provider` でそれを選択する必要があります。さもないと CLI は組み込みのプロバイダーを使い続けます）:

```toml
model_provider = "shunt"

[model_providers.shunt]
base_url = "http://127.0.0.1:3001/v1"
wire_api = "responses"
```

カスタムプロバイダーを使う場合（CLI がローカルログインを必要としないよう `requires_openai_auth = false` を追加してください）、shunt へ向けた時点で Codex CLI 自身の `~/.codex/auth.json` は無関係になります — アカウントはリクエストごとに shunt のプールから来ます。一方 `chatgpt_base_url` の形状は CLI を ChatGPT ログインモードのままにするため、引き続きローカルのログインが必要で、**ゲートされていない**エンドポイントに対してのみ動作します。その ChatGPT ベアラーは設定された shunt トークンではないため、`[server.auth]` はそれを拒否します。

## クライアント認証

shunt に [`[server.auth]`](/ja/guides/shared-gateway/) が設定されている場合 — ループバックを超えるものには推奨です — クライアントトークンを、OpenAI 形式の Bearer キー（`OPENAI_API_KEY` / カスタムプロバイダーの `env_key`、LiteLLM/llmgateway の作法）**または** `x-shunt-token` ヘッダーの**いずれか**で提示します。

```toml
# A. Bearer — built-in openai provider. Set the base URL in ~/.codex/config.toml,
#    NOT via the OPENAI_BASE_URL env var: the env var leaves the CLI's Responses
#    WebSocket pointed at wss://api.openai.com, so it bypasses shunt. See
#    "Point the Codex CLI at shunt" in the connect guide.
openai_base_url = "http://127.0.0.1:3001/v1"
```

```bash
export OPENAI_API_KEY="<shunt-token>"      # sent as Authorization: Bearer
```

```toml
# B. Header — a custom provider carries it (use env_http_headers to keep it out of the file):
[model_providers.shunt]
base_url = "http://127.0.0.1:3001/v1"
wire_api = "responses"
http_headers = { "x-shunt-token" = "<token>" }
```

`[server.auth]` がなければ、このエンドポイントはそこへ到達できる誰にでも開かれています — ループバックや個人利用なら許容できますが、共有ゲートウェイでは不可です。クライアントが提示した認証情報は shunt への認証に**のみ**使われ、それ（および CLI がたまたま送る `Authorization`）は取り除かれ、上流へ転送されることはありません。`[server.admin]` の認証情報ヘッダー（既定では `x-shunt-admin-token`、`[server.admin] header` で指定した名前）も取り除かれます — 管理サーフェスはそのスロットで認証し、管理用の認証情報はアップストリームアカウントをプロビジョニングできるためです。`cookie` ヘッダーもヘッダーごと取り除かれます: 管理サーフェスは書き込み権限のセッション Cookie もそこで受理し、shunt 自身は Cookie ジャーを持たないため、上流がそれに依存することはありません。`x-api-key` も無条件に取り除かれます — `[server.auth]` が設定されていない場合も同様です。対象のプロバイダーは起動時に `chatgpt_oauth` 専用であることが検証されるため、インバウンドの `x-api-key` の値がこのアップストリームに対して有効な認証情報になることは決してありません。Claude Code の `apiKeyHelper` のように `Authorization` と `x-api-key` の両方に同じキーを設定するクライアントであっても、2 つ目のスロット経由でそのキーが漏れることはありません。インバウンドのクライアントが実際の Codex CLI であるため、パススルーはそのリクエストヘッダーをそのまま転送し（`version`、`originator`、`OpenAI-Beta`、`x-codex-*`、…）、差し替えるのは選択されたプールアカウントの `Authorization` ベアラーと `chatgpt-account-id` **だけ**です。認証の詳しい手順は [Codex CLI の接続](/ja/guides/connect-codex-cli/#3-shunt-クライアントトークンを提示するserverauth-設定時)を参照してください。

## WebSocket トランスポート

3 つの Responses パスは HTTP `POST` と認証済み WebSocket `GET` アップグレードの両方を受け付けます。認証は `101 Switching Protocols` より前に完了します。ソケットでは `generate: false` のウォームアップをローカルで完了し、通常の `response.create` は既存の HTTP アカウントプールを再利用して上流ストリーミングを強制し、最初の終端イベントまで各 SSE `data:` ペイロードを WebSocket テキストフレームとして転送します。ターンの置換またはソケット切断はアクティブな上流ボディをキャンセルします。クライアントフレームと SSE イベントは 4 MiB に制限され、送信には有界バックプレッシャーが適用され、エラーフレームには安全なレスポンスメタデータだけが含まれます。

## アカウントのプロビジョニング

[Codex マルチアカウント](/ja/guides/codex-multi-account/#プールを設定する)と同じプールを再利用します。

```bash
codex login
shunt login codex --name main
```

```toml
[[providers.codex.accounts]]
name = "main"
```

`[[providers.codex.accounts]]` が設定されておらず、**かつ shunt のアカウントストアが空**の場合、エンドポイントはデフォルトの `~/.codex/auth.json` 認証情報 1 つへフォールバックします — プーリングもフェイルオーバーもありません。そのため `[server.codex_endpoint]` を設定した時点で、Codex ログイン 1 つで動作します。（ハンドラーはまずアカウントストアをスキャンし、見つかったアカウントをプールするため、インポート済みのストアアカウントがあればプーリングは有効になります。）

## モデルを別のアップストリームへルーティングする

既定ではすべてのリクエストが `[server.codex_endpoint]` に指定した 1 つのプロバイダーへ送られます。任意の `[[server.codex_endpoint.routes]]` テーブルを使うと、Codex CLI がモデル id によって**別の** Responses 互換アップストリームを選べます。ルートのないモデルは従来どおり固定プロバイダーへ送られます。

複数のベンダーが Codex CLI 向けのネイティブ Responses エンドポイントを文書化しています: Z.ai GLM (`https://api.z.ai/api/v1`)、DeepSeek (`https://api.deepseek.com`)、Kimi Code (`https://api.kimi.com/coding/v1`)、MiniMax (`https://api.minimax.io/v1`)、Mimo (`https://api.xiaomimimo.com/v1`)、OpenRouter (`https://openrouter.ai/api/v1`)、Vercel AI Gateway (`https://ai-gateway.vercel.sh/codex/v1`)、そして純正の OpenAI。shunt はプロバイダーの `base_url` に `/responses` を付け足すため、ベンダーが Codex 用として案内しているものと同じ base URL をそのまま設定します。アップストリームは Responses API をネイティブに実装している必要があります — Responses → Chat Completions のアダプターはありません。

```toml
[providers.glm]
kind = "responses"
auth = "api_key"
api_key_env = "GLM_API_KEY"
base_url = "https://api.z.ai/api/v1"

[providers.deepseek]
kind = "responses"
auth = "api_key"
api_key_env = "DEEPSEEK_API_KEY"
base_url = "https://api.deepseek.com"

[server.codex_endpoint]
provider = "codex"

[[server.codex_endpoint.routes]]
model = "glm-5.3"
provider = "glm"

[[server.codex_endpoint.routes]]
model = "deepseek-v4-flash"
provider = "deepseek"
```

`upstream_model` は省略可能で、既定値は `model` です。CLI に入力する id とベンダーが実際に提供する id が異なる場合に指定します。ルーティング先のプロバイダーは実際の資格情報を持つ必要があります — クライアント自身の `Authorization` は常に削除されるため、資格情報を持たない認証モード（`passthrough` または `none`）は起動時に拒否されます。shunt 内蔵の `kimi` プリセットは `kind = "anthropic"` なので、Kimi Code への Codex ルートには別途 `kind = "responses"` のプロバイダーが必要です — Anthropic 種別のプリセットへ Codex モデルをルーティングすると起動時に拒否されます。

CLI 側では Codex を **shunt** に向け、`model` でルートを選びます:

```toml
# ~/.codex/config.toml
model = "glm-5.3"
model_provider = "shunt"
model_catalog_json = "~/.codex/models.json"

[model_providers.shunt]
base_url = "http://127.0.0.1:3001/v1"
wire_api = "responses"
env_key = "SHUNT_TOKEN"
```

shunt は Codex 用のモデルカタログを提供しません — `GET /v1/models` のディスカバリー一覧は Anthropic 形式で、Codex のルートを公開しません。CLI はこれらのベンダーが案内するとおり、`model_catalog_json` が指す `~/.codex/models.json` カタログからスラッグのメタデータを取得します。shunt のルートを選ぶのは `model` の値だけです。

**ChatGPT 以外**のアップストリームへルーティングされたリクエストで変わる点:

- **ヘッダーの許可リスト。** クライアントから引き継ぐのは `content-type` と `accept` のみで、これに解決された資格情報と、ルーティング先のアップストリーム自身が要求する identity が加わります — `OpenAI-Beta: responses=experimental`(xAI/Grok では省略)、および `xai_oauth` ルートの場合は Grok CLI の identity ヘッダー。`authorization`、`x-api-key`、`chatgpt-account-id`、`originator`、`version`、`user-agent`、`session-id`、`x-codex-*`、`x-shunt-*` はいずれもサードパーティに届きません。
- **ボディの `model` 書き換え。** `upstream_model` が要求されたモデルと異なる場合、shunt はトップレベルの `model` だけを書き換え、他のフィールドはそのまま残します。JSON オブジェクトでないボディはそのまま送らず `400` で拒否します。
- **identity エンコーディング。** zstd のリクエストボディはまずデコードされ(純正の Responses API はそのエンコーディングを受け付けません)、`content-encoding` は転送されません。
- **資格情報は 1 つ、フェイルオーバーなし。** ルーティング先のサードパーティの背後にプールはないため、429 や 5xx はローテーションを起こさず `retry-after` とともにそのままリレーされます。

マッチングは完全一致で大文字小文字を区別し、文字種の制限もありません。そのため `MiniMax-M3`、`openai/gpt-5.6-sol`、`~openai/gpt-latest` といったベンダーのスラッグも書いたとおりにルーティングされます。別の `chatgpt_oauth` プロバイダーへのルートであれば、プールのパススルーがそのまま維持されます。ルートはライブ設定から読まれるため、リロードで反映されます。

## `/v1/messages` との違い

- **ネイティブルートは不透明なまま。** Responses ネイティブルートでは、リクエスト本文と上流レスポンスをバイト単位で転送し、変換経路から分離します。
- **圧縮されたリクエストボディはそのまま通過。** 現行の Codex リリースは ChatGPT バックエンドと通信する際にリクエストボディを zstd 圧縮します。これには、このエンドポイントへ向けた `chatgpt_base_url` の形状も含まれます。バイト列とその `content-encoding: zstd` ヘッダーは変更されずに転送されます。shunt は加えて、メトリクス・ログ・スパン用にリクエストの `model` を読み取るためだけに、メモリ上でコピーをデコードします。shunt がデコードできないボディでも中継自体は問題なく行われ、劣化するのは `model` ラベルが `unknown` になることだけで、理由を示す警告が出ます。
- **厳密一致のモデルルーティング。** 一意な厳密一致宣言は Responses ネイティブまたは Anthropic Messages のプロバイダーを 1 つ選べます。プレフィックスのみ・非厳密・未一致は固定ネイティブプロバイダーへフォールバックし、曖昧な宣言は上流送信前に拒否します。
- **Anthropic 変換は厳格かつ有界です。** instructions、テキストと URL/data-URL 画像、関数ツール・呼び出し・結果、tool choice、生成制御、reasoning effort を扱います。HTTP と WebSocket は同じ JSON/SSE 変換器を使います。`end_turn`・`stop_sequence`・`tool_use` は completed、`max_tokens` は incomplete、不正出力・未知の終了理由・ストリームエラー・早期 EOF は failed です。cache read/write の入力トークンも usage に含みます。
- **Collaboration は明示的に有効化します。** `collaboration = true` にすると、厳密一致の Anthropic ルートが宣言済み V2 `collaboration` ツールと平文 `agent_message` task エンベロープを橋渡しします。許可された呼び出しは JSON と SSE レスポンスで collaboration namespace に復元されます。ネイティブ Responses トラフィックはフラグに関係なくバイト単位で不透明なままです。
- **損失を伴う入力は送信前に失敗します。** `previous_response_id`、暗号化 reasoning/compaction 状態、hosted/custom tool、remote file id、不正な tool 関係、未対応フィールドは推測せず拒否します。compaction V2 トリガーを含むネイティブ Responses の透過転送は影響を受けません。
- **隠れたリカバリーはありません。** 暗号文だけの変換 agent task は認証情報解決やネットワーク送信の前に失敗します。shunt は復号・キャッシュ・永続化・課金リカバリー呼び出しを行いません。平文履歴を渡すか、ネイティブ Responses ルートを使ってください。
- **枯渇時はそのまま中継。** プールされたすべてのアカウントを試行し、少なくとも 1 つの上流レスポンスが返っていた場合、shunt はその最後のレスポンスを Anthropic 形式のエラーへ作り直すのではなく、変更せずに中継します。Responses のクライアントは、実際の ChatGPT バックエンドから受け取るはずの生の形を期待するためです。
- **クォータを意識したローテーションには上限があります。** 単独の `429` は一時的なスロットリングです。shunt は、限定されたレスポンス本文に正確な構造化ハードクォータの証拠（`usage_limit_exceeded` または `insufficient_quota`）が含まれている場合のみ、アカウントを有限の内部クールダウンに抑制します。不正、曖昧、サイズ超過、または中断された本文は未検証のままです。有効な `Retry-After` のデルタ秒（小数を安全に切り上げ）と HTTP 日付値は有限の上限内で尊重されます。候補を使い切った場合、最終的な上流ステータス、本文、安全な `Retry-After` メタデータがそのまま残ります。アカウントのローテーションは出力開始前のみ可能であり、無関係なルートレベルのフェイルオーバー動作は変更されません。
- **Compaction は失敗時に閉じ、不透明なままです。** レガシーの HTTP 専用 `POST /v1/responses/compact` は、本文と継続状態を公式 OpenAI API キーバックエンドだけへバイト単位で転送します。ChatGPT/Codex バックエンドはこの独立エンドポイントを提供しなくなったため、ChatGPT OAuth の対象は認証情報の解決やネットワーク送信より前にローカルで失敗します。現在の Codex compaction V2 は通常の Responses ストリームへ末尾の `compaction_trigger` を送り、ネイティブルートはそれを変更せず透過転送します。不正、曖昧、変換が必要、または未対応の対象もネットワーク送信前に拒否されます。shunt は継続状態を復号せず、要約を合成せず、リクエスト履歴を保存しません。
- **ゲートウェイ自身のエラーは OpenAI 形式。** 失敗が shunt 自身のものである場合 — 不正または欠落したクライアントトークン（`401`）、上流レスポンスのないプールの解決不能（`502`）、サイズ超過のリクエストボディ、未設定のエンドポイント — shunt は同じステータスコードのまま、OpenAI Responses のエラー形（`{"error":{"message":…,"type":…,"code":null}}`）で返します。これにより Codex CLI は、Anthropic の `{"type":"error",…}` エンベロープではなく自身のエラー経路でパースできます。中継される*上流*のエラー（バックエンドからの 429/4xx/5xx）は、引き続きそのまま通過します。
- **2 つの受信トランスポート。** HTTP `POST` はバイト忠実性を維持し、WebSocket `GET` は有界イベント配信を追加します。これはプロバイダーの outbound `websocket = true` 設定には依存しません。
- **共有ディスパッチ境界。** HTTP と WebSocket は同じ厳密一致リゾルバーとプロバイダー単位の資格情報フィルタリングを使います。出力開始後にプロバイダー移行や再生はなく、出力前のアカウント切り替えだけが対象です。

## セキュリティ

- ループバックを超えるものでは、このエンドポイントを `[server.auth]` でゲートしてください — プロバイダーはリクエストごとに実際の Codex ベアラーを注入します。
- クライアント自身の認証情報が Codex バックエンドへ届くことはありません。パススルーは Codex CLI 自身のリクエストヘッダーをそのまま転送し、差し替えるのは選択されたプールアカウントのベアラーと `chatgpt-account-id` だけです（shunt のクライアントトークンヘッダー、`[server.admin]` の認証情報ヘッダー、`cookie` ヘッダー全体、内部用の `x-shunt-inbound-client` ラベル、クライアントの `Authorization`/`chatgpt-account-id`、そして `x-api-key` はすべて取り除かれ、転送されることはありません）。
- 起動時に一度だけ決まるのは、エンドポイントの **HTTP ルート登録**だけです。`[server.codex_endpoint]` の実行時のオン/オフ切り替えは、それらのパスを追加・削除するには再起動が必要である旨の警告をログに出力します。テーブルが*保持している*内容はすべてホットリロードされます — 対象の `provider` と `[[server.codex_endpoint.routes]]` のモデルテーブル全体はリクエストごとにライブ設定から読まれるため、ルートの追加・編集・削除はリロードで反映されます。
