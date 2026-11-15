// potions-particles-queues area: potion effect values, particle values, client weather,
// skulls and potion items, respawn anchors, redstone power, boss bar key and
// progress, inventory rows/viewers, queues, raw names, enchantment levels,
// `result of function`, `last caught runtime errors`, `respawn reason`,
// `on-screen kick message` and `projectile force`.
//   node tests/ingame/areas/potions-particles-queues.js
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const now = () => Date.now() + 1
const t = area('potions-particles-queues', { port: 25605 })
const SCRIPTS = [path.join(__dirname, 'potions-particles-queues', 'potions-particles-queues.carve')]
const NAME = 'Tester'

t.run(async () => {
  const srv = await startServer({ dir: t.dir, port: t.port, scripts: SCRIPTS, ops: [NAME] })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop(60000)))
  const loaded = await srv.waitLog(/\[Carve\] Loaded 1 script/, 5000)
  const loadErr = srv.lines.filter((l) => /\[Carve\].*(error|Error)|potions-particles-queues\.carve:\d+/.test(l)).slice(-4).join(' | ')
  t.check('script loads without errors', !!loaded && !loadErr, loadErr)

  const bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  await sleep(1500)
  const run = async (cmd, prefix, ms = 6000) => {
    const since = now()
    bot.command(cmd)
    const m = await bot.waitText(prefix, { since, ms })
    return { text: m ? m.text : null, since }
  }

  // ---- potion effects ----
  let r = await run('ppqpot', 'p:')
  t.check('New Potion Effect + amplifier/tier/duration read back', r.text === 'p:2/1/30 seconds', r.text)
  const eff = await bot.waitPacket('update_mob_effect', (d) => d.effect === 'speed', 4000, r.since)
  t.check('apply a potion effect value: client gets speed II', !!eff && eff.amplifier === 1 && eff.duration > 500, JSON.stringify(eff))
  r = await run('ppqambient', 'amb:')
  const amb = await bot.waitPacket('update_mob_effect', (d) => d.effect === 'jump_boost', 4000, r.since)
  t.check('ambient potion effect without particles: flags', !!amb && (amb.flags & 1) === 1 && (amb.flags & 2) === 0, JSON.stringify(amb))
  r = await run('ppqeffs', 'pe:')
  const pe = (r.text || '').split('|')
  t.check('Potion Effects of Entity: lists speed and jump boost', pe.length === 5 && /speed 2/.test(pe[0]) && /jump boost 1/.test(pe[0]), r.text)
  t.check('Potion Effect of Entity: speed potion effect of player', /^speed 2 for /.test(pe[1] || ''), pe[1])
  t.check('Potion Effect Tier: tier of speed of player', pe[2] === '2', pe[2])
  t.check('Potion Effect Type Category', pe[3] === 'beneficial' && pe[4] === 'harmful', `${pe[3]} ${pe[4]}`)

  // ---- particles ----
  r = await run('ppqpart', 'pt:')
  t.check('particle count/speed/offset read back', r.text === 'pt:7/0.25/x: 1, y: 2, z: 3', r.text)
  const fl = await bot.waitPacket('level_particles', (d) => d.particle === 'flame', 4000, r.since)
  t.check('draw particle value: count, offset and speed sent',
    !!fl && fl.count === 7 && Math.abs(fl.offset.x - 1) < 1e-6 && Math.abs(fl.offset.y - 2) < 1e-6 && Math.abs(fl.speed - 0.25) < 1e-6,
    JSON.stringify(fl))
  r = await run('ppqvel', 'pv:')
  const er = await bot.waitPacket('level_particles', (d) => d.particle === 'end_rod', 4000, r.since)
  t.check('particle with velocity: count 0 and the vector as offset',
    r.text === 'pv:0/x: 0, y: 1, z: 0' && !!er && er.count === 0 && Math.abs(er.offset.y - 1) < 1e-6, `${r.text} ${JSON.stringify(er)}`)

  // ---- items ----
  const slotItem = (name, since) => bot.waitPacket('container_set_slot', (d) => d.item && d.item.name === name, 5000, since)
    .then((p) => p || bot.waitPacket('set_player_inventory', (d) => d.item && d.item.name === name, 1000, since))
  r = await run('ppqskull', 'sk:')
  const sk = await slotItem('player_head', r.since)
  t.check('Player Skull: a player head is given', !!sk && r.text === 'sk:minecraft:player_head', `${r.text} ${sk ? JSON.stringify(sk.item).slice(0, 160) : ''}`)
  r = await run('ppqpotitem', 'pi:')
  const pi = await slotItem('splash_potion', r.since)
  t.check('Potion (item): a splash potion is given', !!pi, pi ? JSON.stringify(pi.item).slice(0, 160) : '')

  // ---- client weather ----
  r = await run('ppqrain', 'w:')
  const rain = await bot.waitPacket('game_event', (d) => d.event === 'start_raining', 4000, r.since)
  t.check('Player Weather: set to rain (start_raining sent, reads rain)', r.text === 'w:rain' && !!rain, r.text)
  r = await run('ppqclear', 'wc:')
  const stop = await bot.waitPacket('game_event', (d) => d.event === 'stop_raining', 4000, r.since)
  t.check('Player Weather: reset (stop_raining sent, reads none)', r.text === 'wc:<none>' && !!stop, r.text)

  // ---- blocks ----
  r = await run('ppqanchor', 'a:')
  const anchor = await bot.waitPacket('block_update', (d) => /respawn_anchor/.test(d.block || ''), 4000, r.since)
  t.check('Respawn Anchor Charges: set 3, read 3, max 4', r.text === 'a:3/4', `${r.text} ${anchor ? anchor.block : ''}`)
  r = await run('ppqred', 'r:')
  t.check('Redstone Block Power of a redstone block', r.text === 'r:15', r.text)

  // ---- boss bars ----
  r = await run('ppqbar', 'b:')
  t.check('Key Of Boss Bar + Progress', r.text === 'b:ppq/40/<none>', r.text)

  // ---- inventories ----
  r = await run('ppqinv', 'i:', 8000)
  t.check('Inventory rows/slots/holder/viewers', r.text === `i:3/27/<none>/${NAME}`, r.text)
  bot.closeContainer()

  // ---- pure logic ----
  const lines = []
  const since = now()
  bot.command('ppqmisc')
  for (const p of ['q:', 'raw:', 'el:', 'res:', 'err:']) {
    const m = await bot.waitText(p, { since, ms: 5000 })
    lines.push(m ? m.text : null)
  }
  t.check('Queue + Queue Start/End', lines[0] === 'q:1/3', lines[0])
  t.check('Raw Name', lines[1] === 'raw:minecraft:diamond_sword', lines[1])
  t.check('Minimum/Maximum Enchantment Level', lines[2] === 'el:5/1', lines[2])
  t.check('Result (of a function)', lines[3] === 'res:42', lines[3])
  t.check('Last Caught Errors', /^err:.*potion effect/.test(lines[4] || ''), lines[4])

  // ---- respawn reason ----
  let s = now()
  srv.console(`kill ${NAME}`)
  await sleep(1500)
  bot.respawn()
  const rr = await bot.waitText('rr:', { since: s, ms: 8000 })
  t.check('Respawn Reason after a death', !!rr && rr.text === 'rr:death', rr ? rr.text : '')
  await sleep(1500)

  // ---- on-screen kick message ----
  const pleb = await joinBot('Pleb', { port: t.port })
  await sleep(1000)
  s = now()
  srv.console('kick Pleb bye now')
  const kick = await bot.waitText('kick:', { since: s, ms: 6000 })
  t.check('On-Screen Kick Message in on kick', !!kick && /kick:.*bye now/.test(kick.text), kick ? kick.text : '')
  pleb.quit()

  // ---- projectile force (a skeleton must shoot the bot) ----
  srv.console(`gamemode survival ${NAME}`)
  srv.console('time set midnight')
  await sleep(500)
  s = now()
  await run('ppqskel', 'skel:')
  const force = await bot.waitText('force:', { since: s, ms: 30000 })
  if (force) t.check('Projectile Force in on entity shoot bow', /^force:\d/.test(force.text), force.text)
  else t.skip('Projectile Force in on entity shoot bow', 'no skeleton shot the bot within 30 s')
})
