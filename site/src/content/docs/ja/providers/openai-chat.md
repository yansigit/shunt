---
title: OpenAI 互換 (Chat Completions)
description: API キーだけでマッピングしたモデルを任意の OpenAI Chat Completions エンドポイントへルーティングする。
---

**OpenAI 互換 (Chat Completions)** は名前付きプロバイダーではなく汎用のプロバイダー種別です:
`kind = "openai_chat"` は OpenAI Chat Completions API（`POST /chat/completions`）を提供する
任意のバックエンドに shunt を向けます。shunt は Claude Code の Anthropic Messages リクエストを
その形状へ変換します — ストリーミングも含みます。カスタムバックエンドは `kind`、
`base_url`、API キーを明示します。別製品の [Command Code API](/ja/providers/command-code/)
にはこの kind の `commandcode` プリセットがあり、サブスクリプション転送とは異なります。

## upstream を設定する

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

順序付き `[[upstreams]]` は shunt の組み込みプロバイダーを置き換えるため、フォールバック先として
残す `anthropic` デフォルトも設定で宣言する必要があります（`server.default_provider` のデフォルトは
`anthropic` です）。

レガシーな `[providers.chat]` テーブル形式も引き続きサポートされます: auth マップの代わりに
`kind`、`base_url`、`auth = "api_key"`、`api_key_env = "CHAT_API_KEY"` を設定してください。
1 つのファイルで `[[upstreams]]` と `[providers.*]` を混在させないでください。

`kind = "openai_chat"` が受け付けるのは `auth = "api_key"` のみです。起動時に他の資格情報モードは
拒否されます — アダプターがリクエストごとに設定されたキーを注入し、それ以外の資格情報経路は
ありません。

## 資格情報

```bash
export CHAT_API_KEY='...'
```

キーを設定ファイルに書き込まないでください。`shunt check` は設定の構造を検証しますが、キーの値は
読み取りません — `CHAT_API_KEY` が未設定のまま `chat` にルーティングされた最初のリクエストは
認証エラーを返します。

## base URL の文法

`base_url` は単純な `http://` または `https://` ルートでなければなりません: クエリ文字列、
フラグメント、userinfo（`user:pass@`）、ドットパスセグメント、空白、バックスラッシュは許されません。
shunt はちょうど 1 つの `/chat/completions` パスを後ろに追加します — すでに `/chat/completions`
で終わるルートはそのまま維持されます — そのため `https://api.example.com/v1` と
`https://api.example.com/v1/chat/completions` は等価です。`shunt check` が起動時に同じ文法を
検証します。

## アダプターが運ぶもの

変換はデフォルト拒否のホワイトリスト方式のため、対応していないリクエストは静かに劣化するのではなく
型付き 400 でフェイルクローズします:

- **テキストターン**（system、user、assistant）と `max_tokens`、`temperature`、`top_p`、
  `stop_sequences`。未対応のトップレベルのリクエストフィールドは拒否されます。単一のテキスト
  ペイロードは UTF-8 で 8 MiB までに制限されます。
- **画像**: user メッセージ内の base64 データまたは URL。
- **ツール**: 宣言、`tool_choice`、ペアになった `tool_use`/`tool_result` ターン。リクエストローカルの
  id レジストリを使うため、並行リクエストが状態を共有することはありません。
- **ストリーミング**: Anthropic SSE へ変換され、使用量が保持されます。ストリーミングリクエストは
  upstream 側の `stream_options.include_usage` 契約を強制します。集約された unary 応答は 32 MiB
  まで、ストリーミングマシンはターンごとに最大 8 MiB のセマンティックバイトを保持します。

## 失敗とキャンセルのセマンティクス

リクエストが upstream に届いた可能性がある以降、生成 POST が再送されることはありません。また
リダイレクトは一切拒否されるため、注入された bearer が設定済みオリジンの外へ持ち出されることは
ありません。upstream の非成功ステータスはゲートウェイ所有のエラーとして表面化し、120 秒を超える
読み取りアイドルはそのターンを失敗させます。リクエストをキャンセルすると upstream 接続が閉じられ、
アドミッションスロットが解放されます。

これらの保証はリポジトリのテストスイートにある合成コンフォーマンスフィクスチャで検証されています。
実プロバイダーの動作についてここで主張・検証するものではありません。

## プロトコルの制限

ツール引数はリクエストごとにバッファリングし、終了境界で一度だけ解析します。
ツールブロックは `message_start` の後に初回到着順で出力されます。上限は128呼び出し、
呼び出しごとに1 MiBの引数です。SSEイベントと未完了の残余ペイロードには、それぞれ
独立した8 MiBの上限があり、フレーム区切りは除外します。累積セマンティック予算の8 MiBは
テキスト、推論、ツール識別子と引数を数え、残余バッファは含みません。

成功には対応する終了理由と `[DONE]` の両方が必要で、EOFだけでは成功しません。
`stop`、`length`、`tool_calls` はそれぞれ `end_turn`、`max_tokens`、`tool_use`
に変換され、他の理由は拒否されます。既にデコードされた後続フレームや残余バイトはエラーです。
終了後の将来のバイトやHTTP EOFは待たず、配信済みテキストは取り消せません。
トークンカウンターは `0..=i64::MAX` の整数でなければなりません。

プレーンテキストのthinkingに対応します。応答の `reasoning` と `reasoning_content` は
プロバイダー拡張であり、普遍的なOpenAI契約ではありません。競合する推論や署名付き・編集済み
推論は拒否します。読み取りアイドル制限は新しい設定キーなしで120秒に固定されています。
このアダプターの `count_tokens` は別の戦略を設定しても常にローカル推定値を使います。
資格情報ファイルへの書き込みは行いません。

## 検証

```bash
shunt check    # -> config ok
shunt run
curl -sS http://127.0.0.1:3001/v1/messages \
  -H 'anthropic-version: 2023-06-01' \
  -H 'content-type: application/json' \
  -d '{"model":"gpt-5.4","max_tokens":16,"messages":[{"role":"user","content":"Reply with OK."}]}'
```
