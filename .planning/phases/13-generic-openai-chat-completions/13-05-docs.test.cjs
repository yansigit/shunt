const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');

test('openai_chat documentation surfaces and locales', () => {
  const files = ['README.md','README.ko.md','README.ja.md','README.zh-CN.md','docs/openai-chat-translation.md'];
  for (const locale of ['', 'ko/', 'ja/', 'zh-cn/']) {
    for (const page of ['providers/openai-chat.md','reference/configuration.md','guides/providers.md']) {
      files.push(`site/src/content/docs/${locale}${page}`);
    }
  }
  for (const file of files) {
    assert.ok(fs.existsSync(file), `missing documentation: ${file}`);
    assert.match(fs.readFileSync(file,'utf8'), /openai_chat/, `missing provider contract: ${file}`);
  }
  const english = fs.readFileSync('site/src/content/docs/providers/openai-chat.md','utf8');
  const config = english.match(/```toml\n([\s\S]*?)```/)[1];
  for (const locale of ['ko','ja','zh-cn']) {
    const text = fs.readFileSync(`site/src/content/docs/${locale}/providers/openai-chat.md`,'utf8');
    assert.equal(text.match(/```toml\n([\s\S]*?)```/)[1], config, `config drift in ${locale}`);
    assert.doesNotMatch(text, /\]\([^)]*#[^)]*\)/, 'new locale pages use fragment-free links');
  }
  assert.match(fs.readFileSync('site/src/lib/i18n.ts','utf8'), /providers\/openai-chat/);
});
