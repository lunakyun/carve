// Language area: variables, lists, text, dates, math, functions, options,
// structures, loops, error reporting, /carve admin, SkBee utilities, NBT and
// the runaway guard. Checks what the server actually sends to the bot.
//   node tests/ingame/areas/language.js
'use strict'
const fs = require('fs')
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('language', { port: 25576 })
const SCRIPT = path.join(__dirname, 'language', 'language.carve')
const scriptsDir = () => path.join(t.dir, 'plugins', 'data', 'carve', 'scripts')

// Run `/cmd`, wait for "<prefix>:done=yes" and collect every "<prefix>:key=value" message.
async function group (bot, cmd, prefix, ms = 8000) {
  const since = Date.now()
  bot.command(cmd)
  const done = await bot.waitText(`${prefix}:done=yes`, { since, ms })
  await sleep(200)
  const out = { _done: !!done }
  for (const p of bot.packets) {
    if (p.t < since || p.name !== 'system_chat' || !p.data || !p.data.text.startsWith(prefix + ':')) continue
    const s = p.data.text.slice(prefix.length + 1)
    const i = s.indexOf('=')
    const k = s.slice(0, i)
    if (!(k in out)) out[k] = s.slice(i + 1)
  }
  return out
}

// expected: a string (exact), a RegExp, a function, or null (the key must not be sent).
function expectAll (name, out, table) {
  if (!out._done) t.check(`${name}: command finished`, false, JSON.stringify(out))
  for (const [k, want] of Object.entries(table)) {
    const got = out[k]
    let ok
    if (want === null) ok = got === undefined
    else if (want instanceof RegExp) ok = got !== undefined && want.test(got)
    else if (typeof want === 'function') ok = got !== undefined && want(got)
    else ok = got === want
    t.check(`${name}: ${k}`, ok, ok ? '' : `got ${JSON.stringify(got)}, want ${want === null ? 'nothing' : want}`)
  }
}

// All system chat lines at or after `since`.
const chatSince = (bot, since) => bot.packets.filter((p) => p.t >= since && p.name === 'system_chat' && p.data).map((p) => p.data.text)

async function say (bot, cmd, parts, ms = 5000) {
  const since = Date.now()
  bot.command(cmd)
  const hit = await bot.waitText(parts, { since, ms })
  if (!hit) console.log(`  /${cmd} replied: ${JSON.stringify(chatSince(bot, since))}`)
  return hit
}

t.run(async () => {
  let srv = await startServer({ dir: t.dir, port: t.port, scripts: [SCRIPT], ops: ['Tester'] })
  let stopped = false
  t.cleanup(async () => { if (!stopped) t.check('server stops cleanly', await srv.stop()) })
  const loadLine = await srv.waitLog(/\[Carve\] Loaded \d+ script/, 5000)
  t.check('language.carve loads without errors', !!loadLine && !srv.lines.some((l) => /language\.carve.*Line \d+|Line \d+: \(language\.carve\)/.test(l)),
    srv.lines.filter((l) => /error|Line \d+:|Can't/i.test(l)).slice(0, 8).join(' | '))

  let bot = await joinBot('Tester', { port: t.port })
  t.cleanup(() => bot.quit())
  await sleep(500)

  // ---- variables ----
  let o = await group(bot, 'lvars', 'lv')
  expectAll('variables', o, { local: '5', case: 'ci', pp: '100', dflt: 'dflt' })
  o = await group(bot, 'lvlocal', 'lv', 3000)
  t.check('variables: locals start empty in a new run', o.fresh === '<none>', `got ${o.fresh}`)
  const before = await group(bot, 'lvshow', 'lr')
  expectAll('variables before restart', before, {
    g: 'keep', n: '3', list: 'a, b and c', date: '2023-11-14 22:13:20', span: '1 minute and 30 seconds',
    nbt: '1', bool: 'true', player: 'Tester', dflt: 'dflt',
  })

  // ---- lists ----
  o = await group(bot, 'llists', 'll')
  expectAll('lists', o, {
    lit: '3, 1 and 2', addrm: '3, 2 and 5', size: '3', amount: '3', sorted: '2, 3 and 5', rev: '5, 2 and 3',
    shuf: '3 10', where: '3 and 5', idx: '1, 3 and 4', first: '3', last: '5', el2: '2', byidx: '5',
    contains: 'yes', notcontains: 'yes', nested: '1 3', nestedval: '2', deleted: '0',
    concat: '3, 2, 5 and 9', sortdesc: '9, 5, 3 and 2', filter: '5 and 3', alpha: 'Apple, fig and pear',
    nums: '1, 2, 3 and 4', rand: '2', transform: '2, 4 and 6', reduce: '12', sum: '12 avg=4 min=2 max=6', empty: '0',
  })

  // ---- text ----
  o = await group(bot, 'ltext', 'tx')
  expectAll('text', o, {
    join: 'a, b, c', split: '3 y', chars: 'a, b and c', replace: 'hell0 w0rld', replfirst: 'baa', replexpr: 'a-b-',
    upper: 'ABC', lower: 'abc', proper: 'Hello World', cap: 'Hello world', sub: 'bcd', first: 'abc', last: 'ef',
    char: 'b', len: '5', index: '3 4 0', regex: 'match', regexany: 'yes', regexbad: null, trim: '[hi]',
    revtext: 'cba', repeat: 'ababab', hash: '900150983cd24fb0d6963f7d28e17f72', starts: 'yes', nl: 'a\nb',
  })

  // ---- dates and timespans ----
  o = await group(bot, 'ldates', 'dt', 10000)
  expectAll('dates', o, {
    fmt: '2023-11-14 22:13:20', print: '2023-11-14 22:13:20', unix: '1700000000', year: String(new Date().getUTCFullYear()),
    span: '1 hour and 30 minutes', min: '30', total: '90', cmp: 'yes', ago: 'yes',
    arith: '1 minute and 30 seconds 3 minutes', later: '2023-11-15 22:13:20', formatted: '1:30:00',
    since: (v) => Number(v) >= 950 && Number(v) <= 1300,
  })
  const w1 = bot.packets.find((p) => p.name === 'system_chat' && p.data && p.data.text === 'dt:w1=yes')
  const w2 = bot.packets.find((p) => p.name === 'system_chat' && p.data && p.data.text === 'dt:w2=yes')
  const gap = w1 && w2 ? w2.t - w1.t : -1
  t.check('dates: wait 1 second waits about 1 s', gap >= 850 && gap <= 1400, `${gap} ms`)

  // ---- math ----
  o = await group(bot, 'lmath', 'mt')
  expectAll('math', o, {
    arith: '14 20 3.5 -3', abs: '3 2 3 3 4 1', minmax: '2 5', trig: '1 1 45', more: '10 120 2 3 1 0',
    base: 'ff 255', fmt: '1,234.50', list: '16 4 2.5 60', comb: '10 20', rgb: /^#?ff0000$/i, special: /^3.14/,
    random: 'ok', rounded: '3.14',
  })
  o = await group(bot, 'lextra', 'ex')
  expectAll('more expressions', o, {
    copy: '3 4', between: 'a, b, c, d and e', codepoint: '65 B', diff: '7', parsed: '12 <none>', empty: 'yes',
    until: (v) => /^1 hour and 59 minutes/.test(v) || /^2 hours/.test(v), valueidx: '2', digits: '3', iteration: '3',
    recursive: '2', unixdate: '2023-11-14 22:13:20', maxof: '9 1',
  })

  // ---- functions and options ----
  o = await group(bot, 'lfuncs', 'fn')
  await bot.waitText('fn:waited=yes', { ms: 2000 })
  await sleep(100)
  for (const p of bot.packets) if (p.name === 'system_chat' && p.data && p.data.text === 'fn:waited=yes') o.waited = 'yes'
  expectAll('functions', o, {
    sq: '49', default: 'Hello Bob', given: 'Hi Bob', rec: '120', list: '8 and 2', listsum: '10', copy: '4',
    tell: 'hey', local: 'local', opt: '[L] 7', optmath: '8', waited: 'yes',
  })

  // ---- structures ----
  o = await group(bot, 'lstruct', 'st')
  expectAll('structures', o, {
    notreached: null, caught: /division by zero/i, caughtfn: /division by zero/i, clear: '<none>',
    tern: 'yes', tern2: 'no', default: 'fb', inline: 'yes', inlineno: null, unless: 'yes', quoted: 'go if you can',
    parseif: 'yes', parseifno: null, elif: 'two', and: 'yes', or: 'yes', whether: 'true',
  })
  t.check('structures: caught errors are not logged', !srv.lines.some((l) => /division by zero/i.test(l)))

  // ---- loops ----
  o = await group(bot, 'lloops', 'lp')
  expectAll('loops', o, {
    while: '3', times: '6', zero: null, list: '4', index: '1, 2, 3, 4 and 5', literal: 'a-b-c', nested: '11, 12, 21 and 22',
    whileexit: '4', split: 'p and q', wait: '3',
  })
  let since = Date.now()
  bot.command('lstop')
  t.check('loops: stop ends the trigger', !!await bot.waitText('lp:beforestop=yes', { since }) &&
    (await sleep(300), !chatSince(bot, since).includes('lp:afterstop=bad')))

  // ---- SkBee utilities ----
  o = await group(bot, 'lskbee', 'sb')
  expectAll('skbee utilities', o, {
    commas: '1,234,567', commas2: '1,234,567.89', abbr: '1.5K 2M', roman: 'MCMXCIV 14', hex: 'ff 255', json: '"a \\"q\\""',
    uuid: /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/, pose: 'standing', locale: /^[a-z]{2}_[a-z]{2}$/,
  })

  // ---- NBT ----
  o = await group(bot, 'lnbt', 'nb')
  expectAll('nbt', o, {
    int: '5', str: 'blade', wrongtype: '<none>', snbt: (v) => /lvl:3/.test(v) && /name:"sword"/.test(v) && /c:3b/.test(v) && !/power/.test(v),
    keys: (v) => ['id', 'lvl', 'c'].every((k) => v.includes(k)), has: 'yes', hasnot: 'yes',
  })
  since = Date.now()
  srv.console('give Tester stick 1')
  await bot.waitPacket('container_set_slot', (d) => d.item && d.item.name === 'stick', 5000, since)
  await sleep(300)
  o = await group(bot, 'ltool', 'nb', 5000)
  t.check('nbt: int tag on the held item', o.tool === '7', `got ${o.tool}`)
  t.check('nbt: custom model data on the held item', o.cmd === '42', `got ${o.cmd}`)

  // ---- runaway guard ----
  let mark = srv.mark()
  since = Date.now()
  bot.command('lrunaway')
  const guard = await srv.waitLog(/ran more than [\d,]+ instructions/, 10000, mark)
  t.check('runaway: endless while loop is stopped with an error', !!guard, guard || 'no guard line in log')
  t.check('runaway: line after the loop never runs', !chatSince(bot, since).includes('rw:after=bad'))
  since = Date.now()
  bot.command('lping')
  const pong = await bot.waitText('rw:pong=yes', { since, ms: 3000 })
  t.check('runaway: server still answers right after', !!pong, pong ? `${Date.now() - since} ms` : 'no reply')
  mark = srv.mark()
  bot.command('ldeep')
  const deep = await srv.waitLog(/64|deep|recursion/i, 5000, mark)
  t.check('runaway: endless recursion stops with an error', !!deep, deep || 'nothing in log')
  since = Date.now()
  bot.command('lping')
  t.check('runaway: server still answers after the recursion', !!await bot.waitText('rw:pong=yes', { since, ms: 3000 }))

  // ---- /carve admin ----
  t.check('admin: help', !!await say(bot, 'carve help', '/carve timings'))
  t.check('admin: list', !!await say(bot, 'carve list', 'language.carve'))
  t.check('admin: info', !!await say(bot, 'carve info', 'functions'))
  t.check('admin: info <script>', !!await say(bot, 'carve info language', 'lsq'))
  t.check('admin: vars set', !!await say(bot, 'carve vars set lang_admin 41', 'Set {lang_admin} to 41'))
  bot.command('carve run add 1 to {lang_admin}')
  await sleep(300)
  t.check('admin: run + vars get', !!await say(bot, 'carve vars get lang_admin', '42'))
  t.check('admin: run calls a script function', !!await say(bot, 'carve run send "adm:%lsq(9)%"', 'adm:81'))
  t.check('admin: run shows parse errors', !!await say(bot, 'carve run brodcast "x"', 'Did you mean'))
  t.check('admin: vars list', !!await say(bot, 'carve vars list lang_', '{lang_admin}'))
  t.check('admin: vars delete', !!await say(bot, 'carve vars delete lang_admin', 'Deleted 1'))
  t.check('admin: timings on', !!await say(bot, 'carve timings on', 'Timings on'))
  await group(bot, 'lmath', 'mt')
  t.check('admin: timings shows the trigger', !!await say(bot, 'carve timings', 'language.carve'))
  t.check('admin: timings reset', !!await say(bot, 'carve timings reset', '[Carve]'))
  t.check('admin: timings off', !!await say(bot, 'carve timings off', 'Timings off'))
  mark = srv.mark()
  srv.console('carve run broadcast "adm:console"')
  t.check('admin: run from the console', !!await bot.waitText('adm:console', { ms: 4000 }))

  // Broken script: reload must report Skript's layout to the player.
  fs.writeFileSync(path.join(scriptsDir(), 'lbroken.carve'), 'command /lbrokenx:\n    trigger:\n        sned "hi" to player\n        send "lb:ok=yes"\n')
  since = Date.now()
  bot.command('carve reload lbroken')
  const enc = await bot.waitText('Encountered 1 error while reloading lbroken.carve!', { since, ms: 6000 })
  await sleep(300)
  const msgs = chatSince(bot, since)
  const all = msgs.join('\n')
  console.log('reload output:\n' + msgs.map((m) => '  | ' + m.replace(/\n/g, '\n  | ')).join('\n'))
  t.check('errors: reload starts with "Reloading lbroken.carve..."', msgs.some((m) => m.includes('[Carve] Reloading lbroken.carve...')))
  t.check('errors: "Line 3: (lbroken.carve)"', /\[Carve\] Line 3: \(lbroken\.carve\)/.test(all))
  t.check('errors: "Can\'t understand this effect" with the line', /\n? {4}Can't understand this effect: 'sned "hi" to player'/.test(all))
  t.check('errors: "Line: sned ..."', /\n? {4}Line: sned "hi" to player/.test(all))
  t.check('errors: "Did you mean" lists send first', /Did you mean: send /.test(all))
  t.check('errors: summary line with time', !!enc && /Encountered 1 error while reloading lbroken\.carve! \(\d+ ?ms\)/.test(enc.text), enc ? enc.text : 'none')
  t.check('admin: errors <script>', !!await say(bot, 'carve errors lbroken', ['Line 3:', '(lbroken.carve)']))
  t.check('admin: list shows the error count', !!await say(bot, 'carve list', 'lbroken'))
  t.check('admin: disable', !!await say(bot, 'carve disable lbroken', ['Successfully disabled', 'lbroken.carve']))
  t.check('admin: disable renames the file', fs.existsSync(path.join(scriptsDir(), '-lbroken.carve')))
  t.check('admin: list disabled', !!await say(bot, 'carve list disabled', 'lbroken'))
  t.check('admin: enable with errors loads and reports them', !!await say(bot, 'carve enable lbroken', 'Encountered 1 error while enabling lbroken.carve!'))
  const until = Date.now() + 3000
  while (Date.now() < until && !fs.existsSync(path.join(scriptsDir(), 'lbroken.carve'))) await sleep(100)
  t.check('admin: enable renames it back', fs.existsSync(path.join(scriptsDir(), 'lbroken.carve')) && !fs.existsSync(path.join(scriptsDir(), '-lbroken.carve')))
  fs.writeFileSync(path.join(scriptsDir(), 'lbroken.carve'), 'on chat:\n    if message is "lbfix":\n        send "lb:fixed=yes" to player\n')
  t.check('admin: reload <script> after fixing', !!await say(bot, 'carve reload lbroken', ['Successfully reloaded', 'lbroken.carve']))
  since = Date.now()
  bot.chat('lbfix')
  t.check('admin: fixed script runs', !!await bot.waitText('lb:fixed=yes', { since }))
  t.check('admin: unknown script', !!await say(bot, 'carve reload nosuch', "Can't find the script"))
  t.check('admin: full reload', !!await say(bot, 'carve reload', 'Successfully reloaded', 8000))

  // ---- variables after a restart ----
  bot.quit()
  await sleep(1500) // let the save timer and the quit settle
  stopped = true
  t.check('server stops cleanly before restart', await srv.stop())
  // The globals live in carve.redb; the restart below reads them back.
  const dataDir = path.join(t.dir, 'plugins', 'data', 'carve')
  t.check('carve.redb written, no variables.json',
    fs.existsSync(path.join(dataDir, 'carve.redb')) && !fs.existsSync(path.join(dataDir, 'variables.json')))
  srv = await startServer({ dir: t.dir, port: t.port, scripts: [SCRIPT], ops: ['Tester'], keepVariables: true })
  stopped = false
  bot = await joinBot('Tester', { port: t.port })
  await sleep(500)
  const after = await group(bot, 'lvshow', 'lr')
  expectAll('variables after restart', after, {
    g: 'keep', n: '3', list: 'a, b and c', date: '2023-11-14 22:13:20', span: '1 minute and 30 seconds',
    nbt: '1', bool: 'true', player: 'Tester', dflt: 'dflt',
  })

  const errs = bot.packets.filter((p) => p.error)
  t.check('every decoded packet parsed', errs.length === 0, errs.slice(0, 3).map((p) => `${p.name}: ${p.error}`).join('; '))
})
