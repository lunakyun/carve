// Server list ping (status state): resolves with the parsed status JSON.
//
//   const { statusPing } = require('../lib/status.js')
//   const st = await statusPing(25571)   // { description, players, version, ... }
'use strict'
const net = require('net')

function varint (n) {
  const out = []; n >>>= 0
  do { let b = n & 0x7f; n >>>= 7; if (n) b |= 0x80; out.push(b) } while (n)
  return Buffer.from(out)
}
const frame = (id, body = Buffer.alloc(0)) => { const p = Buffer.concat([varint(id), body]); return Buffer.concat([varint(p.length), p]) }
function readVarint (b, o) {
  let n = 0; let s = 0
  for (;;) { if (o >= b.length) return null; const x = b[o++]; n |= (x & 0x7f) << s; if (!(x & 0x80)) return [n >>> 0, o]; s += 7 }
}

function statusPing (port, { host = '127.0.0.1', protocol = 777, timeout = 5000 } = {}) {
  return new Promise((resolve, reject) => {
    const sock = net.connect(port, host)
    let buf = Buffer.alloc(0)
    const timer = setTimeout(() => { sock.destroy(); reject(new Error('status ping timed out')) }, timeout)
    sock.on('connect', () => {
      const h = Buffer.from(host, 'utf8'); const pb = Buffer.alloc(2); pb.writeUInt16BE(port)
      sock.write(frame(0, Buffer.concat([varint(protocol), varint(h.length), h, pb, varint(1)])))
      sock.write(frame(0))
    })
    sock.on('data', (d) => {
      buf = Buffer.concat([buf, d])
      const len = readVarint(buf, 0); if (!len || buf.length < len[1] + len[0]) return
      const id = readVarint(buf, len[1]); const sl = readVarint(buf, id[1])
      const json = buf.toString('utf8', sl[1], sl[1] + sl[0])
      clearTimeout(timer); sock.destroy()
      try { resolve(JSON.parse(json)) } catch (e) { reject(e) }
    })
    sock.on('error', (e) => { clearTimeout(timer); reject(e) })
  })
}

module.exports = { statusPing }
