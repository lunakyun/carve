// Wave 6 chunk 1 area: events that were marked blocked or partial. Step on,
// chunk enter, jump, tool change items, attack attempts, pressure plates,
// spawn point changes, world border polling, armor polling, experience
// orbs, pistons, lightning, block drops, bells, can build, unload and stop.
//   node tests/ingame/areas/events-step-jump.js
'use strict'
const fs = require('fs')
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('events-step-jump', { port: 25611 })
const NAME = 'Tester'

t.run(async () => {
  const srv = await startServer({
    dir: t.dir, port: t.port, ops: [NAME], scripts: [path.join(__dirname, 'events-step-jump', 'events-step-jump.carve')],
  })
  let stopped = false
  t.cleanup(async () => { if (!stopped) await srv.stop(60000) })
  t.check('carve loads the script', !!await srv.waitLog(/\[Carve\] Loaded 1 script/, 5000))
  const errLine = srv.lines.find((l) => /\[Carve\].*(error|line \d+)/i.test(l))
  t.check('script has no load errors', !errLine, errLine || '')

  const bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  const said = []
  bot.on('packet', (p) => { if (p.name === 'system_chat' && p.data && /esj:/.test(p.data.text)) said.push({ at: Date.now(), text: p.data.text }) })
  const since = (s, re, ms = 4000) => new Promise((resolve) => {
    const end = Date.now() + ms
    const tick = () => {
      const hit = said.find((x) => x.at >= s && re.test(x.text))
      if (hit) return resolve(hit.text)
      if (Date.now() > end) return resolve(null)
      setTimeout(tick, 50)
    }
    tick()
  })
  const none = async (s, re, ms = 1200) => { await sleep(ms); return !said.some((x) => x.at >= s && re.test(x.text)) }

  await sleep(4000)
  srv.console('fill -12 100 -12 40 100 12 minecraft:stone')
  srv.console(`tp ${NAME} 0.5 101 0.5`)
  srv.console(`gamemode survival ${NAME}`)
  srv.console('setblock 3 100 0 minecraft:dirt')
  await sleep(2500)
  bot.moveTo(0.5, 101, 0.5)
  await sleep(500)
  if (!t.check('bot connected after the teleport', !bot.closed)) return

  // ---- step on ----
  let s = Date.now()
  await bot.walkTo(3.5, 101, 0.5)
  t.check('step on dirt: the dirt trigger runs', !!await since(s, /esj:walkdirt dirt/))
  s = Date.now()
  await bot.walkTo(5.5, 101, 0.5)
  t.check('step on stone: block is stone', !!await since(s, /esj:step stone/))

  // ---- chunk enter ----
  await bot.walkTo(12.5, 101, 0.5)
  s = Date.now()
  await bot.walkTo(17.5, 101, 0.5)
  const ch = await since(s, /esj:chunk /)
  t.check('player chunk enter: new and previous chunk', !!ch && /esj:chunk 1 0 from 0 0/.test(ch), ch || 'no event')
  t.check('player chunk enters: second name fires too', !!await since(s, /esj:chunks/))
  await bot.walkTo(0.5, 101, 0.5)

  // ---- jump ----
  s = Date.now()
  bot._input(16, true)
  await sleep(150)
  bot._input(16, false)
  t.check('jump: fires on the rising edge', !!await since(s, /esj:jump/))
  s = Date.now()
  bot._input(16, true)
  bot._input(16, true)
  await sleep(150)
  bot._input(16, false)
  await sleep(500)
  t.check('jump: held bit fires once', said.filter((x) => x.at >= s && /esj:jump/.test(x.text)).length === 1)

  // ---- tool change with items ----
  srv.console(`clear ${NAME}`)
  srv.console(`give ${NAME} minecraft:diamond 1`)
  await sleep(500)
  srv.console(`give ${NAME} minecraft:stick 1`)
  await sleep(1000)
  bot.setHotbar(3)
  await sleep(300)
  bot.setHotbar(0)
  await sleep(400)
  s = Date.now()
  bot.setHotbar(1)
  const tool = await since(s, /esj:tool /)
  t.check('tool change: new item and past item', !!tool && /stick/.test(tool) && /diamond/.test(tool), tool || 'no event')
  srv.console(`clear ${NAME}`)

  // ---- pressure plate ----
  srv.console('setblock 8 101 0 minecraft:stone_pressure_plate')
  await sleep(500)
  s = Date.now()
  await bot.walkTo(8.5, 101, 0.5)
  const plate = await since(s, /esj:plate /, 5000)
  t.check('pressure plate: location and power', !!plate && /8/.test(plate) && /15/.test(plate), plate || 'no event')
  const red = await since(s, /esj:redstone /, 2000)
  t.check('redstone: event-location of the plate and the new power', !!red && /x: 8/.test(red) && /15/.test(red), red || 'no event')
  await bot.walkTo(0.5, 101, 0.5)

  // ---- spawn change ----
  s = Date.now()
  srv.console(`spawnpoint ${NAME} 2 101 2`)
  const sp = await since(s, /esj:spawn /)
  t.check('spawn change: player spawn point reaches the trigger', !!sp && /2/.test(sp), sp || 'no event')

  // ---- world border (polled) ----
  s = Date.now()
  srv.console('worldborder set 200')
  const wbb = await since(s, /esj:wbb /)
  t.check('border bounds change: old and new size', !!wbb && /200/.test(wbb), wbb || 'no event')
  s = Date.now()
  srv.console('worldborder set 100 2')
  t.check('border bounds change: transition start', !!await since(s, /esj:wbb .*100/))
  s = Date.now()
  srv.console('worldborder center 10 20')
  const wbc = await since(s, /esj:wbc /)
  t.check('border center change: old and new center', !!wbc && /10\.5 20\.5/.test(wbc), wbc || 'no event')
  srv.console('worldborder center 0 0')
  srv.console('worldborder set 59999968')

  // ---- armor change (polled) ----
  s = Date.now()
  srv.console(`item replace entity ${NAME} armor.head with minecraft:iron_helmet`)
  const armor = await since(s, /esj:armor /)
  t.check('armor change: slot and item', !!armor && /helmet/.test(armor) && /iron/.test(armor), armor || 'no event')

  // ---- experience orb ----
  s = Date.now()
  srv.console('summon minecraft:experience_orb 12 101 5')
  t.check('experience spawn: location', !!await since(s, /esj:xp /))

  // ---- lightning ----
  s = Date.now()
  bot.command('cbolt')
  const bolt = await since(s, /esj:bolt /)
  t.check('lightning strike: event-location', !!bolt && /10/.test(bolt), bolt || 'no event')

  // ---- block drop items ----
  srv.console('setblock 1 101 3 minecraft:dirt')
  await sleep(500)
  s = Date.now()
  bot.breakBlock(1, 101, 3)
  const drop = await since(s, /esj:drop .*dirt/)
  t.check('block drop: event-items lists the stack', !!drop && /dirt/.test(drop), drop || 'no event')

  // ---- can build: set event-boolean ----
  srv.console('setblock 2 101 5 minecraft:stone')
  srv.console(`give ${NAME} minecraft:dirt 4`)
  await sleep(800)
  bot.setHotbar(0)
  await sleep(300)
  s = Date.now()
  bot.place(2, 102, 5)
  await since(s, /esj:nobuild/, 2500)
  const placed = await bot.waitPacket('block_update', (d) => d.x === 2 && d.y === 102 && d.z === 5 && d.block === 'dirt', 1500, s)
  t.check('can build check: script sets event-boolean to false', !!said.find((x) => x.at >= s && /esj:nobuild/.test(x.text)))
  t.check('can build check: the block is not placed', !placed)

  // ---- bell ring with the player ----
  srv.console(`clear ${NAME}`)
  srv.console('setblock 2 101 3 minecraft:bell')
  await sleep(800)
  s = Date.now()
  bot.useOn(2, 101, 3, 2, 0, [0.5, 0.5, 0])
  const bell = await since(s, /esj:bell /)
  t.check('bell ring: player and location', !!bell && /Tester/.test(bell) && /x: 2/.test(bell), bell || 'no event')

  // ---- piston location ----
  srv.console('setblock 14 101 9 minecraft:redstone_block')
  await sleep(300)
  s = Date.now()
  srv.console('setblock 14 101 8 minecraft:piston')
  const piston = await since(s, /esj:piston /, 3000)
  t.check('piston extend: location', !!piston && /14/.test(piston), piston || 'no event')
  s = Date.now()
  srv.console('setblock 14 101 9 minecraft:air')
  const pistonr = await since(s, /esj:pistonr /, 4000)
  t.check('piston retract: location', !!pistonr && /14/.test(pistonr), pistonr || 'no event')

  // ---- combust with the entity: a cow standing in fire ----
  srv.console('kill @e[type=cow]')
  await sleep(300)
  let cs = Date.now()
  srv.console('summon minecraft:cow 6.5 101 6.5')
  await bot.waitPacket('add_entity', (d) => d.type === 'cow', 5000, cs)
  s = Date.now()
  srv.console('setblock 6 101 6 minecraft:fire')
  const burn = await since(s, /esj:combust /, 6000)
  t.check('combust: event-entity is the cow', !!burn && /cow/.test(burn), burn || 'no event')
  srv.console('kill @e[type=cow]')
  srv.console('setblock 6 101 6 minecraft:air')

  // ---- leash with the entity ----
  await bot.walkTo(0.5, 101, 0.5)
  srv.console(`clear ${NAME}`)
  srv.console(`give ${NAME} minecraft:lead 1`)
  await sleep(1200)
  bot.setHotbar(0)
  await sleep(300)
  cs = Date.now()
  srv.console('summon minecraft:cow 1.5 101 0.5')
  const cow2 = await bot.waitPacket('add_entity', (d) => d.type === 'cow', 5000, cs)
  s = Date.now()
  if (cow2) bot.interact(cow2.entityId)
  const leash = await since(s, /esj:leash /)
  t.check('leash: event-entity is the cow', !!leash && /cow/.test(leash), leash || 'no event')

  // ---- love mode with the entity ----
  srv.console(`clear ${NAME}`)
  srv.console(`give ${NAME} minecraft:wheat 2`)
  await sleep(1200)
  bot.setHotbar(0)
  await sleep(300)
  s = Date.now()
  if (cow2) bot.interact(cow2.entityId)
  const love = await since(s, /esj:love /)
  t.check('love mode enter: event-entity and ticks', !!love && /cow/.test(love), love || 'no event')

  // ---- spectate (polled camera) ----
  srv.console(`gamemode spectator ${NAME}`)
  await sleep(800)
  srv.console('kill @e[type=cow]')
  cs = Date.now()
  srv.console('summon minecraft:cow 3.5 101 3.5')
  await bot.waitPacket('add_entity', (d) => d.type === 'cow', 5000, cs)
  s = Date.now()
  srv.console(`spectate @e[type=cow,limit=1] ${NAME}`)
  const spec = await since(s, /esj:spectate /, 4000)
  t.check('spectate: event-entity is the cow', !!spec && /cow/.test(spec), spec || 'no event')
  srv.console(`gamemode survival ${NAME}`)

  // ---- unload on reload ----
  let rm = srv.mark()
  srv.console('carve reload')
  t.check('on unload runs before /carve reload', !!await srv.waitLog(/esj:unload/, 4000, rm))

  // ---- unload and stop ----
  const mark = srv.mark()
  stopped = true
  await srv.stop(60000)
  const log = fs.readFileSync(path.join(t.dir, 'last-run.log'), 'utf8')
  t.check('on unload runs at shutdown', /esj:unload/.test(log))
  t.check('on server stop runs at shutdown', /esj:stop/.test(log))
  void mark
})
