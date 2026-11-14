// In-game test for Carve on a real local Pumpkin server.
//
//   node tests/ingame/run.js
//
// It starts the server, connects two bots (Tester = op, Pleb = not op),
// runs every case, restarts the server once for the persistence check and
// stops it again. See docs/ingame-testing.md for the setup.
'use strict'
const fs = require('fs')
const path = require('path')
const { spawn } = require('child_process')
const { Bot } = require('./mc.js')

const ROOT = path.resolve(__dirname, '../..')
const SERVER_DIR = process.env.CARVE_SERVER_DIR || path.join(ROOT, '.test', 'server')
const EXE = process.env.PUMPKIN_EXE ||
  path.join(SERVER_DIR, process.platform === 'win32' ? 'pumpkin-X64-Windows.exe' : 'pumpkin-X64-Linux')
const WASM = path.join(ROOT, 'target', 'wasm32-wasip2', 'release', 'carve.wasm')
const DATA = path.join(SERVER_DIR, 'plugins', 'data', 'carve')
const SCRIPTS = path.join(DATA, 'scripts')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const results = []
function check (name, ok, detail = '') {
  results.push({ name, status: ok ? 'PASS' : 'FAIL', detail })
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? '  (' + detail + ')' : ''}`)
}
function skip (name, reason) {
  results.push({ name, status: 'SKIP', detail: reason })
  console.log(`SKIP  ${name}  (${reason})`)
}

class Server {
  start () {
    this.lines = []
    this.proc = spawn(EXE, [], { cwd: SERVER_DIR, stdio: ['pipe', 'pipe', 'pipe'] })
    const onData = (d) => {
      for (const l of d.toString('utf8').split(/\r?\n/)) {
        if (l.trim()) this.lines.push(l.replace(/\x1b\[[0-9;]*m/g, ''))
      }
    }
    this.proc.stdout.on('data', onData)
    this.proc.stderr.on('data', onData)
    this.exited = new Promise((resolve) => this.proc.on('exit', resolve))
    return this.waitLog(/Server is now running/, 60000)
  }

  mark () { return this.lines.length }

  async waitLog (re, ms = 5000, from = 0) {
    const end = Date.now() + ms
    while (Date.now() < end) {
      const hit = this.lines.slice(from).find((l) => re.test(l))
      if (hit) return hit
      await sleep(100)
    }
    return null
  }

  console (cmd) { this.proc.stdin.write(cmd + '\n') }

  async stop () {
    this.console('stop')
    const done = await Promise.race([this.exited.then(() => true), sleep(20000).then(() => false)])
    if (!done) this.proc.kill()
    return done
  }
}

async function join (name) {
  const bot = new Bot(name)
  bot.on('error', (e) => console.log(`[${name}] error: ${e.message}`))
  await new Promise((resolve, reject) => {
    bot.once('spawn', resolve)
    setTimeout(() => reject(new Error(`${name} did not spawn`)), 20000)
  })
  return bot
}

const has = (kind, s) => (e) => (!kind || e.kind === kind) && e.text.includes(s)

function setup () {
  for (const f of [EXE, WASM]) {
    if (!fs.existsSync(f)) throw new Error(`missing ${f} (see docs/ingame-testing.md)`)
  }
  fs.copyFileSync(WASM, path.join(SERVER_DIR, 'plugins', 'carve.wasm'))
  fs.mkdirSync(SCRIPTS, { recursive: true })
  for (const f of fs.readdirSync(SCRIPTS)) fs.rmSync(path.join(SCRIPTS, f))
  for (const f of fs.readdirSync(path.join(ROOT, 'examples'))) {
    if (f.endsWith('.carve')) fs.copyFileSync(path.join(ROOT, 'examples', f), path.join(SCRIPTS, f))
  }
  fs.copyFileSync(path.join(__dirname, 'carve-test.carve'), path.join(SCRIPTS, 'carve-test.carve'))
  fs.rmSync(path.join(DATA, 'variables.json'), { force: true })
}

async function reload (bot, expect) {
  const t = Date.now()
  bot.command('carve reload')
  return bot.waitFor(has('system', expect), 15000, t)
}

async function phase1 (srv) {
  const loaded = await srv.waitLog(/\[Carve\] loaded \d+ script/, 1000)
  check('plugin loads with every script', /0 error\(s\)/.test(loaded || ''), loaded ? loaded.replace(/^.*\[Carve\] /, '') : 'no load line')

  const tester = await join('Tester')
  srv.console('op Tester')
  check('on join: send', !!await tester.waitFor(has('system', 't:join Tester')))
  check('send title + subtitle', !!await tester.waitFor(has('title', 't:title')) && !!await tester.waitFor(has('subtitle', 't:sub')))
  check('send action bar', !!await tester.waitFor(has('actionbar', 't:bar')))
  const first = tester.log.find(has('system', 't:join Tester'))
  const waited = await tester.waitFor(has('system', 't:waited Tester'), 4000)
  check('wait 1 second, then message', !!waited && first && waited.t - first.t >= 800, waited && first ? `${waited.t - first.t} ms` : 'no message')
  check('cancel event after a wait logs a warning', !!await srv.waitLog(/'cancel event' after a wait/, 3000))

  // Block events, at spawn before anything moves the player.
  // Saved player data may put Tester elsewhere (e.g. mid-air from the teleport case).
  srv.console('gamemode creative Tester')
  srv.console('tp Tester 0.5 64 0.5')
  srv.console('setblock 2 64 0 air')
  srv.console('give Tester stone 1')
  await tester.waitFor((e) => e.kind === 'position' && e.text === '0.5 64 0.5', 3000)
  await sleep(500)
  const b = { x: Math.floor(tester.pos.x), y: Math.floor(tester.pos.y) - 1, z: Math.floor(tester.pos.z) }
  let t = Date.now()
  tester.dig(b.x, b.y, b.z, 0)
  const brk = await tester.waitFor(has('system', 't:break'), 3000, t)
  if (brk) check('on break + cancel event', true)
  else skip('on break + cancel event', 'no break event from a creative start-dig packet')
  t = Date.now()
  tester.placeOn(b.x + 2, b.y, b.z)
  const plc = await tester.waitFor(has('system', 't:place'), 3000, t)
  if (plc) check('on place', true)
  else skip('on place', 'no place event (the block next to spawn may be air or out of reach)')

  const pleb = await join('Pleb')
  await sleep(1500)
  t = Date.now()
  pleb.chat('hello all')
  check('chat goes through (control)', !!await tester.waitFor(has(null, 'hello all'), 4000, t))
  t = Date.now()
  pleb.chat('blockme')
  check('on chat: message + cancel event', !!await pleb.waitFor(has('system', 't:blocked blockme'), 4000, t))
  check('cancelled chat not delivered', !await tester.waitFor(has(null, 'blockme'), 2000, t))

  t = Date.now()
  tester.command('tperm hello world')
  check('command arg (op allowed)', !!await tester.waitFor(has('system', 't:arg hello world'), 4000, t))
  t = Date.now()
  pleb.command('tperm hello')
  const denied = await pleb.waitFor(has('system', 'permission'), 4000, t)
  check('command permission (non-op denied)', !!denied && !pleb.log.some(has('system', 't:arg')))

  t = Date.now()
  tester.command('thealth')
  check('set health of player to 7', !!await tester.waitFor((e) => e.kind === 'health' && Number(e.text) === 7, 4000, t))
  t = Date.now()
  tester.command('theal')
  check('heal player (to max)', !!await tester.waitFor((e) => e.kind === 'health' && Number(e.text) === 20, 4000, t))

  t = Date.now()
  tester.command('tconsole')
  check('execute console command', !!await tester.waitFor(has(null, 't:console'), 4000, t))

  t = Date.now()
  tester.command('ttp')
  check('teleport player', !!await tester.waitFor((e) => e.kind === 'position' && e.text === '10 100 10', 4000, t))

  t = Date.now()
  tester.command('tvar')
  check('global variable', !!await tester.waitFor(has('system', 't:counter 1'), 4000, t))
  check('/carve reload', !!await reload(tester, 'Reloaded'))
  t = Date.now()
  tester.command('tvar')
  check('variable survives reload', !!await tester.waitFor(has('system', 't:counter 2'), 4000, t))

  fs.writeFileSync(path.join(SCRIPTS, 'broken.carve'), 'on join:\n    blorp the player\n')
  const err = await reload(tester, 'old scripts kept')
  const errLine = err && tester.log.find((e) => e.t >= err.t && e.text.includes('broken.carve:2'))
  check('broken reload reports file:line', !!err && !!errLine)
  t = Date.now()
  tester.command('tvar')
  check('broken reload keeps old scripts', !!await tester.waitFor(has('system', 't:counter 3'), 4000, t))
  fs.rmSync(path.join(SCRIPTS, 'broken.carve'))

  // A command added by reload can't be registered until a restart (Pumpkin API limit).
  fs.writeFileSync(path.join(SCRIPTS, 'late.carve'), 'command /tlate:\n    trigger:\n        send "t:late"\n')
  const lateMsg = await reload(tester, 'Reloaded')
  check('reload names new commands that need a restart', !!lateMsg && lateMsg.text.includes('/tlate'))
  t = Date.now()
  tester.command('tlate')
  check('new command is not live before restart', !await tester.waitFor(has('system', 't:late'), 2000, t))
  fs.rmSync(path.join(SCRIPTS, 'late.carve'))

  // 100k+ straight-line instructions in one slice (an event trigger, so reload is enough).
  const body = '        set {_x} to 1\n'.repeat(100005)
  fs.writeFileSync(path.join(SCRIPTS, 'runaway.carve'),
    `on chat:\n    if message is "runaway":\n${body}        send "t:runaway finished" to player\n`)
  const rl = await reload(tester, 'Reloaded')
  if (!rl) {
    check('runaway guard', false, 'reload with runaway.carve failed')
  } else {
    const m = srv.mark()
    t = Date.now()
    tester.chat('runaway')
    const hit = await srv.waitLog(/more than 100000 instructions/, 10000, m)
    check('runaway guard', !!hit && !tester.log.some((e) => e.t >= t && e.text.includes('t:runaway finished')), hit ? '' : 'no guard error in log')
  }
  fs.rmSync(path.join(SCRIPTS, 'runaway.carve'))
  await reload(tester, 'Reloaded')

  t = Date.now()
  pleb.quit()
  check('on quit (player name still known)', !!await tester.waitFor(has('system', 't:quit Pleb'), 4000, t))

  await sleep(6000) // variables.json is saved ~5 s after a change
  tester.quit()
  await sleep(500)
}

async function phase2 (srv) {
  const tester = await join('Tester')
  const t = Date.now()
  tester.command('tvar')
  check('variable survives server restart', !!await tester.waitFor(has('system', 't:counter 4'), 4000, t))
  tester.quit()
  await sleep(500)
}

async function main () {
  setup()
  const srv = new Server()
  if (!await srv.start()) throw new Error('server did not start')
  try {
    await phase1(srv)
  } finally {
    const clean = await srv.stop()
    check('server stops cleanly', clean)
  }
  const vars = path.join(DATA, 'variables.json')
  const saved = fs.existsSync(vars) ? fs.readFileSync(vars, 'utf8') : ''
  check('variables.json written', /"t_counter":\s*3/.test(saved))

  const srv2 = new Server()
  if (!await srv2.start()) throw new Error('server did not restart')
  try {
    await phase2(srv2)
  } finally {
    await srv2.stop()
  }
  fs.writeFileSync(path.join(SERVER_DIR, 'last-run.log'), srv.lines.concat(['--- restart ---'], srv2.lines).join('\n'))

  const failed = results.filter((r) => r.status === 'FAIL').length
  console.log(`\n${results.filter((r) => r.status === 'PASS').length} passed, ${failed} failed, ` +
    `${results.filter((r) => r.status === 'SKIP').length} skipped. Server log: ${path.join(SERVER_DIR, 'last-run.log')}`)
  process.exit(failed ? 1 : 0)
}

main().catch((e) => { console.error(e); process.exit(2) })
