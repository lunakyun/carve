// Runs every in-game area and then run.js, one after another, and prints a
// summary table. Exits with code 1 if anything failed.
//
//   node tests/ingame/all.js            # every area + run.js
//   node tests/ingame/all.js combat     # only the named ones ("run" = run.js)
'use strict'
const fs = require('fs')
const path = require('path')
const { spawn } = require('child_process')

const ROOT = path.resolve(__dirname, '../..')
const AREAS = path.join(__dirname, 'areas')
const TEMPLATE = process.env.CARVE_TEMPLATE_DIR || path.resolve(ROOT, '../pumpscript/.test/server')

const suites = fs.readdirSync(AREAS).filter((f) => f.endsWith('.js')).sort()
  .map((f) => ({ name: f.slice(0, -3), file: path.join(AREAS, f) }))
suites.push({ name: 'run', file: path.join(__dirname, 'run.js') })

const only = process.argv.slice(2)
const todo = only.length ? suites.filter((s) => only.includes(s.name)) : suites

function runSuite ({ name, file }) {
  const env = { ...process.env }
  // run.js uses one fixed server folder; fall back to the template server.
  if (name === 'run' && !env.CARVE_SERVER_DIR && !fs.existsSync(path.join(ROOT, '.test', 'server'))) env.CARVE_SERVER_DIR = TEMPLATE
  return new Promise((resolve) => {
    const started = Date.now()
    const child = spawn(process.execPath, [file], { cwd: ROOT, env, stdio: ['ignore', 'pipe', 'pipe'] })
    const counts = { PASS: 0, FAIL: 0, SKIP: 0 }
    const fails = []
    let buf = ''
    const onLine = (line) => {
      const m = /^(PASS|FAIL|SKIP) {2}(.*)$/.exec(line)
      if (!m) return
      counts[m[1]]++
      if (m[1] === 'FAIL') fails.push(m[2])
      process.stdout.write(`[${name}] ${line}\n`)
    }
    child.stdout.on('data', (d) => {
      buf += d
      const lines = buf.split(/\r?\n/)
      buf = lines.pop()
      lines.forEach(onLine)
    })
    let err = ''
    child.stderr.on('data', (d) => { err += d })
    child.on('close', (code) => {
      if (buf) onLine(buf)
      // A crash with no FAIL line still counts as a failure.
      if (code && !counts.FAIL) { counts.FAIL++; fails.push(`exited with code ${code}: ${err.trim().split('\n').slice(-3).join(' | ')}`) }
      resolve({ name, ...counts, fails, secs: (Date.now() - started) / 1000 })
    })
  })
}

async function main () {
  const rows = []
  for (const s of todo) {
    console.log(`\n== ${s.name} ==`)
    rows.push(await runSuite(s))
  }
  const total = rows.reduce((a, r) => ({ PASS: a.PASS + r.PASS, FAIL: a.FAIL + r.FAIL, SKIP: a.SKIP + r.SKIP }), { PASS: 0, FAIL: 0, SKIP: 0 })
  const pad = (v, n) => String(v).padEnd(n)
  console.log('\n' + pad('area', 10) + pad('pass', 6) + pad('fail', 6) + pad('skip', 6) + 'time')
  for (const r of rows) console.log(pad(r.name, 10) + pad(r.PASS, 6) + pad(r.FAIL, 6) + pad(r.SKIP, 6) + r.secs.toFixed(0) + ' s')
  console.log(pad('total', 10) + pad(total.PASS, 6) + pad(total.FAIL, 6) + pad(total.SKIP, 6))
  for (const r of rows) for (const f of r.fails) console.log(`FAIL [${r.name}] ${f}`)
  process.exit(total.FAIL ? 1 : 0)
}

main()
