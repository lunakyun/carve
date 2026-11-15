// World area: blocks, targeted block, weather, time, game rules, difficulty,
// world border, lightning, explosions, particles, sounds, potion effects,
// fire, display entities, bounds, ray trace, chunks and tags.
//   node tests/ingame/areas/world.js
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('world', { port: 25574 })
const NAME = 'Tester'

t.run(async () => {
  const srv = await startServer({
    dir: t.dir, port: t.port, ops: [NAME], scripts: [path.join(__dirname, 'world', 'world.carve')],
  })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop()))
  t.check('carve loads the world script', !!await srv.waitLog(/\[Carve\] Loaded 1 script/, 5000))
  const errLine = srv.lines.find((l) => /\[Carve\].*(error|line \d+)/i.test(l))
  t.check('world script has no load errors', !errLine, errLine || '')

  const bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  const me = bot.entityId
  bot.on('packet', (p) => { if (p.name === 'disconnect' && p.data) console.log(`kicked: ${p.data.text}`) })
  bot.on('end', () => console.log('bot connection ended'))

  // Every "w:" message, so a failed check can show what came back.
  const said = []
  bot.on('packet', (p) => { if (p.name === 'system_chat' && p.data && p.data.text.startsWith('w:')) said.push(p.data.text) })
  // Run /name and wait for the "w:<tag>" reply. Returns the reply text (or null).
  const run = async (name, tag = name.slice(1), ms = 5000) => {
    const since = Date.now()
    bot.command(name)
    const m = await bot.waitText(`w:${tag}`, { since, ms })
    return m ? m.text : null
  }
  const text = (since, prefix) => { const m = said.slice(since).find((s) => s.startsWith(prefix)); return m || null }
  // A block change at x y z: a single block_update or one entry of a section_blocks_update
  // (Pumpkin batches changes made in the same tick).
  const waitBlock = async (x, y, z, pred = () => true, ms = 3000, since = 0) => {
    const at = (b) => b.x === x && b.y === y && b.z === z && pred(b)
    const one = bot.waitPacket('block_update', at, ms, since)
    const many = bot.waitPacket('section_blocks_update', (d) => d.blocks.some(at), ms, since).then((d) => d && d.blocks.find(at))
    return (await Promise.race([one.then((v) => v || many), many.then((v) => v || one)])) || null
  }

  // Stand on a stone strip at y 100, looking straight down.
  // Let the spawn teleports settle first: confirming an old one gets the bot kicked.
  await sleep(1500)
  srv.console(`tp ${NAME} 0.5 101 0.5`)
  await sleep(2500)
  bot.moveTo(0.5, 101, 0.5)
  await sleep(300)
  if (!t.check('bot still connected after the teleport', !bot.closed)) return
  let since = Date.now()
  t.check('/wpad sets blocks', !!await run('wpad'))
  const pad = await waitBlock(0, 100, 0, (b) => b.block === 'stone', 3000, since)
  t.check('set block sends block_update', !!pad)

  // ---- blocks ----
  since = Date.now()
  await run('wset')
  const dia = await waitBlock(4, 100, 4, undefined, 3000, since)
  t.check('set block at location to diamond block', dia && dia.block === 'diamond_block', dia ? dia.block : 'none')
  const gold = await waitBlock(4, 101, 4, undefined, 3000, since)
  t.check('set block (second block)', gold && gold.block === 'gold_block', gold ? gold.block : 'none')
  const plank = await waitBlock(4, 102, 4, undefined, 3000, since)
  t.check('set a stored block variable to oak planks', plank && plank.block === 'oak_planks', plank ? plank.block : 'none')

  let n = said.length
  await run('wread', 'readdone')
  const has = (prefix, re) => { const s = text(n, prefix); return t.check(`${prefix.slice(2)}: ${re}`, s && re.test(s), s || 'no reply') }
  has('w:type', /^w:type diamond[ _]block$/)
  has('w:above', /^w:above gold[ _]block$/)
  has('w:below', /^w:below stone$/)
  has('w:stored', /^w:stored .*4.*100.*4.* diamond[ _]block$/)
  has('w:north', /^w:north gold[ _]block$/)
  has('w:loc', /x: 1(\.0)?, y: 2(\.0)?, z: 3(\.0)?/)
  t.check('condition: block is diamond block', !!text(n, 'w:is yes'))
  t.check('condition: block is not stone', !!text(n, 'w:isnot yes'))
  t.check('condition: block is stone or gold block', !!text(n, 'w:isor yes'))
  t.check('condition: wrong block type is false', !text(n, 'w:is wrong'))
  t.check('condition: is within radius', !!text(n, 'w:radius yes'))

  since = Date.now()
  const broken = await run('wbreak', 'broken')
  const air1 = await waitBlock(4, 101, 4, (b) => b.block === 'air', 3000, since)
  t.check('break block sends air', !!air1)
  const air2 = await waitBlock(4, 102, 4, (b) => b.block === 'air', 3000, since)
  t.check('destroy block sends air', !!air2)
  t.check('type of a broken block is air', /air$/.test(broken || ''), broken || 'no reply')

  since = Date.now()
  const floor = await run('wfloor', 'floor')
  const em = await waitBlock(0, 100, 0, undefined, 3000, since)
  t.check('set block below player', em && em.block === 'emerald_block', em ? em.block : 'none')
  t.check('type of block below player after set', /emerald[ _]block$/.test(floor || ''), floor || 'no reply')

  // Look straight down at the emerald block.
  bot.look(0, 90)
  await sleep(300)
  n = said.length
  await run('wtarget', 'ray')
  const tg = text(n, 'w:target ')
  t.check('targeted block of player', tg && /0.*100.*0/.test(tg) && /emerald[ _]block$/.test(tg), tg || 'no reply')
  const tg2 = text(n, 'w:target2')
  t.check("player's targeted block", tg2 && /0.*100.*0/.test(tg2), tg2 || 'no reply')
  const ray = text(n, 'w:ray')
  t.check('ray trace from player: hit block and face', ray && /0.*100.*0/.test(ray) && / up /.test(ray), ray || 'no reply')

  // ---- weather ----
  since = Date.now()
  const rain = await run('wrain', 'weather')
  const rl = await bot.waitPacket('game_event', (d) => d.event === 'rain_level_change' && d.value > 0, 3000, since)
  const rs = await bot.waitPacket('game_event', (d) => d.event === 'start_raining' || d.event === 'stop_raining', 3000, since)
  t.check('set weather to rain: game_event rain start', !!rs || !!rl, rs ? rs.event : 'none')
  t.check('weather expression after rain', /rain/.test(rain || ''), rain || 'no reply')
  t.check('condition: it is raining', !!await bot.waitText('w:raining', { since, ms: 1000 }))
  const wev = await bot.waitText('w:ev weather', { since, ms: 2000 })
  t.check('on weather change fires after set weather', wev && /rain/.test(wev.text), wev ? wev.text : 'none')

  since = Date.now()
  const thunder = await run('wthunder', 'thundering', 3000)
  const tl = await bot.waitPacket('game_event', (d) => d.event === 'thunder_level_change' && d.value > 0, 3000, since)
  t.check('set weather to thunder: thunder_level_change', !!tl, tl ? `value ${tl.value}` : 'none')
  t.check('condition: it is thundering', /thunder/.test(thunder || ''), thunder || 'no reply')

  since = Date.now()
  const sunny = await run('wclear', 'sunny', 3000)
  const stop = await bot.waitPacket('game_event', (d) => d.event === 'stop_raining' || d.event === 'start_raining' || (d.event === 'rain_level_change' && d.value === 0), 3000, since)
  t.check('set weather to clear: game_event', !!stop, stop ? stop.event : 'none')
  t.check('condition: it is sunny', /clear/.test(sunny || ''), sunny || 'no reply')
  t.check('condition: weather is clear', !!await bot.waitText('w:clearis', { since, ms: 1000 }))
  const wev2 = await bot.waitText('w:ev weather clear', { since, ms: 2000 })
  t.check('on weather change to clear', !!wev2, wev2 ? wev2.text : 'none')

  since = Date.now()
  await run('wrainfor')
  const rf = await bot.waitPacket('game_event', (d) => d.event === 'rain_level_change' && d.value > 0, 5000, since)
  t.check('set weather to rain for 5 minutes: rain starts', !!rf)
  const rfEv = await bot.waitText('w:ev weather rain', { since, ms: 3000 })
  t.check('on weather change after a weather with a length', !!rfEv)
  const rfCmd = await srv.waitLog(/weather rain 6000|weather rain 6000t/, 1000)
  if (!rfCmd) console.log('note: no weather command line in the log')
  await run('wsun')
  await sleep(300)

  // ---- time ----
  since = Date.now()
  const tm = await run('wtime', 'time')
  const st = await bot.waitPacket('set_time', (d) => d.dayTime != null && Math.abs((d.dayTime % 24000) - 6000) < 40, 3000, since)
  t.check('set time to 6000: set_time packet', !!st, st ? `dayTime ${st.dayTime}` : 'none')
  t.check('time expression', /^w:time 600\d/.test(tm || ''), tm || 'no reply')
  t.check('condition: it is day', !!await bot.waitText('w:day', { since, ms: 1000 }))
  since = Date.now()
  const night = await run('wnight', 'night')
  const sn = await bot.waitPacket('set_time', (d) => d.dayTime != null && Math.abs((d.dayTime % 24000) - 13000) < 40, 3000, since)
  t.check('set the time to night: set_time packet', !!sn, sn ? `dayTime ${sn.dayTime}` : 'none')
  t.check('time after night', /^w:night 1300\d/.test(night || ''), night || 'no reply')
  t.check('condition: it is night', !!await bot.waitText('w:isnight', { since, ms: 1000 }))
  const clock = await run('wclock', 'clock')
  t.check('set time to 6:30 pm, add 500', /^w:clock 1300\d/.test(clock || ''), clock || 'no reply')

  // ---- game rules and difficulty ----
  n = said.length
  await run('wrule', 'cycle')
  t.check('gamerule keepInventory set and read', /^w:rule true$/.test(text(n, 'w:rule ') || ''), text(n, 'w:rule ') || 'no reply')
  t.check('gamerule randomTickSpeed set and read', /^w:rts 10$/.test(text(n, 'w:rts') || ''), text(n, 'w:rts') || 'no reply')
  t.check('condition: gamerule is true', !!text(n, 'w:ruleis'))
  t.check('old gamerule name doDaylightCycle maps', /^w:cycle false$/.test(text(n, 'w:cycle') || ''), text(n, 'w:cycle') || 'no reply')
  const mark = srv.mark()
  srv.console('gamerule keep_inventory')
  const ruleLog = await srv.waitLog(/keep_inventory|keepInventory/i, 3000, mark)
  t.check('server agrees keepInventory is true', ruleLog && /true/.test(ruleLog), ruleLog || 'no log line')

  since = Date.now()
  n = said.length
  bot.command('wdiff')
  const cd = await bot.waitPacket('change_difficulty', (d) => d.difficulty === 'hard', 3000, since)
  t.check('set difficulty to hard: change_difficulty', !!cd, cd ? cd.difficulty : 'none')
  const diff = await bot.waitText('w:diff', { since, ms: 3000 })
  t.check('difficulty expression reads hard', diff && /hard/.test(diff.text), diff ? diff.text : 'no reply')
  t.check('condition: difficulty is hard', !!await bot.waitText('w:diffis', { since, ms: 1000 }))

  // ---- world border ----
  since = Date.now()
  const border = await run('wborder', 'border')
  const bs = await bot.waitPacket('set_border_size', (d) => d.size === 500, 3000, since)
  t.check('set world border size to 500: set_border_size', !!bs)
  t.check('world border size expression', /^w:border 500(\.0)?$/.test(border || ''), border || 'no reply')
  since = Date.now()
  await run('wborder2')
  const lerp = await bot.waitPacket('set_border_lerp_size', (d) => d.size === 1000, 3000, since)
  t.check('world border size over 30 seconds: set_border_lerp_size', lerp && Math.abs(lerp.speed - 30000) < 100, lerp ? `${lerp.oldSize} -> ${lerp.size} in ${lerp.speed} ms` : 'none')

  // ---- lightning and explosion ----
  since = Date.now()
  await run('wbolt')
  const bolt = await bot.waitPacket('add_entity', (d) => d.type === 'lightning_bolt', 3000, since)
  t.check('strike lightning: add_entity lightning_bolt', bolt && Math.abs(bolt.x - 10) < 1.1 && Math.abs(bolt.z - 10) < 1.1, bolt ? `${bolt.x} ${bolt.y} ${bolt.z}` : 'none')
  t.check('on lightning strike fires after strike lightning', !!await bot.waitText('w:ev lightning', { since, ms: 2000 }))
  since = Date.now()
  await run('wflash')
  const flash = await bot.waitPacket('add_entity', (d) => d.type === 'lightning_bolt' && d.x < 0, 3000, since)
  t.check('strike lightning effect: add_entity lightning_bolt', flash && Math.abs(flash.x + 10) < 1.1, flash ? `${flash.x} ${flash.y} ${flash.z}` : 'none')

  since = Date.now()
  await run('wboom')
  const ex = await bot.waitPacket('explode', () => true, 3000, since)
  t.check('create explosion: explode packet', ex && Math.abs(ex.x - 20) < 0.6 && Math.abs(ex.y - 110) < 0.6, ex ? `${ex.x} ${ex.y} ${ex.z} r ${ex.radius}` : 'none')
  t.check('explosion radius matches force 2', ex && Math.abs(ex.radius - 2) < 0.01, ex ? `radius ${ex.radius}` : 'none')
  const boomEv = await bot.waitText('w:ev explode', { since, ms: 2000 })
  t.check('on explode fires after create explosion (yield 1/force)', boomEv && /0\.5$/.test(boomEv.text), boomEv ? boomEv.text : 'none')

  // A real TNT explosion, started outside any script.
  since = Date.now()
  srv.console('summon minecraft:tnt 20 110 -20')
  const exEv = await bot.waitText('w:ev explode', { since, ms: 8000 })
  t.check('on explode fires (tnt)', !!exEv, exEv ? exEv.text : 'none')

  // ---- particles ----
  since = Date.now()
  await run('wpart')
  const fl = await bot.waitPacket('level_particles', (d) => d.particle === 'flame', 3000, since)
  t.check('flame particles: type and count', fl && fl.count === 5, fl ? `count ${fl.count}` : 'none')
  t.check('flame particles: offset 1, 0.5, 1 and extra 0.05', fl && fl.offset && Math.abs(fl.offset.x - 1) < 1e-3 && Math.abs(fl.offset.y - 0.5) < 1e-3 && Math.abs(fl.offset.z - 1) < 1e-3 && Math.abs(fl.speed - 0.05) < 1e-3,
    fl ? JSON.stringify({ o: fl.offset, s: fl.speed }) : 'none')
  t.check('flame particles: position', fl && Math.abs(fl.x - 2) < 1e-6 && Math.abs(fl.y - 102) < 1e-6, fl ? `${fl.x} ${fl.y} ${fl.z}` : 'none')
  const hv = await bot.waitPacket('level_particles', (d) => d.particle === 'happy_villager', 3000, since)
  t.check('happy villager particles: spread and speed', hv && hv.count === 20 && Math.abs(hv.offset.x - 0.5) < 1e-3 && Math.abs(hv.speed - 0.1) < 1e-3, hv ? JSON.stringify({ c: hv.count, o: hv.offset, s: hv.speed }) : 'none')
  const cr = await bot.waitPacket('level_particles', (d) => d.particle === 'crit', 3000, since)
  t.check('crit particle at player, count 1', cr && cr.count === 1 && Math.abs(cr.x - 0.5) < 0.01, cr ? `${cr.count} at ${cr.x} ${cr.y} ${cr.z}` : 'none')
  since = Date.now()
  await run('wforce')
  const fr = await bot.waitPacket('level_particles', (d) => d.particle === 'end_rod', 3000, since)
  t.check('end_rod particle with force', fr && fr.force === true, fr ? `force ${fr.force}` : 'none')
  since = Date.now()
  let m = srv.mark()
  await run('wdust')
  const dustWarn = await srv.waitLog(/dust/i, 2000, m)
  const dustPkt = await bot.waitPacket('level_particles', (d) => d.particle === 'dust', 1000, since)
  if (dustPkt) t.check('dust particle sent', true)
  else t.skip('dust particles with color', `Pumpkin's particle API carries no particle data, Carve skips them${dustWarn ? ' with a warning' : ''}`)
  t.check('bot still connected after dust', !bot.closed)

  // ---- sounds ----
  since = Date.now()
  await run('wsound')
  const pl = await bot.waitPacket('sound', (d) => /note_block\.pling/.test(d.sound), 3000, since)
  t.check('play sound: id', !!pl, pl ? pl.sound : 'none')
  t.check('play sound: volume 0.5 pitch 2', pl && Math.abs(pl.volume - 0.5) < 1e-3 && Math.abs(pl.pitch - 2) < 1e-3, pl ? `vol ${pl.volume} pitch ${pl.pitch}` : 'none')
  const an = await bot.waitPacket('sound', (d) => /anvil\.land/.test(d.sound), 3000, since)
  t.check('play sound at location', an && Math.abs(an.x - 3) < 0.7 && Math.abs(an.z - 3) < 0.7, an ? `${an.x} ${an.y} ${an.z}` : 'none')
  since = Date.now()
  await run('wstop', 'stop1')
  const ss = await bot.waitPacket('stop_sound', (d) => d.sound && /music\.game/.test(d.sound), 3000, since)
  t.check('stop sound "music.game"', !!ss, ss ? ss.sound : 'none')
  since = Date.now()
  await run('wstopall')
  const sa = await bot.waitPacket('stop_sound', (d) => !d.sound && !d.source, 3000, since)
  t.check('stop all sounds', !!sa)

  // ---- potion effects ----
  since = Date.now()
  await run('wpot', 'nopoison')
  const sp = await bot.waitPacket('update_mob_effect', (d) => d.effect === 'speed', 3000, since)
  t.check('apply speed 2 for 10 seconds: update_mob_effect', sp && sp.amplifier === 1 && Math.abs(sp.duration - 200) <= 2 && sp.entityId === me,
    sp ? `amp ${sp.amplifier} dur ${sp.duration} ent ${sp.entityId}/${me}` : 'none')
  const jb = await bot.waitPacket('update_mob_effect', (d) => d.effect === 'jump_boost', 3000, since)
  t.check('apply jump boost: default tier 1, 15 seconds', jb && jb.amplifier === 0 && Math.abs(jb.duration - 300) <= 2, jb ? `amp ${jb.amplifier} dur ${jb.duration}` : 'none')
  t.check('condition: has potion effect speed', !!await bot.waitText('w:hasspeed', { since, ms: 1000 }))
  t.check("condition: doesn't have potion effect poison", !!await bot.waitText('w:nopoison', { since, ms: 1000 }))
  since = Date.now()
  const gone = await run('wunpot', 'speedgone', 3000)
  const rm = await bot.waitPacket('remove_mob_effect', (d) => d.effect === 'speed', 3000, since)
  t.check('remove speed potion effect: remove_mob_effect', !!rm)
  t.check("condition after remove: doesn't have speed", !!gone)
  since = Date.now()
  await run('wunall')
  const rmAll = await bot.waitPacket('remove_mob_effect', (d) => d.effect === 'jump_boost', 3000, since)
  t.check('remove all potion effects: remove_mob_effect jump_boost', !!rmAll)

  // ---- fire ----
  since = Date.now()
  const fire = await run('wfire', 'fire')
  const onFire = await bot.waitPacket('set_entity_data', (d) => d.entityId === me && d.bytes[0] != null && (d.bytes[0] & 1), 3000, since)
  t.check('ignite player: on-fire flag in entity data', !!onFire)
  t.check('burn duration after ignite', fire && /(5|4\.\d+|100) ?(s|sec|seconds|ticks)?/.test(fire) && !/<none>/.test(fire), fire || 'no reply')
  since = Date.now()
  const water = await run('wwater', 'water')
  const offFire = await bot.waitPacket('set_entity_data', (d) => d.entityId === me && d.bytes[0] != null && !(d.bytes[0] & 1), 3000, since)
  t.check('extinguish player: on-fire flag cleared', !!offFire)
  t.check('burn duration after extinguish is 0', water && /\b0\b/.test(water), water || 'no reply')

  // ---- display entities ----
  since = Date.now()
  await run('wdisplay')
  for (const ty of ['text_display', 'block_display', 'item_display']) {
    const d = await bot.waitPacket('add_entity', (e) => e.type === ty, 3000, since)
    t.check(`spawn ${ty.replace('_', ' ')}`, !!d, d ? `${d.x} ${d.y} ${d.z}` : 'none')
  }

  // ---- bounds: walk in and out ----
  const bnd = await run('wbound', 'bound')
  t.check('create bound and read its volume', bnd && /^w:bound 868$/.test(bnd), bnd || 'no reply')
  t.check('player starts outside the bound', !!await run('winbound', 'outside'))
  since = Date.now()
  await bot.walkTo(7.5, 101, 0.5)
  const enter = await bot.waitText('w:enter', { since, ms: 3000 })
  t.check('on bound enter by walking', enter && /zone/.test(enter.text), enter ? enter.text : 'none')
  t.check('player is within bound after walking', !!await run('winbound', 'inside'))
  since = Date.now()
  await bot.walkTo(0.5, 101, 0.5)
  const exit = await bot.waitText('w:exit', { since, ms: 3000 })
  t.check('on bound exit by walking back', exit && /zone/.test(exit.text), exit ? exit.text : 'none')

  // ---- chunks, tags, misc ----
  n = said.length
  await run('wchunk', 'chunkx')
  await sleep(200)
  t.check('chunk at player', /^w:chunk 0 0$/.test(text(n, 'w:chunk ') || ''), text(n, 'w:chunk ') || 'no reply')
  t.check('chunks in radius 1: 9', /^w:chunks 9$/.test(text(n, 'w:chunks') || ''), text(n, 'w:chunks') || 'no reply')
  t.check('chunk x/z of location', /^w:chunkx 2 -2$/.test(text(n, 'w:chunkx') || ''), text(n, 'w:chunkx') || 'no reply')
  t.check('condition: chunk is loaded', !!text(n, 'w:loaded'))

  n = said.length
  await run('wtag', 'tagdone')
  t.check('oak log is tagged with minecraft tag "logs"', !!text(n, 'w:tagged'))
  t.check('minecraft tags of oak log lists logs', /logs/.test(text(n, 'w:tags') || ''), text(n, 'w:tags') || 'no reply')
  const logs = text(n, 'w:logs')
  t.check('minecraft block tag "logs" has members', logs && Number(logs.split(' ')[1]) > 10, logs || 'no reply')
  t.check('block at is tagged with beacon_base_blocks', !!text(n, 'w:beacon'))

  n = said.length
  await run('wmisc', 'miscdone')
  for (const k of ['world', 'biome', 'light', 'env', 'moon', 'high', 'spawn']) {
    const s = text(n, `w:${k} `)
    t.check(`${k} expression has a value`, s && !/<none>|^w:\w+ $|::/.test(s), s || 'no reply')
  }

  await sleep(300)
  t.check('bot still connected after the world area', !bot.closed)
  const errs = bot.packets.filter((p) => p.error)
  t.check('every decoded packet parsed', errs.length === 0, errs.slice(0, 3).map((p) => `${p.name}: ${p.error}`).join('; '))
  const warns = srv.lines.filter((l) => /\[Carve\]/.test(l) && /warn|error/i.test(l))
  if (warns.length) console.log(`carve warnings:\n  ${warns.slice(0, 10).join('\n  ')}`)
})
