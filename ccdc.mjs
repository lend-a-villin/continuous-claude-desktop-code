#!/usr/bin/env node
// CCDC (Continuous Claude Desktop Code): let every account in the Claude desktop app see the same local Code sessions.
//
// The desktop app stores Code session metadata per account/org:
//   <userData>/claude-code-sessions/<accountUuid>/<orgUuid>/local_<id>.json
// The transcripts live in ~/.claude/projects and are shared by every account.
//
// The app reads that folder only when it starts and when someone signs in, so:
//   - on sign-out, the newest copy of every session is copied into every
//     account/org folder (only files that changed), so whichever account signs
//     in next finds them all;
//   - once the app has signed in and read its folder, the copies in the other
//     folders are kept by default ("keepCopies", fewer SSD writes) or removed
//     again when "keepCopies" is false.
// Sign-out and sign-in are read from the app's own log (Logs\main.log).
//
// Settings live in ccdc.json next to this script:
//   { "keepCopies": true, "speed": "fast" | "normal" | "gentle", "language": "", "aliases": {} }
//
// Usage:
//   node ccdc.mjs status
//   node ccdc.mjs watch [--log <file>]
//   node ccdc.mjs estimate [--json]
//   node ccdc.mjs replicate [--dry-run]
//   node ccdc.mjs move [--to <account>[/<org>]] [--dry-run]

import fs from "node:fs";
import path from "node:path";

const SESSIONS_DIR = "claude-code-sessions";
const LOCAL_RE = /^local_([0-9a-f-]{36})\.json$/i;
const DELETED_RE = /^deleted_([0-9a-f-]{36})$/i;
// lines in the app's main.log (the arrows and dashes are matched loosely)
const LOGOUT_RE = /Login-state transition \(loggedOut: false \S+ true/;
const INIT_RE = /\[LocalSessionManager\] Initialization succeeded \S+ accountId=([0-9a-f-]{36}), orgId=([0-9a-f-]{36})/i;
const DISK_RESERVE = 512 * 1024 * 1024; // keep this much free after copying
const HEAD_BYTES = 4096; // lastActivityAt sits near the start of a session file

// copy MB/s and deletes/s per speed setting (Infinity = no limit)
const SPEEDS = {
  fast: { copyMBps: Infinity, deletesPerSec: Infinity },
  normal: { copyMBps: 60, deletesPerSec: 500 },
  gentle: { copyMBps: 15, deletesPerSec: 100 },
};
// assumed disk rates until CCDC has measured this PC
const DEFAULT_RATES = { copyMBps: 150, deletesPerSec: 1500 };

const SCRIPT_DIR = path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1"));
const SETTINGS_FILE = path.join(SCRIPT_DIR, "ccdc.json");
const STATE_FILE = process.env.LOCALAPPDATA ? path.join(process.env.LOCALAPPDATA, "CCDC", "state.json") : null;

function parseArgs(argv) {
  const args = { _: [] };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--dry-run") args.dryRun = true;
    else if (a === "--json") args.json = true;
    else if (a === "--data-dir") args.dataDir = argv[++i];
    else if (a === "--app-log") args.appLog = argv[++i];
    else if (a === "--to") args.to = argv[++i];
    else if (a === "--account") args.account = argv[++i];
    else if (a === "--org") args.org = argv[++i];
    else if (a === "--settle") args.settleMs = Number(argv[++i]);
    else if (a === "--log") logFile = argv[++i];
    else if (a === "-h" || a === "--help") args.help = true;
    else args._.push(a);
  }
  return args;
}

const short = (s) => (s ? s.slice(0, 8) : "none");
const mb = (n) => `${Math.round(n / 1024 / 1024)}MB`;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const freeMemory = () => globalThis.gc?.(); // available when started with --expose-gc
let logFile = null; // --log <file>: watch also appends here (the tray app reads it)
const log = (msg) => {
  const line = `[${new Date().toLocaleTimeString()}] ${msg}`;
  console.log(line);
  if (logFile) {
    try {
      fs.appendFileSync(logFile, line + "\n", "utf8");
    } catch {}
  }
};

// ---------- settings and measured rates ----------

function readJson(file) {
  try {
    return JSON.parse(fs.readFileSync(file, "utf8").replace(/^﻿/, ""));
  } catch {
    return null;
  }
}

function loadSettings() {
  const j = readJson(SETTINGS_FILE) ?? {};
  const speed = SPEEDS[j.speed] ? j.speed : "fast";
  // copies are kept unless turned off: deleting them means rewriting every session at each sign-out
  return { keepCopies: j.keepCopies !== false, speed, aliases: j.aliases ?? {} };
}

function loadRates() {
  const r = (STATE_FILE && readJson(STATE_FILE)?.rates) || {};
  return {
    copyMBps: r.copyMBps > 0 ? r.copyMBps : DEFAULT_RATES.copyMBps,
    deletesPerSec: r.deletesPerSec > 0 ? r.deletesPerSec : DEFAULT_RATES.deletesPerSec,
  };
}

// remembers how fast this PC copied/deleted at "fast" speed, for estimates
function saveRate(key, value) {
  if (!STATE_FILE || !(value > 0) || !Number.isFinite(value)) return;
  try {
    const s = readJson(STATE_FILE) ?? {};
    s.rates = s.rates ?? {};
    const old = s.rates[key];
    s.rates[key] = old > 0 ? Math.round((old * 0.5 + value * 0.5) * 10) / 10 : Math.round(value * 10) / 10;
    fs.mkdirSync(path.dirname(STATE_FILE), { recursive: true });
    fs.writeFileSync(STATE_FILE, JSON.stringify(s, null, 2), "utf8");
  } catch {}
}

// ---------- locating the app's folders ----------

function findDataDirs() {
  const found = [];
  const local = process.env.LOCALAPPDATA;
  if (local) {
    const base = path.join(local, "Claude-Data");
    if (fs.existsSync(base)) {
      for (const d of fs.readdirSync(base, { withFileTypes: true })) {
        if (d.isDirectory()) found.push(path.join(base, d.name));
      }
    }
  }
  if (process.env.APPDATA) found.push(path.join(process.env.APPDATA, "Claude"));
  return found.filter((d) => fs.existsSync(path.join(d, SESSIONS_DIR)));
}

function resolveDataDir(opt) {
  opt ||= process.env.CCDC_DATA_DIR; // lets the tray app be tried on a copy
  if (opt) {
    if (!fs.existsSync(path.join(opt, SESSIONS_DIR))) throw new Error(`No ${SESSIONS_DIR} folder in ${opt}`);
    return opt;
  }
  const dirs = findDataDirs();
  if (dirs.length === 0) throw new Error("Claude desktop data folder not found. Pass --data-dir.");
  if (dirs.length > 1) throw new Error(`Several data folders found, pass --data-dir:\n  ${dirs.join("\n  ")}`);
  return dirs[0];
}

// The app logs to %LOCALAPPDATA%\Claude\Logs\main.log (older builds: <userData>\logs).
function resolveAppLog(dataDir, opt) {
  opt ||= process.env.CCDC_APP_LOG;
  if (opt) return opt;
  const candidates = [
    process.env.LOCALAPPDATA && path.join(process.env.LOCALAPPDATA, "Claude", "Logs", "main.log"),
    path.join(dataDir, "logs", "main.log"),
  ].filter((p) => p && fs.existsSync(p));
  if (candidates.length === 0) return null;
  return candidates.reduce((a, b) => (fs.statSync(b).mtimeMs > fs.statSync(a).mtimeMs ? b : a));
}

function parseLogLine(line) {
  if (LOGOUT_RE.test(line)) return { loggedOut: true, account: null, org: null };
  const m = INIT_RE.exec(line);
  if (m) return { loggedOut: false, account: m[1].toLowerCase(), org: m[2].toLowerCase() };
  return null;
}

// Sign-in state as of the end of the app log: { loggedOut, account, org } or null.
// Reads the log backwards in small chunks instead of loading the whole file.
function readAppState(appLog) {
  if (!appLog) return null;
  let fd;
  try {
    fd = fs.openSync(appLog, "r");
    const size = fs.fstatSync(fd).size;
    const CHUNK = 64 * 1024;
    let end = size;
    let tail = "";
    while (end > 0) {
      const start = Math.max(0, end - CHUNK);
      const buf = Buffer.alloc(end - start);
      fs.readSync(fd, buf, 0, buf.length, start);
      const text = buf.toString("utf8") + tail;
      const lines = text.split(/\r?\n/);
      tail = start > 0 ? lines.shift() : ""; // possibly cut line, finish it with the next chunk
      for (let i = lines.length - 1; i >= 0; i--) {
        const st = parseLogLine(lines[i]);
        if (st) return st;
      }
      end = start;
    }
    return tail ? parseLogLine(tail) : null;
  } catch {
    return null;
  } finally {
    if (fd !== undefined) fs.closeSync(fd);
  }
}

function signedInAccount(dataDir, appLog) {
  const st = readAppState(appLog);
  if (st) return st.loggedOut ? null : st.account;
  const j = readJson(path.join(dataDir, "config.json"));
  return typeof j?.lastKnownAccountUuid === "string" ? j.lastKnownAccountUuid.toLowerCase() : null;
}

// ---------- scanning ----------

// size, mtime and lastActivityAt of a session file; only the first few KB are read
function fileInfo(file) {
  let fd;
  try {
    fd = fs.openSync(file, "r");
    const st = fs.fstatSync(fd);
    if (st.size === 0) return null;
    const buf = Buffer.alloc(Math.min(HEAD_BYTES, st.size));
    fs.readSync(fd, buf, 0, buf.length, 0);
    const m = /"lastActivityAt"\s*:\s*(\d+)/.exec(buf.toString("utf8"));
    return { size: st.size, mtime: Math.trunc(st.mtimeMs), activity: m ? Number(m[1]) : 0 };
  } catch {
    return null;
  } finally {
    if (fd !== undefined) fs.closeSync(fd);
  }
}

// copies keep the source's modified time, so same size + same time = same file
const sameFile = (a, b) => a && b && a.size === b.size && a.mtime === b.mtime;
const newer = (a, b) => a.activity > b.activity || (a.activity === b.activity && a.mtime > b.mtime);

function scan(dataDir) {
  const root = path.join(dataDir, SESSIONS_DIR);
  const leaves = [];
  for (const acc of fs.readdirSync(root, { withFileTypes: true })) {
    if (!acc.isDirectory()) continue;
    for (const org of fs.readdirSync(path.join(root, acc.name), { withFileTypes: true })) {
      if (!org.isDirectory()) continue;
      const dir = path.join(root, acc.name, org.name);
      const files = fs.readdirSync(dir);
      // the org in use is the one whose session files the app wrote last
      let lastWrite = 0;
      for (const f of files) {
        if (LOCAL_RE.test(f) || f === "waiting-input") {
          try {
            lastWrite = Math.max(lastWrite, fs.statSync(path.join(dir, f)).mtimeMs);
          } catch {}
        }
      }
      leaves.push({
        lastWrite,
        account: acc.name.toLowerCase(),
        org: org.name.toLowerCase(),
        dir,
        sessions: files.filter((f) => LOCAL_RE.test(f)),
        deleted: files.filter((f) => DELETED_RE.test(f)),
        mtime: fs.statSync(dir).mtimeMs,
      });
    }
  }
  return { root, leaves };
}

// Which org folder of the account: the one given, else the one whose session
// files the app wrote most recently.
function pickTarget(leaves, account, org) {
  const mine = leaves.filter((l) => l.account === account);
  if (org) return mine.find((l) => l.org === org) ?? null;
  if (mine.length === 0) return null;
  return mine.reduce((a, b) =>
    b.lastWrite > a.lastWrite || (b.lastWrite === a.lastWrite && b.mtime > a.mtime) ? b : a
  );
}

// ---------- throttled file operations ----------

function makeThrottle(limitPerSec) {
  const start = Date.now();
  let done = 0;
  return async (amount) => {
    done += amount;
    if (!Number.isFinite(limitPerSec)) return;
    const ahead = (done / limitPerSec) * 1000 - (Date.now() - start);
    if (ahead > 5) await sleep(ahead);
  };
}

// ---------- replicate (on sign-out) ----------

function deletedIds(s) {
  const ids = new Set();
  for (const l of s.leaves) for (const d of l.deleted) ids.add(DELETED_RE.exec(d)[1].toLowerCase());
  return ids;
}

// What a sign-out would do now: copy the newest version of each session into
// every folder that lacks it or has an older one, and remove copies of
// sessions that were deleted somewhere (so they do not come back).
function planReplicate(s) {
  const deleted = deletedIds(s);
  const best = new Map(); // id -> { file, name, info }
  const infos = new Map(); // file -> info
  const purge = [];
  for (const l of s.leaves) {
    for (const name of l.sessions) {
      const id = LOCAL_RE.exec(name)[1].toLowerCase();
      const file = path.join(l.dir, name);
      if (deleted.has(id)) {
        purge.push(file);
        continue;
      }
      const info = fileInfo(file);
      if (!info) continue;
      infos.set(file, info);
      const cur = best.get(id);
      if (!cur || newer(info, cur.info)) best.set(id, { file, name, info });
    }
  }
  const copies = [];
  // most recently used folders first, in case sign-in comes before we finish
  const leaves = [...s.leaves].sort((a, b) => b.lastWrite - a.lastWrite);
  for (const leaf of leaves) {
    const have = new Set(leaf.sessions.map((f) => f.toLowerCase()));
    for (const src of best.values()) {
      const dest = path.join(leaf.dir, src.name);
      if (dest === src.file) continue;
      if (have.has(src.name.toLowerCase())) {
        const cur = infos.get(dest);
        if (cur && (sameFile(cur, src.info) || !newer(src.info, cur))) continue;
      }
      copies.push({ src: src.file, dest, bytes: src.info.size });
    }
  }
  // a session marked deleted only where the app deleted it: its copies elsewhere go too
  return { copies, purge, sessions: best.size };
}

function freeBytes(dir) {
  if (process.env.CCDC_TEST_FREE_MB) return Number(process.env.CCDC_TEST_FREE_MB) * 1024 * 1024;
  try {
    const st = fs.statfsSync(dir);
    return st.bavail * st.bsize;
  } catch {
    return Infinity;
  }
}

async function replicate(dataDir, { dryRun = false, onStart = null, settings = loadSettings() } = {}) {
  const s = scan(dataDir);
  const { copies, purge } = planReplicate(s);
  const need = copies.reduce((n, c) => n + c.bytes, 0);
  const free = freeBytes(s.root);
  const result = { copies: copies.length, purge: purge.length, need, free, written: 0, purged: 0, lowDisk: false };
  if ((copies.length === 0 && purge.length === 0) || dryRun) return result;
  if (need > free - DISK_RESERVE) {
    result.lowDisk = true;
    return result;
  }
  const speed = SPEEDS[settings.speed];
  onStart?.({ ...result, seconds: estimateSeconds(need, purge.length, settings) });

  const t0 = Date.now();
  const throttleCopy = makeThrottle(speed.copyMBps * 1024 * 1024);
  for (const c of copies) {
    try {
      const tmp = c.dest + ".ccdc.tmp";
      await fs.promises.copyFile(c.src, tmp);
      await fs.promises.rename(tmp, c.dest);
      result.written++;
    } catch (e) {
      log(`copy failed ${path.basename(c.dest)}: ${e.message}`);
    }
    await throttleCopy(c.bytes);
  }
  const copyMs = Date.now() - t0;
  const throttleDel = makeThrottle(speed.deletesPerSec);
  for (const f of purge) {
    try {
      await fs.promises.rm(f);
      result.purged++;
    } catch {}
    await throttleDel(1);
  }
  result.seconds = (Date.now() - t0) / 1000;
  if (settings.speed === "fast" && need > 20 * 1024 * 1024 && copyMs > 200) saveRate("copyMBps", need / 1024 / 1024 / (copyMs / 1000));
  return result;
}

// ---------- consolidate (on sign-in, and the manual "move") ----------

// Leaves one set of session files, in the target folder. For a session found
// elsewhere too: a same or older copy is removed; a newer copy replaces the
// target's file, which is backed up.
function planConsolidate(s, target) {
  const ops = []; // { kind: 'move'|'replace'|'remove', src, dest, name }
  const inTarget = new Set(target.sessions.map((f) => f.toLowerCase()));
  const deletedInTarget = new Set(target.deleted.map((f) => f.toLowerCase()));
  for (const leaf of s.leaves) {
    if (leaf === target) continue;
    for (const name of leaf.deleted) {
      const src = path.join(leaf.dir, name);
      if (deletedInTarget.has(name.toLowerCase())) ops.push({ kind: "remove", src, name });
      else {
        ops.push({ kind: "move", src, dest: path.join(target.dir, name), name });
        deletedInTarget.add(name.toLowerCase());
      }
    }
    for (const name of leaf.sessions) {
      const src = path.join(leaf.dir, name);
      const dest = path.join(target.dir, name);
      if (!inTarget.has(name.toLowerCase())) {
        ops.push({ kind: "move", src, dest, name });
        inTarget.add(name.toLowerCase());
        continue;
      }
      const a = fileInfo(src);
      const b = fileInfo(dest);
      if (!a) continue; // unreadable (being written?): leave it for next time
      if (b && !sameFile(a, b) && newer(a, b)) ops.push({ kind: "replace", src, dest, name });
      else ops.push({ kind: "remove", src, name });
    }
  }
  return ops;
}

async function applyConsolidate(dataDir, root, ops, settings) {
  const backupRoot = path.join(dataDir, "ccdc-backups", new Date().toISOString().replace(/[:.]/g, "-"));
  // moved counts sessions only; deleted_ markers are counted apart
  const r = { moved: 0, markers: 0, removed: 0, replaced: 0, backupRoot: null };
  const throttle = makeThrottle(SPEEDS[settings.speed].deletesPerSec);
  const t0 = Date.now();
  for (const op of ops) {
    try {
      if (op.kind === "remove") {
        await fs.promises.rm(op.src);
        r.removed++;
      } else {
        if (op.kind === "replace") {
          const bak = path.join(backupRoot, path.relative(root, op.dest));
          await fs.promises.mkdir(path.dirname(bak), { recursive: true });
          await fs.promises.rename(op.dest, bak);
          r.backupRoot = backupRoot;
          r.replaced++;
        } else if (DELETED_RE.test(op.name)) r.markers++;
        else r.moved++;
        await fs.promises.rename(op.src, op.dest); // same volume: a rename, not a copy
      }
    } catch (e) {
      log(`skipped ${op.name}: ${e.message}`);
    }
    await throttle(1);
  }
  const ms = Date.now() - t0;
  if (settings.speed === "fast" && r.removed > 200 && ms > 100) saveRate("deletesPerSec", r.removed / (ms / 1000));
  return r;
}

async function consolidate(dataDir, account, org, { dryRun = false, create = false, settings = loadSettings() } = {}) {
  let s = scan(dataDir);
  let target = pickTarget(s.leaves, account, org);
  if (!target && create && org) {
    fs.mkdirSync(path.join(s.root, account, org), { recursive: true });
    s = scan(dataDir);
    target = pickTarget(s.leaves, account, org);
  }
  if (!target) return { target: null, ops: [] };
  const ops = planConsolidate(s, target);
  if (dryRun || ops.length === 0) return { target, ops };
  return { target, ops, ...(await applyConsolidate(dataDir, s.root, ops, settings)) };
}

// ---------- estimates ----------

function estimateSeconds(copyBytes, deletes, settings) {
  const rates = loadRates();
  const speed = SPEEDS[settings.speed];
  const copyRate = Math.min(rates.copyMBps, speed.copyMBps) * 1024 * 1024;
  const delRate = Math.min(rates.deletesPerSec, speed.deletesPerSec);
  return copyBytes / copyRate + deletes / delRate;
}

// How long the next sign-out (copy) and sign-in (cleanup) would take right now.
function estimate(dataDir, settings = loadSettings()) {
  const s = scan(dataDir);
  const plan = planReplicate(s);
  const copyBytes = plan.copies.reduce((n, c) => n + c.bytes, 0);
  // after copying, every folder holds every session; cleanup removes all but one folder's
  const cleanupDeletes = settings.keepCopies ? 0 : plan.sessions * Math.max(0, s.leaves.length - 1);
  const round = (x) => (x <= 0 ? 0 : Math.max(1, Math.ceil(x)));
  return {
    keepCopies: settings.keepCopies,
    speed: settings.speed,
    copyFiles: plan.copies.length,
    copyMB: Math.round(copyBytes / 1024 / 1024),
    purgeFiles: plan.purge.length,
    cleanupFiles: cleanupDeletes,
    copySeconds: round(estimateSeconds(copyBytes, plan.purge.length, settings)),
    cleanupSeconds: round(estimateSeconds(0, cleanupDeletes, settings)),
  };
}

// ---------- commands ----------

function aliasFor(aliases, account) {
  for (const [k, v] of Object.entries(aliases)) if (account.startsWith(k.toLowerCase())) return String(v);
  return "";
}

// JSON list of accounts, for the tray app.
function cmdAccounts(args) {
  const dataDir = resolveDataDir(args.dataDir);
  const s = scan(dataDir);
  const active = signedInAccount(dataDir, resolveAppLog(dataDir, args.appLog));
  const { aliases } = loadSettings();
  const byAcc = new Map();
  for (const l of s.leaves) {
    const e = byAcc.get(l.account) ?? { account: l.account, alias: aliasFor(aliases, l.account), signedIn: l.account === active, sessions: 0 };
    e.sessions += l.sessions.length;
    byAcc.set(l.account, e);
  }
  console.log(JSON.stringify([...byAcc.values()]));
}

function cmdEstimate(args) {
  const e = estimate(resolveDataDir(args.dataDir));
  if (args.json) console.log(JSON.stringify(e));
  else
    console.log(
      `Speed ${e.speed}, ${e.keepCopies ? "copies kept" : "copies removed after sign-in"}.\n` +
        `Next sign-out: copy ${e.copyFiles} file(s), ${e.copyMB}MB, about ${e.copySeconds}s.` +
        (e.keepCopies ? "" : `\nNext sign-in: remove ${e.cleanupFiles} copies, about ${e.cleanupSeconds}s.`)
    );
}

function cmdStatus(args) {
  const dataDir = resolveDataDir(args.dataDir);
  const appLog = resolveAppLog(dataDir, args.appLog);
  const st = readAppState(appLog);
  const s = scan(dataDir);
  const settings = loadSettings();
  console.log(`Data folder: ${dataDir}`);
  console.log(`App log: ${appLog ?? "not found"}`);
  console.log(`App state: ${!st ? "unknown" : st.loggedOut ? "signed out" : `signed in ${short(st.account)} / ${short(st.org)}`}`);
  console.log(`Settings: speed ${settings.speed}, ${settings.keepCopies ? "keep copies" : "remove copies after sign-in"}`);
  for (const l of s.leaves) {
    const live = st && !st.loggedOut && l.account === st.account && l.org === st.org;
    console.log(`  ${short(l.account)} / ${short(l.org)}  sessions=${l.sessions.length}${live ? "  <- in use" : ""}`);
  }
  console.log(`Free disk: ${mb(freeBytes(path.join(dataDir, SESSIONS_DIR)))}`);
}

async function cmdReplicate(args) {
  const dataDir = resolveDataDir(args.dataDir);
  const r = await replicate(dataDir, { dryRun: args.dryRun });
  if (r.lowDisk) {
    console.log(`LOW_DISK: copying needs ${mb(r.need)}, only ${mb(r.free)} free. Nothing written.`);
    process.exitCode = 3;
  } else if (args.dryRun) console.log(`Would write ${r.copies} file(s), ${mb(r.need)}, remove ${r.purge} deleted session copies (free ${mb(r.free)}). Nothing written.`);
  else console.log(`Wrote ${r.written} file(s), ${mb(r.need)}; removed ${r.purged} deleted session copies.`);
}

async function cmdMove(args) {
  const dataDir = resolveDataDir(args.dataDir);
  const appLog = resolveAppLog(dataDir, args.appLog);
  let [account, org] = (args.to ?? "").toLowerCase().split("/");
  if (!account) {
    const st = readAppState(appLog);
    if (st && !st.loggedOut) [account, org] = [st.account, st.org];
    else account = signedInAccount(dataDir, appLog);
  }
  if (!account) throw new Error("No signed-in account found. Pass --to <account>.");
  // allow short prefixes as shown by `status`
  const s = scan(dataDir);
  const full = s.leaves.find((l) => l.account.startsWith(account));
  if (full) account = full.account;
  if (org) org = s.leaves.find((l) => l.account === account && l.org.startsWith(org))?.org ?? org;

  const r = await consolidate(dataDir, account, org, { dryRun: args.dryRun });
  if (!r.target) throw new Error(`Account ${short(account)} has no session folder yet. Open the Code tab once while signed in to it.`);
  const count = (k) => r.ops.filter((o) => o.kind === k).length;
  console.log(`Target: ${short(r.target.account)} / ${short(r.target.org)}`);
  if (args.dryRun) {
    console.log(`Would move ${count("move")}, remove ${count("remove")} copies, replace ${count("replace")} with newer. Nothing written.`);
  } else if (r.ops.length === 0) {
    console.log("Nothing to move.");
  } else {
    console.log(
      `Moved ${r.moved} file(s), removed ${r.removed} copies` +
        (r.replaced ? `, replaced ${r.replaced} with newer (old ones in ${r.backupRoot})` : "") + "."
    );
  }
}

// ---------- the two jobs: what happens on sign-out and after sign-in ----------

async function onSignOut(dataDir, reason) {
  try {
    const settings = loadSettings();
    const r = await replicate(dataDir, {
      settings,
      onStart: (p) =>
        log(`${reason}: copying sessions into every account folder (${p.copies} file(s), ${mb(p.need)}, about ${Math.max(1, Math.ceil(p.seconds))}s)...`),
    });
    if (r.lowDisk) {
      log(`LOW_DISK: copying sessions to every account needs ${mb(r.need)}, only ${mb(r.free)} free. Skipped; sessions stay in the last account.`);
    } else if (r.written || r.purged) {
      log(`Copy done: sessions are in every account folder (${r.written} file(s), ${mb(r.need)}, ${r.seconds.toFixed(1)}s)`);
    } else {
      log(`${reason}: nothing to copy (every folder is up to date)`);
    }
  } catch (e) {
    log(`replicate failed: ${e.message}`);
  }
}

async function onSignIn(dataDir, account, org, reason) {
  try {
    const settings = loadSettings();
    if (settings.keepCopies) return; // copies stay; the next sign-out only copies what changed
    const r = await consolidate(dataDir, account, org, { create: true, settings });
    if (r.moved || r.markers || r.removed || r.replaced) {
      log(
        `${reason}: kept sessions in ${short(account)}/${short(org)}; removed ${r.removed} copies elsewhere` +
          (r.moved ? `, moved in ${r.moved}` : "") +
          (r.replaced ? `, ${r.replaced} replaced with newer` : "")
      );
      if (r.moved) log(`Note: ${r.moved} session(s) moved in after the app read its list; they show after the next sign-in or app restart.`);
    }
  } catch (e) {
    log(`cleanup failed: ${e.message}`);
  }
}

// One-shot jobs for the tray app, which watches the app log itself and starts
// Node only when there is work, so no Node process stays in memory.
//   node ccdc.mjs signout                      -> what a sign-out does
//   node ccdc.mjs signin --account A --org O   -> what happens after a sign-in
//   node ccdc.mjs startup                      -> catch up with the app's current state
async function cmdJob(cmd, args) {
  const dataDir = resolveDataDir(args.dataDir);
  if (cmd === "signout") return onSignOut(dataDir, "Sign-out detected");
  if (cmd === "signin") {
    if (!args.account || !args.org) throw new Error("signin needs --account and --org");
    return onSignIn(dataDir, args.account.toLowerCase(), args.org.toLowerCase(), "sign-in");
  }
  const st = readAppState(resolveAppLog(dataDir, args.appLog));
  if (!st) return log("App state unknown: waiting for the next sign-in or sign-out");
  if (st.loggedOut) {
    log("App is signed out");
    return onSignOut(dataDir, "startup");
  }
  log(`App is signed in: ${short(st.account)}/${short(st.org)}`);
  return onSignIn(dataDir, st.account, st.org, "startup");
}

// Stays running and follows the app's log (for command-line use; the tray
// app does the watching itself and runs the one-shot jobs above).
//   sign-out  -> copy changed sessions into every account/org folder
//   sign-in   -> after the app has read its folder, remove the copies elsewhere
//                (only when keepCopies is false; by default the copies stay)
// Work runs one job at a time; settings are re-read for every job.
function cmdWatch(args) {
  const dataDir = resolveDataDir(args.dataDir);
  const appLog = resolveAppLog(dataDir, args.appLog);
  if (!appLog) throw new Error("Claude app log (Logs\\main.log) not found. Pass --app-log.");
  // the app reads the folder twice within a couple of seconds of sign-in; wait it out
  const settleMs = args.settleMs ?? 6000;
  let cleanupTimer = null;
  let signedIn = null; // { account, org }
  let queue = Promise.resolve();
  const enqueue = (job) => {
    queue = queue.then(job).catch((e) => log(`job failed: ${e.message}`)).finally(freeMemory);
  };

  log(`Watching ${dataDir}`);
  log(`Following app log ${appLog}`);
  const st = readAppState(appLog);
  if (st && !st.loggedOut) signedIn = { account: st.account, org: st.org };
  enqueue(() => cmdJob("startup", args));

  let pos = fs.statSync(appLog).size;
  let carry = "";
  const buf = Buffer.alloc(64 * 1024); // reused for every read
  setInterval(() => {
    let size;
    try {
      size = fs.statSync(appLog).size;
    } catch {
      return; // being rotated
    }
    if (size < pos) pos = 0; // rotated: main.log started over
    if (size === pos) return;
    const fd = fs.openSync(appLog, "r");
    try {
      while (pos < size) {
        const n = fs.readSync(fd, buf, 0, Math.min(buf.length, size - pos), pos);
        if (n <= 0) break;
        pos += n;
        const lines = (carry + buf.toString("utf8", 0, n)).split(/\r?\n/);
        carry = lines.pop();
        if (carry.length > 16 * 1024) carry = ""; // never hold a runaway line
        for (const line of lines) {
          const ev = parseLogLine(line);
          if (!ev) continue;
          if (ev.loggedOut) {
            clearTimeout(cleanupTimer);
            signedIn = null;
            enqueue(() => onSignOut(dataDir, "Sign-out detected"));
          } else {
            const changed = !signedIn || signedIn.account !== ev.account || signedIn.org !== ev.org;
            signedIn = { account: ev.account, org: ev.org };
            if (changed) {
              const keep = loadSettings().keepCopies;
              log(`Sign-in detected: ${short(ev.account)}/${short(ev.org)}, ${keep ? "copies kept" : `removing copies in ${Math.round(settleMs / 1000)}s`}`);
            }
            clearTimeout(cleanupTimer);
            cleanupTimer = setTimeout(() => enqueue(() => onSignIn(dataDir, ev.account, ev.org, "sign-in")), settleMs);
          }
        }
      }
    } finally {
      fs.closeSync(fd);
    }
  }, 500);
}

function help() {
  console.log(`CCDC (Continuous Claude Desktop Code) - share Claude desktop Code sessions between accounts

  node ccdc.mjs status               app state, settings, folders and session counts
  node ccdc.mjs watch                follow the app: copy on sign-out, clean up after sign-in
  node ccdc.mjs estimate [--json]    how long the next sign-out copy / sign-in cleanup would take
  node ccdc.mjs replicate [--dry-run]
                                     copy the newest sessions into every account folder now
  node ccdc.mjs move [--to <acc>[/<org>]] [--dry-run]
                                     keep one set of sessions, in that account's folder

Settings (ccdc.json next to this script):
  "keepCopies": false                remove the copies after sign-in (default true: keep them, fewer SSD writes)
  "speed": "fast"|"normal"|"gentle"  limit copy/delete speed

Options:
  --data-dir <path>   app data folder (default: auto-detect under %LOCALAPPDATA%\\Claude-Data)
  --app-log <path>    app log (default: %LOCALAPPDATA%\\Claude\\Logs\\main.log)`);
}

const args = parseArgs(process.argv.slice(2));
const run = async () => {
  const cmd = args._[0];
  if (args.help || !cmd) help();
  else if (cmd === "status") cmdStatus(args);
  else if (cmd === "move") await cmdMove(args);
  else if (cmd === "replicate") await cmdReplicate(args);
  else if (cmd === "estimate") cmdEstimate(args);
  else if (cmd === "accounts") cmdAccounts(args);
  else if (cmd === "watch") cmdWatch(args);
  else if (cmd === "signout" || cmd === "signin" || cmd === "startup") await cmdJob(cmd, args);
  else {
    help();
    process.exitCode = 2;
  }
};
run().catch((e) => {
  console.error(`Error: ${e.message}`);
  process.exitCode = 1;
});
