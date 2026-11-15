// Case chest area: a user's CS2-style case opening script
// (crates/carve-core/tests/case_chest.carve) on a real server.
//   node tests/ingame/areas/case-chest.js
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const now = () => Date.now() + 1
const t = area('case-chest', { port: 25640 })
const SCRIPTS = [
  path.join(__dirname, '..', '..', '..', 'crates', 'carve-core', 'tests', 'case_chest.carve'),
  // Says whether `player is sneaking` held for each click on a chest.
  path.join(__dirname, 'case-chest', 'probe.carve'),
]
const nameOf = (it) => {
  const c = it && it.components && it.components.custom_name
  // The vanilla give command keeps legacy color codes in the text.
  return c ? c.text.replace(/§./g, '') : null
}

t.run(async () => {
  const srv = await startServer({ dir: t.dir, port: t.port, scripts: SCRIPTS, ops: ['Tester'] })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop()))
  t.check('scripts load', !!await srv.waitLog(/\[Carve\] Loaded 2 scripts/, 3000))
  const errLine = srv.lines.find((l) => /\[Carve\].*(error|Error)/.test(l))
  t.check('no load errors', !errLine, errLine || '')
  // Pumpkin's plugin API v0.1 can't close a window: Carve says so at load, at the line.
  const warnAt = srv.lines.findIndex((l) => /WARN.*\[Carve\] Line 127: \(case_chest\.carve\)/.test(l))
  t.check('load warns that close inventory does nothing here',
    warnAt >= 0 && /does nothing on this server/.test(srv.lines[warnAt + 1] || ''), srv.lines[warnAt + 1] || 'none')

  const bot = await joinBot('Tester', { port: t.port })
  t.cleanup(() => bot.quit())
  await sleep(1500)
  let since = now()
  srv.console('gamemode creative Tester')
  await bot.waitPacket('game_event', (d) => d.gameMode === 'creative', 4000, since)
  since = now()
  srv.console('tp Tester 0.5 64 0.5')
  await bot.waitFor((e) => e.kind === 'position' && e.text === '0.5 64 0.5', 4000, since)
  await sleep(300)
  // A chunk may still be loading after a teleport; retry until setblock works.
  const setblock = async (x, y, z, block) => {
    for (let i = 0; i < 40; i++) {
      const m = srv.mark()
      srv.console(`setblock ${x} ${y} ${z} minecraft:${block}`)
      const r = await srv.waitLog(/Changed the block|not loaded|Could not set/, 2000, m)
      if (r && !/not loaded/.test(r)) return true
      await sleep(250)
    }
    return false
  }
  t.check('setblock chest', await setblock(2, 64, 0, 'chest'))
  await setblock(2, 65, 0, 'air')
  await sleep(300)
  const isCase = (d) => d.type === 'generic_9x3' && /Case Opening/.test(d.title || '')

  // ---- sneak + right click: the plain chest, no case ----
  bot.sneak(true)
  await sleep(300)
  since = now()
  bot.useOn(2, 64, 0, 4)
  const ps = await bot.waitText('probe', { since })
  t.check('sneaking: player is sneaking', !!ps && ps.text.includes('sneak=yes'), ps ? ps.text : 'none')
  t.check('sneaking: no case GUI', !await bot.waitPacket('open_screen', isCase, 1500, since))
  if (bot.window.id) bot.closeContainer()
  bot.sneak(false)
  await sleep(500)

  // ---- right click: the case opens and spins ----
  since = now()
  bot.useOn(2, 64, 0, 4)
  const p = await bot.waitText('probe', { since })
  t.check('standing: player is not sneaking', !!p && p.text.includes('sneak=no'), p ? p.text : 'none')
  const open = await bot.waitPacket('open_screen', isCase, 3000, since)
  t.check('case GUI opens: 3 rows, title', !!open, open ? `${open.type} ${open.title}` : 'none')
  const win = open ? open.windowId : -1
  const msg = await bot.waitText('You unboxed:', { since, ms: 15000 })
  t.check('prize message', !!msg, msg ? msg.text : 'none')
  await sleep(500)
  const slots = bot.packets.filter((x) => x.name === 'container_set_slot' && x.t >= since && x.data && x.data.windowId === win).map((x) => x.data)
  const at = (n) => slots.filter((s) => s.slot === n)
  t.check('filler pane keeps its blank name', at(0).some((s) => s.item && s.item.name === 'gray_stained_glass_pane' && nameOf(s.item) === ' '),
    JSON.stringify(at(0)[0] || null).slice(0, 200))
  t.check('marker pane named', at(22).some((s) => s.item && s.item.name === 'red_stained_glass_pane' && nameOf(s.item) === '^'))
  const reel = slots.filter((s) => s.slot >= 9 && s.slot <= 17 && s.item && s.item.name !== 'gray_stained_glass_pane')
  // A slot whose item did not change sends no packet, so allow a few fewer.
  t.check('reel slots spin (31 x 9 updates)', reel.length >= 31 * 9 / 2, `${reel.length}`)
  t.check('every reel item has its name', reel.length > 0 && reel.every((s) => nameOf(s.item)), JSON.stringify(reel.find((s) => !nameOf(s.item)) || null).slice(0, 200))
  const clicks = bot.packets.filter((x) => x.name === 'sound' && x.t >= since && x.data && x.data.sound === 'ui.button.click')
  t.check('click sound per spin', clicks.length >= 31, `${clicks.length}`)
  t.check('landing sound', !!await bot.waitPacket('sound', (d) => d.sound === 'entity.player.levelup', 1000, since))
  // The prize: the item in the middle of the reel at the end, given with its name.
  const won = at(13).filter((s) => s.item).pop()
  const wonName = won ? nameOf(won.item) : null
  const given = bot.packets.find((x) => x.t >= since && x.data && x.data.item && ['set_player_inventory', 'container_set_slot'].includes(x.name) &&
    !(x.name === 'container_set_slot' && x.data.windowId === win && x.data.slot < 27) &&
    won && x.data.item.name === won.item.name && nameOf(x.data.item) === wonName)
  t.check('prize given with its name', !!given, `${won && won.item.name} ${wonName}`)
  t.check('message names the prize', !!msg && !!wonName && msg.text.includes(wonName), msg ? msg.text : 'none')
  t.check('slot 4 shows the rarity', at(4).some((s) => s.item && /stained_glass_pane/.test(s.item.name) && s.item.name !== 'red_stained_glass_pane'))

  // `close {_p}'s inventory` 2 s later: Pumpkin's plugin API v0.1 has no call for it.
  const closed = await bot.waitPacket('container_close', (d) => d.windowId === win, 3000, since)
  if (closed) t.check('case GUI closes', true)
  else t.skip('case GUI closes', 'Pumpkin plugin API v0.1 has no call to close a window (load warning checked above)')

  // ---- the player closes it; on inventory close clears the guard, so it opens again ----
  if (bot.window.id === win) bot.closeContainer(win)
  await sleep(500)
  since = now()
  bot.useOn(2, 64, 0, 4)
  t.check('case opens again after closing', !!await bot.waitPacket('open_screen', isCase, 3000, since))
  const bad = srv.lines.find((l) => /\[Carve\].*(ERROR|runtime error)/i.test(l))
  t.check('no runtime errors', !bad, bad || '')
})
