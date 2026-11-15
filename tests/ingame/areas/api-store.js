// api-store area: actions that used to be console commands now go through the
// plugin API (sounds, kick, ban, op, give, riding, titles, poison, teams), and
// everything Carve keeps (variables, bounds, first joins) lives in one store,
// carve.redb, that survives a restart and takes over the three old files.
//   node tests/ingame/areas/api-store.js
'use strict'
const fs = require('fs')
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer, offlineUuid } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const now = () => Date.now() + 1
const t = area('api-store', { port: 25650 })
const SCRIPT = path.join(__dirname, 'api-store', 'api-store.carve')
const scriptLines = fs.readFileSync(SCRIPT, 'utf8').split('\n')
const lineOf = (needle) => scriptLines.findIndex((l) => l.includes(needle)) + 1
const dataDir = path.join(t.dir, 'plugins', 'data', 'carve')
const nameText = (it) => JSON.stringify((it && it.components && it.components.custom_name) || null)

t.run(async () => {
  let srv = await startServer({ dir: t.dir, port: t.port, scripts: [SCRIPT], ops: ['Tester'] })
  let stopped = false
  t.cleanup(async () => { if (!stopped) await srv.stop() })
  t.check('script loads', !!await srv.waitLog(/\[Carve\] Loaded 1 script/, 3000))
  const loadErr = srv.lines.find((l) => /\[Carve\].*(error|Error)/.test(l))
  t.check('no load errors', !loadErr, loadErr || '')
  t.check('the store file is made, no old files', fs.existsSync(path.join(dataDir, 'carve.redb')) &&
    !['variables.json', 'bounds.json', 'first_join.txt'].some((f) => fs.existsSync(path.join(dataDir, f))))

  const bots = []
  const join = async (name) => { const b = await joinBot(name, { port: t.port }); bots.push(b); return b }
  t.cleanup(() => bots.forEach((b) => { try { b.quit() } catch {} }))
  let tester = await join('Tester')
  let pleb = await join('Pleb')
  await sleep(1500)
  // Run a command as `bot` and wait for its reply text.
  const say = async (bot, c, expect, ms = 5000) => {
    const since = now()
    bot.command(c)
    return expect ? bot.waitText(expect, { since, ms }) : null
  }
  const logsFrom = (m) => srv.lines.slice(m)

  // ---- sounds go to the client, with no console echo ----
  let since = now()
  let mark = srv.mark()
  tester.command('as_sound')
  const orb = await tester.waitPacket('sound', (d) => d.sound === 'entity.experience_orb.pickup', 4000, since)
  t.check('play sound: the client gets the sound packet', !!orb, orb ? `${orb.sound} vol ${orb.volume} pitch ${orb.pitch}` : 'none')
  const bass = await tester.waitPacket('sound', (d) => d.sound === 'block.note_block.bass', 4000, since)
  t.check('play sound at a location, volume and pitch', !!bass && Math.abs(bass.x - 10) < 0.2 && Math.abs(bass.z - 20) < 0.2 &&
    bass.volume === 2 && Math.abs(bass.pitch - 0.5) < 0.01, bass ? JSON.stringify(bass) : 'none')
  since = now()
  tester.command('as_stop')
  const stopSound = await tester.waitPacket('stop_sound', () => true, 4000, since)
  t.check('stop sound reaches the client', !!stopSound)
  await sleep(300)
  const chatSince = tester.packets.filter((p) => p.name === 'system_chat').map((p) => (p.data && p.data.text) || '')
  t.check('ops see no "[Server: Played sound" in chat', !chatSince.some((x) => /Played sound|Stopped sound|\[Server/.test(x)), chatSince.find((x) => /Played|Stopped|\[Server/.test(x)) || '')
  t.check('server log has no sound command echo', !logsFrom(mark).some((l) => /Played sound|Stopped (the )?sound|playsound/i.test(l)))

  // ---- an unknown sound warns with the script line, and nothing is played ----
  since = now()
  mark = srv.mark()
  tester.command('as_badsound')
  const after = await tester.waitText('as:after-bad', { since })
  t.check('unknown sound: the script goes on', !!after)
  const want = `api-store.carve:${lineOf('entity.nope.nothing')}: play sound: 'entity.nope.nothing' is not a vanilla sound`
  const warn = logsFrom(mark).find((l) => l.includes(want))
  t.check('unknown sound: warning names the script line', !!warn, warn || logsFrom(mark).filter((l) => /WARN/.test(l)).join(' | '))
  t.check('unknown sound: nothing sent to the client', !tester.packets.some((p) => p.name === 'sound' && p.t >= since && /nope/.test(p.data.sound || '')))

  // ---- give, with a name and lore, no § codes ----
  since = now()
  mark = srv.mark()
  tester.command('as_give')
  await tester.waitText('as:given', { since })
  await sleep(300)
  const inv = tester.packets.filter((p) => p.t >= since && p.data && p.data.item && ['set_player_inventory', 'container_set_slot'].includes(p.name))
  const prize = inv.find((p) => p.data.item.name === 'diamond')
  // Pumpkin keeps only the plain text of a name, so the colour is a § code in it (as with the give command).
  t.check('give: the diamond arrives with its name and colour', !!prize && /§6As Prize/.test(nameText(prize.data.item)), prize ? nameText(prize.data.item).slice(0, 200) : 'none')
  t.check('give: no & codes left in the name', !!prize && !/&/.test(nameText(prize.data.item)), prize ? nameText(prize.data.item).slice(0, 200) : 'none')
  t.check('give: a plain stack of 3 sticks', inv.some((p) => p.data.item.name === 'stick' && p.data.item.count === 3))
  const chatNow = tester.packets.filter((p) => p.name === 'system_chat' && p.t >= since).map((p) => (p.data && p.data.text) || '')
  t.check('give: ops see no "Gave ..." message with § codes in chat', !chatNow.some((x) => /Gave \d|§/.test(x)), chatNow.find((x) => /Gave|§/.test(x)) || '')
  t.check('give: no console command echo', !logsFrom(mark).some((l) => /Gave \d|Given|\bgive\b/i.test(l)))

  // ---- experience ----
  const lv = await say(tester, 'as_level', 'as:level')
  t.check('experience: set 2, give 3, remove 1 = level 4', !!lv && /as:level 4\b/.test(lv.text), lv ? lv.text : 'none')

  // ---- riding ----
  await say(tester, 'as_ride', 'as:ride-sent')
  await sleep(500)
  let r = await say(tester, 'as_riding', 'as:riding=')
  t.check('ride: the player is on the pig', !!r && r.text.includes('as:riding=yes'), r ? r.text : 'none')
  await say(tester, 'as_dismount', 'as:dismount-sent')
  await sleep(500)
  r = await say(tester, 'as_riding', 'as:riding=')
  t.check('dismount: the player is off', !!r && r.text.includes('as:riding=no'), r ? r.text : 'none')

  // ---- clear title ----
  since = now()
  tester.command('as_title')
  const shown = await tester.waitPacket('set_title_text', (d) => /as:title/.test(d.text || ''), 3000, since)
  await tester.waitText('as:title-cleared', { since })
  const cleared = await tester.waitPacket('clear_titles', (d) => d.reset === false, 2000, since)
  t.check('clear title: shown, then the client is told to clear it', !!shown && !!cleared)

  // ---- poison and cure ----
  since = now()
  tester.command('as_poison')
  t.check('poison: the effect reaches the client', !!await tester.waitPacket('update_mob_effect', (d) => d.effect === 'poison' && d.duration >= 90, 4000, since))
  since = now()
  tester.command('as_cure')
  t.check('cure: the effect is removed', !!await tester.waitPacket('remove_mob_effect', (d) => d.effect === 'poison', 4000, since))

  // ---- scoreboard and teams on the server scoreboard ----
  since = now()
  tester.command('as_team')
  const tm = await tester.waitText('as:team', { since })
  t.check('team and score are readable', !!tm && /as:team as_team 7/.test(tm.text), tm ? tm.text : 'none')
  t.check('objective reaches the client', !!await tester.waitPacket('set_objective', (d) => d.objective === 'as_obj' && d.mode === 'add', 2000, since))
  t.check('score reaches the client', !!await tester.waitPacket('set_score', (d) => d.objective === 'as_obj' && d.value === 7, 2000, since))
  t.check('team with its prefix reaches the client', !!await tester.waitPacket('set_player_team', (d) => d.team === 'as_team' && /\[As\]/.test(d.prefix || ''), 2000, since))
  since = now()
  tester.command('as_team_end')
  await tester.waitText('as:team-gone', { since })
  t.check('team removal reaches the client', !!await tester.waitPacket('set_player_team', (d) => d.team === 'as_team' && d.method === 'remove', 2000, since))

  // ---- firework ----
  since = now()
  tester.command('as_firework')
  t.check('firework rocket is spawned', !!await tester.waitPacket('add_entity', (d) => d.type === 'firework_rocket', 3000, since))

  // ---- op and deop ----
  await say(tester, 'as_op Pleb', 'as:opped')
  await sleep(300)
  r = await say(pleb, 'as_isop', 'as:isop=')
  t.check('op: the player is an operator at once', !!r && r.text.includes('as:isop=yes'), r ? r.text : 'none')
  await say(tester, 'as_deop Pleb', 'as:deopped')
  await sleep(300)
  r = await say(pleb, 'as_isop', 'as:isop=')
  t.check('deop: the player is not', !!r && r.text.includes('as:isop=no'), r ? r.text : 'none')

  // ---- kick ----
  since = now()
  mark = srv.mark()
  tester.command('as_kick Pleb')
  const kicked = await pleb.waitText('as:kick-reason', { since, ms: 5000 })
  t.check('kick: the player is told the reason', !!kicked, kicked ? kicked.text : 'none')
  t.check('kick: no console command echo', !logsFrom(mark).some((l) => /\[Server|Kicked Pleb: /.test(l)))
  pleb = await join('Pleb')
  await sleep(500)
  since = now()
  mark = srv.mark()
  tester.command('as_kickoff')
  await sleep(800)
  const off = `api-store.carve:${lineOf('kick "Ghost"')}: kick: Ghost is not online`
  t.check('kick of an offline name warns with the script line', logsFrom(mark).some((l) => l.includes(off)), logsFrom(mark).filter((l) => /WARN/.test(l)).join(' | '))

  // ---- ban with a reason, then unban ----
  since = now()
  tester.command('as_ban Pleb')
  const banMsg = await pleb.waitText('as:ban-reason', { since, ms: 5000 })
  t.check('ban: the player is disconnected with the reason', !!banMsg, banMsg ? banMsg.text : 'none')
  r = await tester.waitText('as:ban=', { since })
  t.check('ban: "is banned" is true', !!r && r.text.includes('as:ban=yes'), r ? r.text : 'none')
  let refused = false
  try { await join('Pleb') } catch { refused = true }
  t.check('ban: the banned player can not join', refused)
  const banFile = path.join(t.dir, 'data', 'banned-players.json')
  const bans = fs.existsSync(banFile) ? fs.readFileSync(banFile, 'utf8') : ''
  t.check('ban: the reason is in the ban list', bans.includes('as:ban-reason'), bans.slice(0, 160))
  r = await say(tester, 'as_unban Pleb', 'as:unban=')
  t.check('unban: "is banned" is false', !!r && r.text.includes('as:unban=ok'), r ? r.text : 'none')
  pleb = await join('Pleb')
  t.check('unban: the player can join again', pleb.entityId != null)
  await sleep(500)

  // ---- ban for 1 minute ----
  since = now()
  tester.command('as_banfor Pleb')
  const bf = await tester.waitText('as:banfor=', { since })
  t.check('ban for 1 minute: "is banned" is true', !!bf && bf.text.includes('as:banfor=yes'), bf ? bf.text : 'none')
  const bans2 = fs.existsSync(banFile) ? JSON.parse(fs.readFileSync(banFile, 'utf8') || '[]') : []
  const entry = bans2.find((b) => b.name === 'Pleb')
  t.check('ban for 1 minute: the entry ends in about a minute',
    !!entry && !!entry.expires && entry.expires !== 'forever' && Math.abs(Date.parse(entry.expires.replace(' +0000', 'Z').replace(' ', 'T')) - Date.now() - 60000) < 20000,
    entry ? JSON.stringify(entry) : 'none')
  await sleep(500)
  r = await say(tester, 'as_unban Pleb', 'as:unban=')
  t.check('unban after a timed ban', !!r && r.text.includes('as:unban=ok'), r ? r.text : 'none')
  try { pleb.quit() } catch {}
  pleb = await join('Pleb')

  t.check('no console echo of any action', !srv.lines.some((l) => /\[Server: |Played sound|Banned Pleb|Opped|De-opped|Kicked Pleb/.test(l)),
    srv.lines.find((l) => /\[Server: |Played sound|Banned Pleb|Opped|De-opped|Kicked Pleb/.test(l)) || '')
  const bad = srv.lines.find((l) => /\[Carve\].*(ERROR|runtime error)/i.test(l))
  t.check('no runtime errors', !bad, bad || '')

  // ---- saved state: variables, a bound and first joins survive a restart ----
  const show = async (bot) => (await say(bot, 'as_show', 'as:show', 6000))
  let s = await show(tester)
  t.check('first joins counted: Tester and Pleb', !!s && /firsts=2\b/.test(s.text), s ? s.text : 'none')
  t.check('first run: Tester has not played before', !!s && /played=no/.test(s.text), s ? s.text : 'none')
  await say(tester, 'as_set1', 'as:set1')
  await sleep(6500) // written by the periodic flush
  await say(tester, 'as_set2', 'as:set2')
  await sleep(300)
  // `set2` is only 0.3 s old: the stop has to write it.
  tester.quit(); pleb.quit()
  await sleep(500)
  stopped = true
  t.check('server stops cleanly', await srv.stop())
  t.check('stop: the store is written, no variables.json', fs.existsSync(path.join(dataDir, 'carve.redb')) && !fs.existsSync(path.join(dataDir, 'variables.json')))

  srv = await startServer({ dir: t.dir, port: t.port, scripts: [SCRIPT], ops: ['Tester'], keepVariables: true, keepWorld: true })
  stopped = false
  t.check('restart: nothing imported', !srv.lines.some((l) => /imported \d+ variables/.test(l)))
  tester = await join('Tester')
  await sleep(1000)
  s = await show(tester)
  t.check('restart: the variables are back (text deleted, number 42, list of 3)', !!s && /text=<none>/.test(s.text) && /num=42\b/.test(s.text) && /list=3\b/.test(s.text), s ? s.text : 'none')
  t.check('restart: the bound is back', !!s && /bound=yes/.test(s.text), s ? s.text : 'none')
  t.check('restart: Tester is not a first join again, has played before', !!s && /firsts=2\b/.test(s.text) && /played=yes/.test(s.text), s ? s.text : 'none')
  tester.quit()
  await sleep(500)
  stopped = true
  await srv.stop()

  // ---- the three old files are imported once, then renamed ----
  fs.rmSync(path.join(dataDir, 'carve.redb'), { force: true })
  fs.writeFileSync(path.join(dataDir, 'variables.json'), JSON.stringify({ as_old: 5, as_oldt: 'legacy', as_firsts: 0, 'as_list::1': 'x' }))
  fs.writeFileSync(path.join(dataDir, 'bounds.json'), JSON.stringify({
    bounds: [{ id: 'oldbound', world: 'world', min: [0, 0, 0], max: [3, 3, 3], owners: [], members: [], flags: {} }],
  }))
  fs.writeFileSync(path.join(dataDir, 'first_join.txt'), `${offlineUuid('Tester')}\nsome-other-uuid\n`)
  srv = await startServer({ dir: t.dir, port: t.port, scripts: [SCRIPT], ops: ['Tester'], keepVariables: true, keepWorld: true })
  stopped = false
  const imp = srv.lines.find((l) => /imported 4 variables, 1 bounds and 2 first-join players/.test(l))
  t.check('migration: the log says what was imported', !!imp, srv.lines.filter((l) => /Carve/.test(l)).slice(0, 6).join(' | '))
  t.check('migration: old files renamed to .old', ['variables.json', 'bounds.json', 'first_join.txt'].every((f) =>
    !fs.existsSync(path.join(dataDir, f)) && fs.existsSync(path.join(dataDir, f + '.old'))))
  tester = await join('Tester')
  await sleep(1000)
  s = await show(tester)
  t.check('migration: variables and the bound come through', !!s && /legacy=5\/legacy/.test(s.text) && /old=yes/.test(s.text) && /list=1\b/.test(s.text), s ? s.text : 'none')
  t.check('migration: Tester counts as having joined before (no first join)', !!s && /firsts=0\b/.test(s.text) && /played=yes/.test(s.text), s ? s.text : 'none')
  pleb = await join('Pleb')
  await sleep(1000)
  s = await show(tester)
  t.check('migration: a player not in the old list is a first join', !!s && /firsts=1\b/.test(s.text), s ? s.text : 'none')
  t.check('migration: the old files are not read a second time', !srv.lines.some((l) => /already has data/.test(l)))
  const bad2 = srv.lines.find((l) => /\[Carve\].*(ERROR|runtime error)/i.test(l))
  t.check('no runtime errors after migration', !bad2, bad2 || '')
  const errs = [tester, pleb].flatMap((b) => b.packets.filter((p) => p.error))
  t.check('every decoded packet parsed', errs.length === 0, errs.slice(0, 3).map((p) => `${p.name}: ${p.error}`).join('; '))
  // ---- a crash: what was written stays, the file still opens ----
  await say(tester, 'as_crash', 'as:crash-set')
  await sleep(6500) // the periodic write has run
  tester.quit(); pleb.quit()
  srv.proc.kill() // no stop, no unload
  await new Promise((resolve) => srv.proc.once('exit', resolve))
  stopped = true
  srv = await startServer({ dir: t.dir, port: t.port, scripts: [SCRIPT], ops: ['Tester'], keepVariables: true, keepWorld: true })
  stopped = false
  t.check('after a kill: the store opens, nothing imported again', !!await srv.waitLog(/Loaded 1 script/, 3000) && !srv.lines.some((l) => /imported \d+ variables|could not (open|read)|can't open/.test(l)),
    srv.lines.filter((l) => /Carve/.test(l)).slice(0, 4).join(' | '))
  tester = await join('Tester')
  await sleep(1000)
  s = await show(tester)
  t.check('after a kill: the value written before it is there', !!s && /crash=99\b/.test(s.text) && /legacy=5\/legacy/.test(s.text), s ? s.text : 'none')
  stopped = true
  t.check('server stops cleanly at the end', await srv.stop())
})
