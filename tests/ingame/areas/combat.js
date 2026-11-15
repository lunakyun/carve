// Combat area: on damage (player and mob victims, player and mob attackers),
// set/add/remove/cancel damage, on death with attacker and death message,
// respawn, and entity values (spawn, last spawned, nearest, radius, type,
// name, location, health, velocity, push, teleport, kill, despawn, is a).
//   node tests/ingame/areas/combat.js
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')
const { Reader } = require('../lib/proto.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('combat', { port: 25572 })
const NAME = 'Fighter'
const SCRIPTS = ['combat.carve', 'entities.carve'].map((f) => path.join(__dirname, 'combat', f))

t.run(async () => {
  const srv = await startServer({ dir: t.dir, port: t.port, scripts: SCRIPTS, ops: [NAME] })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop(60000)))
  const loaded = await srv.waitLog(/\[Carve\] Loaded 2 script/, 5000)
  t.check('carve loads both scripts', !!loaded, loaded ? '' : srv.lines.filter((l) => /Carve|error/i.test(l)).slice(-6).join(' | '))

  const bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  // Raw packet names with the entity id (first varint) for packets we don't decode.
  const raw = []
  bot.on('packet', (p) => {
    let eid = null
    try { eid = new Reader(p.buffer).varint() } catch {}
    raw.push({ name: p.name, eid, t: Date.now(), buffer: p.buffer })
  })
  const waitRaw = async (names, eid, since, ms = 4000) => {
    const end = Date.now() + ms
    for (;;) {
      const hit = raw.find((r) => r.t >= since && names.includes(r.name) && (eid == null || r.eid === eid))
      if (hit || Date.now() > end) return hit || null
      await sleep(50)
    }
  }
  // Run a command and wait for the chat line starting with `prefix`.
  const run = async (cmd, prefix, ms = 10000) => {
    const since = Date.now()
    bot.command(cmd)
    const m = await bot.waitText(prefix, { since, ms })
    return m ? m.text : null
  }
  const text = (s, prefix, ms = 10000) => bot.waitText(prefix, { since: s, ms }).then((m) => (m ? m.text : null))
  const health = (since, pred, ms = 8000) => bot.waitPacket('set_health', pred, ms, since)
  // Hit an entity until the script reports the hit by the bot (a hit lands only
  // in reach and outside hurt immunity). Never attack a removed entity: that kicks.
  const hit = async (eid, prefix, tries = 4) => {
    for (let i = 0; i < tries && bot.entities.has(eid); i++) {
      const since = Date.now()
      bot.attack(eid)
      const got = await bot.waitText([prefix, `attacker=${NAME}`], { since, ms: 1500 })
      if (got) return got.text
    }
    return null
  }
  // A fresh pig for the hit and moving tests: knockback can push a pig into a wall,
  // and Pumpkin can deal a pushed pig all its fall damage at once.
  const fresh = async () => {
    const since = Date.now()
    await run('efresh', 'e:fresh')
    const p = await bot.waitPacket('add_entity', (d) => d.type === 'pig', 5000, since)
    await sleep(1500)
    return p ? p.entityId : -1
  }
  const mode = async (m) => { await run(`cmode ${m}`, `c:mode ${m}`); await sleep(600) } // also outlasts hurt immunity

  srv.console(`gamemode survival ${NAME}`)
  srv.console('time set midnight') // the zombie must not burn
  // The bot has no gravity: drop an armor stand, then walk down to where it landed,
  // so mobs on the ground stay in reach.
  await run('eprobe', 'e:probe')
  let gy = NaN
  for (let i = 0, last = NaN; i < 15; i++) { // until it stops falling
    await sleep(1000)
    last = gy
    gy = parseFloat(((await run('eprobey', 'e:probey')) || '').split(' ')[1])
    if (gy === last) break
  }
  await run('eprobegone', 'e:probegone')
  if (gy < bot.pos.y) await bot.walkTo(bot.pos.x, gy, bot.pos.z)
  t.check('bot stands on the ground', gy <= bot.pos.y + 0.01, `ground y ${gy}, bot y ${bot.pos.y}`)
  await bot.waitPacket('game_event', (d) => d.gameMode === 'survival', 5000)
  await sleep(500)

  // ---- player victim, no attacker (/damage) ----
  await mode('none')
  let s = Date.now()
  srv.console(`damage ${NAME} 3`)
  let m = await text(s, 'c:pdmg')
  t.check('on damage of player: victim, damage, no attacker', !!m && m.includes(`victim=${NAME}`) && m.includes('attacker=<none>') && m.includes('dmg=3'), m || 'no message')
  t.check('damage cause in on damage', !!m && /cause=\w+/.test(m) && !m.includes('cause=<none>'), m || '')
  let h = await health(s, (d) => d.health < 20)
  t.check('/damage 3 lowers health to 17', !!h && Math.abs(h.health - 17) < 0.01, h ? `health ${h.health}` : 'no set_health')
  t.check('on damage of zombie does not run for a player', !await text(s, 'c:zdmg', 300))

  const hpBefore = async () => { await sleep(100); return bot.health }
  for (const [md, dealt, expect] of [['set', 5, 1], ['add', 2, 4], ['remove', 3, 2]]) {
    srv.console(`effect clear ${NAME}`)
    srv.console(`effect give ${NAME} instant_health 1 5`)
    await sleep(700)
    await mode(md)
    const before = await hpBefore()
    s = Date.now()
    srv.console(`damage ${NAME} ${dealt}`)
    h = await health(s, (d) => d.health < before)
    t.check(`${md} damage (/damage ${dealt} takes ${expect})`, !!h && Math.abs(before - h.health - expect) < 0.01, h ? `${before} -> ${h.health}` : `no set_health (from ${before})`)
  }
  await mode('cancel')
  let before = await hpBefore()
  s = Date.now()
  srv.console(`damage ${NAME} 4`)
  m = await text(s, 'c:pdmg')
  h = await health(s, (d) => d.health < before, 1500)
  t.check('cancel event in on damage keeps health', !!m && !h, m ? (h ? `health ${before} -> ${h.health}` : `health stays ${before}`) : 'event did not run')

  // ---- entities ----
  await mode('none')
  s = Date.now()
  m = await run('espawn &aBob', 'e:spawned')
  await sleep(300)
  const pigs = bot.packets.filter((p) => p.name === 'add_entity' && p.t >= s && p.data.type === 'pig').map((p) => p.data)
  t.check('spawn 3 pigs sends 3 pigs', pigs.length === 3, `${pigs.length} add_entity pig`)
  t.check('last spawned entity, type and name of entity', !!m && m.includes('type=pig') && m.includes('name=Bob'), m || 'no message')
  t.check('all pigs in radius counts the spawned pigs', !!m && m.includes('count=3'), m || '')
  t.check('last spawned entity prints as its type', !!await text(s, 'e:last pig', 500))
  t.check('entity is a pig', !!await text(s, 'e:isapig yes', 500))
  t.check('entity is not a zombie', !!await text(s, 'e:notzombie yes', 500))
  // The named pig is the last one spawned: its metadata carries the custom name.
  const pig = pigs[pigs.length - 1]
  const named = pig && raw.find((r) => r.t >= s && r.name === 'set_entity_data' && r.eid === pig.entityId && r.buffer.includes(Buffer.from('Bob')))
  t.check('set name of entity reaches the client', !!named, pig ? `entity ${pig.entityId}` : 'no pig')

  m = await run('einfo', 'e:info')
  t.check('health/location/world of entity', !!m && /health=10\b/.test(m) && /x=-?[\d.]+ y=-?[\d.]+ z=-?[\d.]+/.test(m) && !m.includes('world=<none>'), m || 'no message')
  m = await text(Date.now() - 2000, 'e:vel')
  t.check('x/y/z velocity of entity', !!m && /x=-?[\d.]+ y=-?[\d.]+ z=-?[\d.]+/.test(m), m || 'no message')

  m = await run('enear', 'e:nearest ')
  t.check('nearest entity to player', !!m && m.includes('e:nearest pig'), m || 'no message')
  s = Date.now() - 3000
  m = await text(s, 'e:nearestpig')
  t.check('nearest pig within N blocks', !!m && m.includes('pig'), m || '')
  m = await text(s, 'e:nearestzombie')
  t.check('nearest zombie is <none> without zombies', !!m && m.includes('<none>'), m || '')
  m = await text(s, 'e:radius')
  t.check('all pigs / all entities in radius', !!m && m.includes('pigs=3') && /ents=[34]\b/.test(m), m || '')
  m = await text(s, 'e:looped')
  t.check('loop all entities in radius with loop-entity is a pig', !!m && m.includes('e:looped 3'), m || '')


  if (pig) {
    m = await run('ehp', 'e:hp')
    t.check('set health of entity', !!m && m.includes('e:hp 8'), m || '')
    m = await text(Date.now() - 3000, 'e:hp2')
    t.check('remove from / add to health of entity', !!m && m.includes('e:hp2 6'), m || '')

    // ---- mob victim attacked by the player ----
    await mode('none')
    s = Date.now()
    m = await hit(pig.entityId, 'c:mobdmg')
    t.check('on damage of a mob: victim, attacker player, player is <none>', !!m && m.includes('victim=pig') && m.includes(`attacker=${NAME}`) && m.includes('player=<none>'), m || 'no message')
    t.check('name of mob victim is its custom name', !!m && m.includes('name=Bob'), m || '')
    t.check('on damage of zombie does not run for a pig', !await text(s, 'c:zdmg', 300))
    const hurt = await bot.waitPacket('damage_event', (d) => d.entityId === pig.entityId, 2000, s) ||
      await bot.waitPacket('hurt_animation', (d) => d.entityId === pig.entityId, 500, s) ||
      await bot.waitPacket('entity_event', (d) => d.entityId === pig.entityId && d.event === 2, 500, s)
    t.check('player hit on the pig reaches the client', !!hurt)

    await run('ehere', 'e:here')
    await mode('pigcancel')
    m = await run('einfo', 'e:info')
    const hp0 = m && (m.match(/health=([\d.]+)/) || [])[1]
    // The pig can walk out of reach between `ehere` and the hit: bring it back and retry.
    let cm = null
    for (let i = 0; i < 3 && !cm; i++) {
      if (i) { await run('ehere', 'e:here'); await sleep(600) }
      s = Date.now()
      cm = await hit(pig.entityId, 'c:mobdmg')
    }
    await sleep(300)
    m = await run('einfo', 'e:info')
    const hp1 = m && (m.match(/health=([\d.]+)/) || [])[1]
    t.check('cancel damage of a mob keeps its health', !!cm && hp0 != null && hp0 === hp1, `${hp0} -> ${hp1}`)

    // set damage on a mob: kill one of the other pigs with one hit
    const victim = { entityId: await fresh() }
    await mode('pigkill')
    // The pig can walk out of reach before the hit: retry with the hit's own window.
    let died = null
    for (let i = 0; i < 3 && !died; i++) {
      if (i) { await run('ehere', 'e:here'); await sleep(600) }
      s = Date.now()
      m = await hit(victim.entityId, 'c:mobdmg')
      died = await Promise.race([
        bot.waitPacket('entity_event', (d) => d.entityId === victim.entityId && d.event === 3, 3000, s),
        bot.waitPacket('remove_entities', (d) => d.ids.includes(victim.entityId), 3000, s),
      ])
    }
    t.check('set damage to 100 kills the mob', !!died, m || 'no damage event')
    await mode('none')

    let fid = await fresh()
    s = Date.now()
    await run('etp', 'e:tp done')
    const tp = await waitRaw(['teleport_entity', 'entity_position_sync'], fid, s)
    // Read the position right away: the pig's AI walks it off within a second.
    m = await run('einfo', 'e:info')
    await sleep(1000) // the teleport lands on a later tick and resets the velocity
    const num = (k) => parseFloat(((m || '').match(new RegExp(` ${k}=(-?[\\d.]+)`)) || [])[1])
    const near = Math.abs(num('x') - bot.pos.x) < 1.5 && Math.abs(num('z') - (bot.pos.z + 1)) < 1.5
    t.check('teleport entity', !!tp && near, `${tp ? tp.name : 'no teleport packet'}; ${m}`)

    for (const [dir, axis, sign] of [['north', 'z', -1], ['east', 'x', 1]]) {
      s = Date.now()
      // The pig's own AI can overwrite the push on the same tick, so push again
      // (up to 5 times) until a motion packet shows it. A pushed pig can die of
      // fall damage or leave view; then a fresh one takes its place.
      let pm = null
      for (let i = 0; i < 6 && !pm; i++) {
        if (i === 3 || !bot.entities.has(fid)) {
          fid = await fresh()
          await run('etp', 'e:tp done')
          await sleep(1000)
        }
        const since = Date.now()
        await run(`epush ${dir}`, `e:pushed ${dir}`)
        pm = await bot.waitPacket('set_entity_motion', (d) => d.entityId === fid && d.velocity && d.velocity[axis] * sign > 0.05, 2000, since)
      }
      t.check(`push entity ${dir}`, !!pm, pm ? JSON.stringify(pm.velocity) : 'no matching motion packet; got ' + bot.packets.filter((p) => p.name === 'set_entity_motion' && p.t >= s && p.data.entityId === fid).map((p) => JSON.stringify(p.data.velocity)).join(' '))
      await sleep(1000)
    }
    fid = await fresh()
    let mot = null
    for (let i = 0; i < 3 && !mot; i++) {
      s = Date.now()
      m = await run('evel', 'e:setvel')
      mot = await bot.waitPacket('set_entity_motion', (d) => d.entityId === fid && d.velocity && d.velocity.y > 0.5, 4000, s)
    }
    t.check('set y velocity of entity sends motion', !!mot, mot ? JSON.stringify(mot.velocity) : `no motion packet (${m})`)
    // Pumpkin ticks mobs while the script runs, so a tick may already have slowed it down.
    const vy = m ? parseFloat(m.split(' ')[1]) : NaN
    t.check('y velocity of entity reads back', vy > 1 && vy <= 1.5, m || '')
    fid = await fresh()
    // Retried like the pushes above: the pig's AI can overwrite the push.
    let pu = null
    for (let i = 0; i < 3 && !pu; i++) {
      s = Date.now()
      await run('epush up', 'e:pushed up')
      pu = await bot.waitPacket('set_entity_motion', (d) => d.entityId === fid && d.velocity && d.velocity.y > 0.2, 3000, s)
    }
    t.check('push entity up', !!pu, pu ? JSON.stringify(pu.velocity) : 'no matching motion packet')

    s = Date.now()
    await run('epush player', 'e:pushed player')
    const pp = await bot.waitPacket('set_entity_motion', (d) => d.entityId === bot.entityId && d.velocity && d.velocity.y > 0.2, 6000, s)
    t.check('push player upwards', !!pp, pp ? JSON.stringify(pp.velocity) : 'no motion packet for the bot')
    await sleep(2500) // the pushed pig may fall to death; let it land
  }

  // pigs[1] is still alive, the named pig maybe (the push can kill it with fall damage)
  const left = pigs[1]
  if (left) {
    s = Date.now()
    await run('ekillall', 'e:killall')
    const gone = await Promise.race([
      bot.waitPacket('entity_event', (d) => d.entityId === left.entityId && d.event === 3, 4000, s),
      bot.waitPacket('remove_entities', (d) => d.ids.includes(left.entityId), 4000, s),
    ])
    t.check('kill all pigs in radius', !!gone || !bot.entities.has(left.entityId))
  }
  await mode('none')
  s = Date.now()
  m = await run('eremove', 'e:cow')
  const cow = await bot.waitPacket('add_entity', (d) => d.type === 'cow', 2000, s)
  t.check('spawn a cow (one entity)', !!cow && !!m && m.includes('cow'), m || '')
  if (cow) {
    await text(s, 'e:despawned', 4000)
    const rm = await bot.waitPacket('remove_entities', (d) => d.ids.includes(cow.entityId), 3000, s)
    const death = bot.packets.find((p) => p.t >= s && p.name === 'entity_event' && p.data.entityId === cow.entityId && p.data.event === 3)
    t.check('despawn entity removes it without a death', !!rm && !death, rm ? (death ? 'death event seen' : '') : 'no remove packet')
  }

  // ---- mob attacker: a zombie next to the bot ----
  srv.console(`effect give ${NAME} instant_health 1 5`)
  await sleep(700)
  await mode('none')
  s = Date.now()
  m = await run('ezombie', 'e:zombie')
  const zombie = await bot.waitPacket('add_entity', (d) => d.type === 'zombie', 2000, s)
  t.check('spawn a zombie next to the player', !!zombie && !!m && m.includes('zombie'), m || '')
  m = await text(s, 'c:pdmg-by-zombie', 20000)
  t.check('on damage of player: zombie attacker, attacker is a zombie', !!m && m.includes('zombie'), m || 'the zombie never hit the bot in 20 s')
  const zm = m && await text(s, 'c:pdmg victim', 200)
  t.check('attacker prints as the mob type', !!zm && zm.includes('attacker=zombie'), zm || '')

  if (zombie) {
    s = Date.now()
    m = await hit(zombie.entityId, 'c:zdmg')
    t.check('on damage of zombie runs for a zombie victim', !!m && m.includes('victim=zombie') && m.includes(`attacker=${NAME}`), m || 'no message')
  }

  // ---- death by a zombie: attacker in on death and its death message ----
  if (m) {
    await mode('zkill')
    s = Date.now()
    await run('ezhere', 'e:zhere') // the hit knocked it back
    m = await text(s, 'c:death', 20000)
    t.check('on death after a zombie hit: attacker is the zombie', !!m && m.includes('attacker=zombie') && m.includes(`player=${NAME}`), m || 'no death in 20 s')
    t.check('death message set in on death reaches chat', !!await text(s, `c:eaten ${NAME}`, 2000))
    // The bot respawns where the zombie stands: remove it first, or it kills again.
    await run('ezkill', 'e:zkill')
    await mode('none')
    bot.respawn()
    t.check('on respawn after a zombie death', !!await text(s, `c:respawn ${NAME}`))
    await sleep(500)
  }
  await run('ezkill', 'e:zkill')
  await sleep(500)

  // ---- death by /kill: no attacker, custom message, respawn ----
  await mode('msg')
  await sleep(5500) // outlast the 5 s attacker memory
  s = Date.now()
  srv.console(`kill ${NAME}`)
  m = await text(s, 'c:death', 25000) // console commands can lag far behind on a busy machine
  t.check('on death from /kill: victim is the player, no attacker', !!m && m.includes(`victim=${NAME}`) && m.includes('attacker=<none>'), m || 'no message')
  const dm = await text(s, `c:custom death of ${NAME}`, 5000)
  t.check('set death message with colors', !!dm, dm || 'no custom death message')
  s = Date.now()
  bot.respawn()
  t.check('on respawn', !!await text(s, `c:respawn ${NAME}`))
  // Pumpkin sends no health packet on respawn, so take one heart and read the rest.
  await sleep(800)
  s = Date.now()
  srv.console(`damage ${NAME} 1`)
  const hr = await health(s, (d) => d.health < 20, 15000)
  t.check('respawn restores full health', !!hr && Math.abs(hr.health - 19) < 0.01, hr ? `health after 1 damage: ${hr.health}` : 'no set_health')

  t.skip('projectiles (on damage by arrow, shoot)', 'the test bot cannot draw a bow; the shoot section is checked in the w5-chunk6 area')
  await sleep(300)
  t.check('bot still connected', !bot.closed)
  const errs = bot.packets.filter((p) => p.error)
  t.check('every decoded packet parsed', errs.length === 0, errs.slice(0, 3).map((p) => `${p.name}: ${p.error}`).join('; '))
  const lerr = srv.lines.filter((l) => /\[Carve\].*(error|panic)|panicked/i.test(l))
  t.check('no Carve errors in the server log', lerr.length === 0, lerr.slice(0, 3).join(' | '))
})
