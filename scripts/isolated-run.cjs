#!/usr/bin/env node

// Run a command with an isolated OpenCodeX home while comparing the user's
// real OpenCodeX config fingerprint before/after. This is a guard, not an OS
// sandbox: callers must not pass commands which deliberately target production.
const fs = require('fs');
const os = require('os');
const path = require('path');
const crypto = require('crypto');
const cp = require('child_process');

const CURSOR_KEYS = [
  'CURSOR_AUTH_TOKEN',
  'SHUNT_CURSOR_AUTH_TOKEN',
  'SHUNT_CURSOR_AGENT_BASE_URL',
  'SHUNT_CURSOR_CLIENT_VERSION',
];
const SAFE_PORT = '31987';

function productionHome() {
  return path.join(os.homedir(), '.opencodex');
}

/** Snapshot a home supplied by a fixture, or the fixed production home. */
function snapshotHome(root = productionHome()) {
  let homeStat;
  try {
    homeStat = fs.statSync(root);
  } catch (error) {
    if (error.code === 'ENOENT') return { missing: true };
    throw error;
  }
  if (!homeStat.isDirectory()) throw new Error(`OpenCodeX home is not a directory: ${root}`);

  const configPath = path.join(root, 'config.json');
  let configStat;
  try {
    configStat = fs.statSync(configPath);
  } catch (error) {
    // A missing home is the only tolerated ENOENT. A missing config in an
    // existing home is unexpected and must fail closed.
    if (error.code === 'ENOENT') throw new Error(`OpenCodeX config is missing: ${configPath}`);
    throw error;
  }
  if (!configStat.isFile()) throw new Error(`OpenCodeX config is not a file: ${configPath}`);
  const raw = fs.readFileSync(configPath);
  const names = fs.readdirSync(root).filter((name) =>
    name.startsWith('config.json.invalid-') || /backup|bak/.test(name),
  ).sort();
  return {
    missing: false,
    mtimeMs: configStat.mtimeMs,
    sha256: crypto.createHash('sha256').update(raw).digest('hex'),
    names,
  };
}

function lockPath() {
  const user = typeof process.getuid === 'function' ? String(process.getuid()) : os.userInfo().username;
  return path.join(os.tmpdir(), `shunt-isolated-run-${user}.lock`);
}

// The lock is removed only by its owner. In particular, an existing lock is
// never treated as stale and never removed by another run.
function tryAcquireLock(lock = lockPath()) {
  try {
    fs.mkdirSync(lock, { mode: 0o700 });
    const token = `${process.pid}:${crypto.randomBytes(16).toString('hex')}`;
    fs.writeFileSync(path.join(lock, 'owner'), token, { flag: 'wx' });
    return () => {
      try {
        if (fs.readFileSync(path.join(lock, 'owner'), 'utf8') !== token) return;
        fs.unlinkSync(path.join(lock, 'owner'));
        fs.rmdirSync(lock);
      } catch (error) {
        if (error.code !== 'ENOENT') throw error;
      }
    };
  } catch (error) {
    if (error.code === 'EEXIST') return null;
    throw error;
  }
}

function acquireLock(lock = lockPath()) {
  return tryAcquireLock(lock);
}

function childEnvironment() {
  const env = {
    ...process.env,
    OPENCODEX_HOME: fs.mkdtempSync(path.join(os.tmpdir(), 'shunt-isolated-run-')),
    OPENCODEX_PORT: SAFE_PORT,
    SHUNT_PORT: '31711',
    MOCK_PORT: '31712',
  };
  for (const key of CURSOR_KEYS) delete env[key];
  return env;
}

function run(argv = process.argv.slice(2)) {
  if (argv.length === 0) {
    console.error('usage: isolated-run.cjs COMMAND [ARG ...]');
    return 2;
  }
  const release = acquireLock();
  if (!release) {
    console.error('BUSY: another isolated run owns the lock; no command started.');
    return 73;
  }
  let before;
  let child;
  try {
    try {
      before = snapshotHome();
    } catch (error) {
      console.error(`STOP: unable to snapshot production OpenCodeX state: ${error.message}`);
      return 90;
    }
    const env = childEnvironment();
    console.error(`Isolated OPENCODEX_HOME preserved for diagnostics: ${env.OPENCODEX_HOME}`);
    child = cp.spawnSync(argv[0], argv.slice(1), { env, stdio: 'inherit' });
    let after;
    try {
      after = snapshotHome();
    } catch (error) {
      console.error(`STOP: unable to verify production OpenCodeX state: ${error.message}`);
      return 90;
    }
    if (JSON.stringify(after) !== JSON.stringify(before)) {
      console.error('STOP: production OpenCodeX state changed');
      return 90;
    }
    console.error('Production OpenCodeX config mtime/SHA and backup inventory unchanged.');
    if (child.error) console.error(child.error.message);
    return child.status == null ? 1 : child.status;
  } finally {
    release();
  }
}

module.exports = { CURSOR_KEYS, SAFE_PORT, acquireLock, childEnvironment, productionHome, run, snapshotHome, tryAcquireLock };

if (require.main === module) process.exitCode = run();
