// Wire helpers for the 26.3 (protocol 777) play state: a byte reader, network
// NBT, text components and the registries the decoders map ids to.
'use strict'
const fs = require('fs')
const path = require('path')

const REF = process.env.PUMPKIN_REF_DIR ||
  path.resolve(__dirname, '../../../../pumpscript/.ref/pumpkin/assets')

const cache = {}
function asset (file) {
  if (!(file in cache)) {
    try { cache[file] = JSON.parse(fs.readFileSync(path.join(REF, file), 'utf8')) } catch { cache[file] = null }
  }
  return cache[file]
}

// name -> id maps turned around, built on first use
const lookups = {}
function byId (kind) {
  if (lookups[kind]) return lookups[kind]
  const out = []
  if (kind === 'entity' || kind === 'item') {
    const j = asset(kind === 'entity' ? 'entities.json' : 'items.json') || {}
    for (const [name, v] of Object.entries(j)) out[v.id] = name
  } else if (kind === 'block_state') {
    const j = asset('blocks.json')
    for (const b of (j && j.blocks) || []) for (const s of b.states || []) out[s.id] = b.name
  } else {
    const file = { sound: 'sounds.json', particle: 'particles.json', screen: 'screens.json' }[kind]
    const j = asset(file)
    if (Array.isArray(j)) j.forEach((n, i) => { out[i] = n })
  }
  lookups[kind] = out
  return out
}
const nameOf = (kind, id) => byId(kind)[id] || null

function packetNames (state, dir = 'clientbound') {
  const j = asset('packets.json')
  const names = []
  if (j) for (const [n, id] of Object.entries(j[dir][state])) names[id] = n
  return names
}
function packetIds (state, dir = 'serverbound') {
  const j = asset('packets.json')
  return j ? { ...j[dir][state] } : {}
}

class Reader {
  constructor (buf, off = 0) { this.b = buf; this.o = off }
  get left () { return this.b.length - this.o }
  need (n) { if (this.o + n > this.b.length) throw new RangeError('packet too short') }
  u8 () { this.need(1); return this.b[this.o++] }
  i8 () { this.need(1); return this.b.readInt8(this.o++) }
  bool () { return this.u8() !== 0 }
  i16 () { this.need(2); const v = this.b.readInt16BE(this.o); this.o += 2; return v }
  u16 () { this.need(2); const v = this.b.readUInt16BE(this.o); this.o += 2; return v }
  i32 () { this.need(4); const v = this.b.readInt32BE(this.o); this.o += 4; return v }
  i64 () { this.need(8); const v = this.b.readBigInt64BE(this.o); this.o += 8; return v }
  f32 () { this.need(4); const v = this.b.readFloatBE(this.o); this.o += 4; return v }
  f64 () { this.need(8); const v = this.b.readDoubleBE(this.o); this.o += 8; return v }
  bytes (n) { this.need(n); const v = this.b.subarray(this.o, this.o + n); this.o += n; return v }
  varint () {
    let n = 0; let shift = 0; let b
    do { b = this.u8(); n |= (b & 0x7f) << shift; shift += 7 } while (b & 0x80)
    return n
  }
  varlong () {
    let n = 0n; let shift = 0n; let b
    do { b = this.u8(); n |= BigInt(b & 0x7f) << shift; shift += 7n } while (b & 0x80)
    return BigInt.asIntN(64, n)
  }
  str () { const n = this.varint(); return this.bytes(n).toString('utf8') }
  uuid () { const h = this.bytes(16).toString('hex'); return `${h.slice(0, 8)}-${h.slice(8, 12)}-${h.slice(12, 16)}-${h.slice(16, 20)}-${h.slice(20)}` }
  opt (fn) { return this.bool() ? fn() : null }
  pos () {
    const v = this.b.readBigInt64BE((this.need(8), this.o)); this.o += 8
    return {
      x: Number(BigInt.asIntN(26, v >> 38n)),
      y: Number(BigInt.asIntN(12, v)),
      z: Number(BigInt.asIntN(26, v >> 12n)),
    }
  }
  // Packed low-precision vector (velocity since 1.21.9).
  lpvec () {
    const first = this.u8()
    if (first === 0) return { x: 0, y: 0, z: 0 }
    const second = this.u8()
    const mid = this.i32()
    const packed = BigInt(first | (second << 8)) | (BigInt(mid) << 16n)
    const header = Number(packed & 7n)
    let scale = header & 3
    if (header & 4) scale = (this.varint() << 2) | (header & 3)
    const q = (sh) => Number((packed >> sh) & 0x7fffn)
    const f = (v) => ((v / 32766) - 0.5) / 0.5 * scale
    return { x: f(q(3n)), y: f(q(18n)), z: f(q(33n)) }
  }
  // Network NBT: a type byte, then the payload with no root name.
  nbt () {
    const t = this.u8()
    return t === 0 ? null : this.tag(t)
  }
  tag (t) {
    switch (t) {
      case 1: return this.i8()
      case 2: return this.i16()
      case 3: return this.i32()
      case 4: return this.i64()
      case 5: return this.f32()
      case 6: return this.f64()
      case 7: return [...this.bytes(this.i32())]
      case 8: return this.bytes(this.u16()).toString('utf8')
      case 9: { const et = this.u8(); const n = this.i32(); const a = []; for (let i = 0; i < n; i++) a.push(this.tag(et)); return a }
      case 10: {
        const o = {}
        for (;;) {
          const ct = this.u8()
          if (ct === 0) return o
          const k = this.bytes(this.u16()).toString('utf8')
          o[k] = this.tag(ct)
        }
      }
      case 11: { const n = this.i32(); const a = []; for (let i = 0; i < n; i++) a.push(this.i32()); return a }
      case 12: { const n = this.i32(); const a = []; for (let i = 0; i < n; i++) a.push(this.i64()); return a }
      default: throw new Error(`bad NBT tag ${t}`)
    }
  }
  component () { return component(this.nbt()) }
  // Item stack (1.20.5+): count, id, then component patches. Component data
  // has no length prefix, so a stack with patches ends the readable part.
  item () {
    const count = this.varint()
    if (count <= 0) return null
    const id = this.varint()
    const add = this.varint()
    const remove = this.varint()
    const it = { id, name: nameOf('item', id), count, patched: add + remove }
    if (add > 0) { it.partial = true; this.o = this.b.length; return it }
    for (let i = 0; i < remove; i++) this.varint()
    return it
  }
}

// Turn an NBT text component into {text, color, bold..., click, hover, parts}.
// `text` is the plain string with every child and translation argument joined.
function component (n) {
  const out = { text: '', color: null, click: null, hover: null, raw: n }
  const walk = (c, inherit) => {
    if (c == null) return
    if (typeof c !== 'object') { out.text += String(c); return }
    if (Array.isArray(c)) { c.forEach((x) => walk(x, inherit)); return }
    const color = c.color || inherit
    if (!out.color && color) out.color = color
    for (const k of ['bold', 'italic', 'underlined', 'strikethrough', 'obfuscated']) if (c[k]) out[k] = true
    const click = c.click_event || c.clickEvent
    if (click && !out.click) out.click = { action: click.action, value: click.url || click.command || click.value || click.page || null }
    const hover = c.hover_event || c.hoverEvent
    if (hover && !out.hover) {
      const v = hover.value ?? hover.contents ?? hover.text
      out.hover = { action: hover.action, text: v && typeof v === 'object' && !hover.id ? component(v).text : (hover.id || (v == null ? null : String(v))) }
    }
    if (c.text != null) out.text += String(c.text)
    else if (c[''] != null) out.text += String(c[''])
    if (c.translate) {
      out.translate = out.translate || c.translate
      const args = c.with || []
      const lang = asset('en_us_java.json') || {}
      const fmt = lang[c.translate] || c.fallback || c.translate
      if (args.length) {
        let i = 0
        out.text += fmt.replace(/%(\d+\$)?s/g, (_, pos) => {
          const a = args[pos ? parseInt(pos) - 1 : i++]
          return a == null ? '' : component(a).text
        })
      } else out.text += fmt
    }
    if (c.extra) c.extra.forEach((x) => walk(x, color))
  }
  walk(n, null)
  return out
}

module.exports = { Reader, component, nameOf, byId, packetNames, packetIds, asset, REF }
