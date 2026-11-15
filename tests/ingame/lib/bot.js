// Test bot for areas: the mc.js client plus a decoded `packet` stream and
// player actions. mc.js stays as it is for run.js.
//
//   const bot = await joinBot('Smoke', { port })
//   bot.on('packet', (p) => ...)            // every play packet: {id, name, buffer, data?}
//   const p = await bot.waitPacket('block_update', (d) => d.block === 'stone')
'use strict'
const { Bot } = require('../mc.js')
const { Reader, nameOf, packetNames, packetIds, asset } = require('./proto.js')

const CB_NAMES = packetNames('play', 'clientbound')
const SB = packetIds('play', 'serverbound')

const GAME_EVENTS = ['no_respawn_block', 'start_raining', 'stop_raining', 'change_game_mode', 'win_game',
  'demo_event', 'arrow_hit_player', 'rain_level_change', 'thunder_level_change', 'pufferfish_sting',
  'guardian_elder_effect', 'immediate_respawn', 'limited_crafting', 'level_chunks_load_start']
const GAME_MODES = ['survival', 'creative', 'adventure', 'spectator']
const BOSS_COLORS = ['pink', 'blue', 'red', 'green', 'yellow', 'purple', 'white']
const SOUND_SOURCES = ['master', 'music', 'record', 'weather', 'block', 'hostile', 'neutral', 'player', 'ambient', 'voice', 'ui']

function soundRef (r) {
  const n = r.varint()
  if (n === 0) { const name = r.str(); r.opt(() => r.f32()); return name }
  return nameOf('sound', n - 1) || `#${n - 1}`
}

// Each decoder gets a Reader positioned after the packet id and returns plain data.
const DECODE = {
  login: (r) => ({ entityId: r.i32() }),
  system_chat: (r) => { const c = r.component(); return { ...c, overlay: r.bool() } },
  set_action_bar_text: (r) => r.component(),
  set_title_text: (r) => r.component(),
  set_subtitle_text: (r) => r.component(),
  set_titles_animation: (r) => ({ fadeIn: r.i32(), stay: r.i32(), fadeOut: r.i32() }),
  clear_titles: (r) => ({ reset: r.bool() }),
  disguised_chat: (r) => {
    const c = r.component()
    const type = r.varint()
    return { ...c, chatType: type - 1, sender: type ? r.component().text : null }
  },
  player_chat: (r) => {
    const globalIndex = r.varint(); const sender = r.uuid(); r.varint()
    r.opt(() => r.bytes(256))
    const message = r.str(); r.i64(); r.i64()
    const prev = r.varint()
    for (let i = 0; i < prev; i++) if (r.varint() === 0) r.bytes(256)
    const unsigned = r.opt(() => r.component())
    const filter = r.varint()
    if (filter === 2) { const n = r.varint(); for (let i = 0; i < n; i++) r.i64() }
    const chatType = r.varint()
    const name = r.component()
    return { globalIndex, sender, text: unsigned ? unsigned.text : message, message, chatType, senderName: name.text }
  },
  boss_event: (r) => {
    const uuid = r.uuid(); const op = r.varint()
    const d = { uuid, action: ['add', 'remove', 'progress', 'name', 'style', 'flags'][op] || op }
    if (op === 0) {
      const t = r.component()
      Object.assign(d, { text: t.text, title: t, progress: r.f32(), color: BOSS_COLORS[r.varint()], overlay: r.varint(), flags: r.u8() })
    } else if (op === 2) d.progress = r.f32()
    else if (op === 3) { const t = r.component(); d.text = t.text; d.title = t } else if (op === 4) { d.color = BOSS_COLORS[r.varint()]; d.overlay = r.varint() } else if (op === 5) d.flags = r.u8()
    return d
  },
  set_objective: (r) => {
    const d = { objective: r.str(), mode: ['add', 'remove', 'update'][r.u8()] }
    if (d.mode !== 'remove') {
      const t = r.component()
      d.text = t.text; d.display = t; d.render = r.varint() ? 'hearts' : 'integer'
    }
    return d
  },
  set_display_objective: (r) => ({ slot: r.varint(), objective: r.str() }),
  set_score: (r) => {
    const d = { entity: r.str(), objective: r.str(), value: r.varint() }
    const name = r.opt(() => r.component())
    d.display = name ? name.text : null
    return d
  },
  reset_score: (r) => ({ entity: r.str(), objective: r.opt(() => r.str()) }),
  set_player_team: (r) => {
    const d = { team: r.str(), method: ['create', 'remove', 'update', 'add_players', 'remove_players'][r.i8()] }
    if (d.method === 'create' || d.method === 'update') {
      d.display = r.component().text; d.prefix = r.component().text; d.suffix = r.component().text
      d.nametag = r.varint(); d.collision = r.varint()
      d.color = r.opt(() => r.varint()); d.options = r.i8()
    }
    if (['create', 'add_players', 'remove_players'].includes(d.method)) {
      const n = r.varint(); d.players = []
      for (let i = 0; i < n; i++) d.players.push(r.str())
    }
    return d
  },
  sound: (r) => {
    const sound = soundRef(r); const source = SOUND_SOURCES[r.varint()]
    return { sound, source, x: r.i32() / 8, y: r.i32() / 8, z: r.i32() / 8, volume: r.f32(), pitch: r.f32() }
  },
  sound_entity: (r) => ({ sound: soundRef(r), source: SOUND_SOURCES[r.varint()], entityId: r.varint(), volume: r.f32(), pitch: r.f32() }),
  stop_sound: (r) => {
    const f = r.u8()
    return { source: f & 1 ? SOUND_SOURCES[r.varint()] : null, sound: f & 2 ? r.str() : null }
  },
  level_particles: (r) => {
    const id = r.varint()
    // 26.3: id, options (no length), important, force, pos, offset, speed x3, count, mode.
    // The options length depends on the particle, so read the fixed tail from the end.
    const b = r.b; const end = b.length - 1 // last byte: randomization mode varint 0
    let countAt = -1
    for (let k = 1; k <= 5; k++) {
      const at = end - k
      if (at - 50 < r.o) break
      let ok = true
      for (let i = 0; i < k - 1; i++) if (!(b[at + i] & 0x80)) ok = false
      if (ok && !(b[at + k - 1] & 0x80)) { countAt = at; break }
    }
    if (countAt < 0) return { particle: nameOf('particle', id), id }
    const t = new Reader(b, countAt - 50)
    const important = t.bool(); const force = t.bool()
    const x = t.f64(); const y = t.f64(); const z = t.f64()
    const offset = { x: t.f32(), y: t.f32(), z: t.f32() }
    const speed = t.f32(); t.f32(); t.f32()
    return { particle: nameOf('particle', id), id, important, force, x, y, z, offset, speed, count: t.varint() }
  },
  block_update: (r) => { const p = r.pos(); const state = r.varint(); return { ...p, state, block: nameOf('block_state', state) } },
  section_blocks_update: (r) => {
    const s = r.i64()
    const sx = Number(BigInt.asIntN(22, s >> 42n)); const sz = Number(BigInt.asIntN(22, s >> 20n)); const sy = Number(BigInt.asIntN(20, s))
    const n = r.varint(); const blocks = []
    for (let i = 0; i < n; i++) {
      const v = r.varlong(); const state = Number(v >> 12n); const l = Number(v & 0xfffn)
      blocks.push({ x: sx * 16 + (l >> 8), y: sy * 16 + (l & 15), z: sz * 16 + ((l >> 4) & 15), state, block: nameOf('block_state', state) })
    }
    return { section: { x: sx, y: sy, z: sz }, blocks }
  },
  add_entity: (r) => {
    const entityId = r.varint(); const uuid = r.uuid(); const typeId = r.varint()
    const d = { entityId, uuid, typeId, type: nameOf('entity', typeId), x: r.f64(), y: r.f64(), z: r.f64() }
    d.velocity = r.lpvec()
    d.pitch = r.u8() * 360 / 256; d.yaw = r.u8() * 360 / 256; d.headYaw = r.u8() * 360 / 256
    d.data = r.varint()
    return d
  },
  remove_entities: (r) => { const n = r.varint(); const ids = []; for (let i = 0; i < n; i++) ids.push(r.varint()); return { ids } },
  set_entity_motion: (r) => ({ entityId: r.varint(), velocity: r.lpvec() }),
  entity_event: (r) => ({ entityId: r.i32(), event: r.i8() }),
  set_health: (r) => ({ health: r.f32(), food: r.varint(), saturation: r.f32() }),
  set_experience: (r) => ({ progress: r.f32(), level: r.varint(), total: r.varint() }),
  game_event: (r) => {
    const e = r.u8(); const value = r.f32()
    const d = { event: GAME_EVENTS[e] || e, value }
    if (e === 3) d.gameMode = GAME_MODES[value] || value
    return d
  },
  player_info_update: (r) => {
    const actions = r.u8(); const n = r.varint(); const players = []
    for (let i = 0; i < n; i++) {
      const p = { uuid: r.uuid() }
      if (actions & 0x01) {
        p.name = r.str()
        const props = r.varint()
        for (let j = 0; j < props; j++) { r.str(); r.str(); r.opt(() => r.str()) }
      }
      if (actions & 0x02) r.opt(() => { r.uuid(); r.i64(); r.bytes(r.varint()); r.bytes(r.varint()) })
      if (actions & 0x04) p.gameMode = GAME_MODES[r.varint()]
      if (actions & 0x08) p.listed = r.bool()
      if (actions & 0x10) p.latency = r.varint()
      if (actions & 0x20) { const c = r.opt(() => r.component()); p.displayName = c ? c.text : null; p.displayComponent = c }
      if (actions & 0x40) p.listOrder = r.varint()
      if (actions & 0x80) p.showHat = r.bool()
      players.push(p)
    }
    return { actions, players }
  },
  tab_list: (r) => { const h = r.component(); const f = r.component(); return { header: h.text, footer: f.text, headerComponent: h, footerComponent: f } },
  open_screen: (r) => { const windowId = r.varint(); const typeId = r.varint(); const t = r.component(); return { windowId, typeId, type: nameOf('screen', typeId), title: t.text, titleComponent: t } },
  container_set_content: (r) => {
    const windowId = r.varint(); const stateId = r.varint(); const n = r.varint(); const items = []
    for (let i = 0; i < n && r.left > 0; i++) items.push(r.item())
    const carried = r.left > 0 ? r.item() : undefined
    return { windowId, stateId, count: n, items, carried, partial: items.length < n }
  },
  container_set_slot: (r) => ({ windowId: r.varint(), stateId: r.varint(), slot: r.i16(), item: r.item() }),
  container_close: (r) => ({ windowId: r.varint() }),
  // Player inventory slot (0-8 hotbar, 9-35 main, ...), not a window slot.
  set_player_inventory: (r) => ({ slot: r.varint(), item: r.item() }),
  set_cursor_item: (r) => ({ item: r.item() }),
  set_held_slot: (r) => ({ slot: r.varint() }),
  set_equipment: (r) => {
    const entityId = r.varint(); const slots = {}
    const names = ['mainhand', 'offhand', 'feet', 'legs', 'chest', 'head', 'body', 'saddle']
    for (;;) {
      const s = r.i8(); slots[names[s & 0x7f] || s & 0x7f] = r.item()
      if (!(s & 0x80) || r.left === 0) break
    }
    return { entityId, slots }
  },
  explode: (r) => {
    const d = { x: r.f64(), y: r.f64(), z: r.f64(), radius: r.f32(), blockCount: r.i32() }
    d.knockback = r.opt(() => ({ x: r.f64(), y: r.f64(), z: r.f64() }))
    d.particleId = r.varint()
    d.particle = nameOf('particle', d.particleId)
    return d // particle options and sound follow; options have no length prefix
  },
  set_time: (r) => {
    const gameTime = Number(r.i64()); const n = r.varint(); const clocks = []
    for (let i = 0; i < n; i++) clocks.push({ clock: r.varint(), ticks: Number(r.varlong()), partial: r.f32(), rate: r.f32() })
    return { gameTime, clocks, dayTime: clocks.length ? clocks[0].ticks : null }
  },
  initialize_border: (r) => ({ x: r.f64(), z: r.f64(), oldSize: r.f64(), size: r.f64(), speed: Number(r.varlong()), portalBoundary: r.varint(), warningBlocks: r.varint(), warningTime: r.varint() }),
  set_border_size: (r) => ({ size: r.f64() }),
  disconnect: (r) => r.component(),
  damage_event: (r) => ({ entityId: r.varint(), sourceType: r.varint() }),
  hurt_animation: (r) => ({ entityId: r.varint(), yaw: r.f32() }),
  player_combat_kill: (r) => { const playerId = r.varint(); return { playerId, ...r.component() } },
  player_abilities: (r) => {
    const flags = r.i8()
    return { flags, invulnerable: !!(flags & 1), flying: !!(flags & 2), allowFlying: !!(flags & 4), instantBreak: !!(flags & 8), flySpeed: r.f32(), walkSpeed: r.f32() }
  },
  update_attributes: (r) => {
    const entityId = r.varint(); const n = r.varint(); const attributes = {}
    const names = attributeNames()
    for (let i = 0; i < n; i++) {
      const id = r.varint(); const value = r.f64(); const mods = r.varint(); const modifiers = []
      for (let j = 0; j < mods; j++) modifiers.push({ id: r.str(), amount: r.f64(), operation: r.i8() })
      attributes[names[id] || `#${id}`] = { value, modifiers }
    }
    return { entityId, attributes }
  },
  change_difficulty: (r) => ({ difficulty: ['peaceful', 'easy', 'normal', 'hard'][r.u8()], locked: r.bool() }),
  set_border_lerp_size: (r) => ({ oldSize: r.f64(), size: r.f64(), speed: Number(r.varlong()) }),
  update_mob_effect: (r) => {
    const entityId = r.varint(); const id = r.varint()
    return { entityId, id, effect: nameOf('effect', id), amplifier: r.varint(), duration: r.varint(), flags: r.u8() }
  },
  remove_mob_effect: (r) => { const entityId = r.varint(); const id = r.varint(); return { entityId, id, effect: nameOf('effect', id) } },
  // Byte, int, long, float, string and boolean entries are read (bytes and
  // booleans into `bytes`, the rest into `values`); any other type ends the decode.
  set_entity_data: (r) => {
    const entityId = r.varint(); const bytes = {}; const values = {}
    for (;;) {
      const index = r.u8()
      if (index === 0xff) break
      const type = r.varint()
      if (type === 0 || type === 8) bytes[index] = r.u8()
      else if (type === 1) values[index] = r.varint()
      else if (type === 2) values[index] = r.varlong()
      else if (type === 3) values[index] = r.f32()
      else if (type === 4) values[index] = r.str()
      else break
    }
    return { entityId, bytes, values }
  },
}

let ATTRIBUTE_NAMES
function attributeNames () {
  if (!ATTRIBUTE_NAMES) {
    ATTRIBUTE_NAMES = []
    for (const [name, v] of Object.entries(asset('attributes.json') || {})) ATTRIBUTE_NAMES[v.id] = name
  }
  return ATTRIBUTE_NAMES
}

// Text-ish packets that also count as `text` for waitText.
const TEXT_KINDS = { system_chat: 'system', set_action_bar_text: 'actionbar', set_title_text: 'title', set_subtitle_text: 'subtitle', player_chat: 'chat', disguised_chat: 'chat', disconnect: 'kick' }

class TestBot extends Bot {
  constructor (username, opts) {
    super(username, opts)
    this.packets = [] // decoded {name, id, data, t}
    this.entities = new Map() // entityId -> add_entity data
    this.seq = 1
    this.input = 0
    this.entityId = null
    this.window = { id: 0, stateId: 0 }
    this.keep = opts && opts.keep ? new Set(opts.keep) : null
  }

  _packet (id, d) {
    const state = this.state
    super._packet(id, d)
    if (state !== 'play') return
    const name = CB_NAMES[id] || `#${id}`
    const p = { id, name, buffer: d, t: Date.now() }
    const dec = DECODE[name]
    if (dec) {
      try { p.data = dec(new Reader(d)) } catch (e) { p.error = e.message }
      this._track(name, p.data)
      if (!this.keep || this.keep.has(name)) this.packets.push(p)
    }
    this.emit('packet', p)
  }

  _track (name, d) {
    if (!d) return
    if (name === 'login') this.entityId = d.entityId
    else if (name === 'add_entity') this.entities.set(d.entityId, d)
    else if (name === 'remove_entities') d.ids.forEach((i) => this.entities.delete(i))
    else if (name === 'open_screen') this.window = { id: d.windowId, stateId: 0, type: d.type }
    else if (name === 'container_close') this.window = { id: 0, stateId: 0 }
    else if ((name === 'container_set_content' || name === 'container_set_slot') && d.windowId === this.window.id) this.window.stateId = d.stateId
    else if (name === 'set_health') this.health = d.health
    else if (name === 'game_event' && d.gameMode) this.gameMode = d.gameMode
  }

  // Resolve with the first decoded packet `name` whose data passes `pred`, seen at or after `since`.
  waitPacket (name, pred = () => true, ms = 5000, since = 0) {
    const ok = (p) => p.name === name && p.t >= since && p.data && pred(p.data)
    const hit = this.packets.find(ok)
    if (hit) return Promise.resolve(hit.data)
    return new Promise((resolve) => {
      const on = (p) => { if (ok(p)) { done(); resolve(p.data) } }
      const timer = setTimeout(() => { done(); resolve(null) }, ms)
      const done = () => { clearTimeout(timer); this.off('packet', on) }
      this.on('packet', on)
    })
  }

  // Any text packet (chat, action bar, title...) containing every string in `parts`.
  waitText (parts, { kind, ms = 5000, since = 0 } = {}) {
    const want = [].concat(parts)
    const pred = (p) => TEXT_KINDS[p.name] && (!kind || kind === (p.name === 'system_chat' && p.data.overlay ? 'actionbar' : TEXT_KINDS[p.name])) &&
      want.every((s) => (p.data.text || '').includes(s))
    const hit = this.packets.find((p) => p.t >= since && p.data && pred(p))
    if (hit) return Promise.resolve(hit.data)
    return new Promise((resolve) => {
      const on = (p) => { if (p.data && pred(p)) { done(); resolve(p.data) } }
      const timer = setTimeout(() => { done(); resolve(null) }, ms)
      const done = () => { clearTimeout(timer); this.off('packet', on) }
      this.on('packet', on)
    })
  }

  send (name, ...parts) {
    if (!(name in SB)) throw new Error(`unknown serverbound packet ${name}`)
    this._send(SB[name], Buffer.concat(parts.map((p) => Buffer.isBuffer(p) ? p : Buffer.from(p))))
  }

  // Left-click an entity.
  attack (entityId) { this.send('attack', varint(entityId)); this.send('punch') }
  // Right-click an entity (hand 0 = main).
  interact (entityId, { hand = 0, sneaking = false } = {}) { this.send('interact', varint(entityId), varint(hand), [0], [sneaking ? 1 : 0]) }
  // Right-click air with the held item.
  useItem (hand = 0, yaw = 0, pitch = 0) { this.send('use_item', varint(hand), varint(this.seq++), f32(yaw, pitch)) }
  // Right-click face `face` (0 down, 1 up, 2 north, 3 south, 4 west, 5 east) of the block at x y z.
  useOn (x, y, z, face = 1, hand = 0) {
    this.send('use_item_on', varint(hand), blockPos(x, y, z), varint(face), f32(0.5, 1, 0.5), [0, 0], varint(this.seq++))
  }
  // Place the held block at x y z by clicking the top of the block below it.
  place (x, y, z) { this.useOn(x, y - 1, z, 1) }
  // Break the block at x y z. Creative breaks on the start packet; survival also sends "finished".
  breakBlock (x, y, z, { finish = this.gameMode !== 'creative' } = {}) {
    this.send('player_action', varint(0), blockPos(x, y, z), [1], varint(this.seq++))
    if (finish) this.send('player_action', varint(3), blockPos(x, y, z), [1], varint(this.seq++)) // 26.3 wire id for "finished"
  }
  look (yaw, pitch) { this.yaw = yaw; this.pitch = pitch; this.send('move_player_rot', f32(yaw, pitch), [1]) }
  moveTo (x, y, z, onGround = true) { this.pos = { x, y, z }; this.send('move_player_pos', f64(x, y, z), [onGround ? 1 : 0]) }
  // Walk in small steps (default 0.4 blocks every 50 ms) so the server accepts the moves.
  async walkTo (x, y, z, step = 0.4) {
    const from = this.pos || { x, y, z }
    const dist = Math.hypot(x - from.x, y - from.y, z - from.z)
    const n = Math.max(1, Math.ceil(dist / step))
    for (let i = 1; i <= n; i++) {
      this.moveTo(from.x + (x - from.x) * i / n, from.y + (y - from.y) * i / n, from.z + (z - from.z) * i / n)
      await sleep(50)
    }
  }
  setHotbar (slot) { this.send('set_carried_item', i16(slot)) }
  swapHands () { this.send('player_action', varint(7), blockPos(0, 0, 0), [0], varint(0)) }
  _input (bit, on) { this.input = on ? this.input | bit : this.input & ~bit; this.send('player_input', [this.input]) }
  sneak (on = true) { this._input(32, on) }
  sprint (on = true) {
    this._input(64, on)
    if (this.entityId != null) this.send('player_command', varint(this.entityId), varint(on ? 1 : 2), varint(0))
  }
  // Click a slot of the open container (window 0 = own inventory). mode 0 = click, 1 = shift, 2 = hotbar swap, 4 = drop.
  clickSlot (slot, { button = 0, mode = 0, windowId = this.window.id } = {}) {
    this.send('container_click', varint(windowId), varint(this.window.stateId), i16(slot), [button & 0xff], varint(mode), varint(0), [0])
  }
  closeContainer (windowId = this.window.id) { this.send('container_close', varint(windowId)); this.window = { id: 0, stateId: 0 } }
  // Start or stop flying (needs a game mode that allows flight). Flags bit 0x02 = flying.
  fly (on = true) { this.send('player_abilities', [on ? 0x02 : 0]) }
  // Drop the held item (Q); `stack` drops the whole stack. 26.3 wire ids: 4 = stack, 5 = one.
  dropItem (stack = false) { this.send('player_action', varint(stack ? 4 : 5), blockPos(0, 0, 0), [0], varint(0)) }
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
function varint (n) {
  const out = []; n >>>= 0
  do { let b = n & 0x7f; n >>>= 7; if (n) b |= 0x80; out.push(b) } while (n)
  return Buffer.from(out)
}
const f32 = (...xs) => { const b = Buffer.alloc(4 * xs.length); xs.forEach((x, i) => b.writeFloatBE(x, i * 4)); return b }
const f64 = (...xs) => { const b = Buffer.alloc(8 * xs.length); xs.forEach((x, i) => b.writeDoubleBE(x, i * 8)); return b }
const i16 = (n) => { const b = Buffer.alloc(2); b.writeInt16BE(n); return b }
function blockPos (x, y, z) {
  const v = ((BigInt(x) & 0x3ffffffn) << 38n) | ((BigInt(z) & 0x3ffffffn) << 12n) | (BigInt(y) & 0xfffn)
  const b = Buffer.alloc(8); b.writeBigUInt64BE(v); return b
}

// Connect and resolve once the bot has spawned.
async function joinBot (name, { port = 25565, host = '127.0.0.1', timeout = 20000, keep } = {}) {
  const bot = new TestBot(name, { port, host, keep })
  bot.on('error', (e) => console.log(`[${name}] error: ${e.message}`))
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`${name} did not spawn`)), timeout)
    bot.once('spawn', () => { clearTimeout(timer); resolve() })
    bot.once('end', () => { clearTimeout(timer); reject(new Error(`${name} disconnected before spawning`)) })
  })
  return bot
}

module.exports = { TestBot, joinBot, DECODE }
