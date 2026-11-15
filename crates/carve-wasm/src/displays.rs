//! Display entities through the Pumpkin plugin API.

use carve_core::skbee_particles_display::{DisplayKind, DisplaySpec};
use pumpkin_plugin_api::EntityType;
use pumpkin_plugin_api::display::{BillboardMode, ItemDisplayMode, TextAlignment};
use pumpkin_plugin_api::world::Block;
use pumpkin_plugin_api::{BlockTypeExt, DisplayEntityExt, EntityDisplayExt, ItemStack, World};

fn billboard(name: &str) -> BillboardMode {
    match name {
        "vertical" => BillboardMode::Vertical,
        "horizontal" => BillboardMode::Horizontal,
        "center" => BillboardMode::Center,
        _ => BillboardMode::Fixed,
    }
}

fn item_mode(name: &str) -> ItemDisplayMode {
    match name {
        "thirdperson_lefthand" => ItemDisplayMode::ThirdpersonLefthand,
        "thirdperson_righthand" => ItemDisplayMode::ThirdpersonRighthand,
        "firstperson_lefthand" => ItemDisplayMode::FirstpersonLefthand,
        "firstperson_righthand" => ItemDisplayMode::FirstpersonRighthand,
        "head" => ItemDisplayMode::Head,
        "gui" => ItemDisplayMode::Gui,
        "ground" => ItemDisplayMode::Ground,
        "fixed" => ItemDisplayMode::Fixed,
        _ => ItemDisplayMode::None,
    }
}

thread_local! {
    /// The display `spawn` made last, for `last spawned entity`.
    static LAST: std::cell::RefCell<Option<carve_core::EntityRef>> = const { std::cell::RefCell::new(None) };
}

/// The display spawned since the last call, if any.
pub fn take_last() -> Option<carve_core::EntityRef> {
    LAST.with(|l| l.borrow_mut().take())
}

/// Spawn the display and fill it in. `Err` is a message for the log.
pub fn spawn(
    world: &World,
    d: &DisplaySpec,
    text: impl Fn(&str) -> pumpkin_plugin_api::text::TextComponent,
) -> Result<(), String> {
    // Look up the block first, so a bad id spawns nothing.
    let block_state = match (d.kind, d.id.as_deref()) {
        (DisplayKind::Block, Some(id)) => {
            let b: Block = Block::of(id).ok_or_else(|| format!("unknown block '{id}'"))?;
            Some(b.default_state_id)
        }
        _ => None,
    };
    let kind = match d.kind {
        DisplayKind::Text => EntityType::TextDisplay,
        DisplayKind::Block => EntityType::BlockDisplay,
        DisplayKind::Item => EntityType::ItemDisplay,
    };
    let entity = world.spawn_entity(kind, (d.at.x, d.at.y, d.at.z));
    LAST.with(|l| {
        *l.borrow_mut() = Some(carve_core::EntityRef {
            uuid: pumpkin_plugin_api::uuid::to_string(entity.get_uuid()),
            kind: crate::ents::entity_name(entity.get_type()).to_string(),
        })
    });
    let base = entity.as_display().ok_or("spawned entity is not a display")?;
    if let Some(b) = d.billboard {
        base.set_billboard(billboard(b));
    }
    if let Some([x, y, z]) = d.scale {
        base.set_scale(x, y, z);
    }
    if let Some([x, y, z]) = d.translation {
        base.set_translation(x, y, z);
    }
    if let Some(v) = d.view_range {
        base.set_view_range(v);
    }
    if let Some(v) = d.shadow_radius {
        base.set_shadow_radius(v);
    }
    if let Some(v) = d.shadow_strength {
        base.set_shadow_strength(v);
    }
    if let Some(g) = d.glow {
        base.set_glow_color_override(g);
    }
    if let Some(b) = d.brightness {
        let b = i32::from(b);
        base.set_brightness((b << 4) | b);
    }
    match d.kind {
        DisplayKind::Text => {
            let t = entity
                .as_text_display()
                .ok_or("spawned entity is not a text display")?;
            t.set_text(text(d.text.as_deref().unwrap_or("")));
            if let Some(w) = d.line_width {
                t.set_line_width(w);
            }
            if let Some(b) = d.background {
                t.set_background(b);
            }
            if let Some(o) = d.opacity {
                t.set_text_opacity(o as i8);
            }
            t.set_shadow(d.shadow);
            t.set_see_through(d.see_through);
            t.set_default_background(d.default_background);
            if let Some(a) = d.alignment {
                t.set_alignment(match a {
                    "left" => TextAlignment::Left,
                    "right" => TextAlignment::Right,
                    _ => TextAlignment::Center,
                });
            }
        }
        DisplayKind::Block => {
            let b = entity
                .as_block_display()
                .ok_or("spawned entity is not a block display")?;
            if let Some(s) = block_state {
                b.set_block_state_id(s);
            }
        }
        DisplayKind::Item => {
            let i = entity
                .as_item_display()
                .ok_or("spawned entity is not an item display")?;
            if let Some(id) = d.id.as_deref() {
                i.set_item(Some(ItemStack::new(&format!("minecraft:{id}"), 1)));
            }
            if let Some(m) = d.item_mode {
                i.set_item_display_mode(item_mode(m));
            }
        }
    }
    Ok(())
}
