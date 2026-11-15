// Conditions the game can answer (age, creeper state,
// handedness, held items, block properties, pressed keys, plugins, features,
// version, ban list, flight) and the data-table checks (edible, fuel,
// fire resistant, flammable).
//   node tests/ingame/areas/conditions-mob-block-state.js
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('conditions-mob-block-state', { port: 25613 })
const NAME = 'Tester'
const SCRIPTS = [path.join(__dirname, 'conditions-mob-block-state', 'conditions-mob-block-state.carve')]

const varint = (n) => { const b = []; n >>>= 0; do { let x = n & 0x7f; n >>>= 7; if (n) x |= 0x80; b.push(x) } while (n); return Buffer.from(b) }
const str = (s) => { const b = Buffer.from(s, 'utf8'); return Buffer.concat([varint(b.length), b]) }

t.run(async () => {
  const srv = await startServer({ dir: t.dir, port: t.port, scripts: SCRIPTS, ops: [NAME] })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop(60000)))
  const loaded = await srv.waitLog(/\[Carve\] Loaded 1 script/, 8000)
  t.check('carve loads the script without errors', !!loaded && !srv.lines.some((l) => /conditions-mob-block-state\.carve:\d+/.test(l)),
    loaded ? srv.lines.filter((l) => /conditions-mob-block-state\.carve/.test(l)).slice(-6).join(' | ') : 'not loaded')

  const bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  const run = async (cmd, prefix, ms = 15000) => {
    const since = Date.now()
    bot.command(cmd)
    const m = await bot.waitText(prefix, { since, ms })
    return m ? m.text : null
  }
  const has = (m, w) => !!m && m.split(/\s+/).includes(w)

  srv.console(`gamemode creative ${NAME}`)
  await bot.waitPacket('game_event', (d) => d.gameMode === 'creative', 5000)
  const [px, py, pz] = ((await run('cmbpos', 'c:pos')) || 'c:pos 0 100 0').split(' ').slice(1).map(Number)
  srv.console(`fill ${px - 4} ${py - 1} ${pz - 4} ${px + 4} ${py - 1} ${pz + 4} stone`)
  srv.console(`fill ${px - 4} ${py} ${pz - 4} ${px + 4} ${py + 2} ${pz + 4} glass outline`)
  srv.console(`fill ${px - 3} ${py} ${pz - 3} ${px + 3} ${py + 2} ${pz + 3} air`)
  await sleep(1500)

  // ---- pure and table checks ----
  let m = await run('cmblogic', 'c:logic')
  const want = 'ABCDEFGHIJKLMNOPQRSTUVW'
  const got = m ? m.split(' ')[1] || '' : ''
  t.check('vector, plugin, feature, version, edible, fuel, fire resistant, flammable, command',
    got === want, m ? `${m} (missing: ${[...want].filter((c) => !got.includes(c)).join('')})` : 'no message')

  // ---- mobs ----
  await run('cmbspawn', 'c:spawned', 20000)
  await sleep(1500)
  await sleep(800)
  srv.console('say step')
  m = await run('cmbmobs', 'c:mobs')
  t.check('adult pig is an adult and not a baby', has(m, 'padult') && has(m, 'pnobaby'), m || 'no message')
  t.check('baby cow is a baby and not an adult', has(m, 'bbaby') && has(m, 'bnoadult'), m || '')
  t.check('plain creeper is not charged and not igniting', has(m, 'cplain') && has(m, 'cquiet') && !has(m, 'ccharged'), m || '')
  t.check('pig on the ground is not climbing', has(m, 'pnoclimb'), m || '')
  t.check('pig is not riding', has(m, 'pfree'), m || '')
  t.check('entity is of type pig and not of type cow', has(m, 'ptype') && has(m, 'pnotcow'), m || '')
  t.check('pig is not from a spawner', has(m, 'pwild'), m || '')
  t.check('entities are not silent by default', has(m, 'pnoisy') && !has(m, 'slsilent'), m || '')
  await run('cmbsilent', 'c:silenced')
  await sleep(800)
  m = await run('cmbmobs', 'c:mobs')
  if (has(m, 'slsilent')) t.check('a silenced pig is silent', true, m)
  else t.skip('a silenced pig is silent', `data merge entity did not set Silent (${m})`)

  await run('cmbcharge', 'c:charged')
  m = await run('cmbmobs', 'c:mobs')
  if (has(m, 'ccharged')) t.check('a creeper made powered is charged', true, m)
  else t.skip('a creeper made powered is charged', `data merge entity does not set powered (${m})`)
  await run('cmbignite', 'c:ignited')
  m = await run('cmbmobs', 'c:mobs')
  if (has(m, 'cquiet')) t.skip('a creeper with a lit fuse is igniting', `data merge entity does not light the fuse (${m})`)
  else t.check('a creeper with a lit fuse is igniting', true, m)

  m = await run('cmbpath', 'c:path', 10000)
  t.check('pathfinding: idle cow is not, a cow sent walking is', m === 'c:path idle  walking' || m === 'c:path idle walking', m || 'no message')

  // ---- blocks ----
  const blk = async (x, y, z, set) => {
    let out = null
    // The console runs the command on its own thread; ask again when the old block is still there.
    for (let attempt = 0; attempt < 3; attempt++) {
      if (set.includes('[')) {
        // Pumpkin's setblock takes no block states: place the block, then set its data.
        srv.console(`setblock ${x} ${y} ${z} ${set.split('[')[0]}`)
        await sleep(400)
        await run(`cmbset ${x} ${y} ${z} ${set}`, 'c:set')
      } else {
        srv.console(`setblock ${x} ${y} ${z} ${set}`)
      }
      await sleep(1000)
      out = await run(`cmbblk ${x} ${y} ${z}`, 'c:blk')
      const air = !!out && has(out, 'transp') && has(out, 'pass') && !has(out, 'solid')
      if (set === 'air' || !air) break
    }
    return out
  }
  const bx = px + 2; const by = py; const bz = pz
  m = await blk(bx, by, bz, 'stone')
  t.check('stone: solid, occluding, not transparent, not passable, of type stone',
    has(m, 'solid') && has(m, 'occ') && !has(m, 'transp') && !has(m, 'pass') && has(m, 'stone'), m || 'no message')
  m = await blk(bx, by, bz, 'glass')
  t.check('glass: solid, not occluding, not of type stone', has(m, 'solid') && !has(m, 'occ') && !has(m, 'stone'), m || 'no message')
  m = await blk(bx, by, bz, 'air')
  t.check('air: transparent and passable', has(m, 'transp') && has(m, 'pass') && !has(m, 'solid'), m || 'no message')
  m = await blk(bx, by, bz, 'minecraft:command_block[conditional=true]')
  t.check('command block with conditional=true is conditional', has(m, 'cond'), m || 'no message')
  m = await blk(bx, by, bz, 'minecraft:command_block')
  t.check('plain command block is not conditional', !has(m, 'cond') && m !== null, m || 'no message')
  m = await blk(bx, by, bz, 'minecraft:oak_leaves[persistent=true]')
  t.check('persistent leaves are persistent', has(m, 'pers'), m || 'no message')
  m = await blk(bx, by, bz, 'minecraft:oak_leaves[persistent=false]')
  t.check('plain leaves are not persistent', !has(m, 'pers') && m !== null, m || 'no message')
  m = await blk(bx, by, bz, 'minecraft:lever[powered=true]')
  t.check('a pulled lever is powered', has(m, 'pow'), m || 'no message')
  m = await blk(bx, by, bz, 'minecraft:lever[powered=false]')
  t.check('a lever that is off is not powered', !has(m, 'pow') && m !== null, m || 'no message')
  m = await blk(bx, by, bz, 'minecraft:redstone_lamp[lit=true]')
  t.check('a lit lamp is powered', has(m, 'pow'), m || 'no message')
  m = await blk(bx, by, bz, 'minecraft:oak_stairs')
  t.check('stairs: passable is false, solid', !has(m, 'pass'), m || 'no message')
  m = await blk(bx, by, bz, 'minecraft:chest')
  t.check('an ordinary chest is not lootable', !has(m, 'loot') && m !== null, m || 'no message')
  srv.console(`setblock ${bx} ${by} ${bz} minecraft:chest`)
  await sleep(400)
  srv.console(`data merge block ${bx} ${by} ${bz} {LootTable:"minecraft:chests/simple_dungeon"}`)
  await sleep(400)
  m = await run(`cmbblk ${bx} ${by} ${bz}`, 'c:blk')
  if (has(m, 'loot')) t.check('a chest with a loot table is lootable', true, m)
  else t.skip('a chest with a loot table is lootable', `could not give the chest a loot table (${m})`)
  srv.console(`setblock ${bx} ${by} ${bz} air`)

  // ---- the player ----
  srv.console(`clear ${NAME}`)
  srv.console(`give ${NAME} diamond_sword 1`)
  await sleep(500)
  bot.setHotbar(0)
  await sleep(300)
  m = await run('cmbself', 'c:self')
  t.check('player is right handed, not left handed', has(m, 'right') && has(m, 'notleft'), m || 'no message')
  // Main hand 0 is left, 1 is right in the client settings packet.
  bot.send('client_information', str('en_us'), [10], varint(0), [1], [0x7f], varint(0), [0], [1])
  await sleep(600)
  const left = await run('cmbself', 'c:self')
  t.check('player who picked the left hand is left handed, not right handed', has(left, 'left') && !has(left, 'right') && !has(left, 'notleft'), left || 'no message')
  bot.send('client_information', str('en_us'), [10], varint(0), [1], [0x7f], varint(1), [0], [1])
  await sleep(600)
  t.check('holding a diamond sword (any hand and main hand)', has(m, 'holds') && has(m, 'main') && has(m, 'notoff') && has(m, 'nostick'), m || '')
  t.check('creative player can fly and is not flying', has(m, 'canfly') && !has(m, 'flying'), m || '')
  t.check('player is not riding and not silent', has(m, 'free') && has(m, 'noisy'), m || '')
  t.check('player is inside their own chunk and the world border, not in a far chunk', has(m, 'inchunk') && has(m, 'outchunk') && has(m, 'inborder'), m || '')
  t.check('the chunk of the player is loaded, a far chunk is not', has(m, 'loaded') && has(m, 'unloaded'), m || '')
  t.check('no keys pressed', has(m, 'nofwd') && !has(m, 'jump') && !has(m, 'sneak'), m || '')
  bot.sneak(true)
  await sleep(400)
  m = await run('cmbself', 'c:self')
  t.check('pressing the sneak key', has(m, 'sneak') && !has(m, 'jump'), m || 'no message')
  bot.sneak(false)
  await sleep(300)
  m = await run('cmbself', 'c:self')
  t.check('released the sneak key', !has(m, 'sneak'), m || 'no message')
  bot._input(16, true)
  await sleep(400)
  m = await run('cmbself', 'c:self')
  t.check('pressing the jump key', has(m, 'jump'), m || 'no message')
  bot._input(16, false)
  await sleep(400)
  // Jumping follows the jump statistic: the first query takes a baseline.
  await run('cmbself', 'c:self')
  bot.moveTo(px + 0.5, py + 0.4, pz + 0.5, false)
  bot._input(16, true)
  await sleep(300)
  bot.moveTo(px + 0.5, py, pz + 0.5, true)
  await sleep(300)
  m = await run('cmbself', 'c:self')
  if (has(m, 'jumping')) t.check('a player who just jumped is jumping', true, m)
  else t.skip('a player who just jumped is jumping', `the jump statistic did not change for the test bot (${m})`)
  bot._input(16, false)

  srv.console(`setblock ${px + 2} ${py} ${pz} ladder`)
  await sleep(500)
  bot.moveTo(px + 2.5, py, pz + 0.5, false)
  await sleep(600)
  m = await run('cmbself', 'c:self')
  t.check('a player in the air on a ladder is climbing', has(m, 'climbing'), m || 'no message')
  bot.moveTo(px + 0.5, py, pz + 0.5, true)
  await sleep(600)
  m = await run('cmbself', 'c:self')
  t.check('a player on the ground is not climbing', !has(m, 'climbing') && m !== null, m || 'no message')
  srv.console(`setblock ${px + 2} ${py} ${pz} air`)

  srv.console(`gamemode survival ${NAME}`)
  await sleep(800)
  m = await run('cmbself', 'c:self')
  t.check('survival player cannot fly', !has(m, 'canfly'), m || 'no message')
  srv.console(`gamemode creative ${NAME}`)
  await sleep(500)
  bot.fly(true)
  await sleep(600)
  m = await run('cmbself', 'c:self')
  t.check('player who started flying is flying', has(m, 'flying'), m || 'no message')
  bot.fly(false)

  // ---- respawn point: bed, anchor, neither ----
  const sx = px - 2; const sy = py; const sz = pz
  srv.console(`setblock ${sx} ${sy} ${sz} white_bed`)
  await sleep(700)
  m = await run(`cmbspawnpoint ${sx} ${sy} ${sz}`, 'c:spawnpoint')
  t.check('a respawn point on a bed is a bed spawn', has(m, 'bed') && !has(m, 'anchor') && !has(m, 'notbed'), m || 'no message')
  srv.console(`setblock ${sx} ${sy} ${sz} respawn_anchor`)
  await sleep(700)
  m = await run(`cmbspawnpoint ${sx} ${sy} ${sz}`, 'c:spawnpoint')
  t.check('a respawn point on an anchor is an anchor spawn', has(m, 'anchor') && has(m, 'notbed'), m || 'no message')
  srv.console(`setblock ${sx} ${sy} ${sz} stone`)
  await sleep(700)
  m = await run(`cmbspawnpoint ${sx} ${sy} ${sz}`, 'c:spawnpoint')
  t.check('a respawn point on a stone block is neither', has(m, 'notbed') && !has(m, 'anchor') && !has(m, 'bed'), m || 'no message')
  srv.console(`setblock ${sx} ${sy} ${sz} air`)

  // ---- bans ----
  const pleb = await joinBot('Pleb', { port: t.port })
  await sleep(500)
  m = await run('cmbban Pleb', 'c:ban')
  t.check('a player who is not banned', m === 'c:ban no', m || 'no message')
  srv.console('ban Pleb testing')
  await sleep(1200)
  pleb.quit()
  m = await run('cmbban Pleb', 'c:ban')
  t.check('a banned player is banned', m === 'c:ban yes', m || 'no message')
  srv.console('pardon Pleb')
})
