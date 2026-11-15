// Syntax from popular public scripts: command argument types and defaults,
// permission message, message, player and console command, slot lists, offline
// players and the small conditions around them.
//   node tests/ingame/areas/popular.js
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('popular', { port: 25680 })
const NAME = 'Tester'

t.run(async () => {
  const srv = await startServer({
    dir: t.dir, port: t.port, ops: [], scripts: [path.join(__dirname, 'popular', 'popular.carve')],
  })
  let stopped = false
  t.cleanup(async () => { if (!stopped) await srv.stop(60000) })
  t.check('carve loads the script', !!await srv.waitLog(/\[Carve\] Loaded 1 script/, 5000))
  const errLine = srv.lines.find((l) => /\[Carve\].*(error|line \d+)/i.test(l))
  t.check('script has no load errors', !errLine, errLine || '')

  const bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  const said = []
  bot.on('packet', (p) => { if (p.name === 'system_chat' && p.data && /pop:/.test(JSON.stringify(p.data))) said.push({ at: Date.now(), text: JSON.stringify(p.data) }) })
  const since = (s, re, ms = 4000) => new Promise((resolve) => {
    const end = Date.now() + ms
    const tick = () => {
      const hit = said.find((x) => x.at >= s && re.test(x.text))
      if (hit) return resolve(hit.text)
      if (Date.now() > end) return resolve(null)
      setTimeout(tick, 50)
    }
    tick()
  })
  const none = async (s, re, ms = 1200) => { await sleep(ms); return !said.some((x) => x.at >= s && re.test(x.text)) }

  await sleep(3000)
  srv.console(`gamemode survival ${NAME}`)
  await sleep(500)

  // ---- command arguments and defaults ----
  let s = Date.now()
  bot.command('pop-args Alex yes')
  t.check('offline player, boolean and defaults', !!await since(s, /pop:args Alex true 5 hi/))
  s = Date.now()
  bot.command('pop-args Alex off 9 hello there')
  t.check('given values replace the defaults', !!await since(s, /pop:args Alex false 9 hello there/))
  s = Date.now()
  bot.command('pop-args Alex maybe')
  t.check('a bad boolean is refused', await none(s, /pop:args/))
  s = Date.now()
  bot.command('pop-me')
  t.check('<player=%player%> defaults to the sender', !!await since(s, new RegExp(`pop:me ${NAME}`)))

  // ---- permission message ----
  s = Date.now()
  bot.command('pop-perm')
  t.check('permission message is sent', !!await since(s, /pop:denied/))
  t.check('the command did not run', await none(s, /pop:perm-ran/, 500))

  // ---- player and console commands, exit trigger ----
  s = Date.now()
  bot.command('pop-cmds')
  t.check('message effect', !!await since(s, /pop:cmds-done/))
  t.check('player command runs as the player', !!await since(s, new RegExp(`pop:me ${NAME}`)))
  t.check('execute player command with an offline player', !!await since(s, /pop:args Zed true 5 hi/))
  t.check('send to all players', !!await since(s, /pop:all/))
  t.check('exit trigger stops the trigger', await none(s, /pop:never/, 500))
  t.check('console command runs', !!await srv.waitLog(/pop:ping console/, 3000))

  // ---- chest with slots ----
  s = Date.now()
  bot.command('pop-gui')
  t.check('set slots and open the menu', !!await since(s, /pop:gui-open/))
  await sleep(800)
  const open = bot.screen !== undefined ? bot.screen : null
  t.check('the menu opened on the client', open !== null && open !== 0 || bot.screen === undefined, `screen ${open}`)

  // ---- offline players loop ----
  s = Date.now()
  bot.command('pop-offline')
  const off = await since(s, /pop:offline (\d+)/)
  t.check('loop all offline players includes who joined', !!off && /pop:offline [1-9]/.test(off), off || 'no reply')

  // ---- misc ----
  s = Date.now()
  bot.command('pop-misc')
  t.check('misc finishes', !!await since(s, /pop:misc-done/))
  t.check('random integer from..to stays in range', await none(s, /pop:bad-random/, 300))
  t.check('was less than a minute ago', !!await since(s, /pop:recent/, 500))
  t.check('difference between and less than', !!await since(s, /pop:close/, 500))
  t.check("player doesn't have permission", !!await since(s, /pop:noperm/, 500))
  t.check('world is "world"', !!await since(s, /pop:world/, 500))
  t.check('mod operator', !!await since(s, /pop:odd/, 500))
  t.check('exit 2 loops leaves both loops', !!await since(s, /pop:in 1/, 500) && await none(s, /pop:in 2/, 300))

  bot.quit()
  stopped = await srv.stop(60000)
  t.check('server stops cleanly', stopped)
  stopped = true
})
