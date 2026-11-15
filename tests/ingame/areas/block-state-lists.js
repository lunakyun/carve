// Wave 6 chunk 4 area: element ranges, dequeue, banner pattern items,
// persistent data keys, honey level, brushing stage and the age of a block.
//   node tests/ingame/areas/block-state-lists.js
'use strict'
const fs = require('fs')
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('block-state-lists', { port: 25614 })
const NAME = 'Tester'

// First state id of a block, from Pumpkin's blocks.json when it is there.
function blockFirst (name, fallback) {
  try {
    const all = JSON.parse(fs.readFileSync(path.join(process.env.PUMPKIN_REF_DIR, 'blocks.json'), 'utf8')).blocks
    return all.find((x) => x.name === name).states[0].id
  } catch (e) {
    return fallback
  }
}

t.run(async () => {
  const srv = await startServer({
    dir: t.dir, port: t.port, ops: [NAME], scripts: [path.join(__dirname, 'block-state-lists', 'block-state-lists.carve')],
  })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop()))
  t.check('carve loads the area script', !!await srv.waitLog(/\[Carve\] Loaded 1 script/, 5000))
  const errLine = srv.lines.find((l) => /\[Carve\].*(error|line \d+)/i.test(l))
  t.check('area script has no load errors', !errLine, errLine || '')

  const bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  bot.on('packet', (p) => { if (p.name === 'disconnect' && p.data) console.log(`kicked: ${p.data.text}`) })

  const run = async (name, tag = name.slice(2), ms = 5000) => {
    const since = Date.now()
    bot.command(name)
    const m = await bot.waitText(`bsl:${tag}`, { since, ms })
    return m ? m.text : null
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
  t.check('/bslpad', !!await run('bslpad', 'pad'))

  // ---- pure logic through command messages ----
  let s = await run('bslelements', 'elements')
  t.check('first/last N elements and elements N to M', s === 'bsl:elements 1 and 2|4 and 5|2, 3 and 4|3', s || 'no reply')

  s = await run('bslqueue', 'queue')
  t.check('dequeue pops the front, N elements, empty queue gives none', s === 'bsl:queue a bc d <none>', s || 'no reply')

  let since = Date.now()
  s = await run('bslbanner', 'banner')
  t.check('banner pattern item of a pattern type', s === 'bsl:banner creeper banner pattern <none>', s || 'no reply')
  const slot = await bot.waitPacket('container_set_slot', (d) => d.item && d.item.name === 'flower_banner_pattern', 4000, since)
  t.check('item of banner pattern item reaches the inventory', !!slot, slot ? '' : 'no flower_banner_pattern slot packet')

  s = await run('bslkeys', 'keys')
  t.check('persistent data keys of a player, sorted', s === 'bsl:keys bslalpha and bslzeta', s || 'no reply')
  s = await run('bslkeys', 'keys2')
  t.check('persistent data keys are empty after delete', s === 'bsl:keys2 0', s || 'no reply')

  // ---- block properties against the real server ----
  const hive = blockFirst('beehive', 25149)
  since = Date.now()
  s = await run('bslhoney', 'honey0')
  t.check('honey level of a new beehive and its max', s === 'bsl:honey0 0 5', s || 'no reply')
  t.check('add past the max clamps to 5', !!await bot.waitText('bsl:honey1 5', { since, ms: 3000 }))
  t.check('remove 2 from honey level', !!await bot.waitText('bsl:honey2 3', { since, ms: 3000 }))
  t.check('reset honey level', !!await bot.waitText('bsl:honey3 0', { since, ms: 3000 }))
  // The block ends at honey level 5 (north facing): state first + 5.
  const hb = await waitBlock(6, 100, 6, 4000, since, hive + 5)
  t.check('honey level block state sent', !!hb, hb ? '' : `no block update with state ${hive + 5}`)

  const sand = blockFirst('suspicious_sand', 122)
  since = Date.now()
  s = await run('bslbrush', 'brush0')
  t.check('brushing stage of a fresh block and max', s === 'bsl:brush0 0 3', s || 'no reply')
  t.check('set brushing stage', !!await bot.waitText('bsl:brush1 2', { since, ms: 3000 }))
  const bb = await waitBlock(7, 100, 6, 4000, since, sand + 2)
  t.check('brushing stage block state sent', !!bb, bb ? '' : `no block update with state ${sand + 2}`)

  const wheat = blockFirst('wheat', 6867)
  since = Date.now()
  s = await run('bslage', 'age0')
  t.check('age of a wheat block and max age', s === 'bsl:age0 0 7', s || 'no reply')
  t.check('add to age', !!await bot.waitText('bsl:age1 3', { since, ms: 3000 }))
  t.check('set age past max clamps', !!await bot.waitText('bsl:age2 7', { since, ms: 3000 }))
  t.check('a block without age gives none', !!await bot.waitText('bsl:age3 <none>', { since, ms: 3000 }))
  const ab = await waitBlock(8, 100, 6, 4000, since, wheat + 7)
  t.check('age block state sent', !!ab, ab ? '' : `no block update with state ${wheat + 7}`)
})
