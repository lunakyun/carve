// Wave 5 chunk 4: block data, block spheres and lines, written books, boss bar
// by id, brewing stand slots, world border settings, potion effect values,
// item damage, ender chests, enderman blocks, AI, attributes, enchanting
// values, inventory actions, cursor slot and event values.
//   node tests/ingame/areas/w5-chunk4.js
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('w5-chunk4', { port: 25604 })
const NAME = 'Tester'

t.run(async () => {
  const srv = await startServer({ dir: t.dir, port: t.port, ops: [NAME], scripts: [path.join(__dirname, 'w5-chunk4', 'w5-chunk4.carve')] })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop()))
  t.check('carve loads the script', !!await srv.waitLog(/\[Carve\] Loaded 1 script/, 8000))
  const errLines = srv.lines.filter((l) => /\[Carve\].*(error|Error)/.test(l))
  t.check('script loads without errors', errLines.length === 0, errLines.join(' / '))

  const bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  const seen = []
  bot.on('packet', (p) => seen.push({ name: p.name, t: Date.now(), data: p.data }))
  const packets = (name, since, pred = () => true) => seen.filter((p) => p.name === name && p.t >= since && pred(p.data || {}))
  srv.console(`gamemode creative ${NAME}`)
  await sleep(1500)

  const run = async (cmd, prefix, ms = 6000) => {
    const since = Date.now()
    bot.command(cmd)
    const m = await bot.waitText(prefix, { since, ms })
    return m ? m.text : ''
  }
  const field = (m, i) => (m.split(' ').slice(1).join(' ').split('|')[i] || '').trim()
  const base = () => ({ x: Math.floor(bot.pos.x), y: Math.floor(bot.pos.y), z: Math.floor(bot.pos.z) })

  // ---- block data ----
  let b = base()
  let since = Date.now()
  let m = await run(`c4bd ${b.x + 2} ${b.y + 3} ${b.z}`, 'c4:bd')
  await sleep(400) // block changes go out at the end of the tick, after the chat line
  t.check('block data: set and read', /oak_stairs\[.*facing=west/.test(m) && /half=top/.test(m), m)
  const at = (d) => d.x === b.x + 2 && d.y === b.y + 3 && d.z === b.z
  const bu = packets('block_update', since, at).map((p) => p.data)
    .concat(...packets('section_blocks_update', since).map((p) => p.data.blocks.filter(at)))
  t.check('block data: block update for the stairs', bu.some((d) => /oak_stairs/.test(d.block || '')), bu.map((d) => d.block).join(','))

  // ---- block sphere ----
  since = Date.now()
  m = await run(`c4sphere ${b.x - 4} ${b.y + 4} ${b.z}`, 'c4:sphere')
  await sleep(400)
  t.check('blocks in radius 1: 7 blocks, set to glass', field(m, 0) === '7' && /glass/.test(field(m, 1)) && !/glass/.test(field(m, 2)), m)
  const glass = packets('block_update', since, (d) => /glass/.test(d.block || '')).length + packets('section_blocks_update', since).length
  t.check('blocks in radius: the client sees the glass', glass > 0, `${glass} updates`)

  // ---- blocks between / from to / above ----
  m = await run(`c4blocks ${b.x} ${b.y + 6} ${b.z + 4}`, 'c4:blocks')
  t.check('blocks between a and b: 8', field(m, 0) === '8', m)
  t.check('blocks from a to b: a line of 4', field(m, 1) === '4' && /stone/.test(field(m, 3)), m)
  t.check('blocks above x: 100', field(m, 2) === '100', m)

  // ---- books ----
  since = Date.now()
  m = await run('c4book', 'c4:book')
  t.check('book title / author / pages of player\'s tool', m === 'c4:book Tales|Ann|2|page two', m)
  const bs = packets('container_set_slot', since, (d) => d.item && d.item.name === 'written_book').concat(packets('set_player_inventory', since, (d) => d.item && d.item.name === 'written_book'))
  t.check('book: the written book reaches the client', bs.length > 0, `${bs.length} slot packets`)

  // ---- boss bar from key ----
  since = Date.now()
  m = await run('c4bar', 'c4:bar')
  t.check('boss bar with id: the id, none for an unknown id', m === 'c4:bar c4bar|<none>', m)
  const bar = await bot.waitPacket('boss_event', () => true, 2000, since)
  t.check('show bossbar {_b}: the client gets the bar', !!bar, bar ? JSON.stringify(bar).slice(0, 80) : 'none')

  // ---- brewing stand slots ----
  b = base()
  m = await run(`c4brew ${b.x - 2} ${b.y} ${b.z - 3}`, 'c4:brew')
  t.check('brewing stand slots: set and read bottle and ingredient', /^c4:brew potion\|nether wart\|<none>$/.test(m), m)

  // ---- world border ----
  since = Date.now()
  m = await run('c4border', 'c4:border')
  t.check('world border center / damage amount / damage buffer', m === 'c4:border 10|20|2|7', m)
  const center = packets('set_border_center', since).length + packets('initialize_border', since, (d) => d.x === 10).length
  t.check('world border center: the client gets the new center', center > 0, `${center} packets`)

  // ---- potion effect values ----
  since = Date.now()
  await run('c4effect', 'c4:effect')
  await sleep(300)
  const eff = packets('update_mob_effect', since)
  const speed = eff.find((p) => p.data.effect === 'speed')
  const jump = eff.find((p) => p.data.effect === 'jump_boost')
  t.check('potion effect of speed of tier 2 for 10 seconds', speed && speed.data.amplifier === 1 && speed.data.duration === 200, speed ? JSON.stringify(speed.data) : eff.map((p) => p.data.effect).join(','))
  t.check('ambient ... without particles', jump && (jump.data.flags & 1) === 1 && (jump.data.flags & 2) === 0, jump ? JSON.stringify(jump.data) : 'none')

  // ---- item damage ----
  srv.console(`gamemode survival ${NAME}`)
  await sleep(500)
  bot.setHotbar(0)
  since = Date.now()
  bot.command('c4dmg')
  const d1 = await bot.waitText('c4:dmg1', { since, ms: 6000 })
  const d2 = await bot.waitText('c4:dmg2', { since, ms: 6000 })
  await sleep(400)
  const dslots = packets('container_set_slot', since, (d) => d.item && d.item.name === 'diamond_sword').concat(packets('set_player_inventory', since, (d) => d.item && d.item.name === 'diamond_sword'))
  const dmgs = dslots.map((p) => p.data.item.components && p.data.item.components.damage)
  t.check('give diamond sword with damage 10', dmgs.includes(10) && d1 && d1.text === 'c4:dmg1 10|1551', `${d1 ? d1.text : 'none'} wire ${dmgs.join(',')}`)
  t.check('set damage of player\'s tool / durability', dmgs.includes(5) && d2 && d2.text === 'c4:dmg2 5|1556', `${d2 ? d2.text : 'none'} wire ${dmgs.join(',')}`)

  // ---- ender chest ----
  since = Date.now()
  m = await run('c4ender', 'c4:ender')
  t.check('slot of ender chest: set and read', m === 'c4:ender diamond|3 emerald|2' || m === 'c4:ender diamond|3 emeralds|2', m)
  const open = await bot.waitPacket('open_screen', () => true, 3000, since)
  t.check('open player\'s ender chest', open && /generic_9x3$/.test(open.type), open ? `${open.type} "${open.title}"` : 'none')
  await sleep(400)
  const content = open && packets('container_set_content', since, (d) => d.windowId === open.windowId)[0]
  t.check('ender chest window holds the diamond', content && content.data.items[0] && content.data.items[0].name === 'diamond', content ? content.data.items.slice(0, 3).map((i) => i && i.name).join(',') : 'none')
  if (open) bot.closeContainer()
  m = await run('c4enderclear', 'c4:enderclear')
  t.check('clear ender chest of player', m === 'c4:enderclear 0', m)

  // ---- enderman block, AI ----
  m = await run('c4mobs', 'c4:carry')
  t.check('carried block of an enderman: set and read', /grass_block/.test(m), m)
  const ai = await bot.waitText('c4:ai ', { since: Date.now() - 3000, ms: 3000 })
  const ai2 = await bot.waitText('c4:ai2', { since: Date.now() - 3000, ms: 3000 })
  t.check('ai of a mob: set to false and read; a player has none', ai && ai.text === 'c4:ai false|<none>', ai ? ai.text : 'none')
  t.check('ai of a mob: set back to true', ai2 && ai2.text === 'c4:ai2 true', ai2 ? ai2.text : 'none')

  // ---- attributes ----
  since = Date.now()
  m = await run('c4attr', 'c4:attr')
  t.check('movement speed attribute of player: set and read', /^c4:attr 0\.3\|20$/.test(m), m)
  const attr = await bot.waitPacket('update_attributes', (d) => d.entityId === bot.entityId && d.attributes.movement_speed && Math.abs(d.attributes.movement_speed.value - 0.3) < 1e-6, 3000, since)
  t.check('attribute change reaches the client', !!attr, attr ? JSON.stringify(attr.attributes.movement_speed) : 'no update_attributes with 0.3')
  m = await run('c4attrreset', 'c4:attrreset')
  t.check('reset attribute', /^c4:attrreset 0\.1/.test(m), m)

  // ---- enchanting ----
  b = base()
  const tx = b.x + 1; const ty = b.y; const tz = b.z + 2
  await run(`c4ench ${tx} ${ty} ${tz}`, 'c4:enchready')
  await sleep(600)
  since = Date.now()
  bot.useOn(tx, ty, tz, 1)
  const table = await bot.waitPacket('open_screen', (d) => /enchant/.test(d.type), 3000, since)
  if (!table) {
    t.check('open the enchanting table', false, 'no enchantment window')
  } else {
    await sleep(400)
    since = Date.now()
    bot.clickSlot(29, { mode: 1, button: 0 }) // sword from hotbar 0 into the item slot
    await sleep(300)
    bot.clickSlot(30, { mode: 1, button: 0 }) // lapis from hotbar 1 into the lapis slot
    const prep = await bot.waitText('c4:prep', { since, ms: 4000 })
    const pf = prep ? prep.text : ''
    t.check('enchantment offers / bonus / enchant item in prepare', /^c4:prep [1-3]\|0\|diamond_sword\|\d+\|\{.*enchantment.*\}/.test(pf), pf)
    const click = await bot.waitText('c4:click', { since, ms: 1000 })
    t.check('inventory action of a shift click', !!click && /move to other inventory/.test(click.text), click ? click.text : 'none')
    await sleep(300)
    since = Date.now()
    bot.send('container_button_click', Buffer.from([table.windowId & 0x7f]), Buffer.from([0]))
    const en = await bot.waitText('c4:ench ', { since, ms: 4000 })
    const ef = en ? en.text : ''
    const offer1 = pf.match(/\{.*\}/)
    const hint = /enchantment:"?([a-z_]+)/.exec(offer1 ? offer1[0] : '')
    t.check('enchant item / exp cost / enchantment hint in on enchant', /^c4:ench diamond_sword\|\d+\|[a-z_]+\|\d+$/.test(ef), ef)
    t.check('enchantment hint is the shown offer', !!hint && ef.includes(`|${hint[1]}|`), `${hint ? hint[1] : '?'} vs ${ef}`)
    bot.closeContainer()
  }
  m = await run('c4level', 'c4:level')
  t.check('enchantment level of "sharpness 5" / "sharpness IV"', m === 'c4:level 5|4', m)

  // ---- inventory action and cursor slot in the player's inventory ----
  srv.console(`clear ${NAME}`)
  await sleep(300)
  srv.console(`give ${NAME} stone 5`)
  await sleep(800)
  since = Date.now()
  bot.clickSlot(36, { windowId: 0 })
  let c = await bot.waitText('c4:click', { since, ms: 3000 })
  t.check('inventory action: pickup all, empty cursor', c && /^c4:click pickup all\|<none>\|left$/.test(c.text), c ? c.text : 'none')
  await sleep(300)
  since = Date.now()
  bot.clickSlot(38, { windowId: 0 })
  c = await bot.waitText('c4:click', { since, ms: 3000 })
  t.check('inventory action: place all, cursor slot has the stack', c && /^c4:click place all\|5 stone\|left$/.test(c.text), c ? c.text : 'none')

  // ---- consumed item ----
  await run('c4bread', 'c4:bread')
  srv.console(`effect give ${NAME} minecraft:hunger 5 50`)
  await sleep(2000)
  bot.setHotbar(0)
  since = Date.now()
  bot.useItem()
  const cs = await bot.waitText('c4:consumed', { since, ms: 4000 })
  t.check('consumed item in on consume', cs && cs.text === 'c4:consumed bread', cs ? cs.text : 'none')
  srv.console(`effect clear ${NAME}`)

  // ---- broadcast message ----
  since = Date.now()
  srv.console('say hello from c4')
  const bc = await bot.waitText('c4:bc', { since, ms: 4000 })
  t.check('broadcast message in on broadcast', bc && /c4:bc .*hello from c4/.test(bc.text), bc ? bc.text : 'none')

  // ---- the zombie in on death ----
  srv.console('time set midnight')
  await run('c4zombie', 'c4:zombie')
  await sleep(800)
  since = Date.now()
  srv.console('kill @e[type=minecraft:zombie]')
  const dead = await bot.waitText('c4:dead', { since, ms: 4000 })
  const df = dead ? dead.text : ''
  t.check('the zombie / the creature in on death, event-skeleton is none', !!dead && field(df, 0) !== '<none>' && field(df, 0) === field(df, 1) && field(df, 2) === '<none>', df)
})
