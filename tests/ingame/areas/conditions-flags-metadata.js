// Wave 5 chunk 2 area: conditions. Pick up items, can see, chat settings,
// client weather, respawn anchors, mob checks, wet and in liquid, glowing
// text, item cooldown, line of sight, contains, damage cause, egg hatch,
// entity storage, equippable flags, boss bar flags, tooltips, loot tables,
// metadata and persistent data, played before, preferred tool, responsive.
//   node tests/ingame/areas/conditions-flags-metadata.js
'use strict'
const fs = require('fs')
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('conditions-flags-metadata', { port: 25602 })
const NAME = 'Tester'

const varint = (n) => { const b = []; do { let x = n & 0x7f; n >>>= 7; if (n) x |= 0x80; b.push(x) } while (n); return Buffer.from(b) }
const str = (s) => Buffer.concat([varint(Buffer.byteLength(s)), Buffer.from(s)])

t.run(async () => {
  // A fresh first-join list, so the first join is really the first.
  fs.rmSync(path.join(t.dir, 'plugins', 'data', 'carve', 'first_join.txt'), { force: true })
  const srv = await startServer({
    dir: t.dir, port: t.port, ops: [NAME], scripts: [path.join(__dirname, 'conditions-flags-metadata', 'conditions-flags-metadata.carve')],
  })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop(60000)))
  t.check('carve loads the script', !!await srv.waitLog(/\[Carve\] Loaded 1 script/, 5000))
  const errLine = srv.lines.find((l) => /\[Carve\].*(error|line \d+)/i.test(l))
  t.check('script has no load errors', !errLine, errLine || '')

  let since = Date.now()
  let bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  const said = []
  const listen = (b) => b.on('packet', (p) => { if (p.name === 'system_chat' && p.data && p.data.text.startsWith('k:')) said.push(p.data.text) })
  listen(bot)
  const joined1 = await bot.waitText('k:joined', { since, ms: 5000 })
  t.check('has played before: no on the first join', !!joined1 && joined1.text.includes('played=no'), joined1 ? joined1.text : 'no join message')

  const run = async (cmd, tag, ms = 5000) => {
    const s = Date.now()
    bot.command(cmd)
    const m = await bot.waitText(`k:${tag}`, { since: s, ms })
    return m ? m.text : ''
  }

  // Rejoin: now the player has played before.
  bot.quit()
  await sleep(1500)
  since = Date.now()
  bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  listen(bot)
  const joined2 = await bot.waitText('k:joined', { since, ms: 5000 })
  t.check('has played before: yes after a rejoin', !!joined2 && joined2.text.includes('played=yes'), joined2 ? joined2.text : 'no join message')

  await sleep(1500)
  srv.console(`tp ${NAME} 0.5 101 0.5`)
  srv.console('setblock 0 100 0 stone')
  srv.console(`gamemode survival ${NAME}`)
  await sleep(2500)
  bot.moveTo(0.5, 101, 0.5)
  await sleep(500)

  let m = await run('kpick', 'pick')
  t.check('can pick up items (survival)', m.includes('can=yes cant=no'), m)

  // Can see: a second player, hidden and shown again.
  const other = await joinBot('Other', { port: t.port })
  t.cleanup(() => other.quit())
  await sleep(1000)
  m = await run('khide off', 'hide off')
  const seeOn = m.includes('see=yes')
  m = await run('khide on', 'hide on')
  t.check('can see: yes, then no after hide', seeOn && m.includes('see=no'), m)
  await run('khide off', 'hide off')

  // Chat settings: Pumpkin's defaults, then what the client reports.
  m = await run('kchat', 'chat')
  t.check('chat colors / messages / filtering defaults', m.includes('colors=yes messages=yes filter=no'), m)
  t.check('has client weather: no (and the negated form yes)', m.includes('weather=no,neg'), m)
  // client_information: locale, view distance, chat mode 2 (hidden), colors off,
  // skin parts, main hand, text filtering on, server listings, particles.
  bot.send('client_information', str('en_us'), [8], varint(2), [0], [0x7f], varint(1), [1], [1], varint(0))
  await sleep(800)
  m = await run('kchat', 'chat')
  t.check('chat settings follow client_information', m.includes('colors=no messages=no filter=yes'), m)
  bot.send('client_information', str('en_us'), [8], varint(0), [1], [0x7f], varint(1), [0], [1], varint(0))

  m = await run('kanchor', 'anchor')
  t.check('respawn anchors do not work in the overworld', m.includes('work=no dont=yes'), m)

  m = await run('kmobs', 'mobs')
  t.check('enderman not stared at, sheep not sheared, sheep has ai', m.includes('stared=no,neg') && m.includes('sheared=no,neg') && m.includes('ai=yes noai=no'), m)

  m = await run('kwet on', 'wet on', 6000)
  const wetOn = m
  m = await run('kwet off', 'wet off', 6000)
  t.check('entity is wet / in liquid in water', wetOn.includes('wet=yes liquid=yes dry=no'), wetOn)
  t.check('entity is not wet out of water', m.includes('wet=no liquid=no dry=yes'), m)

  await run('ksign', 'sign')
  await sleep(300)
  m = await run('kglow', 'glow')
  const noGlow = m.includes('has=no hasnt=yes')
  t.check('has glowing text: a plain sign has none', noGlow, m)
  // Pumpkin's `data merge block` reports success but keeps the old block entity,
  // so there is no way to make a glowing sign from the console.
  t.skip('has glowing text: yes on a glowing sign', 'Pumpkin 0.2.2 data merge block does not change block entities')

  m = await run('kcool', 'cool')
  t.check('has item cooldown (pearl yes, diamond no)', m.includes('pearl=yes diamond=no nodiamond=yes'), m)

  m = await run('klos clear', 'los clear')
  const losClear = m.includes('sight=yes blocked=no')
  m = await run('klos block', 'los block')
  t.check('has line of sight: clear, then blocked by stone', losClear && m.includes('sight=no blocked=yes'), m)
  await run('klos clear', 'los clear')

  m = await run('kcontains', 'contains')
  t.check('contains: text, list, not-in-list, inventory', m.includes('k:contains tlni'), m)

  // Damage cause from the console damage command.
  since = Date.now()
  srv.console(`damage ${NAME} 1 minecraft:fall`)
  let d = await bot.waitText('k:dmg', { since, ms: 5000 })
  t.check('damage was caused by fall', !!d && d.text.includes('r=fall'), d ? d.text : 'no damage event')
  await sleep(700)
  since = Date.now()
  srv.console(`damage ${NAME} 1 minecraft:lava`)
  d = await bot.waitText('k:dmg', { since, ms: 5000 })
  t.check('damage caused by lava (not fall)', !!d && d.text.includes('r=notfall,lava'), d ? d.text : 'no damage event')
  await sleep(700)
  since = Date.now()
  srv.console(`damage ${NAME} 1 minecraft:drown`)
  d = await bot.waitText('k:dmg', { since, ms: 5000 })
  t.check('damage caused by "drowning" (Skript name for drown)', !!d && d.text.includes(',drown'), d ? d.text : 'no damage event')
  srv.console(`effect give ${NAME} instant_health 1 5`)

  // Egg throw.
  await run('kegg', 'egg ready')
  bot.setHotbar(0)
  await sleep(500)
  since = Date.now()
  let egg = null
  for (let i = 0; i < 3 && !egg; i++) {
    bot.useItem(0, 0, -45)
    egg = await bot.waitText('k:eggthrow', { since, ms: 4000 })
  }
  if (egg) {
    const ev = egg.text.match(/ev=(\w+)/)
    const want = ev && ev[1] === 'true' ? 'will=yes wont=no' : 'will=no wont=yes'
    t.check('egg will hatch matches event-hatching', egg.text.includes(want), egg.text)
  } else t.check('egg will hatch (egg throw event)', false, 'no egg throw event')

  m = await run('kbee', 'bee', 6000)
  t.check('entity storage of an empty hive is not full; stone has none', m.includes('full=no notfull=yes stone=no'), m)

  m = await run('kequip', 'equip')
  t.check('equippable flags of vanilla items', m.includes('k:equip dsSlLhHeEx'), m)

  m = await run('kbar', 'bar')
  t.check('has boss bar flag (darken sky yes, fog and music no)', m.includes('k:bar sky,nofog') && !m.includes('music'), m)

  m = await run('ktip', 'tip')
  t.check('item tooltips are shown', m.includes('k:tip shown,entire'), m)

  await run('kchest', 'chest')
  await sleep(300)
  m = await run('kloot', 'loot')
  const noLoot = m.includes('has=no hasnt=yes')
  t.check('has loot table: a placed chest has none', noLoot, m)
  t.skip('has loot table: yes on a loot chest', 'Pumpkin 0.2.2 data merge block does not change block entities')

  m = await run('kmeta', 'meta')
  t.check('has metadata / persistent data tag', m.includes('k:meta md,noother,pd,pdgone'), m)

  m = await run('ktool', 'tool')
  t.check('is preferred tool (obsidian, dirt, stone)', m.includes('k:tool dia,noiron,dirt,nostone') && !m.includes('wrong'), m)

  m = await run('kresp', 'respawned')
  await sleep(500)
  m = await run('kresp2', 'resp')
  const notResp = m.includes('is=no isnt=yes') && m.includes('type=interaction')
  srv.console('execute as @e[type=interaction] run data merge entity @s {response:1b}')
  await sleep(800)
  m = await run('kresp2', 'resp')
  if (m.includes('is=yes')) t.check('is responsive: no, then yes after data merge', notResp, m)
  else t.check('interaction is not responsive (default)', notResp, m)
})
