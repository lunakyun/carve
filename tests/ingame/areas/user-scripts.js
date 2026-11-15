// User-scripts area: lines from real server scripts. Sounds at a place for
// everyone, `on place of`, `event-block's location`, priorities, damage cause
// lists, number words, `subtract`, `region at` (loads with a warning).
//   node tests/ingame/areas/user-scripts.js
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const now = () => Date.now() + 1
const t = area('user-scripts', { port: 25670 })
const SCRIPTS = [path.join(__dirname, 'user-scripts', 'user-scripts.carve')]

t.run(async () => {
  const srv = await startServer({ dir: t.dir, port: t.port, scripts: SCRIPTS, ops: ['Tester'] })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop()))
  const loadErr = srv.lines.find((l) => /\[Carve\].*(error|Error)/.test(l))
  t.check('script loads without errors', !!await srv.waitLog(/\[Carve\] Loaded 1 script/, 5000) && !loadErr, loadErr || '')
  t.check('region at warns at load', !!await srv.waitLog(/WorldGuard/, 3000))

  const bot = await joinBot('Tester', { port: t.port })
  t.cleanup(() => bot.quit())
  const other = await joinBot('Other', { port: t.port })
  t.cleanup(() => other.quit())
  await sleep(1500)

  // ---- priority: lowest first, monitor last ----
  const prio = other.packets
    .filter((p) => p.name === 'system_chat' && p.data && /^us:prio-/.test(p.data.text || ''))
    .map((p) => p.data.text.slice('us:prio-'.length))
  t.check('priorities run lowest to monitor', JSON.stringify(prio) === JSON.stringify(['lowest', 'normal', 'highest', 'monitor']), prio.join(','))

  // ---- sound at a place, to everyone ----
  let since = now()
  bot.command('us_sound')
  for (const [who, b] of [['Tester', bot], ['Other', other]]) {
    const s = await b.waitPacket('sound', (d) => d.sound === 'entity.tnt.primed', 4000, since)
    t.check(`sound at a place reaches ${who}`, !!s && Math.abs(s.x - 10) < 0.2 && Math.abs(s.y - 70) < 0.2 && Math.abs(s.z - 20) < 0.2 && Math.abs(s.volume - 0.7) < 0.01,
      s ? `${s.x} ${s.y} ${s.z} vol ${s.volume}` : 'none')
  }
  since = now()
  bot.command('us_sound_one')
  for (const [who, b] of [['Tester', bot], ['Other', other]]) {
    const s = await b.waitPacket('sound', (d) => d.sound === 'block.note_block.pling', 4000, since)
    t.check(`sound to all players reaches ${who}`, !!s && Math.abs(s.pitch - 2) < 0.01, s ? `pitch ${s.pitch}` : 'none')
  }

  // ---- number words, subtract, random element out of, region at ----
  since = now()
  bot.command('us_compare')
  t.check('player count is greater than', !!await bot.waitText('us:count-gt-1', { since }))
  t.check('subtract', !!await bot.waitText('us:subtract 7', { since }))
  t.check('random element out of a list', !!await bot.waitText('us:random-ok', { since }))
  t.check('region at is <none>', !!await bot.waitText('us:region-none', { since }))

  // ---- on place of ----
  const setblock = async (x, y, z, block) => {
    for (let i = 0; i < 40; i++) {
      const m = srv.mark()
      srv.console(`setblock ${x} ${y} ${z} minecraft:${block}`)
      const r = await srv.waitLog(/Changed the block|not loaded|Could not set/, 2000, m)
      if (r && /Changed the block/.test(r)) return
      await sleep(250)
    }
  }
  // Spawn protection only lets ops build, and the bot must stand within reach.
  srv.console('gamemode creative Tester')
  srv.console('tp Tester 0.5 64 0.5')
  await sleep(1500)
  await setblock(3, 64, 1, 'stone')
  await setblock(4, 64, 1, 'stone')
  since = now()
  srv.console('give Tester tnt 2')
  await bot.waitPacket('container_set_slot', (d) => d.item && d.item.name === 'tnt', 4000, since)
  bot.setHotbar(0)
  await sleep(300)
  since = now()
  bot.place(3, 65, 1)
  const tnt = await bot.waitText('us:place-tnt', { since })
  t.check('on place of tnt runs for tnt', !!tnt, tnt ? tnt.text : 'none')
  t.check('event-block and its location', !!tnt && /us:place-tnt tnt 3 65 1/.test(tnt.text), tnt ? tnt.text : 'none')
  t.check('on place of chest does not run for tnt', !await bot.waitText('us:place-box', { since, ms: 800 }))
  t.check('plain on place also runs', !!await bot.waitText('us:place-any', { since }))
  since = now()
  srv.console('give Tester chest 1')
  await bot.waitPacket('container_set_slot', (d) => d.item && d.item.name === 'chest', 4000, since)
  await sleep(300)
  bot.setHotbar(1)
  await sleep(300)
  since = now()
  bot.place(4, 65, 1)
  const box = await bot.waitText('us:place-box', { since })
  t.check('on place of chest or barrel runs for a chest', !!box, box ? box.text : 'none')
  t.check('on place of tnt does not run for a chest', !await bot.waitText('us:place-tnt', { since, ms: 800 }))

  // ---- damage cause lists ----
  srv.console('gamemode survival Tester')
  since = now()
  srv.console('damage Tester 1 minecraft:fall')
  t.check('damage cause is a list of causes', !!await bot.waitText('us:cause-hit', { since, ms: 4000 }))
})
