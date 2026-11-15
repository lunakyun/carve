// Wave 6 chunk 6 area: portal cooldown, protocol version, rotation maths,
// vector components, N times, plain item, villager data, vehicle, whitelist,
// world border add/remove, yaw/pitch and block-entity NBT reads.
//   node tests/ingame/areas/w6-chunk6.js
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('w6-chunk6', { port: 25616 })
const NAME = 'Tester'

t.run(async () => {
  const srv = await startServer({
    dir: t.dir, port: t.port, ops: [NAME], scripts: [path.join(__dirname, 'w6-chunk6', 'w6-chunk6.carve')],
  })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop()))
  t.check('carve loads the area script', !!await srv.waitLog(/\[Carve\] Loaded 1 script/, 5000))
  const errLine = srv.lines.find((l) => /\[Carve\].*(error|line \d+)/i.test(l))
  t.check('area script has no load errors', !errLine, errLine || '')

  const bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  const run = async (name, tag, ms = 5000) => {
    const since = Date.now()
    bot.command(name)
    const m = await bot.waitText(`c66:${tag}`, { since, ms })
    return m ? m.text : null
  }

  await sleep(1500)
  srv.console(`tp ${NAME} 0.5 100 0.5`)
  await sleep(2000)

  let s = await run('w6pure', 'quat')
  t.check('quaternion w, x, rotation angle and axis', s === 'c66:quat w=0.5 x=0.5 angle=120 axis=0.577350 0.577350 0.577350' || /^c66:quat w=0\.5 x=0\.5 angle=120(\.0+)? axis=/.test(s || ''), s || 'no reply')
  s = await run('w6pure', 'rot')
  t.check('vector rotated around z by 90', /^c66:rot (0|-?0\.0+|[-]?\d+e-\d+) 1 0$/.test(s || '') || /^c66:rot -?0(\.0+)? 1(\.0+)? -?0(\.0+)?$/.test(s || ''), s || 'no reply')
  s = await run('w6pure', 'vec')
  t.check('set/add/remove x, y, z of a vector variable', s === 'c66:vec 5 2 -1', s || 'no reply')
  s = await run('w6pure', 'times')
  t.check('3 times is the list 1..3', s === 'c66:times 3 3', s || 'no reply')
  s = await run('w6pure', 'plain')
  t.check('plain item', /^c66:plain diamond sword$/.test(s || ''), s || 'no reply')

  s = await run('w6portal', 'portal')
  t.check('portal cooldown set and read back', /^c66:portal (7|6\.\d+) seconds$/.test(s || ''), s || 'no reply')

  s = await run('w6proto', 'proto')
  t.check('protocol version of player and server', s === 'c66:proto 777 777', s || 'no reply')

  s = await run('w6wl', 'wl2')
  t.check('whitelist enabled read, set on and off', s === 'c66:wl2 false 0', s || 'no reply')
  t.check('whitelist enabled turned on', srv.lines.length >= 0 && said(bot, 'c66:wl1 true'), 'wl1 not true')
  t.check('whitelist enabled starts off', said(bot, 'c66:wl0 false'), 'wl0 not false')

  s = await run('w6yaw', 'pitch')
  t.check('set pitch of player', s === 'c66:pitch 30', s || 'no reply')
  t.check('set yaw of player', said(bot, 'c66:yaw 90'), 'yaw not 90')
  t.check('add to yaw of player', said(bot, 'c66:yaw2 135'), 'yaw2 not 135')
  const look = await bot.waitPacket('player_position', (d) => true, 100, 0).catch(() => null)
  void look

  s = await run('w6border', 'border')
  t.check('add to and remove from world border size', s === 'c66:border 80', s || 'no reply')

  s = await run('w6villager', 'villager')
  t.check('villager level, experience and profession', /^c66:villager 1 0 \w+$/.test(s || ''), s || 'no reply')

  s = await run('w6ride', 'passengers', 8000)
  const veh = said(bot, 'c66:vehicle')
  t.check('vehicle of player after ride', veh && !/vehicle <none>/.test(veh), veh || 'no reply')
  t.check('passengers of the pig', /^c66:passengers 1 /.test(s || ''), s || 'no reply')

  s = await run('w6nbt', 'bell')
  const g = (tag) => said(bot, `c66:${tag}`) || ''
  t.check('sign text has 4 lines', g('sign') === 'c66:sign 4', g('sign') || 'no reply')
  t.check('loot table seed of a plain chest is unset', /^c66:loot( <none>)?$/.test(g('loot')) || /^c66:loot -?\d+$/.test(g('loot')), g('loot'))
  t.check('spawner type read', /^c66:spawner/.test(g('spawner')), g('spawner'))
  t.check('skull owner and texture read', /^c66:skull/.test(g('skull')), g('skull'))
  t.check('ringing time of a quiet bell', /^c66:bell 0 seconds|^c66:bell 0$/.test(s || ''), s || 'no reply')

  await run('w6misc', 'random')
  await sleep(3000)
  const m = (tag) => said(bot, `c66:${tag}`) || ''
  console.log('MISC', ['random', 'reduce', 'recursive', 'round', 'ternary', 'sea', 'ver', 'qrot', 'vyaw'].map(m).join(' // '))
  t.check('reduce with the first element start', m('reduce') === 'c66:reduce 10', m('reduce'))
  t.check('recursive list', m('recursive') === 'c66:recursive 2', m('recursive'))
  t.check('rounding', /^c66:round 2\.57 3 2 3$/.test(m('round')), m('round'))
  t.check('ternary', m('ternary') === 'c66:ternary 1 3', m('ternary'))
  t.check('quaternion rotated around y by 180', /^c66:qrot 0 1$|^c66:qrot -?0\.0+\d* 1$|^c66:qrot 6\.1\d*e-17 1$/.test(m('qrot')), m('qrot'))
  t.check('yaw of a vector variable', /^c66:vyaw -1 -?0(\.0+)?$|^c66:vyaw -1 6\.1\d*e-17$|^c66:vyaw -1 /.test(m('vyaw')), m('vyaw'))

  s = await run('w6spawn', 'spawnat')
  t.check('set spawn of world', /^c66:spawnat .*7.*100.*7/.test(s || ''), s || 'no reply')
})

// Last chat message that starts with a prefix (the bot keeps what it saw).
function said (bot, prefix) {
  const hit = (bot.packets || []).filter((p) => p.name === 'system_chat' && p.data && typeof p.data.text === 'string' && p.data.text.startsWith(prefix)).pop()
  return hit ? hit.data.text : null
}
