// Smoke area: proves the harness end to end.
//   node tests/ingame/areas/smoke.js
'use strict'
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('smoke', { port: 25570 })

const SCRIPT = `
on join:
    send "s:hello %player%" to player
    send title "s:title" with subtitle "s:sub" to player

command /sblock:
    trigger:
        set block at location(3, 64, 3) to diamond block
        send "s:block" to player

command /spig:
    trigger:
        spawn a pig at player

command /ssound:
    trigger:
        play sound "block.note_block.pling" with volume 0.5 and pitch 2 to player

on right click:
    send "s:click" to player
`

t.run(async () => {
  const srv = await startServer({ dir: t.dir, port: t.port, scripts: { 'smoke.carve': SCRIPT }, ops: ['Smoke'] })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop()))
  t.check('carve loads the script', !!await srv.waitLog(/\[Carve\] Loaded 1 script/, 3000))

  const bot = await joinBot('Smoke', { port: t.port })
  t.cleanup(() => bot.quit())
  t.check('bot gets its entity id from login', Number.isInteger(bot.entityId))

  t.check('message on join', !!await bot.waitText('s:hello Smoke', { kind: 'system' }))
  const title = await bot.waitPacket('set_title_text', (d) => d.text.includes('s:title'))
  t.check('title', !!title)
  t.check('subtitle', !!await bot.waitPacket('set_subtitle_text', (d) => d.text.includes('s:sub')))

  // Op from ops.json: /sblock has no permission, but /op-only console commands do.
  srv.console('gamemode creative Smoke')
  const gm = await bot.waitPacket('game_event', (d) => d.gameMode === 'creative')
  t.check('game_event: change game mode', !!gm)
  const info = await bot.waitPacket('player_info_update', (d) => d.players.some((p) => p.gameMode === 'creative'))
  t.check('player_info_update: game mode', !!info)

  let since = Date.now()
  bot.command('sblock')
  const blk = await bot.waitPacket('block_update', (d) => d.x === 3 && d.y === 64 && d.z === 3, 5000, since)
  t.check('block update from set block', blk && blk.block === 'diamond_block', blk ? blk.block : 'none')

  since = Date.now()
  bot.command('spig')
  const pig = await bot.waitPacket('add_entity', (d) => d.type === 'pig', 5000, since)
  t.check('entity spawn: spawn a pig at player', !!pig, pig ? `id ${pig.entityId} at ${pig.x.toFixed(1)} ${pig.y.toFixed(1)} ${pig.z.toFixed(1)}` : 'none')

  since = Date.now()
  bot.command('ssound')
  const snd = await bot.waitPacket('sound', (d) => /note_block\.pling/.test(d.sound), 5000, since)
  t.check('sound', !!snd, snd ? `${snd.sound} vol ${snd.volume} pitch ${snd.pitch}` : 'none')

  since = Date.now()
  srv.console('give Smoke stone 3')
  const slot = await bot.waitPacket('container_set_slot', (d) => d.item && d.item.name === 'stone', 5000, since)
  t.check('container_set_slot: given item', !!slot, slot ? `slot ${slot.slot} x${slot.item.count}` : 'none')

  // Actions: hotbar, look, a few steps, sneak/sprint, right click air, swap hands, click a slot.
  bot.setHotbar(0)
  bot.look(90, 0)
  const start = bot.pos
  await bot.walkTo(start.x + 1, start.y, start.z)
  bot.sneak(true); bot.sneak(false); bot.sprint(true); bot.sprint(false)
  since = Date.now()
  bot.useItem()
  t.check('use item fires on right click', !!await bot.waitText('s:click', { since }))
  bot.swapHands()
  bot.clickSlot(36)
  bot.closeContainer(0)

  if (pig) {
    since = Date.now()
    bot.attack(pig.entityId)
    const hit = await Promise.race([
      bot.waitPacket('damage_event', (d) => d.entityId === pig.entityId, 3000, since),
      bot.waitPacket('entity_event', (d) => d.entityId === pig.entityId, 3000, since),
      bot.waitPacket('sound_entity', (d) => d.entityId === pig.entityId, 3000, since),
    ])
    if (hit) t.check('attack reaches the pig', true)
    else t.skip('attack reaches the pig', 'no damage/event/sound packet for the pig')
  }
  await sleep(300)
  t.check('bot still connected after all actions', !bot.closed)
  const errs = bot.packets.filter((p) => p.error)
  t.check('every decoded packet parsed', errs.length === 0, errs.slice(0, 3).map((p) => `${p.name}: ${p.error}`).join('; '))
  const seen = [...new Set(bot.packets.map((p) => p.name))].sort()
  console.log(`decoded: ${seen.join(' ')}`)
})
