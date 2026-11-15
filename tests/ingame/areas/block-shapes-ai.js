// Wave 5 chunk 4: block data, block spheres and lines, written books, boss bar
// by id, brewing stand slots, world border settings, potion effect values,
// item damage, ender chests, enderman blocks, AI, attributes, enchanting
// values, inventory actions, cursor slot and event values.
//   node tests/ingame/areas/block-shapes-ai.js
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('block-shapes-ai', { port: 25604 })
const NAME = 'Tester'

t.run(async () => {
  const srv = await startServer({ dir: t.dir, port: t.port, ops: [NAME], scripts: [path.join(__dirname, 'block-shapes-ai', 'block-shapes-ai.carve')] })
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
  let m = await run(`bsabd ${b.x + 2} ${b.y + 3} ${b.z}`, 'bsa:bd')
  await sleep(400) // block changes go out at the end of the tick, after the chat line
  t.check('block data: set and read', /oak_stairs\[.*facing=west/.test(m) && /half=top/.test(m), m)
  const at = (d) => d.x === b.x + 2 && d.y === b.y + 3 && d.z === b.z
  const bu = packets('block_update', since, at).map((p) => p.data)
    .concat(...packets('section_blocks_update', since).map((p) => p.data.blocks.filter(at)))
  t.check('block data: block update for the stairs', bu.some((d) => /oak_stairs/.test(d.block || '')), bu.map((d) => d.block).join(','))

  // ---- block sphere ----
  since = Date.now()
  m = await run(`bsasphere ${b.x - 4} ${b.y + 4} ${b.z}`, 'bsa:sphere')
  await sleep(400)
  t.check('blocks in radius 1: 7 blocks, set to glass', field(m, 0) === '7' && /glass/.test(field(m, 1)) && !/glass/.test(field(m, 2)), m)
  const glass = packets('block_update', since, (d) => /glass/.test(d.block || '')).length + packets('section_blocks_update', since).length
  t.check('blocks in radius: the client sees the glass', glass > 0, `${glass} updates`)

  // ---- blocks between / from to / above ----
  m = await run(`bsablocks ${b.x} ${b.y + 6} ${b.z + 4}`, 'bsa:blocks')
  t.check('blocks between a and b: 8', field(m, 0) === '8', m)
  t.check('blocks from a to b: a line of 4', field(m, 1) === '4' && /stone/.test(field(m, 3)), m)
  t.check('blocks above x: 100', field(m, 2) === '100', m)

  // ---- books ----
  since = Date.now()
  m = await run('bsabook', 'bsa:book')
  t.check('book title / author / pages of player\'s tool', m === 'bsa:book Tales|Ann|2|page two', m)
  const bs = packets('container_set_slot', since, (d) => d.item && d.item.name === 'written_book').concat(packets('set_player_inventory', since, (d) => d.item && d.item.name === 'written_book'))
  t.check('book: the written book reaches the client', bs.length > 0, `${bs.length} slot packets`)

  // ---- boss bar from key ----
  since = Date.now()
  m = await run('bsabar', 'bsa:bar')
  t.check('boss bar with id: the id, none for an unknown id', m === 'bsa:bar bsabar|<none>', m)
  const bar = await bot.waitPacket('boss_event', () => true, 2000, since)
  t.check('show bossbar {_b}: the client gets the bar', !!bar, bar ? JSON.stringify(bar).slice(0, 80) : 'none')

  // ---- brewing stand slots ----
  b = base()
  m = await run(`bsabrew ${b.x - 2} ${b.y} ${b.z - 3}`, 'bsa:brew')
  t.check('brewing stand slots: set and read bottle and ingredient', /^bsa:brew potion\|nether wart\|<none>$/.test(m), m)

  // ---- world border ----
  since = Date.now()
  m = await run('bsaborder', 'bsa:border')
  t.check('world border center / damage amount / damage buffer', m === 'bsa:border 10|20|2|7', m)
  const center = packets('set_border_center', since).length + packets('initialize_border', since, (d) => d.x === 10).length
  t.check('world border center: the client gets the new center', center > 0, `${center} packets`)

  // ---- potion effect values ----
  since = Date.now()
  await run('bsaeffect', 'bsa:effect')
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
  bot.command('bsadmg')
  const d1 = await bot.waitText('bsa:dmg1', { since, ms: 6000 })
  const d2 = await bot.waitText('bsa:dmg2', { since, ms: 6000 })
  await sleep(400)
  const dslots = packets('container_set_slot', since, (d) => d.item && d.item.name === 'diamond_sword').concat(packets('set_player_inventory', since, (d) => d.item && d.item.name === 'diamond_sword'))
  const dmgs = dslots.map((p) => p.data.item.components && p.data.item.components.damage)
  t.check('give diamond sword with damage 10', dmgs.includes(10) && d1 && d1.text === 'bsa:dmg1 10|1551', `${d1 ? d1.text : 'none'} wire ${dmgs.join(',')}`)
  t.check('set damage of player\'s tool / durability', dmgs.includes(5) && d2 && d2.text === 'bsa:dmg2 5|1556', `${d2 ? d2.text : 'none'} wire ${dmgs.join(',')}`)

  // ---- ender chest ----
  since = Date.now()
  m = await run('bsaender', 'bsa:ender')
  t.check('slot of ender chest: set and read', m === 'bsa:ender diamond|3 emerald|2' || m === 'bsa:ender diamond|3 emeralds|2', m)
  const open = await bot.waitPacket('open_screen', () => true, 3000, since)
  t.check('open player\'s ender chest', open && /generic_9x3$/.test(open.type), open ? `${open.type} "${open.title}"` : 'none')
  await sleep(400)
  const content = open && packets('container_set_content', since, (d) => d.windowId === open.windowId)[0]
  t.check('ender chest window holds the diamond', content && content.data.items[0] && content.data.items[0].name === 'diamond', content ? content.data.items.slice(0, 3).map((i) => i && i.name).join(',') : 'none')
  if (open) bot.closeContainer()
  m = await run('bsaenderclear', 'bsa:enderclear')
  t.check('clear ender chest of player', m === 'bsa:enderclear 0', m)

  // ---- enderman block, AI ----
  m = await run('bsamobs', 'bsa:carry')
  t.check('carried block of an enderman: set and read', /grass_block/.test(m), m)
  const ai = await bot.waitText('bsa:ai ', { since: Date.now() - 3000, ms: 3000 })
  const ai2 = await bot.waitText('bsa:ai2', { since: Date.now() - 3000, ms: 3000 })
  t.check('ai of a mob: set to false and read; a player has none', ai && ai.text === 'bsa:ai false|<none>', ai ? ai.text : 'none')
  t.check('ai of a mob: set back to true', ai2 && ai2.text === 'bsa:ai2 true', ai2 ? ai2.text : 'none')

  // ---- attributes ----
  since = Date.now()
  m = await run('bsaattr', 'bsa:attr')
  t.check('movement speed attribute of player: set and read', /^bsa:attr 0\.3\|20$/.test(m), m)
  const attr = await bot.waitPacket('update_attributes', (d) => d.entityId === bot.entityId && d.attributes.movement_speed && Math.abs(d.attributes.movement_speed.value - 0.3) < 1e-6, 3000, since)
  t.check('attribute change reaches the client', !!attr, attr ? JSON.stringify(attr.attributes.movement_speed) : 'no update_attributes with 0.3')
  m = await run('bsaattrreset', 'bsa:attrreset')
  t.check('reset attribute', /^bsa:attrreset 0\.1/.test(m), m)

  // ---- enchanting ----
  b = base()
  const tx = b.x + 1; const ty = b.y; const tz = b.z + 2
  await run(`bsaench ${tx} ${ty} ${tz}`, 'bsa:enchready')
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
    const prep = await bot.waitText('bsa:prep', { since, ms: 4000 })
    const pf = prep ? prep.text : ''
    t.check('enchantment offers / bonus / enchant item in prepare', /^bsa:prep [1-3]\|0\|diamond_sword\|\d+\|\{.*enchantment.*\}/.test(pf), pf)
    const click = await bot.waitText('bsa:click', { since, ms: 1000 })
    t.check('inventory action of a shift click', !!click && /move to other inventory/.test(click.text), click ? click.text : 'none')
    await sleep(300)
    since = Date.now()
    bot.send('container_button_click', Buffer.from([table.windowId & 0x7f]), Buffer.from([0]))
    const en = await bot.waitText('bsa:ench ', { since, ms: 4000 })
    const ef = en ? en.text : ''
    const offer1 = pf.match(/\{.*\}/)
    const hint = /enchantment:"?([a-z_]+)/.exec(offer1 ? offer1[0] : '')
    t.check('enchant item / exp cost / enchantment hint in on enchant', /^bsa:ench diamond_sword\|\d+\|[a-z_]+\|\d+$/.test(ef), ef)
    t.check('enchantment hint is the shown offer', !!hint && ef.includes(`|${hint[1]}|`), `${hint ? hint[1] : '?'} vs ${ef}`)
    bot.closeContainer()
  }
  m = await run('bsalevel', 'bsa:level')
  t.check('enchantment level of "sharpness 5" / "sharpness IV"', m === 'bsa:level 5|4', m)

  // ---- inventory action and cursor slot in the player's inventory ----
  srv.console(`clear ${NAME}`)
  await sleep(300)
  srv.console(`give ${NAME} stone 5`)
  await sleep(800)
  since = Date.now()
  bot.clickSlot(36, { windowId: 0 })
  let c = await bot.waitText('bsa:click', { since, ms: 3000 })
  t.check('inventory action: pickup all, empty cursor', c && /^bsa:click pickup all\|<none>\|left$/.test(c.text), c ? c.text : 'none')
  await sleep(300)
  since = Date.now()
  bot.clickSlot(38, { windowId: 0 })
  c = await bot.waitText('bsa:click', { since, ms: 3000 })
  t.check('inventory action: place all, cursor slot has the stack', c && /^bsa:click place all\|5 stone\|left$/.test(c.text), c ? c.text : 'none')

  // ---- consumed item ----
  await run('bsabread', 'bsa:bread')
  srv.console(`effect give ${NAME} minecraft:hunger 5 50`)
  await sleep(2000)
  bot.setHotbar(0)
  since = Date.now()
  bot.useItem()
  const cs = await bot.waitText('bsa:consumed', { since, ms: 4000 })
  t.check('consumed item in on consume', cs && cs.text === 'bsa:consumed bread', cs ? cs.text : 'none')
  srv.console(`effect clear ${NAME}`)

  // ---- broadcast message ----
  since = Date.now()
  srv.console('say hello from bsa')
  const bc = await bot.waitText('bsa:bc', { since, ms: 4000 })
  t.check('broadcast message in on broadcast', bc && /bsa:bc .*hello from bsa/.test(bc.text), bc ? bc.text : 'none')

  // ---- the zombie in on death ----
  srv.console('time set midnight')
  await run('bsazombie', 'bsa:zombie')
  await sleep(800)
  since = Date.now()
  srv.console('kill @e[type=minecraft:zombie]')
  const dead = await bot.waitText('bsa:dead', { since, ms: 4000 })
  const df = dead ? dead.text : ''
  t.check('the zombie / the creature in on death, event-skeleton is none', !!dead && field(df, 0) !== '<none>' && field(df, 0) === field(df, 1) && field(df, 2) === '<none>', df)
})
