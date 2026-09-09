const assert = require('assert/strict');
const cp = require('child_process');
const fs = require('fs');
const os = require('os');
const path = require('path');
const { snapshotHome, acquireLock, tryAcquireLock, SAFE_PORT } = require('./isolated-run.cjs');

const temp = fs.mkdtempSync(path.join(os.tmpdir(), 'shunt-isolated-fixture-'));
const home = path.join(temp, 'home');
fs.mkdirSync(home);
fs.writeFileSync(path.join(home, 'config.json'), Buffer.from('{"fixture":true}\n'));
fs.writeFileSync(path.join(home, 'config.json.invalid-before'), 'old');
fs.writeFileSync(path.join(home, 'catalog-backup.json'), 'old');
const first = snapshotHome(home);
assert.equal(first.missing, false);
assert.equal(first.names.length, 2);
assert.equal(first.sha256.length, 64);
fs.appendFileSync(path.join(home, 'config.json'), 'changed');
assert.notEqual(snapshotHome(home).sha256, first.sha256);
const changed = snapshotHome(home);
fs.utimesSync(path.join(home, 'config.json'), new Date(1000000), new Date(1000000));
assert.notEqual(snapshotHome(home).mtimeMs, changed.mtimeMs);
fs.writeFileSync(path.join(home, 'config.json.invalid-new'), 'fixture');
assert.equal(snapshotHome(home).names.length, 3);
assert.deepEqual(snapshotHome(path.join(temp, 'missing')), { missing: true });
fs.mkdirSync(path.join(temp, 'empty'));
assert.throws(() => snapshotHome(path.join(temp, 'empty')), /config is missing/);

const wrapper = path.join(__dirname, 'isolated-run.cjs');
const inherited = cp.spawnSync(process.execPath, [wrapper, process.execPath, '-e', [
  "const fs=require('fs');",
  "if (!process.env.OPENCODEX_HOME || process.env.OPENCODEX_HOME === 'caller-home' || !fs.statSync(process.env.OPENCODEX_HOME).isDirectory() || process.env.OPENCODEX_PORT !== '31987') process.exit(3);",
  "if (process.env.SHUNT_PORT !== '31711' || process.env.MOCK_PORT !== '31712') process.exit(5);",
  "for (const k of ['CURSOR_AUTH_TOKEN','SHUNT_CURSOR_AUTH_TOKEN','SHUNT_CURSOR_AGENT_BASE_URL','SHUNT_CURSOR_CLIENT_VERSION']) if (process.env[k]) process.exit(4);",
].join('')], {
  env: { ...process.env, CURSOR_AUTH_TOKEN: 'secret', OPENCODEX_HOME: 'caller-home', OPENCODEX_PORT: '10100', SHUNT_PORT: '10100', MOCK_PORT: '10100' },
});
assert.equal(inherited.status, 0);

const lock = path.join(temp, 'lock');
const release = acquireLock(lock);
assert.throws(() => fs.mkdirSync(lock), /EEXIST/);
assert.equal(tryAcquireLock(lock), null);
release();
assert.doesNotThrow(() => fs.mkdirSync(lock));
fs.rmdirSync(lock);

const fail = cp.spawnSync(process.execPath, [wrapper], { encoding: 'utf8' });
assert.equal(fail.status, 2);
assert.match(fail.stderr, /usage/);
const childFailure = cp.spawnSync(process.execPath, [wrapper, process.execPath, '-e', 'process.exit(7)'], { encoding: 'utf8' });
assert.equal(childFailure.status, 7);
const missingCommand = cp.spawnSync(process.execPath, [wrapper, path.join(temp, 'absent-command')], { encoding: 'utf8' });
assert.equal(missingCommand.status, 1);
const ownRelease = acquireLock();
assert.ok(ownRelease);
try {
  const busy = cp.spawnSync(process.execPath, [wrapper, process.execPath, '-e', 'process.exit(99)'], { encoding: 'utf8', timeout: 2000 });
  assert.equal(busy.status, 73);
  assert.match(busy.stderr, /no command started/);
} finally { ownRelease(); }
assert.equal(SAFE_PORT, '31987');
console.log('isolated-run fixtures passed');
