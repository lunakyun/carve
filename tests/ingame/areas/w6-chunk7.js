// Wave 6 chunk 7 area: variadic min/max, round places, world(), entity(uuid),
// axisAngle, for each, alias patterns/plurals, suppress and using lines.
//   node tests/ingame/areas/w6-chunk7.js
'use strict'
const fs = require('fs')
const path = require('path')
const zlib = require('zlib')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')
const { statusPing } = require('../lib/status.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('w6-chunk7', { port: 25617 })
const NAME = 'Tester'

// A real 64x64 PNG (one gray color), so the icon check reads a valid file.
function png64 () {
  const crcTable = []
  for (let n = 0; n < 256; n++) { let c = n; for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1; crcTable[n] = c >>> 0 }
  const crc = (b) => { let c = 0xffffffff; for (const x of b) c = crcTable[(c ^ x) & 0xff] ^ (c >>> 8); return (c ^ 0xffffffff) >>> 0 }
  const chunk = (type, data) => {
    const len = Buffer.alloc(4); len.writeUInt32BE(data.length)
    const td = Buffer.concat([Buffer.from(type), data])
    const c = Buffer.alloc(4); c.writeUInt32BE(crc(td))
    return Buffer.concat([len, td, c])
  }
  const ihdr = Buffer.alloc(13); ihdr.writeUInt32BE(64, 0); ihdr.writeUInt32BE(64, 4); ihdr[8] = 8; ihdr[9] = 0
  const raw = Buffer.alloc(64 * 65, 0x80); for (let y = 0; y < 64; y++) raw[y * 65] = 0
  return Buffer.concat([Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]), chunk('IHDR', ihdr), chunk('IDAT', zlib.deflateSync(raw)), chunk('IEND', Buffer.alloc(0))])
}

t.run(async () => {
  const icon = png64()
  const carveData = path.join(t.dir, 'plugins', 'data', 'carve')
  fs.mkdirSync(carveData, { recursive: true })
  fs.writeFileSync(path.join(carveData, 'icon.png'), icon)
  const srv = await startServer({
    dir: t.dir, port: t.port, ops: [NAME], scripts: [path.join(__dirname, 'w6-chunk7', 'w6-chunk7.carve'), path.join(__dirname, 'w6-chunk7', 'w6-auto.carve')],
  })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop()))
  t.check('carve loads the area script', !!await srv.waitLog(/\[Carve\] Loaded 2 script/, 5000))
  const errLine = srv.lines.find((l) => /\[Carve\].*(error|line \d+)/i.test(l))
  t.check('area script has no load errors', !errLine, errLine || '')

  const bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  const run = async (name, tag = name.slice(3), ms = 5000) => {
    const since = Date.now()
    bot.command(name)
    const m = await bot.waitText(`c7:${tag}`, { since, ms })
    return m ? m.text : null
  }
  await sleep(1500)
  srv.console(`tp ${NAME} 0.5 100 0.5`)
  await sleep(2000)

  let s = await run('c7math', 'math')
  t.check('max/min with many args, round places', s === 'c7:math 9 2 3.14 3 7', s || 'no reply')
  s = await run('c7world', 'world')
  t.check('world("name") call form', s === 'c7:world world', s || 'no reply')
  s = await run('c7entity', 'entity')
  t.check('entity(uuid) and entity from uuid find a live pig', s === 'c7:entity pig pig', s || 'no reply')
  s = await run('c7player', 'player')
  t.check('entity(uuid) of a player gives the player', s === `c7:player ${NAME}`, s || 'no reply')
  s = await run('c7axis', 'axis')
  t.check('axisAngle and quaternion() come out', !!s && /w: 0, x: 0, y: 1, z: 0 w: 0\.7071, x: 0\.7071, y: 0, z: 0 w: 1, x: 0, y: 2, z: 0$/.test(s), s || 'no reply')
  s = await run('c7each', 'each')
  t.check('for each sets the loop variable', s === 'c7:each 1,2,3,', s || 'no reply')
  s = await run('c7alias', 'alias')
  t.check('alias pattern and plural give items', s === 'c7:alias', s || 'no reply')
  s = await run('c7suppress', 'suppress')
  t.check('locally suppress line runs', s === 'c7:suppress', s || 'no reply')

  s = await run('c7types', 'types')
  t.check('abstract and loose types work as function parameter types', s === 'c7:types types-ok', s || 'no reply')

  const mark = srv.mark()
  s = await run('c7team', 'team')
  t.check('death message visibility of a team reads back', s === 'c7:team hide for other teams', s || 'no reply')
  const dm = await srv.waitLog(/deathmessagevisibility|death message visibility/i, 3000, mark)
  t.check('death message visibility reached /team modify', !!dm, dm || 'no server log line')

  // ---- server icon (type Server Icon) ----
  await run('c7icon', 'icon')
  const st = await statusPing(t.port).catch((e) => ({ error: String(e) }))
  const want = 'data:image/png;base64,' + icon.toString('base64')
  t.check('server icon loaded and sent in the ping', st.favicon === want, st.error || (st.favicon ? st.favicon.slice(0, 40) : 'no favicon'))

  // ---- auto reload ----
  s = await run('c7auto', 'auto')
  t.check('auto reload script starts at v1', s === 'c7:auto v1', s || 'no reply')
  await sleep(4500) // the first look at the folder is the baseline
  const file = path.join(carveData, 'scripts', 'w6-auto.carve')
  fs.writeFileSync(file, fs.readFileSync(file, 'utf8').replace('v1', 'v2'))
  const reloaded = await srv.waitLog(/auto reload: w6-auto.carve changed/, 12000)
  t.check('auto reload noticed the changed file', !!reloaded)
  await sleep(500)
  s = await run('c7auto', 'auto')
  t.check('auto reload ran the new version', s === 'c7:auto v2', s || 'no reply')
  // A script without the line is left alone.
  const other = path.join(carveData, 'scripts', 'w6-chunk7.carve')
  fs.appendFileSync(other, '\n# touched\n')
  await sleep(5000)
  t.check('files without `auto reload` are not reloaded', !srv.lines.some((l) => /auto reload: w6-chunk7.carve/.test(l)))
})
