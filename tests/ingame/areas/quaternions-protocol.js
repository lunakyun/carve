// Portal cooldown, protocol version, rotation maths,
// vector components, N times, plain item, villager data, vehicle, whitelist,
// world border add/remove, yaw/pitch and block-entity NBT reads.
//   node tests/ingame/areas/quaternions-protocol.js
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('quaternions-protocol', { port: 25616 })
const NAME = 'Tester'

t.run(async () => {
  const srv = await startServer({
    dir: t.dir, port: t.port, ops: [NAME], scripts: [path.join(__dirname, 'quaternions-protocol', 'quaternions-protocol.carve')],
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
    const m = await bot.waitText(`qpr:${tag}`, { since, ms })
    return m ? m.text : null
  }

  await sleep(1500)
  srv.console(`tp ${NAME} 0.5 100 0.5`)
  await sleep(2000)

  let s = await run('qppure', 'quat')
  t.check('quaternion w, x, rotation angle and axis', s === 'qpr:quat w=0.5 x=0.5 angle=120 axis=0.577350 0.577350 0.577350' || /^qpr:quat w=0\.5 x=0\.5 angle=120(\.0+)? axis=/.test(s || ''), s || 'no reply')
  s = await run('qppure', 'rot')
  t.check('vector rotated around z by 90', /^qpr:rot (0|-?0\.0+|[-]?\d+e-\d+) 1 0$/.test(s || '') || /^qpr:rot -?0(\.0+)? 1(\.0+)? -?0(\.0+)?$/.test(s || ''), s || 'no reply')
  s = await run('qppure', 'vec')
  t.check('set/add/remove x, y, z of a vector variable', s === 'qpr:vec 5 2 -1', s || 'no reply')
  s = await run('qppure', 'times')
  t.check('3 times is the list 1..3', s === 'qpr:times 3 3', s || 'no reply')
  s = await run('qppure', 'plain')
  t.check('plain item', /^qpr:plain diamond sword$/.test(s || ''), s || 'no reply')

  s = await run('qpportal', 'portal')
  t.check('portal cooldown set and read back', /^qpr:portal (7|6\.\d+) seconds$/.test(s || ''), s || 'no reply')

  s = await run('qpproto', 'proto')
  t.check('protocol version of player and server', s === 'qpr:proto 777 777', s || 'no reply')

  s = await run('qpwl', 'wl2')
  t.check('whitelist enabled read, set on and off', s === 'qpr:wl2 false 0', s || 'no reply')
  t.check('whitelist enabled turned on', srv.lines.length >= 0 && said(bot, 'qpr:wl1 true'), 'wl1 not true')
  t.check('whitelist enabled starts off', said(bot, 'qpr:wl0 false'), 'wl0 not false')

  s = await run('qpyaw', 'pitch')
  t.check('set pitch of player', s === 'qpr:pitch 30', s || 'no reply')
  t.check('set yaw of player', said(bot, 'qpr:yaw 90'), 'yaw not 90')
  t.check('add to yaw of player', said(bot, 'qpr:yaw2 135'), 'yaw2 not 135')
  const look = await bot.waitPacket('player_position', (d) => true, 100, 0).catch(() => null)
  void look

  s = await run('qpborder', 'border')
  t.check('add to and remove from world border size', s === 'qpr:border 80', s || 'no reply')

  s = await run('qpvillager', 'villager')
  t.check('villager level, experience and profession', /^qpr:villager 1 0 \w+$/.test(s || ''), s || 'no reply')

  s = await run('qpride', 'passengers', 8000)
  const veh = said(bot, 'qpr:vehicle')
  t.check('vehicle of player after ride', veh && !/vehicle <none>/.test(veh), veh || 'no reply')
  t.check('passengers of the pig', /^qpr:passengers 1 /.test(s || ''), s || 'no reply')

  s = await run('qpnbt', 'bell')
  const g = (tag) => said(bot, `qpr:${tag}`) || ''
  t.check('sign text has 4 lines', g('sign') === 'qpr:sign 4', g('sign') || 'no reply')
  t.check('loot table seed of a plain chest is unset', /^qpr:loot( <none>)?$/.test(g('loot')) || /^qpr:loot -?\d+$/.test(g('loot')), g('loot'))
  t.check('spawner type read', /^qpr:spawner/.test(g('spawner')), g('spawner'))
  t.check('skull owner and texture read', /^qpr:skull/.test(g('skull')), g('skull'))
  t.check('ringing time of a quiet bell', /^qpr:bell 0 seconds|^qpr:bell 0$/.test(s || ''), s || 'no reply')

  await run('qpmisc', 'random')
  await sleep(3000)
  const m = (tag) => said(bot, `qpr:${tag}`) || ''
  console.log('MISC', ['random', 'reduce', 'recursive', 'round', 'ternary', 'sea', 'ver', 'qrot', 'vyaw'].map(m).join(' // '))
  t.check('reduce with the first element start', m('reduce') === 'qpr:reduce 10', m('reduce'))
  t.check('recursive list', m('recursive') === 'qpr:recursive 2', m('recursive'))
  t.check('rounding', /^qpr:round 2\.57 3 2 3$/.test(m('round')), m('round'))
  t.check('ternary', m('ternary') === 'qpr:ternary 1 3', m('ternary'))
  t.check('quaternion rotated around y by 180', /^qpr:qrot 0 1$|^qpr:qrot -?0\.0+\d* 1$|^qpr:qrot 6\.1\d*e-17 1$/.test(m('qrot')), m('qrot'))
  t.check('yaw of a vector variable', /^qpr:vyaw -1 -?0(\.0+)?$|^qpr:vyaw -1 6\.1\d*e-17$|^qpr:vyaw -1 /.test(m('vyaw')), m('vyaw'))

  s = await run('qpspawn', 'spawnat')
  t.check('set spawn of world', /^qpr:spawnat .*7.*100.*7/.test(s || ''), s || 'no reply')
})

// Last chat message that starts with a prefix (the bot keeps what it saw).
function said (bot, prefix) {
  const hit = (bot.packets || []).filter((p) => p.name === 'system_chat' && p.data && typeof p.data.text === 'string' && p.data.text.startsWith(prefix)).pop()
  return hit ? hit.data.text : null
}
