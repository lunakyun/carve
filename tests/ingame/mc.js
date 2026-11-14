// Minimal Minecraft Java 26.3 (protocol 777) test client for Pumpkin.
// mineflayer only goes up to 26.1, and the login/config packets changed, so
// this speaks the few packets the Carve tests need. Packet ids come from
// Pumpkin's `pumpkin-data/src/generated/packet.rs`; offline mode only.
'use strict'
const net = require('net')
const zlib = require('zlib')
const crypto = require('crypto')
const { EventEmitter } = require('events')

const PROTOCOL = 777
const SB = {
  HANDSHAKE: 0, HELLO: 0, LOGIN_ACK: 3,
  C_CLIENT_INFO: 0, C_FINISH: 3, C_KEEP_ALIVE: 4, C_PONG: 5, C_KNOWN_PACKS: 7, C_ACCEPT_COC: 9,
  ACCEPT_TP: 0, CHAT_COMMAND: 7, CHAT: 9, CHUNK_BATCH_RECEIVED: 11, KEEP_ALIVE: 28, MOVE_POS: 30,
  PLAYER_ACTION: 41, PLAYER_LOADED: 44, PONG: 45, USE_ITEM_ON: 66, CLIENT_COMMAND: 12,
}
const CB = {
  L_DISCONNECT: 0, L_FINISHED: 2, L_COMPRESSION: 3,
  C_DISCONNECT: 2, C_FINISH: 3, C_KEEP_ALIVE: 4, C_PING: 5, C_KNOWN_PACKS: 15, C_COC: 20,
  CHUNK_BATCH_FINISHED: 11, DISCONNECT: 32, DISGUISED_CHAT: 33, KEEP_ALIVE: 45, PING: 62,
  PLAYER_CHAT: 66, PLAYER_POSITION: 73, RESPAWN: 84, ACTION_BAR: 89, SET_HEALTH: 106, SUBTITLE: 114,
  TITLE: 116, SYSTEM_CHAT: 124,
}

function varint (n) {
  const out = []
  n >>>= 0
  do {
    let b = n & 0x7f
    n >>>= 7
    if (n) b |= 0x80
    out.push(b)
  } while (n)
  return Buffer.from(out)
}
function readVarint (buf, off) {
  let n = 0; let shift = 0; let b
  do {
    if (off >= buf.length) return null
    b = buf[off++]
    n |= (b & 0x7f) << shift
    shift += 7
  } while (b & 0x80)
  return [n, off]
}
const str = (s) => { const b = Buffer.from(s, 'utf8'); return Buffer.concat([varint(b.length), b]) }
const u16 = (n) => { const b = Buffer.alloc(2); b.writeUInt16BE(n); return b }
const f64 = (...xs) => { const b = Buffer.alloc(8 * xs.length); xs.forEach((x, i) => b.writeDoubleBE(x, i * 8)); return b }
const f32 = (...xs) => { const b = Buffer.alloc(4 * xs.length); xs.forEach((x, i) => b.writeFloatBE(x, i * 4)); return b }
const i64 = (n) => { const b = Buffer.alloc(8); b.writeBigInt64BE(BigInt(n)); return b }
const u8 = (n) => Buffer.from([n & 0xff])
function blockPos (x, y, z) {
  const v = ((BigInt(x) & 0x3ffffffn) << 38n) | ((BigInt(z) & 0x3ffffffn) << 12n) | (BigInt(y) & 0xfffn)
  const b = Buffer.alloc(8); b.writeBigUInt64BE(v); return b
}

class Bot extends EventEmitter {
  constructor (username, { host = '127.0.0.1', port = 25565 } = {}) {
    super()
    this.username = username
    this.state = 'login'
    this.threshold = -1
    this.buf = Buffer.alloc(0)
    this.health = null
    this.pos = null
    this.log = [] // {kind, text, t}
    this.loaded = false
    this.sock = net.connect(port, host, () => this._start())
    this.sock.on('data', (d) => this._data(d))
    this.sock.on('error', (e) => this.emit('error', e))
    this.sock.on('close', () => { this.closed = true; this.emit('end') })
  }

  _start () {
    this._send(SB.HANDSHAKE, Buffer.concat([varint(PROTOCOL), str('127.0.0.1'), u16(25565), varint(2)]))
    const uuid = crypto.createHash('md5').update('OfflinePlayer:' + this.username).digest()
    this._send(SB.HELLO, Buffer.concat([str(this.username), uuid]))
  }

  _send (id, data = Buffer.alloc(0)) {
    if (this.closed) return
    let body = Buffer.concat([varint(id), data])
    if (this.threshold >= 0) body = Buffer.concat([varint(0), body])
    this.sock.write(Buffer.concat([varint(body.length), body]))
  }

  _data (d) {
    this.buf = Buffer.concat([this.buf, d])
    for (;;) {
      const lenR = readVarint(this.buf, 0)
      if (!lenR) return
      const [len, off] = lenR
      if (this.buf.length < off + len) return
      let pkt = this.buf.subarray(off, off + len)
      this.buf = this.buf.subarray(off + len)
      if (this.threshold >= 0) {
        const [dataLen, o2] = readVarint(pkt, 0)
        pkt = dataLen === 0 ? pkt.subarray(o2) : zlib.inflateSync(pkt.subarray(o2))
      }
      const [id, o3] = readVarint(pkt, 0)
      try { this._packet(id, pkt.subarray(o3)) } catch (e) { this.emit('error', e) }
    }
  }

  _record (kind, text) {
    const e = { kind, text, t: Date.now() }
    this.log.push(e)
    this.emit('record', e)
  }

  _packet (id, d) {
    if (this.state === 'login') {
      if (id === CB.L_COMPRESSION) this.threshold = readVarint(d, 0)[0]
      else if (id === CB.L_FINISHED) {
        this._send(SB.LOGIN_ACK)
        this.state = 'config'
        this._send(SB.C_CLIENT_INFO, Buffer.concat([str('en_us'), u8(8), varint(0), u8(1), u8(0x7f), varint(1), u8(0), u8(1)]))
      } else if (id === CB.L_DISCONNECT) this._record('kick', d.toString('utf8'))
      return
    }
    if (this.state === 'config') {
      if (id === CB.C_KNOWN_PACKS) this._send(SB.C_KNOWN_PACKS, d)
      else if (id === CB.C_KEEP_ALIVE) this._send(SB.C_KEEP_ALIVE, d)
      else if (id === CB.C_PING) this._send(SB.C_PONG, d)
      else if (id === CB.C_COC) this._send(SB.C_ACCEPT_COC)
      else if (id === CB.C_DISCONNECT) this._record('kick', d.toString('utf8'))
      else if (id === CB.C_FINISH) { this._send(SB.C_FINISH); this.state = 'play'; this.emit('play') }
      return
    }
    switch (id) {
      case CB.KEEP_ALIVE: this._send(SB.KEEP_ALIVE, d); break
      case CB.PING: this._send(SB.PONG, d); break
      case CB.CHUNK_BATCH_FINISHED: this._send(SB.CHUNK_BATCH_RECEIVED, f32(25)); break
      case CB.PLAYER_POSITION: {
        const [tp, o] = readVarint(d, 0)
        this.pos = { x: d.readDoubleBE(o), y: d.readDoubleBE(o + 8), z: d.readDoubleBE(o + 16) }
        const yaw = d.readFloatBE(o + 48); const pitch = d.readFloatBE(o + 52)
        // Pumpkin kicks a client that confirms a teleport after a newer one was
        // sent, so confirm only the latest one, once teleports stop coming.
        // 26.3 adds the position and rotation to the confirm.
        const pos = this.pos
        clearTimeout(this.tpTimer)
        this.tpTimer = setTimeout(() => {
          this._send(SB.ACCEPT_TP, Buffer.concat([varint(tp), f64(pos.x, pos.y, pos.z), f32(yaw, pitch)]))
        }, this.loaded ? 60 : 0)
        this._record('position', `${this.pos.x} ${this.pos.y} ${this.pos.z}`)
        if (!this.loaded) {
          const first = this.firstSpawn === undefined
          this.firstSpawn = false
          this.loaded = true
          setTimeout(() => this._send(SB.PLAYER_LOADED), first ? 0 : 100)
          if (first) this.emit('spawn')
        }
        break
      }
      case CB.SET_HEALTH: this.health = d.readFloatBE(0); this._record('health', String(this.health)); break
      // Text components arrive as network NBT; tests only match substrings.
      case CB.SYSTEM_CHAT: this._record(d[d.length - 1] ? 'actionbar' : 'system', text(d)); break
      case CB.PLAYER_CHAT: this._record('chat', text(d)); break
      case CB.DISGUISED_CHAT: this._record('chat', text(d)); break
      case CB.TITLE: this._record('title', text(d)); break
      case CB.SUBTITLE: this._record('subtitle', text(d)); break
      case CB.ACTION_BAR: this._record('actionbar', text(d)); break
      case CB.DISCONNECT: this._record('kick', text(d)); break
      // After a respawn the client must say "loaded" again, or Pumpkin ignores clicks.
      case CB.RESPAWN: this.loaded = false; this._record('respawn', ''); break
    }
  }

  command (cmd) { this._send(SB.CHAT_COMMAND, str(cmd.replace(/^\//, ''))) }

  chat (msg) {
    const salt = crypto.randomBytes(8)
    this._send(SB.CHAT, Buffer.concat([str(msg), i64(Date.now()), salt, u8(0), varint(0), Buffer.alloc(3), u8(0)]))
  }

  dig (x, y, z, status = 0) { this._send(SB.PLAYER_ACTION, Buffer.concat([varint(status), blockPos(x, y, z), u8(1), varint(1)])) }

  placeOn (x, y, z) {
    this._send(SB.USE_ITEM_ON, Buffer.concat([varint(0), blockPos(x, y, z), varint(1), f32(0.5, 1, 0.5), u8(0), u8(0), varint(2)]))
  }

  // Press "Respawn" on the death screen.
  respawn () { this._send(SB.CLIENT_COMMAND, varint(0)) }

  quit () { this.sock.end(); this.closed = true }

  // Resolve with the first record matching `pred` (including ones already seen after `since`).
  waitFor (pred, ms = 5000, since = 0) {
    const hit = this.log.find((e) => e.t >= since && pred(e))
    if (hit) return Promise.resolve(hit)
    return new Promise((resolve) => {
      const on = (e) => { if (pred(e)) { cleanup(); resolve(e) } }
      const timer = setTimeout(() => { cleanup(); resolve(null) }, ms)
      const cleanup = () => { clearTimeout(timer); this.off('record', on) }
      this.on('record', on)
    })
  }
}

// Pull readable strings out of a network-NBT text component.
function text (d) {
  return d.toString('utf8').replace(/[\x00-\x1f�]+/g, ' ').trim()
}

module.exports = { Bot }
