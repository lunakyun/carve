//! Inventory access through the Pumpkin plugin API.
//!
//! `open_gui` takes the GUI resource, so the menu's inventory handle is kept
//! here (one per player) to read and write the open menu later. The server
//! shares the inventory with the open window. The entry is dropped when the
//! player closes the window.

use std::cell::RefCell;
use std::collections::HashMap;

use carve_core::Value;
use carve_core::gaps::InvVal;
use pumpkin_plugin_api::common::ClickType;
use pumpkin_plugin_api::gui::Gui;
use pumpkin_plugin_api::inventory::Inventory;
use pumpkin_plugin_api::item_stack::ItemStack;
use pumpkin_plugin_api::{Player, Screen};

/// An open chest menu made by a script.
struct Menu {
    inv: Inventory,
    /// The inventory value it was built from (0 for `open chest ...`).
    id: u64,
    title: String,
    rows: u8,
}

thread_local! {
    /// Open chest menus made by scripts, by player uuid.
    static MENUS: RefCell<HashMap<String, Menu>> = RefCell::new(HashMap::new());
}

fn screen(rows: u8) -> Screen {
    match rows {
        1 => Screen::Generic9x1,
        2 => Screen::Generic9x2,
        3 => Screen::Generic9x3,
        4 => Screen::Generic9x4,
        5 => Screen::Generic9x5,
        _ => Screen::Generic9x6,
    }
}

/// Split a registry key into the bare id and the amount.
pub fn pair(stack: &ItemStack) -> (String, u32) {
    let key = stack.get_registry_key();
    let id = key.strip_prefix("minecraft:").unwrap_or(&key).to_string();
    (id, u32::from(stack.get_count()))
}

fn stack_of(id: &str, count: u32) -> ItemStack {
    ItemStack::new(&format!("minecraft:{id}"), count.clamp(1, 99) as u8)
}

pub fn open_chest(player: &Player, uuid: &str, rows: u8, title: &str) {
    open_menu(player, uuid, rows, title, 0);
}

fn open_menu(player: &Player, uuid: &str, rows: u8, title: &str, id: u64) {
    let gui = Gui::new(screen(rows), crate::text(title));
    let inv = gui.get_inventory();
    player.open_gui(gui);
    // Opening a window closes the old one first, and that close event drops
    // the old entry. Insert after, so the new menu stays.
    let menu = Menu {
        inv,
        id,
        title: title.to_string(),
        rows,
    };
    MENUS.with(|m| m.borrow_mut().insert(uuid.to_string(), menu));
}

/// Open a menu built from an inventory value, items with their meta.
pub fn open_value(player: &Player, uuid: &str, v: &InvVal) {
    open_menu(player, uuid, v.rows, &v.title, v.id);
    with_menu(uuid, |inv| {
        for (i, item) in v.slots.iter().enumerate() {
            if let Some(s) = crate::gaps::stack_of(item) {
                inv.set_item(i as u32, Some(s));
            }
        }
    });
}

/// Id, title and rows of the player's open menu.
pub fn menu_info(uuid: &str) -> Option<(u64, String, u8)> {
    MENUS.with(|m| m.borrow().get(uuid).map(|x| (x.id, x.title.clone(), x.rows)))
}

/// Set a slot in every open menu built from inventory value `id`.
pub fn set_by_id(id: u64, slot: u32, item: &Value) {
    MENUS.with(|m| {
        for menu in m.borrow().values().filter(|x| x.id == id) {
            if slot < menu.inv.get_size() {
                menu.inv.set_item(slot, crate::gaps::stack_of(item));
            }
        }
    });
}

/// Set a slot of the player's open menu.
pub fn set_in_menu(uuid: &str, slot: u32, item: &Value) {
    with_menu(uuid, |inv| {
        if slot < inv.get_size() {
            inv.set_item(slot, crate::gaps::stack_of(item));
        }
    });
}

/// Uuids of the players who have this menu open. `id` 0: the menu opened for `viewer`.
pub fn viewers(id: u64, viewer: Option<&str>) -> Vec<String> {
    MENUS.with(|m| {
        m.borrow()
            .iter()
            .filter(|(uuid, menu)| {
                if id == 0 {
                    viewer == Some(uuid.as_str())
                } else {
                    menu.id == id
                }
            })
            .map(|(uuid, _)| uuid.clone())
            .collect()
    })
}

/// Uuids of the players with a menu open that was built from inventory value `id`.
pub fn viewers_of(id: u64) -> Vec<String> {
    MENUS.with(|m| {
        m.borrow()
            .iter()
            .filter(|(_, x)| x.id == id && id != 0)
            .map(|(u, _)| u.clone())
            .collect()
    })
}

/// The player closed a window.
pub fn forget(uuid: &str) {
    MENUS.with(|m| m.borrow_mut().remove(uuid));
}

fn with_menu<R>(uuid: &str, f: impl FnOnce(&Inventory) -> R) -> Option<R> {
    MENUS.with(|m| m.borrow().get(uuid).map(|x| f(&x.inv)))
}

pub fn get(player: &Player, uuid: &str, current: bool, slot: i64) -> Option<(String, u32)> {
    let slot = u32::try_from(slot).ok()?;
    if current && let Some(r) = with_menu(uuid, |inv| (slot < inv.get_size()).then(|| inv.get_item(slot))) {
        return r.flatten().as_ref().map(pair);
    }
    let pi = player.get_inventory();
    let stack = match slot {
        0..=35 => pi.as_inventory().get_item(slot),
        36 => pi.get_boots(),
        37 => pi.get_leggings(),
        38 => pi.get_chestplate(),
        39 => pi.get_helmet(),
        40 => pi.get_off_hand(),
        _ => None,
    };
    stack.as_ref().map(pair)
}

pub fn set(player: &Player, uuid: &str, current: bool, slot: i64, item: Option<(&str, u32)>) {
    set_stack(player, uuid, current, slot, item.map(|(id, n)| stack_of(id, n)));
}

/// Put a stack (with whatever name and lore it has) in a slot; `None` empties it.
pub fn set_stack(player: &Player, uuid: &str, current: bool, slot: i64, stack: Option<ItemStack>) {
    let Ok(slot) = u32::try_from(slot) else {
        return;
    };
    if current && menu_info(uuid).is_some() {
        with_menu(uuid, |inv| {
            if slot < inv.get_size() {
                inv.set_item(slot, stack);
            }
        });
        return;
    }
    let pi = player.get_inventory();
    match slot {
        0..=35 => pi.as_inventory().set_item(slot, stack),
        36 => pi.set_boots(stack),
        37 => pi.set_leggings(stack),
        38 => pi.set_chestplate(stack),
        39 => pi.set_helmet(stack),
        40 => pi.set_off_hand(stack),
        _ => {}
    }
}

pub fn clear(player: &Player, uuid: &str, current: bool) {
    if current && with_menu(uuid, Inventory::clear).is_some() {
        return;
    }
    player.get_inventory().clear_all();
}

/// `click type` text: `left`, `shift left`, `double click`, ...
pub fn click_name(c: ClickType) -> &'static str {
    match c {
        ClickType::Left => "left",
        ClickType::Right => "right",
        ClickType::ShiftLeft => "shift left",
        ClickType::ShiftRight => "shift right",
        ClickType::Middle => "middle",
        ClickType::Drop => "drop",
        ClickType::ControlDrop => "control drop",
        ClickType::DoubleClick => "double click",
        ClickType::NumberKey => "number key",
        ClickType::Unknown => "unknown",
    }
}

/// Take up to `count` of an item out of the 36 main slots.
pub fn remove(player: &Player, id: &str, count: u32) -> u32 {
    let inv = player.get_inventory().as_inventory();
    let mut left = count;
    for slot in 0..inv.get_size() {
        if left == 0 {
            break;
        }
        let Some(stack) = inv.get_item(slot) else {
            continue;
        };
        // Pumpkin gives the bare key (`diamond`); accept a namespaced one too.
        let key = stack.get_registry_key();
        if key.strip_prefix("minecraft:").unwrap_or(&key) != id {
            continue;
        }
        let have = u32::from(stack.get_count());
        let take = have.min(left);
        left -= take;
        if take == have {
            inv.set_item(slot, None);
        } else {
            stack.set_count((have - take) as u8);
            inv.set_item(slot, Some(stack));
        }
    }
    count - left
}
