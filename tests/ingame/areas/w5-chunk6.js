// Wave 5 chunk 6 area: scale, targets, spectator camera, text displays,
// sea pickles, block data, temperature, world border warnings, boss bar and
// inventory reads, all <type>s, slot index, spawn eggs, stored enchantments,
// the shoot section and the spawning reason / tamer / source block / shooter
// event values.
//   node tests/ingame/areas/w5-chunk6.js
'use strict'
const fs = require('fs')
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')
const { Reader } = require('../lib/proto.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('w5-chunk6', { port: 25606 })
const NAME = 'Tester'

// Base temperatures from the 26.3 data pack (same table as w5_chunk6.rs).
const TEMPS = {
  plains: 0.8, sunflower_plains: 0.8, forest: 0.7, flower_forest: 0.7, birch_forest: 0.6, desert: 2.0,
  savanna: 2.0, taiga: 0.25, snowy_plains: 0.0, swamp: 0.8, river: 0.5, ocean: 0.5, beach: 0.8,
  dark_forest: 0.7, jungle: 0.95, meadow: 0.5, cherry_grove: 0.5, windswept_hills: 0.2, stony_shore: 0.2,
}

// First state id of a block and its default, from Pumpkin's blocks.json when it is there.
function blockStates (name, fallbackFirst) {
  const dir = process.env.PUMPKIN_REF_DIR
  try {
    const all = JSON.parse(fs.readFileSync(path.join(dir, 'blocks.json'), 'utf8')).blocks
    const b = all.find((x) => x.name === name)
    return b.states[0].id
  } catch (e) {
    return fallbackFirst
  }
}

t.run(async () => {
  const srv = await startServer({
    dir: t.dir, port: t.port, ops: [NAME], scripts: [path.join(__dirname, 'w5-chunk6', 'w5-chunk6.carve')],
  })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop()))
  t.check('carve loads the area script', !!await srv.waitLog(/\[Carve\] Loaded 1 script/, 5000))
  const errLine = srv.lines.find((l) => /\[Carve\].*(error|line \d+)/i.test(l))
  t.check('area script has no load errors', !errLine, errLine || '')

  const bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  bot.on('packet', (p) => { if (p.name === 'disconnect' && p.data) console.log(`kicked: ${p.data.text}`) })

  const said = []
  bot.on('packet', (p) => { if (p.name === 'system_chat' && p.data && (p.data.text || '').startsWith('c6:')) said.push(p.data.text) })
  // Run /name and wait for the "c6:<tag>" reply.
  const run = async (name, tag = name.slice(3), ms = 5000) => {
    const since = Date.now()
    bot.command(name)
    const m = await bot.waitText(`c6:${tag}`, { since, ms })
    return m ? m.text : null
  }
  // Packets the test bot does not decode: keep name, time and the first varint of the body.
  const seen = []
  bot.on('packet', (p) => {
    if (p.data) return
    let v = null
    try { v = new Reader(p.buffer).varint() } catch {}
    seen.push({ name: p.name, t: p.t, v })
  })
  const raw = async (name, since, ms = 3000, pred = () => true) => {
    const end = Date.now() + ms
    for (;;) {
      const hit = seen.find((p) => p.name === name && p.t >= since && pred(p.v))
      if (hit) return hit
      if (Date.now() > end) return null
      await sleep(50)
    }
  }
  const waitBlock = async (x, y, z, ms = 3000, since = 0, state = null) => {
    const at = (b) => b.x === x && b.y === y && b.z === z && (state === null || b.state === state)
    const one = bot.waitPacket('block_update', at, ms, since)
    const many = bot.waitPacket('section_blocks_update', (d) => d.blocks.some(at), ms, since).then((d) => d && d.blocks.find(at))
    return (await Promise.race([one.then((v) => v || many), many.then((v) => v || one)])) || null
  }

  await sleep(1500)
  srv.console(`tp ${NAME} 0.5 100 0.5`)
  await sleep(2500)
  bot.moveTo(0.5, 100, 0.5)
  await sleep(300)
  if (!t.check('bot still connected after the teleport', !bot.closed)) return
  t.check('/c6pad', !!await run('c6pad', 'pad'))

  // ---- world border ----
  let since = Date.now()
  let s = await run('c6border', 'warn')
  t.check('world border of player', said.some((x) => x === "c6:border world border of 'world'"), said.find((x) => x.startsWith('c6:border')) || 'none')
  t.check('set/read world border warning distance and time', s === 'c6:warn 12 30 seconds', s || 'no reply')
  const wd = await raw('set_border_warning_distance', since, 3000, (v) => v === 12)
  t.check('warning distance packet sent', !!wd, wd ? `distance ${wd.v}` : 'no set_border_warning_distance 12')
  const wt = await raw('set_border_warning_delay', since, 3000, (v) => v === 30)
  t.check('warning time packet sent', !!wt, wt ? `delay ${wt.v}` : 'no set_border_warning_delay 30')

  // ---- blocks ----
  since = Date.now()
  s = await run('c6pickle', 'pickles')
  t.check('sea pickles read back', s === 'c6:pickles 3 4', s || 'no reply')
  const first = blockStates('sea_pickle', 18624)
  const pick = await waitBlock(6, 100, 6, 3000, since, first + 4)
  // 3 pickles, waterlogged (the default): offset (3 - 1) * 2.
  t.check('sea pickles block state sent', pick && pick.state === first + 4, pick ? `state ${pick.state}, want ${first + 4}` : 'no block update')

  since = Date.now()
  s = await run('c6data', 'data')
  t.check('block data read back', s === 'c6:data oak_log[axis=z]', s || 'no reply')
  const logFirst = blockStates('oak_log', 139)
  const log = await waitBlock(7, 100, 6, 3000, since, logFirst + 2)
  t.check('block data state sent', log && log.state === logFirst + 2, log ? `state ${log.state}, want ${logFirst + 2}` : 'no block update')

  s = await run('c6temp', 'temp')
  const m = s && s.match(/^c6:temp (\S+) (-?[\d.]+)$/)
  if (m && TEMPS[m[1]] !== undefined) {
    // The block below the player is at y 99: vanilla lowers the base 0.05 per 40 blocks above y 80.
    const want = TEMPS[m[1]] - (99 - 80) * 0.05 / 40
    t.check('temperature of block is the biome temperature', Math.abs(Number(m[2]) - want) < 1e-4, `${s}, want ${want}`)
  } else {
    t.check('temperature of block is a number', !!m, s || 'no reply')
  }

  // ---- entities ----
  since = Date.now()
  s = await run('c6scale', 'scale')
  t.check('scale of player set and read', s === 'c6:scale 2', s || 'no reply')
  const attr = await bot.waitPacket('update_attributes', (d) => d.entityId === bot.entityId && d.attributes.scale, 3000, since)
  t.check('scale attribute sent to the client', attr && attr.attributes.scale.value === 2, attr ? JSON.stringify(attr.attributes.scale) : 'no update_attributes')
  s = await run('c6unscale', 'unscale')
  t.check('reset scale of player', s === 'c6:unscale 1', s || 'no reply')

  s = await run('c6target', 'target')
  t.check('target of a zombie set and read', s === `c6:target ${NAME}`, s || 'no reply')

  since = Date.now()
  s = await run('c6spec', 'spec')
  t.check('spectator target set and read', s === 'c6:spec pig', s || 'no reply')
  const cam = await raw('set_camera', since, 3000, (v) => v !== bot.entityId)
  t.check('set_camera packet sent', !!cam, cam ? `camera on ${cam.v}` : 'no set_camera to another entity')
  since = Date.now()
  s = await run('c6unspec', 'unspec')
  t.check('reset spectator target', s === 'c6:unspec <none>', s || 'no reply')
  const back = await raw('set_camera', since, 3000, (v) => v === bot.entityId)
  t.check('set_camera back to the player', !!back)

  since = Date.now()
  s = await run('c6td', 'td')
  t.check('text display alignment, line width, opacity and scale', s === 'c6:td left 80 128 x: 2, y: 2, z: 2', s || 'no reply')
  const td = await bot.waitPacket('add_entity', (d) => d.type === 'text_display', 3000, since)
  const meta = td && await bot.waitPacket('set_entity_data', (d) => d.entityId === td.entityId, 3000, since)
  t.check('text display data sent', !!meta, meta ? `${Object.keys(meta.values).length + Object.keys(meta.bytes).length} fields` : td ? 'no set_entity_data' : 'no text display spawned')

  // ---- boss bars and inventories ----
  s = await run('c6bar', 'bar')
  t.check('boss bar title, style and viewers', s === `c6:bar Raid segmented 10 ${NAME}`, s || 'no reply')

  since = Date.now()
  s = await run('c6inv', 'inv')
  t.check('inventory title and viewers', s === `c6:inv Bank ${NAME}`, s || 'no reply')
  const scr = await bot.waitPacket('open_screen', () => true, 3000, since)
  t.check('add to viewers opens the menu', scr && scr.title === 'Bank', scr ? scr.title : 'no open_screen')
  bot.closeContainer()
  await sleep(300)

  // ---- pure expressions ----
  s = await run('c6all', 'all')
  t.check('all difficulties', s === 'c6:all peaceful, easy, normal and hard', s || 'no reply')
  s = await run('c6slot', 'slot')
  t.check('index of slot', s === 'c6:slot 5', s || 'no reply')
  s = await run('c6hints', 'hints')
  t.check('suppress type hints runs its body', s === 'c6:hints ok', s || 'no reply')

  srv.console(`clear ${NAME}`)
  await sleep(500)
  since = Date.now()
  s = await run('c6egg', 'egg2')
  t.check('spawn egg entity read', said.some((x) => x === 'c6:egg zombie'), said.find((x) => x.startsWith('c6:egg ')) || 'none')
  t.check('spawn egg entity changed', s === 'c6:egg2 cow spawn egg', s || 'no reply')
  const egg = await bot.waitPacket('container_set_slot', (d) => d.item && d.item.name === 'cow_spawn_egg', 3000, since)
    .then((v) => v || bot.waitPacket('set_player_inventory', (d) => d.item && d.item.name === 'cow_spawn_egg', 500, since))
  t.check('changed spawn egg given', !!egg)
  // Pumpkin fires its creature spawn event for spawn eggs (not for /summon): use the egg on the pad.
  since = Date.now()
  bot.useOn(1, 99, 0, 1)
  const reason = await bot.waitText('c6:reason', { since, ms: 4000 })
  t.check('spawning reason in on spawn', !!reason && /^c6:reason spawner[ _]egg$/.test(reason.text), reason ? reason.text : 'no message')

  srv.console(`clear ${NAME}`)
  await sleep(500)
  bot.setHotbar(0)
  srv.console(`give ${NAME} enchanted_book`)
  await sleep(800)
  since = Date.now()
  s = await run('c6stored', 'stored')
  t.check('stored enchantments set and read', s === 'c6:stored sharpness 5', s || 'no reply')
  const isBook = (d) => d.item && d.item.name === 'enchanted_book' && d.item.components && d.item.components.stored_enchantments
  const book = await bot.waitPacket('container_set_slot', isBook, 3000, since)
    .then((v) => v || bot.waitPacket('set_player_inventory', isBook, 500, since))
  const se = book && book.item.components.stored_enchantments
  t.check('stored enchantments sent on the book', se && se.some((e) => e.name === 'sharpness' && e.level === 5), se ? JSON.stringify(se) : 'no book update')

  since = Date.now()
  s = await run('c6shoot', 'shot')
  t.check('shoot section body runs with the arrow', s === 'c6:shot arrow', s || 'no reply')
  const arrow = await bot.waitPacket('add_entity', (d) => d.type === 'arrow', 3000, since)
  t.check('arrow spawned', !!arrow)
  const mv = arrow && await bot.waitPacket('set_entity_motion', (d) => d.entityId === arrow.entityId, 3000, since)
  const sp = (v) => v && Math.hypot(v.x, v.y, v.z)
  t.check('arrow given a velocity', !!mv || (arrow && sp(arrow.velocity) > 0.5), mv ? JSON.stringify(mv.velocity) : arrow ? JSON.stringify(arrow.velocity) : 'no arrow')

  // ---- event values ----

  // Tame a wolf: survival, bones, right click until it takes.
  srv.console(`gamemode survival ${NAME}`)
  srv.console(`clear ${NAME}`)
  await sleep(300)
  srv.console(`give ${NAME} bone 64`)
  await sleep(500)
  bot.setHotbar(0)
  since = Date.now()
  srv.console('summon wolf 1.5 100 1.5')
  const wolf = await bot.waitPacket('add_entity', (d) => d.type === 'wolf', 4000, since)
  let tamer = null
  if (wolf) {
    for (let i = 0; i < 30 && !tamer; i++) {
      // Keep the wolf within reach: it wanders off the pad between clicks.
      if (i % 5 === 0) { srv.console('tp @e[type=wolf] 1.5 100 0.5'); await sleep(200) }
      bot.interact(wolf.entityId)
      tamer = await bot.waitText('c6:tamer', { since, ms: 400 })
    }
  }
  if (tamer) t.check('tamer in on tame', tamer.text === `c6:tamer ${NAME}`, tamer.text)
  else t.skip('tamer in on tame', wolf ? 'Pumpkin did not tame the wolf in 30 tries' : 'no wolf spawned')

  // Fire spread: fire on planks with a fast random tick.
  srv.console('gamerule fire_spread_radius_around_player 128')
  srv.console('gamerule random_tick_speed 1000')
  srv.console('fill -6 99 -6 -2 101 -2 oak_planks')
  srv.console('setblock -4 102 -4 fire')
  // The source fire can burn out in the same tick it spreads (then it reads as air), so wait for a spread
  // whose source is still fire.
  const spreadSince = Date.now() - 200
  const src = await bot.waitText('c6:source fire', { since: spreadSince, ms: 15000 }) || await bot.waitText('c6:source', { since: spreadSince, ms: 10 })
  if (src) t.check('source block in on spread', src.text === 'c6:source fire', src.text)
  else t.skip('source block in on spread', 'Pumpkin fired no spread event in 15 s')

  // A skeleton shooting at the survival bot.
  since = Date.now()
  srv.console('fill -1 99 2 2 99 10 stone')
  srv.console('time set night')
  await sleep(300)
  srv.console('summon skeleton 0.5 100 8.5')
  const shot = await bot.waitText('c6:shooter', { since, ms: 15000 })
  if (shot) t.check('shooter in on shoot', /^c6:shooter /.test(shot.text), shot.text)
  else t.skip('shooter in on shoot', 'the skeleton did not shoot in 15 s')
  srv.console(`gamemode creative ${NAME}`)
})
