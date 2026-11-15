// w6-chunk5 area: parsed as integer/boolean/player, formatted human-readable,
// key of (items, loot tables), loot contexts, max stack size from the server,
// item values with components, and checks of rows that were wrongly marked
// blocked (hex code, memory, id, highest solid block, motd).
//   node tests/ingame/areas/w6-chunk5.js
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const now = () => Date.now() + 1
const t = area('w6-chunk5', { port: 25615 })
const SCRIPTS = [path.join(__dirname, 'w6-chunk5', 'w6-chunk5.carve')]
const NAME = 'Tester'

t.run(async () => {
  const srv = await startServer({ dir: t.dir, port: t.port, scripts: SCRIPTS, ops: [NAME] })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop(60000)))
  const loaded = await srv.waitLog(/\[Carve\] Loaded 1 script/, 5000)
  const loadErr = srv.lines.filter((l) => /\[Carve\].*(error|Error)|w6-chunk5\.carve:\d+/.test(l)).slice(-4).join(' | ')
  t.check('script loads without errors', !!loaded && !loadErr, loadErr)

  const bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  await sleep(1500)
  const run = async (cmd, prefix, ms = 6000) => {
    const since = now()
    bot.command(cmd)
    const m = await bot.waitText(prefix, { since, ms })
    return { text: m ? m.text : null, since }
  }

  let r = await run('w6parse', 'pa:')
  t.check('Parse: integer, boolean, player', r.text === `pa:12|<none>|true|<none>|${NAME}|<none>`, r.text)
  r = await run('w6fmt', 'fm:')
  t.check('Formatted Time: human-readable and clock', r.text === 'fm:1 hour and 5 minutes|1:05:00', r.text)
  r = await run('w6key', 'ky:')
  t.check('Keyed: item and loot table keys', r.text === 'ky:minecraft:diamond|minecraft:chests/simple_dungeon|mymod:stuff', r.text)
  r = await run('w6loot', 'lc:')
  t.check('Loot Context + location/luck/looter/looted entity', r.text !== null && /^lc:.*1.*64.*2\|3\|Tester\|<none>$/.test(r.text), r.text)
  r = await run('w6stack', 'st:')
  t.check('Max Stack Size read from the server', r.text === 'st:16|1|64|16', r.text)

  const slotItem = (name, since) => bot.waitPacket('container_set_slot', (d) => d.item && d.item.name === name, 5000, since)
    .then((p) => p || bot.waitPacket('set_player_inventory', (d) => d.item && d.item.name === name, 1000, since))
  const withComp = (item, comp, since) => bot.waitPacket('container_set_slot', (d) => d.item && d.item.name === item && d.item.components && d.item.components[comp] !== undefined, 5000, since)
    .then((p) => p || bot.waitPacket('set_player_inventory', (d) => d.item && d.item.name === item && d.item.components && d.item.components[comp] !== undefined, 1000, since))
  r = await run('w6item', 'it:')
  const sw = await withComp('diamond_sword', 'custom_model_data', r.since)
  t.check('Item with Custom Model Data: client item carries custom_model_data 7', !!sw && sw.item.components.custom_model_data.floats[0] === 7, sw ? JSON.stringify(sw.item.components) : r.text)
  r = await run('w6item2', 'it2:')
  const g = await withComp('stick', 'enchantment_glint_override', r.since)
  t.check('Item with Enchantment Glint: component true on the client item', !!g && g.item.components.enchantment_glint_override === true, g ? JSON.stringify(g.item.components) : r.text)
  const m = await withComp('stick', 'item_model', r.since)
  t.check('Item with Model: item_model component on the client item', !!m && /blaze_rod/.test(JSON.stringify(m.item.components.item_model)), m ? JSON.stringify(m.item.components) : r.text)
  r = await run('w6misc', 'hx:')
  t.check('Hex Code of a color name and #rrggbb', r.text === 'hx:FF5555|00FF7F', r.text)
  const rest = []
  for (const p of ['id:', 'hs:', 'mo:']) {
    const m = await bot.waitText(p, { since: r.since, ms: 5000 })
    rest.push(m ? m.text : null)
  }
  t.check('Id of an item', rest[0] === 'id:minecraft:diamond', rest[0])
  t.check('Highest Solid Block', /^hs:/.test(rest[1] || '') && !/<none>/.test(rest[1] || ''), rest[1])
  t.check('Message of the Day is readable', !!rest[2] && rest[2].length > 4 && !/<none>/.test(rest[2]), rest[2])
})
