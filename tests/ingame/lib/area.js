// Tiny runner for one test area.
//
//   const t = area('items')
//   t.check('give works', ok, 'detail'); t.skip('needs X', 'reason')
//   t.cleanup(() => srv.stop())   // runs in done(), also after a crash
//   await t.done()                // prints the summary, exits 1 on any FAIL
'use strict'
const path = require('path')
const { ROOT } = require('./server.js')

function area (name, { port } = {}) {
  const results = []
  const cleanups = []
  const started = Date.now()
  let finished = false
  const t = {
    name,
    port,
    // Each area has its own server folder (gitignored through /.test/).
    dir: path.join(ROOT, '.test', 'areas', name),
    results,
    check (label, ok, detail = '') {
      results.push({ label, status: ok ? 'PASS' : 'FAIL', detail })
      console.log(`${ok ? 'PASS' : 'FAIL'}  ${label}${detail ? '  (' + detail + ')' : ''}`)
      return !!ok
    },
    skip (label, reason = '') {
      results.push({ label, status: 'SKIP', detail: reason })
      console.log(`SKIP  ${label}${reason ? '  (' + reason + ')' : ''}`)
    },
    cleanup (fn) { cleanups.unshift(fn) },
    // Run the area body; a throw counts as a FAIL and still runs the cleanups.
    async run (body) {
      try { await body(t) } catch (e) { t.check('area ran without an error', false, e.stack || String(e)) }
      return t.done()
    },
    async done () {
      if (finished) return
      finished = true
      for (const fn of cleanups) {
        try { await fn() } catch (e) { console.log(`cleanup error: ${e.message}`) }
      }
      const n = (s) => results.filter((r) => r.status === s).length
      console.log(`\n[${name}] ${n('PASS')} passed, ${n('FAIL')} failed, ${n('SKIP')} skipped in ${((Date.now() - started) / 1000).toFixed(1)} s`)
      process.exit(n('FAIL') ? 1 : 0)
    },
  }
  process.on('unhandledRejection', (e) => { t.check('no unhandled rejection', false, e && e.stack); t.done() })
  return t
}

module.exports = { area }
