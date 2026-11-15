// Goat horns, silence, screaming, trees,
// zombifying, allay clones, script switching, reset and remove all, vector
// rotation, flight, fake blocks, interaction and text display flags, sound
// categories, shearing, taming and timed bans.
//   node tests/ingame/areas/effects-entity-commands.js
'use strict'
const fs = require('fs')
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('effects-entity-commands', { port: 25612 })
const NAME = 'Tester'

t.run(async () => {
  const scripts = ['effects-entity-commands.carve', 'effects-entity-commands-1.carve'].map((f) => path.join(__dirname, 'effects-entity-commands', f))
  const srv = await startServer({ dir: t.dir, port: t.port, ops: [NAME, 'Pleb'], scripts })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop(60000)))
  t.check('carve loads both scripts', !!await srv.waitLog(/\[Carve\] Loaded 2 script/, 5000))
  const errLine = srv.lines.find((l) => /\[Carve\].*(error|line \d+)/i.test(l))
  t.check('scripts have no load errors', !errLine, errLine || '')

  const bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  const packets = []
  bot.on('packet', (p) => packets.push({ ...p, at: Date.now() }))
  const text = async (since, part, ms = 5000) => { const m = await bot.waitText(part, { since, ms }); return m ? m.text : null }
  const run = async (cmd, tag, ms = 5000) => { const s = Date.now(); bot.command(cmd); return text(s, `eec:${tag}`, ms) }
  // A command whose effect is a console command: wait until the console ran it.
  const runM = async (cmd, tag, what = 'Modified entity data') => {
    const mark = srv.mark()
    const r = await run(cmd, tag)
    await srv.waitLog(new RegExp(what), 4000, mark)
    return r
  }
  // `data get entity ...` and its answer from the console log.
  const dataGet = async (sel, p) => {
    const mark = srv.mark()
    await sleep(500)
    srv.console(`data get entity ${sel} ${p}`)
    await sleep(600)
    return srv.lines.slice(mark).join(' | ')
  }

  await sleep(1500)
  srv.console('fill -12 100 -12 40 100 40 minecraft:stone')
  srv.console('fill 28 100 28 32 100 32 minecraft:grass_block')
  srv.console(`tp ${NAME} 0.5 101 0.5`)
  srv.console(`gamemode survival ${NAME}`)
  await sleep(2500)
  bot.moveTo(0.5, 101, 0.5)
  await sleep(300)
  if (!t.check('bot connected after the teleport', !bot.closed)) return
  t.check('control: a command answers', !!await run('eecping', 'ping'))

  // ---- goat: horns, screaming ----
  await runM('eechorn', 'horn')
  let out = await dataGet('@e[type=goat,x=3,y=101,z=3,distance=..6,limit=1]', 'HasLeftHorn')
  t.check('remove the left horn: HasLeftHorn is 0b', /0b|false/i.test(out) && !/1b/.test(out), out.slice(-160))
  out = await dataGet('@e[type=goat,x=3,y=101,z=3,distance=..6,limit=1]', 'HasRightHorn')
  t.check('remove the left horn: the right horn stays', /1b|true/i.test(out), out.slice(-160))
  await runM('eechorn2', 'horn2')
  out = await dataGet('@e[type=goat,x=3,y=101,z=3,distance=..6,limit=1]', 'HasRightHorn')
  t.check('remove goat\'s right horn: HasRightHorn is 0b', /0b|false/i.test(out), out.slice(-160))
  await runM('eechorn3', 'horn3')
  out = await dataGet('@e[type=goat,x=3,y=101,z=3,distance=..6,limit=1]', 'HasLeftHorn')
  const out2 = await dataGet('@e[type=goat,x=3,y=101,z=3,distance=..6,limit=1]', 'HasRightHorn')
  t.check('regrow both horns: both are 1b', /1b|true/i.test(out) && /1b|true/i.test(out2), `${out.slice(-80)} // ${out2.slice(-80)}`)
  await runM('eecscream', 'scream')
  out = await dataGet('@e[type=goat,x=3,y=101,z=3,distance=..6,limit=1]', 'IsScreamingGoat')
  t.check('make goat scream: IsScreamingGoat is 1b', /1b|true/i.test(out), out.slice(-160))

  // ---- silence ----
  await runM('eecsilent', 'silent')
  out = await dataGet('@e[type=cow,x=9,y=101,z=3,distance=..6,limit=1]', 'Silent')
  t.check('silence a cow: Silent is 1b', /1b|true/i.test(out), out.slice(-160))
  await runM('eecsilent2', 'silent2')
  out = await dataGet('@e[type=cow,x=9,y=101,z=3,distance=..6,limit=1]', 'Silent')
  t.check('unsilence a cow: Silent is gone or 0b', !/1b|true/i.test(out) || /0b|false|no element|not found|no data/i.test(out), out.slice(-160))

  // ---- tree ----
  await run('eectree', 'tree')
  await sleep(1500)
  const tree = await run('eectreeat', 'treeat')
  const logs = tree ? Number((tree.match(/treeat (\d+)/) || [])[1]) : 0
  t.check('grow oak at a location: a log trunk stands there', logs >= 3, tree || 'no reply')

  // ---- zombify ----
  let s = Date.now()
  const z0 = await run('eeczomb0', 'zomb0')
  t.check('control: the villager has its name', !!z0 && /Bob/.test(z0), z0 || 'no reply')
  await run('eeczomb', 'zomb')
  await sleep(1500)
  out = await dataGet('@e[type=zombie_villager,x=12,y=101,z=3,distance=..8,limit=1]', '')
  t.check('zombify a villager: a zombie villager with the name is there', /Bob/.test(out) && !/No entity was found/.test(out), out.slice(-200))
  out = await dataGet('@e[type=villager,x=12,y=101,z=3,distance=..8,limit=1]', 'Health')
  t.check('zombify a villager: the villager is gone', /No entity was found/.test(out), out.slice(-160))

  // ---- allay clone ----
  s = Date.now()
  await run('eecallay', 'allay')
  await sleep(800)
  const allays = new Set(packets.filter((p) => p.name === 'add_entity' && p.at >= s && p.data && p.data.type === 'allay').map((p) => p.data.entityId))
  t.check('make allay duplicate: two allays exist', allays.size >= 2, `${allays.size} allay add_entity packets`)

  // ---- script switching ----
  t.check('control: effects-entity-commands-1 command works', !!await run('eecb', 'b here'))
  await run('eecdis', 'dis')
  await sleep(800)
  s = Date.now()
  bot.command('eecb')
  t.check('disable script: its command is gone', !await text(s, 'eec:b here', 1500))
  await run('eecen', 'en')
  await sleep(800)
  t.check('enable script: its command is back', !!await run('eecb', 'b here'))
  await run('eecrel', 'rel')
  await sleep(800)
  t.check('reload script: its command still works', !!await run('eecb', 'b here'))

  // ---- pure logic ----
  const reset = await run('eecreset', 'reset')
  t.check('reset: number 0, text empty, boolean false; remove all 1 from a list', reset === 'eec:reset 0||false|2 and 3', reset || 'no reply')
  const reset2 = await text(0, 'eec:reset2', 1500)
  t.check('reset a list variable empties it', !!reset2 && /reset2 0/.test(reset2), reset2 || 'no reply')
  const rot = await run('eecrot', 'rot')
  t.check('rotate vectors around z, a vector and the x axis (also in lists)', rot === 'eec:rot x: 0, y: 1, z: 0|x: 1, y: 0, z: 0|x: 0, y: 0, z: 1|x: 0, y: -1, z: 0', rot || 'no reply')
  t.check('locally suppress warnings and suppress type hints: body runs', !!await run('eecwarn', 'warn body ran'))

  // ---- flight ----
  s = Date.now()
  await run('eecfly', 'fly')
  const fly = await bot.waitPacket('player_abilities', (d) => d.allowFlying, 3000, s)
  t.check('allow flight: abilities allow flying', !!fly)
  s = Date.now()
  await run('eecnofly', 'nofly')
  const nofly = await bot.waitPacket('player_abilities', (d) => !d.allowFlying, 3000, s)
  t.check('disallow flight: abilities no longer allow flying', !!nofly)

  // ---- fake block ----
  s = Date.now()
  await run('eecfake', 'fake')
  const fake = await bot.waitPacket('block_update', (d) => d.x === 5 && d.y === 101 && d.z === 5 && d.block === 'diamond_block', 3000, s)
  t.check('make player see a block as diamond block: block_update on the wire', !!fake)
  s = Date.now()
  await run('eecunfake', 'unfake')
  const unfake = await bot.waitPacket('block_update', (d) => d.x === 5 && d.y === 101 && d.z === 5 && d.block === 'air', 3000, s)
  t.check('make player see the original block: real block resent', !!unfake)

  // ---- interaction and text display ----
  const inter = await run('eecinter', 'inter')
  t.check('make interaction responsive: reads back as responsive', !!inter && /inter yes/.test(inter), inter || 'no reply')
  const inter2 = await run('eecinter2', 'inter2')
  t.check('make interaction unresponsive: reads back as not responsive', !!inter2 && /inter2 no/.test(inter2), inter2 || 'no reply')
  s = Date.now()
  await run('eectd', 'td')
  await sleep(1500)
  const tdAdd = packets.find((p) => p.name === 'add_entity' && p.at >= s && p.data && p.data.type === 'text_display')
  const tdId = tdAdd && tdAdd.data.entityId
  const flagsOf = (since) => {
    const m = {}
    for (const p of packets) if (p.name === 'set_entity_data' && p.at >= since && p.data && p.data.entityId === tdId) Object.assign(m, p.data.bytes)
    return m
  }
  const before = flagsOf(s)
  s = Date.now()
  await run('eectd2', 'td2')
  await sleep(1500)
  const after = flagsOf(s)
  // The flags byte after "apply drop shadow" + "prevent see through": shadow bit set, see-through bit clear.
  t.check('text display: drop shadow on and see-through off on the wire', Object.values(after).includes(1), `before ${JSON.stringify(before)} after ${JSON.stringify(after)}`)

  // ---- stop sound categories ----
  s = Date.now()
  await run('eecstop', 'stop')
  const stop = await bot.waitPacket('stop_sound', (d) => d.source === 'music' && /cow\.ambient/.test(d.sound || ''), 3000, s)
  t.check('stop sound in music: the packet names the sound and category', !!stop, stop ? '' : 'no stop_sound packet')
  s = Date.now()
  await run('eecstopall', 'stopall')
  const stopall = await bot.waitPacket('stop_sound', (d) => d.source === 'master' && !d.sound, 3000, s)
  t.check('stop all sounds in master: the packet has only the category', !!stopall)

  // ---- shear and tame ----
  await runM('eecsheep', 'sheep')
  out = await dataGet('@e[type=sheep,x=18,y=101,z=3,distance=..6,limit=1]', 'Sheared')
  t.check('shear a sheep: Sheared is 1b', /1b|true/i.test(out), out.slice(-160))
  await runM('eecsheep2', 'sheep2')
  out = await dataGet('@e[type=sheep,x=18,y=101,z=3,distance=..6,limit=1]', 'Sheared')
  t.check('unshear a sheep: Sheared is 0b', /0b|false/i.test(out), out.slice(-160))
  await run('eectame', 'tame')
  out = await dataGet('@e[type=wolf,x=10,y=101,z=3,distance=..30,limit=1]', 'Owner')
  t.check('tame a wolf: it has an owner', /\[I;|Owner|\d{5,}/.test(out) && !/no entity|not found|no element/i.test(out), out.slice(-160))
  await run('eecuntame', 'untame')
  out = await dataGet('@e[type=wolf,x=10,y=101,z=3,distance=..30,limit=1]', 'Owner')
  t.check('untame a wolf: the owner is gone', /Found no elements matching Owner/.test(out), out.slice(-160))

  // ---- ride, swing, equip on any entity ----
  s = Date.now()
  await run('eecride', 'ride')
  await sleep(1500)
  const adds = packets.filter((p) => p.name === 'add_entity' && p.at >= s && p.data)
  const zId = (adds.find((p) => p.data.type === 'zombie') || { data: {} }).data.entityId
  const pigId = (adds.find((p) => p.data.type === 'pig') || { data: {} }).data.entityId
  const readVar = (b, o = 0) => { let n = 0; let sh = 0; for (;;) { const x = b[o++]; n |= (x & 0x7f) << sh; if (!(x & 0x80)) return [n, o]; sh += 7 } }
  const passengers = (since) => packets.filter((p) => p.name === 'set_passengers' && p.at >= since).map((p) => {
    let [veh, o] = readVar(p.buffer); let [n, o2] = readVar(p.buffer, o); const ids = []
    for (let i = 0; i < n; i++) { const r = readVar(p.buffer, o2); ids.push(r[0]); o2 = r[1] }
    return { veh, ids }
  })
  const ridden = passengers(s).find((x) => x.veh === pigId && x.ids.includes(zId))
  t.check('make zombie ride a pig: the pig carries the zombie', !!ridden && pigId !== undefined, JSON.stringify(passengers(s)))
  s = Date.now()
  await run('eecdismount', 'dismount')
  await sleep(1200)
  const freed = passengers(s).find((x) => x.veh === pigId && !x.ids.includes(zId))
  t.check('make zombie dismount: the pig carries nobody', !!freed, JSON.stringify(passengers(s)))
  s = Date.now()
  await run('eecswing', 'swing')
  await sleep(800)
  const anim = (since, hand) => packets.find((p) => p.name === 'swing_animation' && p.at >= since && readVar(p.buffer)[0] === zId && p.buffer[readVar(p.buffer)[1]] === hand)
  t.check('make zombie swing main hand: swing animation sent', !!anim(s, 0))
  s = Date.now()
  await run('eecswing2', 'swing2')
  await sleep(800)
  t.check('make zombie swing off hand: off hand swing sent', !!anim(s, 1))
  s = Date.now()
  await run('eecequip', 'equip')
  await sleep(1500)
  const equipped = packets.filter((p) => p.name === 'set_equipment' && p.at >= s && p.data && p.data.entityId === zId)
  const slotNames = new Set(); for (const p of equipped) for (const [k, v] of Object.entries(p.data.slots)) if (v) slotNames.add(`${k}:${v.id || v.name || JSON.stringify(v).slice(0, 40)}`)
  t.check('equip zombie with a helmet and boots: both slots sent', [...slotNames].some((x) => x.startsWith('head')) && [...slotNames].some((x) => x.startsWith('feet')), [...slotNames].join(','))

  // ---- worlds and boss bar flags ----
  const created = await run('eecworld', 'world')
  t.check('create world with environment nether: the command ran', !!created, created || 'no reply')
  const wc = await run('eecworldchk', 'worldchk')
  t.check('a created world is in the world list', !!wc && /worldchk yes/.test(wc), wc || 'no reply')
  const eec = await run('eecworld2', 'world2')
  t.check('unload world: it is no longer loaded', !!eec && /world2 no/.test(eec), eec || 'no reply')
  const bar = await run('eecbar', 'bar')
  t.check('boss bar flags read back: darken sky set, fog not set', !!bar && /bar dark$/.test(bar.trim()), bar || 'no reply')

  // ---- timed ban ----
  const pleb = await joinBot('Pleb', { port: t.port })
  t.cleanup(() => pleb.quit())
  let plebKicked = null
  pleb.on('packet', (p) => { if (p.name === 'disconnect') plebKicked = p.data ? p.data.text || 'kicked' : 'kicked' })
  await sleep(1500)
  const mark = srv.mark()
  await run('eecban', 'ban')
  await sleep(1200)
  t.check('ban for a time: the player is kicked with the reason', !!plebKicked && /eec test|ban/i.test(plebKicked), plebKicked || 'not kicked')
  let entry = null
  try { entry = JSON.parse(fs.readFileSync(path.join(t.dir, 'data', 'banned-players.json'), 'utf8')).find((b) => b.name === 'Pleb') } catch (e) { entry = { err: String(e) } }
  const left = entry && entry.expires ? Date.parse(entry.expires) - Date.now() : -1
  t.check('ban for a time: the ban list holds Pleb for about 2 minutes', left > 60000 && left < 150000, JSON.stringify(entry))
  srv.console('pardon Pleb')
})
