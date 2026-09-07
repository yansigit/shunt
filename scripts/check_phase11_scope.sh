#!/usr/bin/env bash
# Phase 11 release scope gate. Bash 3.2/macOS compatible; Node uses only built-ins.
# Git-visible paths only: ignored build outputs are not a filesystem forensic scan.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
node <<'NODE'
const fs = require('node:fs');
const cp = require('node:child_process');
const git = (...args) => cp.execFileSync('git', args, {encoding:'utf8', maxBuffer:32*1024*1024});
const phase = '.planning/phases/11-antigravity-protocol-and-credential-hardening/';
const baseline = git('log','--reverse','--diff-filter=A','--format=%H','--',phase+'11-01-PLAN.md').trim().split('\n')[0];
if (!/^[0-9a-f]{40}$/.test(baseline || '')) throw Error('Cannot resolve Phase 11 baseline');
const roots = ['README.md','README.ko.md','README.ja.md','README.zh-CN.md'];
const sites = ['', 'ko/', 'ja/', 'zh-cn/'].map(locale => 'site/src/content/docs/'+locale+'providers/antigravity.mdx');
const allowed = new Set([
  ...roots, ...sites,
  'docs/antigravity-tool-identities.md','docs/notes/antigravity-daily-host.md',
  'scripts/check_phase11_scope.sh',
  '.planning/STATE.md','.planning/ROADMAP.md','.planning/REQUIREMENTS.md',
  // Exact mandatory GSD regression and post-execution review evidence, not broad exceptions.
  '.planning/phases/09-provider-conformance-foundation/09-VERIFICATION.md',
  ...['11-CONTEXT.md','11-RESEARCH.md','11-PATTERNS.md','COVERAGE.md','11-VALIDATION.md','11-VERIFICATION.md','11-REVIEW.md'].map(p=>phase+p),
  ...Array.from({length:7},(_,i)=>String(i+1).padStart(2,'0')).flatMap(n=>[phase+'11-'+n+'-PLAN.md',phase+'11-'+n+'-SUMMARY.md']),
  'src/adapters/gemini/mod.rs',
  'src/adapters/responses/inbound.rs','src/adapters/responses/request.rs','src/adapters/responses/websocket.rs',
  'src/auth/antigravity/auth.rs','src/auth/antigravity/catalog.rs','src/auth/mod.rs','src/auth/shared.rs',
  'src/model/antigravity_request.rs','src/model/gemini.rs','src/model/gemini_request.rs',
  'src/retry.rs','src/server.rs','src/server/antigravity_replay_tests.rs',
  'src/server/antigravity_replay_tests/fixtures.rs','src/upstream_timeout.rs',
  'tests/antigravity_catalog.rs','tests/antigravity_tool_scope.rs','tests/gemini_conformance.rs'
]);
const ignored = p => p.startsWith('.gsd/') || p === '.planning/config.json' || p === '.planning/milestone.lock';
const split = s => s.split('\0').filter(Boolean);
const untracked = split(git('ls-files','--others','--exclude-standard','-z'));
const paths = new Set([
  ...split(git('diff','--name-only','--no-renames','-z',baseline,'HEAD')),
  ...split(git('diff','--name-only','--no-renames','-z','HEAD')),
  ...untracked
]);
let failures = 0;
const fail = (p, reason) => { console.error(JSON.stringify(p)+': '+reason); failures++; };
for (const p of paths) if (!ignored(p) && !allowed.has(p)) fail(p,'outside exact Phase 11 scope');
const required = ['daily-cloudcode-pa.googleapis.com','cloudcode-pa.googleapis.com','fetchAvailableModels','streamGenerateContent?alt=sse','call_antigravity_v2_','401','Google AI Studio Web','antigravity_cli'];
for (const p of [...roots,...sites]) {
  if (!fs.existsSync(p)) { fail(p,'missing maintained documentation'); continue; }
  const text=fs.readFileSync(p,'utf8');
  for (const term of required) if (!text.includes(term)) fail(p,'missing contract term '+term);
  if (!/Google AI Studio Web[^\n]*(?:excluded|제외|対象外|不在支持范围)/.test(text)) fail(p,'missing explicit exclusion');
}
const synthetic = value => /^(?:synthetic|test|fake|dummy|fixture|mock|example|placeholder|native-fixture|pre-401|post-401|refreshed|stored-refresh|token-[ab])(?:[-_:./]|$)/i.test(value) || /^(?:account-[ab]|catalog-token|new-account-token)$/.test(value);
function entropy(value) {
  const counts=new Map();
  for (const c of value) counts.set(c,(counts.get(c)||0)+1);
  return [...counts.values()].reduce((sum,n)=>sum-(n/value.length)*Math.log2(n/value.length),0);
}
function secretKind(line) {
  for (const match of line.matchAll(/(?:ya29\.[A-Za-z0-9_-]{12,}|AIza[A-Za-z0-9_-]{30,}|eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,})/g)) {
    if (!synthetic(match[0])) return 'credential-shaped token';
  }
  for (const match of line.matchAll(/Bearer[ \t]+([A-Za-z0-9._~+/-]+)/g)) {
    const value=match[1];
    if (value.length >= 8 && !synthetic(value) && !/^(?:payloads?|token|bearer|credentials?|can|from|to|is|must|never|refusing)$/i.test(value)) return 'literal bearer payload';
  }
  // Heuristic, not a guarantee: high-entropy quoted payloads may be secrets.
  // Identifier-like words, explicit synthetic placeholders and contextual SHA evidence are exempt.
  for (const match of line.matchAll(/["'`]([A-Za-z0-9_+./=-]{40,})["'`]/g)) {
    const value=match[1];
    if (synthetic(value) || allowed.has(value) || /^[a-z][a-z0-9]*(?:_[a-z0-9]+)+$/.test(value)) continue;
    if (/^[a-f0-9]{40,64}$/.test(value) && /sha|hash|commit|baseline|fingerprint|state_head|v1:/i.test(line)) continue;
    if (entropy(value)>4.3) return 'high-entropy quoted value (heuristic)';
  }
  return null;
}
// Deterministic scanner probes use assembled, non-credential test strings.
for (const probe of [
  ['ya','29.'].join('')+'a'.repeat(20),
  ['AI','za'].join('')+'b'.repeat(32),
  ['ey','J'].join('')+'c'.repeat(12)+'.'+'d'.repeat(12)+'.'+'e'.repeat(12),
  'Bearer '+'Ab9Zp2Qt7Lm4Vc8N',
  '"'+['Ab9+/Qx2Lm7=zT4_pR6.vC8','Yk0Ne3Hs5Wd1FjUoGiBaPqS'].join('')+'"'
]) if (!secretKind(probe)) throw Error('Secret-scanner negative probe failed');
for (const probe of ['access_token','refresh_token','Bearer synthetic-token','"synthetic-fixture-value"'])
  if (secretKind(probe)) throw Error('Secret-scanner placeholder probe failed');
for (const p of paths) {
  if (ignored(p) || !allowed.has(p)) continue;
  // Separate committed and uncommitted additions so a later deletion cannot hide a committed secret.
  const patches = [
    git('diff','--no-ext-diff','--no-renames','--unified=0',baseline,'HEAD','--',p),
    git('diff','--no-ext-diff','--no-renames','--unified=0','HEAD','--',p)
  ];
  let lines = patches.flatMap(patch=>patch.split('\n').filter(l=>l.startsWith('+')&&!l.startsWith('+++')).map(l=>l.slice(1)));
  if (untracked.includes(p) && fs.statSync(p).isFile()) lines.push(...fs.readFileSync(p,'utf8').split('\n'));
  for (let i=0;i<lines.length;i++) {
    const line=lines[i];
    const kind=secretKind(line);
    if (kind) fail(p,'added-line '+(i+1)+' '+kind); // Never print the value.
    if (/^(?:src|tests)\//.test(p) && /SAPISIDHASH|MakerSuite|aistudio\.google\.com|alkalimakersuite|google_ai_studio|GoogleAiStudio/.test(line))
      fail(p,'excluded Google AI Studio implementation marker');
    if (/^(?:README|docs\/|site\/)/.test(p) && /Google AI Studio Web/.test(line) &&
        !/excluded|exclud|not supported|제외|対象外|不在支持范围/i.test(line))
      fail(p,'unqualified excluded-product support claim');
  }
}
if (failures) { console.error('Phase 11 gate FAILED: '+failures+' finding(s).'); process.exit(1); }
console.log('Phase 11 gate passed: baseline '+baseline+', '+paths.size+' Git-visible changed paths; 8 maintained docs surfaces; exclusions and heuristic secret scan.');
NODE
