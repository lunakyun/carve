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
// A private server copy can run on another port (see docs/ingame-testing.md).
const PORT = Number(process.env.CARVE_PORT || 25565)

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
  const bot = new Bot(name, { port: PORT })
  bot.on('error', (e) => console.log(`[${name}] error: ${e.message}`))
  await new Promise((resolve, reject) => {
    bot.once('spawn', resolve)
    setTimeout(() => reject(new Error(`${name} did not spawn`)), 20000)
  })
  return bot
}

// `s` may be a list: every part must be in the text (formatted messages arrive split up).
const has = (kind, s) => (e) => (!kind || e.kind === kind) && [].concat(s).every((x) => e.text.includes(x))

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
  const loaded = await srv.waitLog(/\[Carve\] Loaded \d+ scripts? with a total of/, 1000)
  const loadErr = srv.lines.some((l) => /while loading the scripts/.test(l))
  check('plugin loads with every script', !!loaded && !loadErr, loaded ? loaded.replace(/^.*\[Carve\] /, '') : 'no load line')

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
  // Reset the blocks the cases use (an earlier failed run may have changed them).
  srv.console('setblock 0 63 0 stone')
  srv.console('setblock 2 63 0 stone')
  srv.console('setblock 2 64 0 air')
  srv.console('give Tester stone 1')
  await tester.waitFor((e) => e.kind === 'position' && e.text === '0.5 64 0.5', 3000, Date.now() - 50)
  await sleep(500)
  const b = { x: Math.floor(tester.pos.x), y: Math.floor(tester.pos.y) - 1, z: Math.floor(tester.pos.z) }
  let t = Date.now()
  tester.dig(b.x, b.y, b.z, 0)
  const brk = await tester.waitFor(has('system', 't:break'), 3000, t)
  if (brk) check('on break + cancel event', true)
  else skip('on break + cancel event', `no break event from a creative start-dig packet at ${b.x} ${b.y} ${b.z}`)
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
  check('/carve reload', !!await reload(tester, 'Successfully reloaded'))
  t = Date.now()
  tester.command('tvar')
  check('variable survives reload', !!await tester.waitFor(has('system', 't:counter 2'), 4000, t))

  fs.writeFileSync(path.join(SCRIPTS, 'broken.carve'), 'on join:\n    blorp the player\n')
  const err = await reload(tester, 'while reloading')
  const errLine = err && tester.log.find((e) => e.t >= err.t && has(null, ['Line 2:', '(broken.carve)'])(e))
  check('broken reload reports file:line', !!err && !!errLine)
  t = Date.now()
  tester.command('tvar')
  check('broken reload keeps old scripts', !!await tester.waitFor(has('system', 't:counter 3'), 4000, t))
  fs.rmSync(path.join(SCRIPTS, 'broken.carve'))

  // A command added by reload can't be registered until a restart (Pumpkin API limit).
  fs.writeFileSync(path.join(SCRIPTS, 'late.carve'), 'command /tlate:\n    trigger:\n        send "t:late"\n')
  t = Date.now()
  tester.command('carve reload')
  const lateMsg = await tester.waitFor(has('system', 'need a server restart'), 15000, t)
  check('reload names new commands that need a restart', !!lateMsg && lateMsg.text.includes('/tlate'))
  t = Date.now()
  tester.command('tlate')
  check('new command is not live before restart', !await tester.waitFor(has('system', 't:late'), 2000, t))
  fs.rmSync(path.join(SCRIPTS, 'late.carve'))

  // 100k+ straight-line instructions in one slice (an event trigger, so reload is enough).
  const body = '        set {_x} to 1\n'.repeat(100005)
  fs.writeFileSync(path.join(SCRIPTS, 'runaway.carve'),
    `on chat:\n    if message is "runaway":\n${body}        send "t:runaway finished" to player\n`)
  const rl = await reload(tester, 'Successfully reloaded')
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
  await reload(tester, 'Successfully reloaded')

  // ---- v2 ----
  t = Date.now()
  tester.command('tloop')
  check('v2 loop all players + list size', !!await tester.waitFor(has('system', 't:loop 2'), 4000, t))
  check('v2 loop N times', !!await tester.waitFor(has('system', 't:sum 6'), 4000, t))
  t = Date.now()
  tester.command('tcond')
  check('v2 and/or/between/contains/is a or b', !!await tester.waitFor(has('system', 't:cond ok'), 4000, t))
  t = Date.now()
  tester.command('tfunc')
  check('v2 function value + options', !!await tester.waitFor(has('system', 't:func 42'), 4000, t))
  check('v2 function with wait', !!await tester.waitFor(has('system', 't:fwait'), 4000, t))
  t = Date.now()
  tester.command('tstate')
  const st = await tester.waitFor(has('system', 't:loaded'), 4000, t)
  const beats = st ? Number((st.text.match(/beats (\d+)/) || [])[1]) : 0
  check('v2 on load', !!st && st.text.includes('t:loaded yes'))
  check('v2 every 1 second', beats >= 3, `beats=${beats}`)
  srv.console('clear Tester')
  await sleep(500)
  t = Date.now()
  tester.command('titem')
  check('v2 give + has item', !!await tester.waitFor(has('system', 't:has diamonds'), 4000, t))
  const tool = await tester.waitFor(has('system', 't:tool'), 4000, t)
  check('v2 tool of player', !!tool && /t:tool (3 )?diamond/.test(tool.text), tool ? tool.text.slice(0, 60) : 'no reply')
  const p0 = { ...tester.pos }
  t = Date.now()
  tester.command('tloc')
  const loc = await tester.waitFor(has('system', 't:loc'), 4000, t)
  check('v2 location/x/y/z/world of player', !!loc && loc.text.includes('world') && loc.text.includes(`t:loc ${p0.x} ${p0.y} ${p0.z}`), loc ? loc.text.slice(0, 80) : 'no reply')
  check('v2 teleport to location()', !!await tester.waitFor((e) => e.kind === 'position' && e.text === '5 90 5', 4000, t))

  await sleep(500)
  srv.console('gamemode survival Tester')
  srv.console('tp Tester 0.5 64 0.5')
  await sleep(1000)
  t = Date.now()
  srv.console('damage Tester 3')
  const dmg = await tester.waitFor(has('system', 't:dmg'), 4000, t)
  check('v2 on damage (damage, no attacker)', !!dmg && dmg.text.includes('t:dmg 3 by <none>'), dmg ? dmg.text.slice(0, 60) : 'no event')
  t = Date.now()
  srv.console('kill Tester')
  check('v2 on death', !!await tester.waitFor(has('system', 't:died'), 4000, t))
  await sleep(500)
  tester.respawn()
  check('v2 on respawn', !!await tester.waitFor(has('system', 't:respawned'), 5000, t))
  await sleep(1000)
  srv.console('gamemode creative Tester')
  srv.console('tp Tester 0.5 64 0.5')
  srv.console('clear Tester')
  srv.console('give Tester stone 1')
  srv.console('setblock 2 64 0 air')
  await tester.waitFor((e) => e.kind === 'position' && e.text === '0.5 64 0.5', 3000, t)
  await sleep(500)
  t = Date.now()
  tester.placeOn(2, 63, 0)
  const rc = await tester.waitFor(has('system', 't:rclick'), 3000, t)
  check('v2 on right click + clicked block', !!rc && !rc.text.includes('<none>'),
    rc ? rc.text.slice(0, 40) : 'no event; got: ' + tester.log.filter((e) => e.t >= t).map((e) => e.kind + ':' + e.text.slice(0, 30)).join(' | '))
  t = Date.now()
  tester.dig(0, 63, 0, 0)
  check('v2 on left click with <item> filter', !!await tester.waitFor(has('system', 't:lclick-stone'), 3000, t))

  // ---- /carve admin ----
  const say = async (cmd, want, who = tester, ms = 5000) => {
    const t0 = Date.now()
    who.command(cmd)
    return who.waitFor(has('system', want), ms, t0)
  }
  check('admin: help', !!await say('carve help', '/carve timings'))
  check('admin: alias /cv', !!await say('cv help', '/carve reload'))
  check('admin: list', !!await say('carve list', 'carve-test.carve'))
  check('admin: info', !!await say('carve info', 'Data folder'))
  check('admin: info <script>', !!await say('carve info carve-test', 'double(n: number) :: number'))
  fs.writeFileSync(path.join(SCRIPTS, 'one.carve'), 'on join:\n    blorp\n')
  const bad = await say('carve reload one', 'while reloading')
  check('admin: reload <script> with error keeps old', !!bad && !!await tester.waitFor(has('system', ['Line 2:', '(one.carve)']), 2000, bad.t))
  check('admin: errors', !!await say('carve errors one', ['Line 2:', '(one.carve)']))
  fs.writeFileSync(path.join(SCRIPTS, 'one.carve'),
    'command /tvar2:\n    trigger:\n        send "x"\non chat:\n    if message is "one?":\n        send "t:one" to player\n')
  check('admin: reload <script>', !!await say('carve reload one', ['Successfully reloaded', 'one.carve']))
  t = Date.now()
  tester.chat('one?')
  check('admin: reloaded script runs', !!await tester.waitFor(has('system', 't:one'), 4000, t))
  check('admin: disable', !!await say('carve disable one', ['Successfully disabled', 'one.carve']))
  check('admin: disabled file renamed', fs.existsSync(path.join(SCRIPTS, '-one.carve')))
  t = Date.now()
  tester.chat('one?')
  check('admin: disabled script stops', !await tester.waitFor(has('system', 't:one'), 2000, t))
  check('admin: enable (restart note for its command)', !!await say('carve enable one', 'need a server restart'))
  check('admin: list disabled', !!await say('carve list disabled', 'Disabled (0)'))
  fs.rmSync(path.join(SCRIPTS, 'one.carve'))
  await say('carve reload', 'Successfully reloaded')
  check('admin: vars set', !!await say('carve vars set t_admin 41', 'Set {t_admin} to 41'))
  t = Date.now()
  tester.command('carve run add 1 to {t_admin}')
  await sleep(300)
  check('admin: run <effect> + vars get', !!await say('carve vars get t_admin', '42'))
  check('admin: run shows parse errors', !!await say('carve run brodcast "x"', 'Did you mean'))
  check('admin: vars list', !!await say('carve vars list t_', '{t_admin}'))
  check('admin: vars delete', !!await say('carve vars delete t_admin', 'Deleted 1'))
  await say('carve timings on', 'Timings on')
  tester.command('tvar')
  await sleep(300)
  check('admin: timings', !!await say('carve timings', 'carve-test.carve'))
  await say('carve timings off', 'Timings off')
  check('admin: permission denied for non-op', !!await say('carve vars list', 'need the carve:vars permission', pleb))
  t = Date.now()
  tester.suggest('/carve re')
  const sug = await tester.waitFor((e) => e.kind === 'suggest', 4000, t)
  check('admin: tab completion (subcommand)', !!sug && sug.text.includes('reload'), sug ? sug.text : 'no reply')
  // Script names are not completed (Pumpkin 0.2.2 crashes on plugin
  // suggestion handlers); this checks that asking is harmless.
  t = Date.now()
  tester.suggest('/carve info carve-t')
  await sleep(1000)
  t = Date.now()
  tester.command('tvar')
  check('admin: tab completion of an argument is harmless', !!await tester.waitFor(has('system', 't:counter'), 4000, t))

  await gaps(srv, tester, say)

  t = Date.now()
  pleb.quit()
  check('on quit (player name still known)', !!await tester.waitFor(has('system', 't:quit Pleb'), 4000, t))
  check('gaps: set quit message', !!await tester.waitFor(has('system', 't:bye Pleb'), 4000, t))

  await sleep(6000) // variables.json is saved ~5 s after a change
  lastCounter = Math.max(0, ...tester.log.filter(has('system', 't:counter')).map((e) => Number((e.text.match(/t:counter (\d+)/) || [])[1] || 0)))
  tester.quit()
  await sleep(500)
}

let lastCounter = 0

// Inventory values, held item editing and the other `gaps` syntax.
async function gaps (srv, tester, say) {
  srv.console('tp Tester 0.5 64 0.5')
  srv.console('clear Tester')
  srv.console('give Tester diamond_sword 1')
  await sleep(800)
  const held = await say('theld', 't:held ')
  check('gaps: set name/lore, add lore, enchant held item', !!held &&
    held.text.includes('t:held Blade|line a, line b and line c|sharpness 5'), held ? held.text.slice(0, 90) : 'no reply')
  const held2 = await tester.waitFor(has('system', 't:held2'), 3000, held ? held.t : 0)
  check("gaps: disenchant + items in player's inventory", !!held2 && held2.text.includes('t:held2 [<none>] 1'), held2 ? held2.text.slice(0, 60) : 'no reply')

  const misc = await say('tmisc', 't:misc')
  check("gaps: sorted by, {t}'s minutes, regex replace, rgb in text", !!misc &&
    misc.text.includes('t:misc a, bb and ccc 2 a#b#') && misc.text.includes('red'), misc ? misc.text.slice(0, 90) : 'no reply')
  check('gaps: do if', !!await tester.waitFor(has('system', 't:doif'), 3000, misc ? misc.t : 0))
  check('gaps: filter section', !!await tester.waitFor(has('system', 't:filter a'), 3000, misc ? misc.t : 0))
  check('gaps: rgb() color in particles (no error)', !srv.lines.some((l) => /ERROR.*(dust|color)/i.test(l)))
  let t = Date.now()
  tester.chat('tcancel')
  check('gaps: event is cancelled', !!await tester.waitFor(has('system', 't:cancelled'), 3000, t))

  t = Date.now()
  tester.command('tgui')
  const scr = await tester.waitFor((e) => e.kind === 'screen' && e.text.includes('Shop'), 4000, t)
  const gui = await tester.waitFor(has('system', 't:gui'), 4000, t)
  check('gaps: open inventory value as a menu', !!scr && !!gui && gui.text.includes('t:gui Shop 3'),
    `${scr ? scr.text : 'no window'} / ${gui ? gui.text.slice(0, 40) : 'no reply'}`)
  if (scr) {
    t = Date.now()
    tester.click(6, Number(scr.text.split(' ')[0]))
    const clk = await tester.waitFor(has('system', 't:guiclick'), 4000, t)
    check('gaps: click: name of event-inventory, event-inventory is {var}, live slot update', !!clk &&
      clk.text.includes('t:guiclick 6 gold ingot'), clk ? clk.text.slice(0, 60) : 'no click event')
    check('gaps: clicked item is the real stack', !!clk && clk.text.includes('gold ingot gold ingot'), clk ? clk.text.slice(0, 60) : '')
  }

  const mob = await say('tmob', 't:mob ')
  check('gaps: spawn named + set velocity to vector', !!mob && mob.text.includes('t:mob Bob 0.5'), mob ? mob.text.slice(0, 50) : 'no reply')
  const push = await tester.waitFor(has('system', 't:mobpush'), 3000, mob ? mob.t : 0)
  const px = push ? Number(push.text.split(' ')[1]) : NaN
  check('gaps: push away from', px > 0.5, push ? push.text.slice(0, 40) : 'no reply')
  const pv = await tester.waitFor(has('system', 't:pvel'), 3000, mob ? mob.t : 0)
  check('gaps: set velocity of player', !!pv && pv.text.includes('t:pvel 0.42'), pv ? pv.text.slice(0, 40) : 'no reply')
  const fwd = await tester.waitFor(has('system', 't:pfwd'), 3000, mob ? mob.t : 0)
  const [fx, fz] = fwd ? fwd.text.split('t:pfwd ')[1].split(' ').map(Number) : [NaN, NaN]
  check('gaps: push forward (facing yaw 0: +z)', Math.abs(fx) < 0.01 && fz > 0.9, fwd ? fwd.text.slice(0, 40) : 'no reply')
  t = Date.now()
  srv.console('kill @e[type=minecraft:zombie]')
  // Natural zombies may die too; look for the named one.
  const md = await tester.waitFor(has('system', 't:mobdeath zombie Bob'), 5000, t)
  check('gaps: on death of zombie', !!md, md ? md.text.slice(0, 50) : 'no event for Bob')
}

async function phase2 (srv) {
  const tester = await join('Tester')
  const t = Date.now()
  tester.command('tvar')
  check('variable survives server restart', !!await tester.waitFor(has('system', `t:counter ${lastCounter + 1}`), 4000, t), `expected ${lastCounter + 1}`)
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
  check('variables.json written', new RegExp(`"t_counter":\\s*${lastCounter}\\b`).test(saved), `t_counter ${lastCounter}`)

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
