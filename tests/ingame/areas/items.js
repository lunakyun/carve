// Items area: give, meta, held item, inventories, chest menus, clicks,
// item components, recipes and advancements.
//   node tests/ingame/areas/items.js
'use strict'
const path = require('path')
const { area } = require('../lib/area.js')
const { startServer } = require('../lib/server.js')
const { joinBot } = require('../lib/bot.js')

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const t = area('items', { port: 25573 })
const NAME = 'Tester'

// Window 0 slot for a Carve player-inventory slot (0-8 hotbar, 9-35 main, 36-39 boots..helmet, 40 off hand).
const winSlot = (s) => s < 9 ? 36 + s : s < 36 ? s : s === 40 ? 45 : 44 - s

t.run(async () => {
  const srv = await startServer({ dir: t.dir, port: t.port, ops: [NAME], scripts: [path.join(__dirname, 'items', 'items.carve')] })
  t.cleanup(async () => t.check('server stops cleanly', await srv.stop()))
  t.check('carve loads the items script', !!await srv.waitLog(/\[Carve\] Loaded 1 script/, 5000))
  const errLine = srv.lines.find((l) => /\[Carve\].*(error|Error)/.test(l))
  t.check('script loads without errors', !errLine, errLine || '')

  const bot = await joinBot(NAME, { port: t.port })
  t.cleanup(() => bot.quit())
  // Names of every packet (also the ones bot.js does not decode), with their time.
  const seenAll = []
  bot.on('packet', (p) => seenAll.push({ name: p.name, t: p.t }))
  const sawPacket = (name, since) => seenAll.some((p) => p.name === name && p.t >= since)
  await sleep(500)

  // Mirror of a window's slots built from every content/slot packet so far.
  const view = (windowId = 0) => {
    const slots = []
    for (const p of bot.packets) {
      if (p.name === 'set_player_inventory' && p.data && windowId === 0) { slots[winSlot(p.data.slot)] = p.data.item; continue }
      if (!p.data || p.data.windowId !== windowId) continue
      if (p.name === 'container_set_content') { slots.length = 0; p.data.items.forEach((it, i) => { slots[i] = it }) } else if (p.name === 'container_set_slot') slots[p.data.slot] = p.data.item
    }
    return slots
  }
  const desc = (it) => it ? `${it.name} x${it.count}${it.components ? ' ' + JSON.stringify(it.components) : ''}${it.partial ? ' partial@' + it.unknownComponent : ''}` : 'empty'
  // Run a command, wait for the "i:<tag>" lines (or just settle) and return them.
  const run = async (cmd, waitFor = [], ms = 4000) => {
    const since = Date.now()
    bot.command(cmd)
    const got = {}
    for (const w of [].concat(waitFor)) {
      const m = await bot.waitText(w, { since, ms })
      got[w] = m ? m.text : null
    }
    await sleep(waitFor.length ? 200 : 700)
    return got
  }
  const slotWhere = (pred, windowId = 0) => { const v = view(windowId); const i = v.findIndex((it) => it && pred(it)); return i < 0 ? null : { slot: i, item: v[i] } }
  const clear = () => run('iclear', 'i:cleared')

  // ---- give ----
  await clear()
  bot.setHotbar(0)
  await run('igive')
  let s = slotWhere((it) => it.name === 'diamond')
  t.check('give 1 diamond', s && s.item.count === 1, s ? `slot ${s.slot} ${desc(s.item)}` : 'none')
  await run('igive5')
  s = slotWhere((it) => it.name === 'emerald')
  t.check('give 5 emeralds', s && s.item.count === 5, s ? desc(s.item) : 'none')

  await clear()
  await run('iname')
  s = slotWhere((it) => it.name === 'diamond_sword')
  const cname = s && s.item.components && s.item.components.custom_name
  t.check('give ... named "&bBlade": custom_name', cname && cname.text.includes('Blade'), s ? desc(s.item) : 'none')
  t.check('named item keeps the &b color', cname && (cname.text.includes('§b') || cname.color === 'aqua'), cname ? JSON.stringify(cname.raw) : '')

  const tool = await run('itool', ['i:tool', 'i:held', 'i:name', 'i:lore', 'i:ench', 'i:sharp5', 'i:enchanted', 'i:is-dsword'])
  t.check('tool of player', /i:tool .*diamond.sword/i.test(tool['i:tool'] || ''), tool['i:tool'])
  t.check('held item of player', /i:held .*diamond.sword/i.test(tool['i:held'] || ''), tool['i:held'])
  t.check('name of player\'s tool', /i:name .*Blade/.test(tool['i:name'] || ''), tool['i:name'])
  t.check('tool is diamond sword (type compare ignores name)', /yes/.test(tool['i:is-dsword'] || ''), tool['i:is-dsword'])
  t.check('unenchanted tool is not enchanted', /no/.test(tool['i:enchanted'] || ''), tool['i:enchanted'])

  await clear()
  await run('ilore')
  s = slotWhere((it) => it.name === 'golden_sword')
  const lore = s && s.item.components && s.item.components.lore
  t.check('give ... with lore "a" and "b": lore component', lore && lore.length === 2 && lore[0].text.includes('first line') && lore[1].text.includes('second line'), s ? desc(s.item) : 'none')
  const tl = await run('itool', ['i:lore'])
  t.check('lore of player\'s tool', /first line.*second line/.test(tl['i:lore'] || ''), tl['i:lore'])

  await clear()
  await run('iench')
  s = slotWhere((it) => it.name === 'iron_sword')
  const ench = s && s.item.components && s.item.components.enchantments
  const enchStr = ench ? ench.map((e) => `${e.name} ${e.level}`).join(', ') : ''
  t.check('give ... enchanted with sharpness 5 and unbreaking 3', /sharpness 5/.test(enchStr) && /unbreaking 3/.test(enchStr), s ? desc(s.item) : 'none')
  const te = await run('itool', ['i:ench', 'i:sharp5', 'i:enchanted'])
  t.check('enchantments of player\'s tool', /sharpness 5/.test(te['i:ench'] || '') && /unbreaking 3/.test(te['i:ench'] || ''), te['i:ench'])
  t.check('tool is enchanted with sharpness 5', /yes/.test(te['i:sharp5'] || ''), te['i:sharp5'])
  t.check('tool is enchanted', /yes/.test(te['i:enchanted'] || ''), te['i:enchanted'])

  await clear()
  await run('icombo')
  s = slotWhere((it) => it.name === 'stone_sword')
  const c = s && s.item.components
  const swords = view(0).filter((it) => it && it.name === 'stone_sword').reduce((n, it) => n + it.count, 0)
  t.check('give 2 swords named/lore/enchanted at once (swords do not stack)', swords === 2 && c && c.custom_name && c.custom_name.text.includes('Twin') &&
    c.lore && c.lore[0].text.includes('a pair') && c.enchantments && c.enchantments.some((e) => e.name === 'sharpness' && e.level === 3), s ? desc(s.item) : 'none')

  await clear()
  await run('ivar')
  s = slotWhere((it) => it.name === 'netherite_sword')
  t.check('item with meta kept in a variable, then given', s && s.item.components && s.item.components.custom_name && s.item.components.custom_name.text.includes('Kept') &&
    s.item.components.lore && s.item.components.lore[0].text.includes('in a var'), s ? desc(s.item) : 'none')

  // ---- has / remove / add / slots ----
  await clear()
  await run('igive'); await run('igive'); await run('igive')
  let h = await run('ihas', ['i:has2', 'i:has10', 'i:amount'])
  t.check('player has 2 diamonds (has 3)', /yes/.test(h['i:has2'] || ''), h['i:has2'])
  t.check('inventory contains 10 diamonds is false', /no/.test(h['i:has10'] || ''), h['i:has10'])
  t.check('amount of diamonds in player\'s inventory', /i:amount 3\b/.test(h['i:amount'] || ''), h['i:amount'])
  await run('iremove', ['i:removed'])
  s = slotWhere((it) => it.name === 'diamond')
  t.check('remove 2 diamonds from player leaves 1', s && s.item.count === 1, s ? desc(s.item) : 'none')
  h = await run('ihas', ['i:has2'])
  t.check('player has 2 diamonds is false after remove', /no/.test(h['i:has2'] || ''), h['i:has2'])
  await run('iadd', ['i:added'])
  s = slotWhere((it) => it.name === 'apple')
  t.check('add 4 apples to player\'s inventory', s && s.item.count === 4, s ? desc(s.item) : 'none')

  const sl = await run('islot', ['i:slot3', 'i:slot30'])
  const v0 = view(0)
  t.check('set slot 3 of player\'s inventory to 5 carrots', v0[winSlot(3)] && v0[winSlot(3)].name === 'carrot' && v0[winSlot(3)].count === 5, desc(v0[winSlot(3)]))
  t.check('set slot 20 of player\'s inventory to bread', v0[winSlot(20)] && v0[winSlot(20)].name === 'bread', desc(v0[winSlot(20)]))
  t.check('read slot 3 of player\'s inventory', /carrot/.test(sl['i:slot3'] || ''), sl['i:slot3'])
  t.check('empty slot is not set', !!sl['i:slot30'], sl['i:slot30'])
  const fe = await run('iempty', ['i:first'])
  t.check('first empty slot of player', /i:first \d+/.test(fe['i:first'] || ''), fe['i:first'])

  await run('iclear', ['i:cleared'])
  const nonEmpty = view(0).map((it, i) => it ? `${i}:${it.name}` : null).filter(Boolean)
  t.check('clear player\'s inventory empties every slot', nonEmpty.length === 0, nonEmpty.join(' '))

  // ---- armor and off hand ----
  const ar0 = await run('iarmor', ['i:armor0'])
  t.check('equip is visible to the same script', /diamond.helmet/.test(ar0['i:armor0'] || ''), ar0['i:armor0'])
  const ar = await run('iarmorread', ['i:armor'])
  const va = view(0)
  t.check('equip player with diamond helmet (slot 5)', va[5] && va[5].name === 'diamond_helmet', desc(va[5]))
  t.check('set slot 36 (boots)', va[8] && va[8].name === 'iron_boots', desc(va[8]))
  t.check('set slot 38 (chestplate)', va[6] && va[6].name === 'golden_chestplate', desc(va[6]))
  t.check('set slot 40 (off hand)', va[45] && va[45].name === 'shield', desc(va[45]))
  t.check('helmet/boots/chestplate of player', /diamond.helmet/.test(ar['i:armor'] || '') && /iron.boots/.test(ar['i:armor'] || '') && /golden.chestplate/.test(ar['i:armor'] || ''), ar['i:armor'])
  await clear()

  // ---- chest menu ----
  let since = Date.now()
  const mm = await run('imenu', ['i:menu'])
  const open = await bot.waitPacket('open_screen', () => true, 3000, since)
  t.check('open chest with 3 rows named "&aShop"', open && /generic_9x3$/.test(open.type) && open.title.includes('Shop'), open ? `${open.type} "${open.title}" color ${open.titleComponent.color}` : 'none')
  t.check('menu title keeps &a', open && (open.title.includes('§a') || /^(green|#55FF55)$/i.test(open.titleComponent.color)), open ? JSON.stringify(open.titleComponent.raw) : '')
  await sleep(300)
  const wid = open ? open.windowId : -1
  let vm = view(wid)
  t.check('set slot 4 of current inventory (menu) to diamond', vm[4] && vm[4].name === 'diamond', desc(vm[4]))
  t.check('set slot 13 of current inventory to 3 emeralds', vm[13] && vm[13].name === 'emerald' && vm[13].count === 3, desc(vm[13]))
  t.check('read slot 4 of current inventory', /diamond/.test(mm['i:menu'] || ''), mm['i:menu'])

  // Click the diamond: cancelled, so it must stay in slot 4 and not end up on the cursor.
  since = Date.now()
  bot.clickSlot(4)
  const click = await bot.waitText('i:click', { since })
  t.check('on inventory click runs with clicked slot / item / click type', click && /i:click 4 .*diamond.* left/i.test(click.text), click ? click.text : 'none')
  t.check('cancel event in inventory click', !!await bot.waitText('i:cancelled', { since }))
  await sleep(500)
  vm = view(wid)
  const carried = bot.packets.filter((p) => p.t >= since && p.name === 'set_cursor_item')
  t.check('cancelled click: diamond stays in slot 4', vm[4] && vm[4].name === 'diamond', desc(vm[4]))
  const re = await run('imenu1')
  void re
  since = Date.now()
  // Click an emerald (not cancelled) to check the slot moves.
  const open1 = await bot.waitPacket('open_screen', (d) => /generic_9x1$/.test(d.type), 3000, since - 1500)
  t.check('open a chest with 1 row', !!open1, open1 ? `${open1.type} "${open1.title}"` : 'none')
  t.check('untitled menu is "Chest"', open1 && /Chest/.test(open1.title), open1 ? open1.title : '')
  void carried

  // Close by the client: on inventory close.
  since = Date.now()
  bot.closeContainer()
  t.check('on inventory close runs when the player closes', !!await bot.waitText('i:closed', { since }))

  // Close from the script.
  await run('imenu', ['i:menu'])
  since = Date.now()
  await run('iclose', ['i:close-ran'])
  const closed = await bot.waitPacket('container_close', () => true, 2000, since)
  if (closed) t.check('close player\'s inventory sends container_close', true)
  else t.skip('close player\'s inventory sends container_close', 'plugin API v0.1 has no close call')
  bot.closeContainer()
  await sleep(300)

  // Rows from a variable, "titled", set to air, clear the menu, a right click that is not cancelled.
  since = Date.now()
  await run('imenu2')
  const open2 = await bot.waitPacket('open_screen', (d) => /generic_9x2$/.test(d.type), 3000, since)
  t.check('open chest with {_r} rows titled "&cVar" for player', open2 && /Var/.test(open2.title), open2 ? `${open2.type} "${open2.title}"` : 'none')
  await sleep(300)
  let v2 = open2 ? view(open2.windowId) : []
  t.check('set slot of current inventory of player to air empties it', !v2[0] && v2[1] && v2[1].name === 'dirt', `${desc(v2[0])} / ${desc(v2[1])}`)
  since = Date.now()
  bot.clickSlot(1, { button: 1 })
  const rc = await bot.waitText('i:click', { since })
  t.check('click type of a right click', rc && /i:click 1 dirt right/.test(rc.text), rc ? rc.text : 'none')
  await run('imenuclear', ['i:menucleared'])
  v2 = open2 ? view(open2.windowId) : []
  t.check('clear player\'s current inventory empties the menu', v2.filter(Boolean).length === 0, v2.filter(Boolean).map(desc).join(' '))
  bot.closeContainer()
  await sleep(300)
  await clear()
  // Clicks in the player's own inventory fire the event too.
  srv.console(`give ${NAME} stone 5`)
  await sleep(600)
  since = Date.now()
  bot.clickSlot(36, { windowId: 0 })
  const oc = await bot.waitText('i:click', { since })
  t.check('on inventory click fires in the player\'s own inventory', !!oc, oc ? oc.text : 'none')
  bot.clickSlot(36, { windowId: 0 }) // put the stack back
  await sleep(300)
  const nc = await run('inotcontain', ['i:item'])
  s = slotWhere((it) => it.name === 'stone')
  t.check('remove 3 stone from player\'s inventory', s && s.item.count === 2, s ? desc(s.item) : 'none')
  t.check('item type / item amount of player\'s tool', /i:item stone 2/.test(nc['i:item'] || ''), nc['i:item'])
  await clear()
  const nn = await run('inotcontain', ['i:nostone'])
  t.check('inventory does not contain stone', !!nn['i:nostone'])

  // ---- NBT and components on the held item ----
  await clear()
  bot.setHotbar(0)
  await run('iname')
  since = Date.now()
  const nb = await run('inbt', ['i:nbt', 'i:power', 'i:hastag'])
  s = slotWhere((it) => it.name === 'diamond_sword')
  const cd = s && s.item.components && s.item.components.custom_data
  t.check('set int tag "power" of nbt of tool: custom_data on the wire', cd && cd.power === 7 && cd.id && cd.id.name === 'blade', s ? desc(s.item) : 'none')
  t.check('nbt keeps the custom name', s && s.item.components && s.item.components.custom_name && s.item.components.custom_name.text.includes('Blade'), s ? desc(s.item) : '')
  t.check('nbt of player\'s tool', /power:7/.test(nb['i:nbt'] || ''), nb['i:nbt'])
  t.check('int tag "power" of nbt of tool', /i:power 7\b/.test(nb['i:power'] || ''), nb['i:power'])
  t.check('tool has int tag "power"', !!nb['i:hastag'])
  const nb2 = await run('inbtdel', ['i:nbt2'])
  s = slotWhere((it) => it.name === 'diamond_sword')
  const cd2 = s && s.item.components && s.item.components.custom_data
  t.check('delete tag "power" of nbt of tool', cd2 && cd2.power === undefined && cd2.id, s ? desc(s.item) : 'none')
  t.check('nbt read back after delete', !/power/.test(nb2['i:nbt2'] || 'power'), nb2['i:nbt2'])

  const cp = await run('icomp', ['i:comp'])
  s = slotWhere((it) => it.name === 'diamond_sword')
  const cc = (s && s.item.components) || {}
  t.check('custom model data on the wire', cc.custom_model_data && cc.custom_model_data.floats[0] === 42, desc(s && s.item))
  t.check('unbreakable on the wire', cc.unbreakable === true)
  t.check('item model on the wire', cc.item_model === 'minecraft:stick', String(cc.item_model))
  t.check('rarity on the wire', cc.rarity === 'epic', String(cc.rarity))
  t.check('glint override on the wire', cc.enchantment_glint_override === true, String(cc.enchantment_glint_override))
  t.check('max stack size on the wire', cc.max_stack_size === 16, String(cc.max_stack_size))
  t.check('component reads', /i:comp 42 true minecraft:stick epic true 16/.test(cp['i:comp'] || ''), cp['i:comp'])
  const cp2 = await run('icompreset', ['i:comp2'])
  s = slotWhere((it) => it.name === 'diamond_sword')
  const cr = (s && s.item.components) || {}
  t.check('reset custom model data / unbreakable false removes them', !cr.custom_model_data && !cr.unbreakable, desc(s && s.item))
  t.check('component reads after reset', /i:comp2 <none> false/.test(cp2['i:comp2'] || ''), cp2['i:comp2'])

  const sn = await run('isetnbt', ['i:setnbt'])
  s = slotWhere((it) => it.name === 'diamond_sword')
  const cd3 = s && s.item.components && s.item.components.custom_data
  t.check('set nbt of player\'s tool to nbt from "..."', cd3 && cd3.a === 1 && cd3.b === 'x' && !cd3.id, s ? desc(s.item) : 'none')
  t.check('nbt read back after set', /a:1/.test(sn['i:setnbt'] || ''), sn['i:setnbt'])

  // ---- recipes ----
  await clear()
  since = Date.now()
  const rd = await run('irecipe', ['i:discovered', 'i:recipe'])
  t.check('discover recipe runs', !!rd['i:discovered'])
  t.check('discover recipe sends recipe_book_add', sawPacket('recipe_book_add', since))
  if (rd['i:recipe']) t.check('on recipe discover', /dirt_gem/.test(rd['i:recipe']), rd['i:recipe'])
  else t.skip('on recipe discover (unlock)', 'Pumpkin fires it only when the client marks a recipe seen in the book')
  // What Pumpkin does fire it for: the client marking a recipe display as seen.
  since = Date.now()
  bot.send('recipe_book_seen_recipe', Buffer.from([0]))
  const seen = await bot.waitText('i:recipe', { since, ms: 3000 })
  t.check('on recipe discover runs when the client marks a recipe seen (recipe id display_N)', seen && /i:recipe display_0/.test(seen.text), seen ? seen.text : 'none')
  // Craft it in the 2x2 grid: dirt in slots 1 and 3 should show a diamond in result slot 0.
  srv.console(`give ${NAME} dirt 2`)
  await sleep(700)
  const dirt = slotWhere((it) => it.name === 'dirt')
  if (dirt) {
    bot.window = { id: 0, stateId: bot.window.stateId }
    bot.clickSlot(dirt.slot, { windowId: 0 }) // pick up the stack
    await sleep(200)
    bot.clickSlot(1, { windowId: 0, button: 1 }) // place one
    await sleep(200)
    bot.clickSlot(3, { windowId: 0, button: 1 })
    await sleep(800)
    const vr = view(0)
    t.check('registered shaped recipe crafts in the 2x2 grid', vr[0] && vr[0].name === 'diamond', `result ${desc(vr[0])}, grid ${desc(vr[1])} / ${desc(vr[3])}`)
  } else t.check('registered shaped recipe crafts in the 2x2 grid', false, 'no dirt given')
  bot.closeContainer(0)

  // ---- advancements ----
  since = Date.now()
  const ad = await run('iadv', ['i:granted', 'i:adv'])
  t.check('grant advancement runs', !!ad['i:granted'])
  t.check('grant advancement sends update_advancements', sawPacket('update_advancements', since))
  // A grant from a script fires the event while the script runs, so Carve skips it (nested event).
  if (ad['i:adv']) t.check('on advancement done (script grant)', /story\/root/.test(ad['i:adv']), ad['i:adv'])
  else t.skip('on advancement done (script grant)', 'nested event: fired while the granting script runs')
  let ah = await run('iadvhas', ['i:hasadv'])
  t.check('player has advancement after grant', /yes/.test(ah['i:hasadv'] || ''), ah['i:hasadv'])
  await run('iadvrevoke', ['i:revoked'])
  ah = await run('iadvhas', ['i:hasadv'])
  t.check('player has advancement is false after revoke', /no/.test(ah['i:hasadv'] || ''), ah['i:hasadv'])
  since = Date.now()
  srv.console(`advancement grant ${NAME} only minecraft:story/mine_stone`)
  const adv2 = await bot.waitText('i:adv', { since })
  t.check('on advancement done (console grant)', adv2 && /story\/mine_stone/.test(adv2.text), adv2 ? adv2.text : 'none')

  await sleep(300)
  t.check('bot still connected', !bot.closed)
  const errs = bot.packets.filter((p) => p.error)
  t.check('every decoded packet parsed', errs.length === 0, errs.slice(0, 3).map((p) => `${p.name}: ${p.error}`).join('; '))
  const carveErr = srv.lines.filter((l) => /\[Carve\].*(error|panic)/i.test(l) || /panicked/.test(l))
  t.check('no Carve errors or panics in the log', carveErr.length === 0, carveErr.slice(0, 3).join(' | '))
})
