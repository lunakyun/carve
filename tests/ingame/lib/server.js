// Start a throwaway Pumpkin server for one area.
//
//   const srv = await startServer({ dir, port, wasm, scripts: { 'a.carve': '...' }, ops: ['Smoke'] })
//   srv.console('give Smoke stone 1'); await srv.waitLog(/Carve/); await srv.stop()
//
// The first start copies the template server (CARVE_TEMPLATE_DIR, default the
// pumpscript test server): the binary, pumpkin.toml and data/*.json, not the
// world. Every start rewrites the ports, ops.json and the script folder, and
// deletes the world (and with it saved player data) unless keepWorld is set.
'use strict'
const fs = require('fs')
const path = require('path')
const crypto = require('crypto')
const { spawn } = require('child_process')

const ROOT = path.resolve(__dirname, '../../..')
const TEMPLATE = process.env.CARVE_TEMPLATE_DIR || path.resolve(ROOT, '../pumpscript/.test/server')
const EXE_NAME = process.platform === 'win32' ? 'pumpkin-X64-Windows.exe' : 'pumpkin-X64-Linux'
const DEFAULT_WASM = path.join(ROOT, 'target', 'wasm32-wasip2', 'release', 'carve.wasm')
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))

const running = new Set()
process.on('exit', () => { for (const p of running) try { p.kill() } catch {} })

// Pumpkin's offline-mode UUID: the first 16 bytes of sha256(name).
function offlineUuid (name) {
  const h = crypto.createHash('sha256').update(name).digest('hex')
  return `${h.slice(0, 8)}-${h.slice(8, 12)}-${h.slice(12, 16)}-${h.slice(16, 20)}-${h.slice(20)}`
}

function copyTemplate (dir) {
  fs.mkdirSync(path.join(dir, 'data'), { recursive: true })
  const exe = path.join(dir, EXE_NAME)
  if (!fs.existsSync(exe)) {
    const src = path.join(TEMPLATE, EXE_NAME)
    if (!fs.existsSync(src)) throw new Error(`no Pumpkin binary at ${src} (see docs/ingame-testing.md)`)
    try { fs.linkSync(src, exe) } catch { fs.copyFileSync(src, exe) }
  }
  if (!fs.existsSync(path.join(dir, 'pumpkin.toml'))) fs.copyFileSync(path.join(TEMPLATE, 'pumpkin.toml'), path.join(dir, 'pumpkin.toml'))
  const data = path.join(TEMPLATE, 'data')
  for (const f of fs.existsSync(data) ? fs.readdirSync(data) : []) {
    if (f.endsWith('.json') && f !== 'ops.json' && !fs.existsSync(path.join(dir, 'data', f))) fs.copyFileSync(path.join(data, f), path.join(dir, 'data', f))
  }
}

// Java on `port`; every other listener off and moved out of the way.
function setPorts (toml, port) {
  const moved = { 'networking.bedrock': 10000, 'networking.query': 20000, 'networking.management': 30000 }
  let section = ''
  return toml.split(/\r?\n/).map((line) => {
    const h = line.match(/^\s*\[([^\]]+)\]/)
    if (h) { section = h[1].trim(); return line }
    if (/^\s*address\s*=/.test(line)) {
      const p = section === 'networking.java' ? port : port + (moved[section] || 35000)
      return `address = "127.0.0.1:${p}"`
    }
    if (/^\s*enabled\s*=/.test(line) && (section in moved || section === 'networking.lan_broadcast' || section === 'networking.bedrock.nethernet')) return 'enabled = false'
    return line
  }).join('\n')
}

async function startServer ({ dir, port, wasm = DEFAULT_WASM, scripts = {}, ops = [], keepVariables = false, keepWorld = false, env = {} }) {
  if (!dir || !port) throw new Error('startServer needs dir and port')
  copyTemplate(dir)
  const tomlPath = path.join(dir, 'pumpkin.toml')
  fs.writeFileSync(tomlPath, setPorts(fs.readFileSync(tomlPath, 'utf8'), port))
  fs.writeFileSync(path.join(dir, 'data', 'ops.json'), JSON.stringify(
    [].concat(ops).map((name) => ({ uuid: offlineUuid(name), name, level: 4, bypassesPlayerLimit: false })), null, 2))
  if (!keepWorld) fs.rmSync(path.join(dir, 'world'), { recursive: true, force: true })

  if (!fs.existsSync(wasm)) throw new Error(`missing ${wasm} (cargo build -p carve-wasm --target wasm32-wasip2 --release)`)
  const carveData = path.join(dir, 'plugins', 'data', 'carve')
  const scriptDir = path.join(carveData, 'scripts')
  fs.mkdirSync(scriptDir, { recursive: true })
  fs.copyFileSync(wasm, path.join(dir, 'plugins', 'carve.wasm'))
  for (const f of fs.readdirSync(scriptDir)) fs.rmSync(path.join(scriptDir, f), { recursive: true, force: true })
  // scripts: { 'name.carve': 'source' } or a list of .carve file paths
  if (Array.isArray(scripts)) for (const f of scripts) fs.copyFileSync(f, path.join(scriptDir, path.basename(f)))
  else for (const [name, src] of Object.entries(scripts)) fs.writeFileSync(path.join(scriptDir, name), src)
  if (!keepVariables) fs.rmSync(path.join(carveData, 'variables.json'), { force: true })

  const lines = []
  const proc = spawn(path.join(dir, EXE_NAME), [], { cwd: dir, stdio: ['pipe', 'pipe', 'pipe'], env: { ...process.env, ...env } })
  running.add(proc)
  const onData = (d) => {
    for (const l of d.toString('utf8').split(/\r?\n/)) if (l.trim()) lines.push(l.replace(/\x1b\[[0-9;]*m/g, ''))
  }
  proc.stdout.on('data', onData)
  proc.stderr.on('data', onData)
  let exitCode
  const exited = new Promise((resolve) => proc.on('exit', (c) => { exitCode = c; running.delete(proc); resolve(c) }))

  const srv = {
    dir,
    port,
    lines,
    proc,
    get alive () { return exitCode === undefined },
    mark: () => lines.length,
    console (cmd) { if (exitCode === undefined) proc.stdin.write(cmd + '\n') },
    // First log line matching `re` from line index `from` on, or null after `ms`.
    async waitLog (re, ms = 5000, from = 0) {
      const end = Date.now() + ms
      for (;;) {
        const hit = lines.slice(from).find((l) => re.test(l))
        if (hit || Date.now() > end || exitCode !== undefined) return hit || null
        await sleep(100)
      }
    },
    // Stop with the console command; kill it if it hasn't exited after `ms`. True if it stopped cleanly.
    async stop (ms = 20000) {
      if (exitCode !== undefined) return true
      srv.console('stop')
      const clean = await Promise.race([exited.then(() => true), sleep(ms).then(() => false)])
      if (!clean) { proc.kill(); await exited }
      fs.writeFileSync(path.join(dir, 'last-run.log'), lines.join('\n'))
      return clean
    },
  }
  const ok = await srv.waitLog(/Server is now running/, 60000)
  if (!ok) {
    await srv.stop(5000)
    throw new Error(`server on port ${port} did not start:\n${lines.slice(-15).join('\n')}`)
  }
  return srv
}

module.exports = { startServer, offlineUuid, setPorts, ROOT, TEMPLATE }
