// Freeze area: scripts that try to hang or stall the server. After each one
// the server must still answer (a chat round trip within 3 s) and the script
// must have been stopped with an error that names the script and line.
//   node tests/ingame/areas/freeze.js [case-name ...]
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('freeze', { port: 25660 })
const SCRIPT = path.join(__dirname, 'freeze', 'freeze.carve')

const GUARD = /freeze\.carve:\d+: .*(ran more than [\d,]+ instructions|too much work|too many|longer than|too deep|nested|levels deep|too complex|took more than|goes up to|gives at most)/

// name: [command or {chat}, log pattern the run must produce (null: none needed), extra wait in ms]
const CASES = [
  ['fzwhile', GUARD], ['fznested', GUARD], ['fzloopmax', GUARD], ['fzloopbig', GUARD],
  ['fzloopneg', null], ['fzloopnan', /freeze\.carve:\d+:/], ['fzloopinf', /freeze\.carve:\d+:/],
  ['fzwhilewait0', GUARD], ['fzwhilewaits0', GUARD], ['fzwaitmany', null],
  ['fzrecurse', /levels deep/], ['fzmutual', /levels deep/], ['fzrecwait', null, 4000], ['fzfork', GUARD],
  ['fzcmdself', /freeze\.carve:\d+: .*command .* skipped|nest more than/, 1500],
  ['fzcmdconsole', /freeze\.carve:\d+: .*command .* skipped|nest more than/, 1500],
  ['fzcmdping', /freeze\.carve:\d+: .*command .* skipped|nest more than/, 1500],
  [{ chat: 'fzsay one' }, null, 1500],
  [{ chat: 'fzbroadcast one' }, null, 800],
  ['fztp', null, 1500], ['fzdamage', null, 1500],
  ['fzblocks', null], ['fzblocksloop', GUARD], ['fzentities', GUARD],
  ['fzrepeat', GUARD], ['fzrepeatnest', GUARD], ['fzdoubling', GUARD], ['fzjoin', GUARD],
  ['fzlistmillions', GUARD], ['fzsorted', GUARD],
  ['fzregex', /too complex/], ['fzregexlong', /too complex/], ['fzregexmany', /too complex/],
  ['fzrange', GUARD], ['fzrangeloop', GUARD], ['fzfill', GUARD],
  ['fzspawn', null, 1500], ['fzspawnloop', GUARD, 1500], ['fzparticles', null, 1500],
  ['fzparticleloop', GUARD, 1500], ['fzexplode', GUARD, 1500],
  ['fzexplode1', null], ['fzexplode20', null], ['fzparticle1', null], ['fzparticle100', null],
  ['fzspin', GUARD, 1500], ['fzspinoff', null],
]
const only = process.argv.slice(2)

t.run(async () => {
  let srv
  let bot
  const boot = async () => {
    srv = await startServer({ dir: t.dir, port: t.port, scripts: [SCRIPT], ops: ['Tester'] })
    bot = await joinBot('Tester', { port: t.port })
    await sleep(500)
    srv.console('gamemode creative Tester')
    await sleep(300)
  }
  await boot()
  t.cleanup(async () => { try { bot.quit() } catch {} ; await srv.stop() })
  t.check('freeze.carve loads without errors', !!await srv.waitLog(/\[Carve\] Loaded \d+ script/, 5000) &&
    !srv.lines.some((l) => /freeze\.carve.*Line \d+|Line \d+: \(freeze\.carve\)/.test(l)),
    srv.lines.filter((l) => /error|Line \d+:|Can't/i.test(l)).slice(0, 4).join(' | '))

  // Chat round trip: the server answers within 3 s.
  const ping = async () => {
    const since = Date.now()
    bot.command('fzping')
    const hit = await bot.waitText('fz:pong', { since, ms: 3000 })
    return hit ? Date.now() - since : -1
  }

  for (const [what, want, extra = 0] of CASES) {
    const name = typeof what === 'string' ? what : 'chat:' + what.chat
    if (only.length && !only.some((o) => name.includes(o))) continue
    const mark = srv.mark()
    const since = Date.now()
    if (typeof what === 'string') bot.command(what)
    else bot.chat(what.chat)
    // The ping is queued behind the case, so the wait is how long the server was busy.
    bot.command('fzping')
    const pong = await bot.waitText('fz:pong', { since, ms: 3000 })
    const busy = pong ? Date.now() - since : -1
    const hit = want ? await srv.waitLog(want, 3000, mark) : null
    if (extra) await sleep(extra)
    const ms = pong && extra ? await ping() : busy
    const answered = ms >= 0
    t.check(`${name}: server answers within 1 s (no stall)`, answered && busy < 1000, answered ? `${busy} ms` : 'no reply in 3 s')
    if (want) t.check(`${name}: logged an error with script and line`, !!hit, hit ? hit.slice(0, 140) : srv.lines.slice(mark).slice(-3).join(' | ').slice(0, 200))
    if (!answered) {
      // Frozen: kill it and start over so the next case is judged on its own.
      try { srv.proc.kill() } catch {}
      try { bot.quit() } catch {}
      await sleep(1000)
      await boot()
    }
  }
  // Nothing in the log may be a crash.
  t.check('no panic or trap in the server log', !srv.lines.some((l) => /panicked|wasm trap|unreachable|stack overflow|out of fuel/i.test(l)),
    srv.lines.filter((l) => /panicked|trap|overflow/i.test(l)).slice(0, 3).join(' | '))
})
