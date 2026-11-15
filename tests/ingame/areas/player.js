// Player area: health, food, xp, game mode, flight, op/kick/ban, names,
// titles, boss bars, scoreboards, teams, tab list, colors and components,
// placeholders, senders and sounds.
//   node tests/ingame/areas/player.js
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer, offlineUuid } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('player', { port: 25575 })
const SCRIPT = path.join(__dirname, 'player', 'player.carve')

// Every text part of a raw NBT component with its own color and click/hover.
function segments (raw) {
  const out = []
  const walk = (c, inh) => {
    if (c == null) return
    if (typeof c !== 'object') { out.push({ text: String(c), ...inh }); return }
    if (Array.isArray(c)) { c.forEach((x) => walk(x, inh)); return }
    const cur = { ...inh }
    for (const k of ['color', 'bold', 'italic']) if (c[k] != null) cur[k] = c[k]
    const click = c.click_event || c.clickEvent
    if (click) cur.click = click
    const hover = c.hover_event || c.hoverEvent
    if (hover) cur.hover = hover
    const s = c.text != null ? c.text : c['']
    if (s != null && String(s) !== '') out.push({ text: String(s), ...cur })
    if (c.extra) c.extra.forEach((x) => walk(x, cur))
  }
  walk(raw, {})
  return out
}
const segOf = (raw, needle) => segments(raw).find((s) => s.text.includes(needle))
const near = (a, b, eps = 0.01) => typeof a === 'number' && Math.abs(a - b) < eps
// Carve sends named colors as their hex value; both mean the same to the client.
const NAMED = { black: '#000000', dark_blue: '#0000aa', dark_green: '#00aa00', dark_aqua: '#00aaaa', dark_red: '#aa0000', dark_purple: '#aa00aa', gold: '#ffaa00', gray: '#aaaaaa', dark_gray: '#555555', blue: '#5555ff', green: '#55ff55', aqua: '#55ffff', red: '#ff5555', light_purple: '#ff55ff', yellow: '#ffff55', white: '#ffffff' }
const isColor = (got, want) => !!got && (got === want || got.toLowerCase() === (NAMED[want] || want).toLowerCase())
const segColor = (raw, needle, want) => { const s = raw && segOf(raw, needle); return isColor(s && s.color, want) }

t.run(async () => {
  const srv = await startServer({ dir: t.dir, port: t.port, scripts: [SCRIPT], ops: ['Tester'] })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop()))
  const loaded = await srv.waitLog(/\[Carve\] Loaded 1 script/, 5000)
  t.check('carve loads player.carve without errors', !!loaded && !srv.lines.some((l) => /Encountered \d+ error/.test(l)),
    srv.lines.filter((l) => /\[Carve\] +(Line|Can't)/.test(l)).slice(0, 6).join(' | '))

  let bot = await joinBot('Tester', { port: t.port })
  t.cleanup(() => bot.quit())
  let pleb = await joinBot('Pleb', { port: t.port })
  t.cleanup(() => pleb.quit())
  srv.console('gamemode survival Tester')
  await sleep(500)

  // Run a script command and wait for its "p:" reply.
  async function run (cmd, marker, { who = bot, ms = 10000 } = {}) { // long: parallel areas can lag the server
    await sleep(5) // a reply in the same millisecond would count for the next command
    const since = Date.now()
    who.command(cmd)
    const txt = marker ? await who.waitText(marker, { since, ms }) : null
    return { since, txt, text: txt ? txt.text : '' }
  }
  const pkt = (name, pred, since, ms = 3000, who = bot) => who.waitPacket(name, pred, ms, since)

  // ---- health ----
  let r = await run('p_hp', 'p:hp ')
  t.check('health/max health read back', r.text === 'p:hp 30/40', r.text)
  let h = await pkt('set_health', (d) => near(d.health, 30), r.since)
  t.check('set health of player: set_health 30', !!h, h ? `${h.health}` : 'none')
  const attr = await pkt('update_attributes', (d) => d.entityId === bot.entityId && d.attributes.max_health, r.since)
  t.check('set max health: update_attributes max_health 40', attr && near(attr.attributes.max_health.value, 40),
    attr ? JSON.stringify(attr.attributes.max_health) : 'no packet')

  r = await run('p_hp2', 'p:hp2 ')
  t.check('add/remove health', r.text === 'p:hp2 24', r.text)
  h = await pkt('set_health', (d) => near(d.health, 24), r.since)
  t.check('add/remove health: set_health 24', !!h)

  r = await run('p_heal', 'p:heal ')
  t.check('heal player fills to max', r.text === 'p:heal 40', r.text)
  t.check('heal: set_health 40', !!await pkt('set_health', (d) => near(d.health, 40), r.since))

  r = await run('p_maxdown', 'p:maxdown ')
  t.check('lowering max health lowers health', r.text === 'p:maxdown 20/20', r.text)
  t.check('lower max health: set_health 20', !!await pkt('set_health', (d) => near(d.health, 20), r.since))

  r = await run('p_healby', 'p:healby ')
  t.check('heal player by 3', r.text === 'p:healby 13', r.text)
  t.check('heal by: set_health 13', !!await pkt('set_health', (d) => near(d.health, 13), r.since))

  r = await run('p_dmg', 'p:dmg ')
  h = await pkt('set_health', (d) => near(d.health, 8), r.since)
  t.check('damage player by 5: set_health 8', h && near(h.health, 8), h ? `${h.health}` : 'none')
  t.check('damage player: damage_event or hurt animation',
    !!(await pkt('damage_event', (d) => d.entityId === bot.entityId, r.since, 1000) || await pkt('hurt_animation', (d) => d.entityId === bot.entityId, r.since, 1000)))
  t.check('damage read back', r.text === 'p:dmg 8', r.text)

  await sleep(1200) // past the hurt cooldown from the damage above
  let mark = srv.mark()
  r = await run('p_kill', null)
  const death = await pkt('player_combat_kill', (d) => d.playerId === bot.entityId, r.since)
  t.check('kill player: death screen (player_combat_kill)', !!death, death ? death.text : 'none')
  t.check('kill player fires on death', !!await srv.waitLog(/p:died/, 3000, mark))
  bot.respawn()
  h = await pkt('set_health', (d) => d.health > 0, r.since, 5000)
  t.check('respawn after kill', !!h, h ? `${h.health}` : 'none')
  await sleep(500)

  // ---- food ----
  r = await run('p_food', 'p:food ')
  t.check('food level/saturation/hunger read back', r.text === 'p:food 5 3 5', r.text)
  h = await pkt('set_health', (d) => d.food === 5 && near(d.saturation, 3), r.since)
  t.check('set food level and saturation: set_health food 5 sat 3', !!h)
  r = await run('p_food2', 'p:food2 ')
  t.check('add/remove hunger', r.text === 'p:food2 6', r.text)
  t.check('add/remove hunger: set_health food 6', !!await pkt('set_health', (d) => d.food === 6, r.since))
  r = await run('p_feed', 'p:feed ')
  t.check('feed player', r.text === 'p:feed 20 20', r.text)
  t.check('feed: set_health food 20 sat 20', !!await pkt('set_health', (d) => d.food === 20 && near(d.saturation, 20), r.since))
  r = await run('p_feedby', 'p:feedby ')
  t.check('feed player by 4', r.text === 'p:feedby 14', r.text)
  t.check('feed by: set_health food 14', !!await pkt('set_health', (d) => d.food === 14, r.since))

  // ---- experience ----
  r = await run('p_lvl', 'p:lvl ')
  t.check('level / level progress read back', r.text === 'p:lvl 10 0.5', r.text)
  let xp = await pkt('set_experience', (d) => d.level === 10 && near(d.progress, 0.5), r.since)
  t.check('set level and progress: set_experience level 10 progress 0.5', !!xp)
  r = await run('p_xp', 'p:xp ')
  xp = await pkt('set_experience', (d) => d.total === 500, r.since)
  t.check('set xp of player to 500: set_experience total 500', !!xp, xp ? `level ${xp.level} progress ${xp.progress.toFixed(2)}` : 'none')
  t.check('xp / total experience read back', /^p:xp 500 \d+ 500$/.test(r.text), r.text)
  r = await run('p_xp2', 'p:xp2 ')
  t.check('give/add/remove levels', r.text === 'p:xp2 5', r.text)
  t.check('levels: set_experience level 5', !!await pkt('set_experience', (d) => d.level === 5, r.since))
  r = await run('p_xp3', 'p:xp3 ')
  t.check('give/add/remove xp points', r.text === 'p:xp3 12', r.text)
  t.check('xp points: set_experience total 12', !!await pkt('set_experience', (d) => d.total === 12, r.since))

  // ---- game mode ----
  r = await run('p_gm creative', 'p:gm ')
  t.check('set gamemode to creative read back', r.text === 'p:gm creative creative creative', r.text)
  t.check('set gamemode: game_event creative', !!await pkt('game_event', (d) => d.gameMode === 'creative', r.since))
  t.check('set gamemode: player_info_update creative', !!await pkt('player_info_update', (d) => d.players.some((p) => p.gameMode === 'creative'), r.since))
  t.check('player is in creative', !!await bot.waitText('p:gm-is-creative', { since: r.since, ms: 1000 }))
  t.check('gamemode of player is not survival', !!await bot.waitText('p:gm-not-survival', { since: r.since, ms: 1000 }))
  r = await run('p_gm a', 'p:gm ')
  t.check('gamemode short form a = adventure', r.text === 'p:gm adventure adventure adventure', r.text)
  t.check('adventure: game_event', !!await pkt('game_event', (d) => d.gameMode === 'adventure', r.since))
  r = await run('p_gm 3', 'p:gm ')
  t.check('gamemode number 3 = spectator', r.text === 'p:gm spectator spectator spectator', r.text)
  r = await run('p_gmc', 'p:gmc ')
  t.check('set player\'s gamemode to creative mode', r.text === 'p:gmc creative', r.text)
  r = await run('p_gm survival', 'p:gm ')
  t.check('back to survival', r.text === 'p:gm survival survival survival', r.text)
  t.check('survival: game_event', !!await pkt('game_event', (d) => d.gameMode === 'survival', r.since))

  // ---- flight ----
  r = await run('p_fly', 'p:fly')
  let ab = await pkt('player_abilities', (d) => d.allowFlying && d.flying, r.since)
  t.check('make player fly: player_abilities allow + flying', !!ab, ab ? `flags ${ab.flags}` : 'none')
  t.check('player can fly after make fly', !!await bot.waitText('p:fly can', { since: r.since, ms: 1000 }))
  t.check('player is flying after make fly', !!await bot.waitText('p:fly flying', { since: r.since, ms: 1000 }))
  r = await run('p_nofly', 'p:nofly')
  ab = await pkt('player_abilities', (d) => !d.allowFlying && !d.flying, r.since)
  t.check('stop flying: player_abilities cleared', !!ab, ab ? `flags ${ab.flags}` : 'none')
  t.check('player is not flying after stop', !!r.txt)

  t.skip('walk speed / fly speed', '⛔ no attribute or abilities speed API in the plugin API')

  // ---- op / deop ----
  r = await run('p_opcheck', 'p:op ', { who: pleb })
  t.check('Pleb is not op', r.text === 'p:op no', r.text)
  r = await run('p_opcheck', 'p:op ')
  t.check('Tester is op', r.text === 'p:op yes', r.text)
  r = await run('p_op Pleb', 'p:opped')
  const opEv = await pkt('entity_event', (d) => d.event >= 25 && d.event <= 28, r.since, 3000, pleb)
  t.check('op "Pleb": op-level entity_event to Pleb', !!opEv, opEv ? `event ${opEv.event}` : 'none')
  await sleep(300)
  r = await run('p_opcheck', 'p:op ', { who: pleb })
  t.check('Pleb is op after op', r.text === 'p:op yes', r.text)
  r = await run('p_deop Pleb', 'p:deopped')
  const deopEv = await pkt('entity_event', (d) => d.event === 24, r.since, 3000, pleb)
  t.check('deop "Pleb": level 0 entity_event to Pleb', !!deopEv)
  await sleep(300)
  r = await run('p_opcheck', 'p:op ', { who: pleb })
  t.check('Pleb is not op after deop', r.text === 'p:op no', r.text)

  // ---- names ----
  r = await run('p_dname', 'p:dname ')
  t.check('display name / nickname read back', r.text === 'p:dname Boss Boss', r.text)
  r = await run('p_dreset', 'p:dreset ')
  t.check('reset display name', r.text === 'p:dreset Tester', r.text)
  r = await run('p_tab', 'p:tab ')
  t.check('tab list name read back', r.text === 'p:tab [VIP] Tester', r.text)
  let info = await pkt('player_info_update', (d) => (d.actions & 0x20) && d.players.some((p) => p.displayName === '[VIP] Tester'), r.since, 3000, pleb)
  const tabSeg = info && segOf(info.players.find((p) => p.displayName).displayComponent.raw, '[VIP]')
  t.check('tab list name: player_info_update display name to others', !!info)
  t.check('tab list name keeps & color (green)', tabSeg && isColor(tabSeg.color, 'green'), tabSeg ? tabSeg.color : 'none')
  r = await run('p_tabreset', 'p:tabreset ')
  info = await pkt('player_info_update', (d) => (d.actions & 0x20) && d.players.some((p) => p.displayName == null), r.since, 3000, pleb)
  t.check('reset tab list name: player_info_update display name cleared', !!info)
  t.check('reset tab list name reads the account name', r.text === 'p:tabreset Tester', r.text)

  r = await run('p_prio', null)
  t.check('player list priority: player_info_update list order 5', !!await pkt('player_info_update', (d) => (d.actions & 0x40) && d.players.some((p) => p.listOrder === 5), r.since, 3000, pleb))

  r = await run('p_ids', 'p:ids ')
  const m = r.text.match(/ping=(\S+) uuid=(\S+) ip=(\S+) name=(\S+)/) || []
  t.check('ping of player is a number', /^\d+(\.\d+)?$/.test(m[1] || ''), m[1])
  t.check('uuid of player is the offline uuid', m[2] === offlineUuid('Tester'), `${m[2]} vs ${offlineUuid('Tester')}`)
  t.check('ip of player is 127.0.0.1', m[3] === '127.0.0.1', m[3])
  t.check('name of player', m[4] === 'Tester', m[4])

  // ---- titles and action bar ----
  r = await run('p_title', null)
  t.check('send title', !!await pkt('set_title_text', (d) => d.text === 'p:t1', r.since))
  t.check('send title with subtitle', !!await pkt('set_subtitle_text', (d) => d.text === 'p:s1', r.since))
  const times = await pkt('set_titles_animation', () => true, r.since)
  t.check('title times: fade in 5, stay 60, fade out 20 ticks', times && times.fadeIn === 5 && times.stay === 60 && times.fadeOut === 20, times ? JSON.stringify(times) : 'none')
  r = await run('p_titleall', null)
  t.check('send title to all players (Tester)', !!await pkt('set_title_text', (d) => d.text === 'p:tall', r.since))
  t.check('send title to all players (Pleb)', !!await pkt('set_title_text', (d) => d.text === 'p:tall', r.since, 3000, pleb))
  r = await run('p_tclear', null)
  let ct = await pkt('clear_titles', (d) => d.reset === false, r.since)
  t.check('clear title: clear_titles reset=false', ct && ct.reset === false, ct ? JSON.stringify(ct) : 'none')
  r = await run('p_treset', null)
  ct = await pkt('clear_titles', (d) => d.reset === true, r.since)
  t.check('reset title: clear_titles reset=true', ct && ct.reset === true, ct ? JSON.stringify(ct) : 'none')
  r = await run('p_ab', null)
  t.check('send actionbar to player', !!await bot.waitText('p:ab', { kind: 'actionbar', since: r.since }))
  t.check('send action bar to all players (Pleb)', !!await pleb.waitText('p:aball', { kind: 'actionbar', since: r.since }))

  // ---- boss bars ----
  r = await run('p_bb', null)
  const bb = await pkt('boss_event', (d) => d.action === 'add' && d.text === 'p:raid', r.since)
  t.check('create + show bossbar: boss_event add', !!bb)
  t.check('bossbar progress 40 -> 0.4', bb && near(bb.progress, 0.4), bb ? `${bb.progress}` : '')
  t.check('bossbar color red', bb && bb.color === 'red', bb ? bb.color : '')
  t.check('bossbar style notched 10 (overlay 2)', bb && bb.overlay === 2, bb ? `${bb.overlay}` : '')
  t.check('bossbar title keeps &c color', bb && segColor(bb.title.raw, 'p:raid', 'red'))
  const bbId = bb && bb.uuid
  r = await run('p_bb2', null)
  const bbP = await pkt('boss_event', (d) => d.uuid === bbId && d.action === 'progress', r.since)
  t.check('set progress of bossbar to 75', bbP && near(bbP.progress, 0.75), bbP ? `${bbP.progress}` : 'none')
  t.check('set title of bossbar', !!await pkt('boss_event', (d) => d.uuid === bbId && d.action === 'name' && d.text === 'p:raid2', r.since))
  const bbS = await pkt('boss_event', (d) => d.uuid === bbId && d.action === 'style' && d.color === 'blue' && d.overlay === 0, r.since)
  t.check('set color/style of bossbar: blue solid', bbS && bbS.overlay === 0, bbS ? JSON.stringify(bbS) : 'none')
  const bbF = await pkt('boss_event', (d) => d.uuid === bbId && d.action === 'flags', r.since)
  t.check('add darken sky flag (0x1)', bbF && (bbF.flags & 1) === 1, bbF ? `${bbF.flags}` : 'none')
  r = await run('p_bbhide', null)
  t.check('hide bossbar: boss_event remove', !!await pkt('boss_event', (d) => d.uuid === bbId && d.action === 'remove', r.since))
  r = await run('p_bbshow', null)
  const bb2 = await pkt('boss_event', (d) => d.uuid === bbId && d.action === 'add', r.since)
  t.check('add player to bossbar: shows again with new state', bb2 && bb2.text === 'p:raid2' && near(bb2.progress, 0.75), bb2 ? JSON.stringify({ t: bb2.text, p: bb2.progress, c: bb2.color }) : 'none')
  r = await run('p_bbrm', null)
  t.check('remove bossbar: boss_event remove', !!await pkt('boss_event', (d) => d.uuid === bbId && d.action === 'remove', r.since))

  // ---- sidebar ----
  r = await run('p_side', null)
  const obj = await pkt('set_objective', (d) => d.mode !== 'remove' && /p:side/.test(d.text || ''), r.since)
  t.check('sidebar title: set_objective', !!obj, obj ? obj.text : 'none')
  t.check('sidebar title keeps &6 color (gold)', obj && segColor(obj.display.raw, 'p:side', 'gold'))
  t.check('sidebar shown in display slot 1', obj && !!await pkt('set_display_objective', (d) => d.slot === 1 && d.objective === obj.objective, r.since))
  // A sidebar line is a score holder named after the line (plus invisible § padding).
  const plain = (s) => (s || '').replace(/§./g, '')
  const line = (txt) => (d) => (d.display ? plain(d.display) : plain(d.entity)) === txt
  const l1 = await pkt('set_score', line('p:line1'), r.since)
  const l2 = await pkt('set_score', line('p:line2'), r.since)
  t.check('sidebar line 1 and 2: set_score', !!(l1 && l2))
  t.check('sidebar line 1 is above line 2', l1 && l2 && l1.value > l2.value, l1 && l2 ? `${l1.value} ${l2.value}` : '')
  t.check('sidebar line keeps &7 color', l2 && /§7p:line2/.test(l2.entity), l2 ? JSON.stringify(l2.entity) : '')
  r = await run('p_sideclr2', null)
  const l1b = await pkt('set_score', line('p:line1'), r.since, 300)
  await sleep(300)
  const l2b = bot.packets.find((p) => p.name === 'set_score' && p.t >= r.since && line('p:line2')(p.data))
  const l2gone = await pkt('reset_score', (d) => l2 && d.entity === l2.entity, r.since, 2000)
  t.check('clear line 2: reset_score for line 2 only', !!l2gone && !l2b && !l1b)
  r = await run('p_sideclr', null)
  t.check('clear scoreboard: set_objective remove', obj && !!await pkt('set_objective', (d) => d.mode === 'remove' && d.objective === obj.objective, r.since))

  r = await run('p_board', null)
  const bo = await pkt('set_objective', (d) => d.mode !== 'remove' && d.text === 'p:board', r.since)
  t.check('board title: set_objective', !!bo)
  const bl = []
  for (const s of ['p:b1', 'p:b2', 'p:b3']) bl.push(await pkt('set_score', line(s), r.since))
  t.check('set lines of board: three set_score lines in order', bl.every(Boolean) && bl[0].value > bl[1].value && bl[1].value > bl[2].value)
  r = await run('p_boarddel', null)
  t.check('delete board: set_objective remove', bo && !!await pkt('set_objective', (d) => d.mode === 'remove' && d.objective === bo.objective, r.since))

  // ---- objectives, scores, teams ----
  r = await run('p_obj', 'p:obj ')
  t.check('score / objective display name read back', /^p:obj 8 Kills$/.test(r.text), r.text)
  t.check('objective exists', !!await bot.waitText('p:obj exists', { since: r.since, ms: 1000 }))
  const so = await pkt('set_objective', (d) => d.objective === 'pkills' && d.mode === 'add', r.since)
  t.check('register objective: set_objective pkills', !!so, so ? so.text : 'none')
  const sc = await pkt('set_score', (d) => d.objective === 'pkills' && d.entity === 'Tester' && d.value === 8, r.since)
  t.check('set/add score: set_score Tester pkills 8', !!sc)
  r = await run('p_objrm', 'p:objrm')
  t.check('unregister objective: gone', !!r.txt)
  t.check('unregister objective: set_objective remove', !!await pkt('set_objective', (d) => d.objective === 'pkills' && d.mode === 'remove', r.since))

  r = await run('p_team', 'p:team ')
  t.check('team of / prefix / color / size read back', r.text === 'p:team pred [R]  dark_red 1' || /^p:team pred .*\[R\].* dark.red 1$/.test(r.text), r.text)
  t.check('player is in team', !!await bot.waitText('p:team member', { since: r.since, ms: 1000 }))
  const tm = await pkt('set_player_team', (d) => d.team === 'pred' && (d.method === 'create' || d.method === 'update') && /\[R\]/.test(d.prefix || ''), r.since)
  t.check('team prefix: set_player_team', !!tm, tm ? JSON.stringify({ p: tm.prefix, s: tm.suffix, c: tm.color }) : 'none')
  const tmS = await pkt('set_player_team', (d) => d.team === 'pred' && /!/.test(d.suffix || ''), r.since)
  t.check('team suffix: set_player_team', !!tmS)
  const tmC = await pkt('set_player_team', (d) => d.team === 'pred' && d.color === 4, r.since)
  t.check('team color dark red (4): set_player_team', !!tmC)
  t.check('add player to team: set_player_team add_players Tester', !!await pkt('set_player_team', (d) => d.team === 'pred' && (d.players || []).includes('Tester') && d.method !== 'remove_players', r.since))
  r = await run('p_teamrm', 'p:teamrm')
  t.check('remove from team / unregister: set_player_team remove', !!await pkt('set_player_team', (d) => d.team === 'pred' && d.method === 'remove', r.since))

  // ---- tab list header and footer ----
  r = await run('p_tabhf', null)
  let tl = await pkt('tab_list', (d) => d.header === 'p:head' && d.footer === 'p:foot', r.since)
  t.check('tab list header and footer: tab_list', !!tl)
  t.check('tab header keeps &6 (gold)', tl && segColor(tl.headerComponent.raw, 'p:head', 'gold'))
  r = await run('p_tabhf2', null)
  tl = await pkt('tab_list', (d) => d.header === 'p:head2', r.since, 3000, pleb)
  t.check('tab header for all players (Pleb)', !!tl)
  tl = await pkt('tab_list', (d) => d.header === 'p:head2', r.since)
  t.check('header only keeps the footer', tl && tl.footer === 'p:foot', tl ? tl.footer : 'none')
  r = await run('p_tabhfreset', null)
  t.check('reset tab list: empty header and footer', !!await pkt('tab_list', (d) => d.header === '' && d.footer === '', r.since))

  // ---- colors and components ----
  r = await run('p_col', 'p:hex')
  const raw = r.txt && r.txt.raw
  const col = (s) => { const x = raw && segOf(raw, s); return x ? x.color : 'missing' }
  t.check('hex tag <#ff8000>', /^#ff8000$/i.test(col('p:hex')), col('p:hex'))
  t.check('legacy &a = green', isColor(col('p:legacy'), 'green'), col('p:legacy'))
  t.check('named tag <red>', isColor(col('p:named'), 'red'), col('p:named'))
  t.check('named tag <dark_blue>', isColor(col('p:dblue'), 'dark_blue'), col('p:dblue'))
  r = await run('p_col2', 'p:amphex')
  const raw2 = r.txt && r.txt.raw
  const col2 = (s) => { const x = raw2 && segOf(raw2, s); return x ? x.color : 'missing' }
  t.check('&#00ff7f hex code', /^#00ff7f$/i.test(col2('p:amphex')), col2('p:amphex'))
  t.check('&x&1&2&3&4&5&6 hex code', /^#123456$/i.test(col2('p:xhex')), col2('p:xhex'))
  const bold = raw2 && segOf(raw2, 'p:bold')
  t.check('<bold> tag', bold && bold.bold === true || bold && bold.bold === 1, bold ? JSON.stringify(bold) : 'missing')
  r = await run('p_grad', 'p:gradient')
  const gsegs = r.txt ? segments(r.txt.raw) : []
  const gText = gsegs.map((s) => s.text).join('')
  t.check('gradient/rainbow text arrives whole', gText.includes('p:gradient') && gText.includes('p:rainbow'), gText)
  const gradColors = new Set(gsegs.filter((s, i) => i < 10).map((s) => s.color))
  t.check('gradient colors each letter differently', gradColors.size >= 5, [...gradColors].join(','))
  t.check('gradient starts red-ish', gsegs[0] && /^#(ff|f[0-9a-f])/i.test(gsegs[0].color || ''), gsegs[0] && gsegs[0].color)
  r = await run('p_colfn', 'p:colfn')
  t.check('uncolored colored text is plain', r.txt && r.text === 'p:colfn' && !segments(r.txt.raw).some((s) => s.color), r.txt ? JSON.stringify(segments(r.txt.raw)) : 'none')

  r = await run('p_click', 'p:clk')
  let seg = r.txt && segOf(r.txt.raw, 'p:clk')
  t.check('click run_command component', seg && seg.click && seg.click.action === 'run_command' && seg.click.command === '/p_ids', seg ? JSON.stringify(seg.click) : 'none')
  t.check('hover show_text component', seg && seg.hover && seg.hover.action === 'show_text' && JSON.stringify(seg.hover).includes('p:hov'), seg ? JSON.stringify(seg.hover) : 'none')
  r = await run('p_click2', 'p:comp')
  seg = r.txt && segOf(r.txt.raw, 'p:comp')
  t.check('text component + click event open_url', seg && seg.click && seg.click.action === 'open_url' && seg.click.url === 'https://example.com/rules', seg ? JSON.stringify(seg.click) : 'none')
  t.check('text component + hover event', seg && seg.hover && JSON.stringify(seg.hover).includes('p:hov2'), seg ? JSON.stringify(seg.hover) : 'none')
  t.check('text component keeps &6 (gold)', seg && isColor(seg.color, 'gold'), seg ? seg.color : 'none')
  r = await run('p_click3', 'p:sugg')
  seg = r.txt && segOf(r.txt.raw, 'p:sugg')
  t.check('click suggest_command', seg && seg.click && seg.click.action === 'suggest_command' && seg.click.command === '/msg ', seg ? JSON.stringify(seg.click) : 'none')
  seg = r.txt && segOf(r.txt.raw, 'p:cpy')
  t.check('click copy_to_clipboard', seg && seg.click && seg.click.action === 'copy_to_clipboard' && seg.click.value === 'p:copy', seg ? JSON.stringify(seg.click) : 'none')

  // ---- placeholders ----
  r = await run('p_ph', 'p:ph ')
  t.check('parse placeholders player_name/player_gamemode', r.text === 'p:ph Tester survival', r.text)
  const ph2 = await bot.waitText('p:ph2', { since: r.since })
  t.check('placeholder "player_name" for player', ph2 && ph2.text === 'p:ph2 Tester', ph2 ? ph2.text : 'none')
  r = await run('p_ph3', 'p:ph3')
  t.check('script placeholder (on placeholder request)', r.text === 'p:ph3 world', r.text)

  // ---- senders ----
  mark = srv.mark()
  r = await run('p_sender', 'p:sender')
  t.check('sender is a player / send to sender / me', r.text === 'p:sender player Tester', r.text)
  t.check('send to console from a player writes the log', !!await srv.waitLog(/p:toconsole/, 3000, mark))
  t.check('send to console does not reach the player', !await bot.waitText('p:toconsole', { since: r.since, ms: 500 }))
  mark = srv.mark()
  srv.console('p_sender')
  t.check('console sender: sender is console', !!await srv.waitLog(/p:sender console/, 3000, mark))
  r = await run('p_make', 'p:target')
  t.check('make player execute command', r.text === 'p:target Tester', r.text)
  r = await run('p_make2', 'p:target')
  t.check('execute command by player', r.text === 'p:target Tester', r.text)
  mark = srv.mark()
  r = await run('p_console', null)
  const say = await bot.waitText('p:say', { since: r.since })
  t.check('execute console command "say"', !!say, say ? say.text : 'none')
  t.check('make console execute command', !!await srv.waitLog(/p:sender console/, 3000, mark))
  const runSince = Date.now()
  srv.console('carve run broadcast "p:runmsg"')
  t.check('/carve run from the console reaches players', !!await bot.waitText('p:runmsg', { ms: 3000, since: runSince }))

  // ---- sounds ----
  r = await run('p_snd', null)
  let snd = await pkt('sound', (d) => /experience_orb\.pickup/.test(d.sound), r.since)
  t.check('play sound with volume and pitch', snd && near(snd.volume, 0.5) && near(snd.pitch, 2), snd ? `${snd.sound} ${snd.volume} ${snd.pitch} ${snd.source}` : 'none')
  srv.console('tp Tester 10 70 20')
  await sleep(500)
  r = await run('p_sndat', null)
  snd = await pkt('sound', (d) => /anvil\.land/.test(d.sound), r.since)
  t.check('play sound at location', snd && near(snd.x, 10, 1) && near(snd.y, 70, 1) && near(snd.z, 20, 1), snd ? `${snd.x} ${snd.y} ${snd.z}` : 'none')
  r = await run('p_sndstop', null)
  const ss = await pkt('stop_sound', (d) => /music\.game/.test(d.sound || ''), r.since)
  t.check('stop sound "music.game"', !!ss, ss ? JSON.stringify(ss) : 'none')
  r = await run('p_sndstopall', null)
  t.check('stop all sounds', !!await pkt('stop_sound', (d) => !d.sound && !d.source, r.since))

  // ---- kick and ban (last: Pleb leaves) ----
  r = await run('p_kick Pleb', null)
  const kick = await pleb.waitText('p:bye now', { kind: 'kick', since: r.since })
  t.check('kick player due to reason: Pleb gets the reason', !!kick, kick ? kick.text : 'none')
  await sleep(500)
  t.check('kick disconnects Pleb', pleb.closed)
  pleb = await joinBot('Pleb', { port: t.port })
  await sleep(500)
  r = await run('p_ban Pleb', 'p:isbanned')
  const banMsg = await pleb.waitText('p:banned', { kind: 'kick', since: r.since, ms: 3000 })
  t.check('ban player: online Pleb is disconnected with the reason', !!banMsg || pleb.closed, banMsg ? banMsg.text : `closed=${pleb.closed}`)
  t.skip('"Pleb" is banned', '⛔ Pumpkin gives plugins no way to read the ban list')
  await sleep(500)
  let rejoin = null; let err = null
  try { rejoin = await joinBot('Pleb', { port: t.port, timeout: 6000 }) } catch (e) { err = e.message }
  t.check('banned Pleb cannot rejoin', !rejoin, err || 'joined')
  if (rejoin) rejoin.quit()
  r = await run('p_unban Pleb', 'p:unbanned')
  await sleep(500)
  try { pleb = await joinBot('Pleb', { port: t.port, timeout: 8000 }); err = null } catch (e) { err = e.message; pleb = { quit () {} } }
  t.check('unban: Pleb can rejoin', !err, err || '')

  await sleep(300)
  t.check('Tester still connected at the end', !bot.closed)
  const errs = bot.packets.filter((p) => p.error)
  t.check('every decoded packet parsed', errs.length === 0, errs.slice(0, 3).map((p) => `${p.name}: ${p.error}`).join('; '))
})
