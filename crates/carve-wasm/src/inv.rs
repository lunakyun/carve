//! Inventory access through the Pumpkin plugin API.
//!
//! `open_gui` takes the GUI resource, so the menu's inventory handle is kept
//! here (one per player) to read and write the open menu later. The server
//! shares the inventory with the open window. The entry is dropped when the
//! player closes the window.

use std::cell::RefCell;
use std::collections::HashMap;

use pumpkin_plugin_api::gui::Gui;
use pumpkin_plugin_api::inventory::Inventory;
use pumpkin_plugin_api::item_stack::ItemStack;
use pumpkin_plugin_api::text::TextComponent;
use pumpkin_plugin_api::{Player, Screen};

thread_local! {
    /// Open chest menus made by scripts, by player uuid.
    static MENUS: RefCell<HashMap<String, Inventory>> = RefCell::new(HashMap::new());
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

pub fn open_chest(player: &Player, uuid: &str, rows: u8, title: TextComponent) {
    let gui = Gui::new(screen(rows), title);
    let inv = gui.get_inventory();
    player.open_gui(gui);
    // Opening a window closes the old one first, and that close event drops
    // the old entry. Insert after, so the new menu stays.
    MENUS.with(|m| m.borrow_mut().insert(uuid.to_string(), inv));
}

/// The player closed a window.
pub fn forget(uuid: &str) {
    MENUS.with(|m| m.borrow_mut().remove(uuid));
}

fn with_menu<R>(uuid: &str, f: impl FnOnce(&Inventory) -> R) -> Option<R> {
    MENUS.with(|m| m.borrow().get(uuid).map(f))
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
    let Ok(slot) = u32::try_from(slot) else {
        return;
    };
    let stack = item.map(|(id, n)| stack_of(id, n));
    if current && MENUS.with(|m| m.borrow().contains_key(uuid)) {
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

/// Take up to `count` of an item out of the 36 main slots.
pub fn remove(player: &Player, id: &str, count: u32) -> u32 {
    let key = format!("minecraft:{id}");
    let inv = player.get_inventory().as_inventory();
    let mut left = count;
    for slot in 0..inv.get_size() {
        if left == 0 {
            break;
        }
        let Some(stack) = inv.get_item(slot) else {
            continue;
        };
        if stack.get_registry_key() != key {
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
