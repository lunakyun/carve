// Wave 6 chunk 7 area: variadic min/max, round places, world(), entity(uuid),
// axisAngle, for each, alias patterns/plurals, suppress and using lines.
//   node tests/ingame/areas/w6-chunk7.js
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('w6-chunk7', { port: 25617 })
const NAME = 'Tester'

t.run(async () => {
  const srv = await startServer({
    dir: t.dir, port: t.port, ops: [NAME], scripts: [path.join(__dirname, 'w6-chunk7', 'w6-chunk7.carve')],
  })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop()))
  t.check('carve loads the area script', !!await srv.waitLog(/\[Carve\] Loaded 1 script/, 5000))
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
  t.check('axisAngle quaternions come out', !!s && /0\.7071/.test(s), s || 'no reply')
  s = await run('c7each', 'each')
  t.check('for each sets the loop variable', s === 'c7:each 1,2,3,', s || 'no reply')
  s = await run('c7alias', 'alias')
  t.check('alias pattern and plural give items', s === 'c7:alias', s || 'no reply')
  s = await run('c7suppress', 'suppress')
  t.check('locally suppress line runs', s === 'c7:suppress', s || 'no reply')
})
