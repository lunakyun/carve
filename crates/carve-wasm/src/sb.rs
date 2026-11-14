//! Sidebar scoreboards, boss bars and the tab list header and footer, on
//! top of the Pumpkin WIT API. The core only knows script-chosen ids and line
//! numbers, so the handles and bookkeeping live here.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};

use carve_core::scoreboards_bossbars::{BarColor, BarFlag, BarStyle, MAX_LINES};
use carve_core::PlayerRef;
use pumpkin_plugin_api::boss_bar::{BossBar, BossBarColor, BossBarDivision};
use pumpkin_plugin_api::logging::LogLevel as ApiLevel;
use pumpkin_plugin_api::scoreboard::{DisplaySlot, NumberFormat, RenderType};
use pumpkin_plugin_api::{Player, uuid};

use crate::{WasmHost, plog, text};

/// The objective every Carve sidebar uses.
const OBJECTIVE: &str = "carve_sidebar";

#[derive(Default)]
struct Sidebar {
    title: String,
    /// Line number to the score holder name that shows it.
    lines: BTreeMap<u32, String>,
}

#[derive(Default)]
struct Sb {
    bars: HashMap<String, BossBar>,
    /// Bar id to the uuids shown it, so a leaving player can be forgotten.
    viewers: HashMap<String, Vec<String>>,
    sidebars: HashMap<String, Sidebar>,
    tabs: HashMap<String, (String, String)>,
}

thread_local! {
    static SB: RefCell<Sb> = RefCell::new(Sb::default());
}

/// A player left: their sidebar and tab list are gone with their session.
pub fn forget(uuid: &str) {
    SB.with(|s| {
        let mut s = s.borrow_mut();
        s.sidebars.remove(uuid);
        s.tabs.remove(uuid);
        for v in s.viewers.values_mut() {
            v.retain(|u| u != uuid);
        }
    });
}

fn color(c: BarColor) -> BossBarColor {
    match c {
        BarColor::Pink => BossBarColor::Pink,
        BarColor::Blue => BossBarColor::Blue,
        BarColor::Red => BossBarColor::Red,
        BarColor::Green => BossBarColor::Green,
        BarColor::Yellow => BossBarColor::Yellow,
        BarColor::Purple => BossBarColor::Purple,
        BarColor::White => BossBarColor::White,
    }
}

fn division(s: BarStyle) -> BossBarDivision {
    match s {
        BarStyle::Solid => BossBarDivision::NoDivision,
        BarStyle::Notches6 => BossBarDivision::Notches6,
        BarStyle::Notches10 => BossBarDivision::Notches10,
        BarStyle::Notches12 => BossBarDivision::Notches12,
        BarStyle::Notches20 => BossBarDivision::Notches20,
    }
}

fn unknown_bar(id: &str) {
    plog(ApiLevel::Warn, &format!("no bossbar '{id}': create it first"));
}

impl WasmHost<'_> {
    fn fresh(&self, p: &PlayerRef) -> Option<Player> {
        uuid::parse(&p.uuid).and_then(|id| self.server.get_player_by_uuid(id))
    }

    pub fn sb_send_title_timed(
        &mut self,
        p: &PlayerRef,
        title: &str,
        subtitle: Option<&str>,
        fade_in: u32,
        stay: u32,
        fade_out: u32,
    ) {
        if let Some(pl) = self.get(p) {
            pl.send_title_animation(fade_in as i32, stay as i32, fade_out as i32);
            if let Some(s) = subtitle {
                pl.show_subtitle(text(s));
            }
            pl.show_title(text(title));
        }
    }

    pub fn sb_set_tab_list(&mut self, p: &PlayerRef, header: Option<&str>, footer: Option<&str>) {
        let (h, f) = SB.with(|s| {
            let mut s = s.borrow_mut();
            let cur = s.tabs.entry(p.uuid.clone()).or_default();
            if let Some(h) = header {
                cur.0 = h.to_string();
            }
            if let Some(f) = footer {
                cur.1 = f.to_string();
            }
            cur.clone()
        });
        if let Some(pl) = self.get(p) {
            pl.set_tab_list_header_footer(text(&h), text(&f));
        }
    }

    /// Make the player's sidebar objective if it is not there yet.
    fn sb_ensure(&mut self, p: &PlayerRef) -> Option<pumpkin_plugin_api::scoreboard::Scoreboard> {
        let java = self.get(p)?.as_java();
        let Some(java) = java else {
            plog(ApiLevel::Warn, "scoreboards are only supported for Java players");
            return None;
        };
        let sb = java.get_scoreboard();
        let fresh = SB.with(|s| {
            let mut s = s.borrow_mut();
            if s.sidebars.contains_key(&p.uuid) {
                false
            } else {
                s.sidebars.insert(p.uuid.clone(), Sidebar::default());
                true
            }
        });
        if fresh {
            sb.add_objective(OBJECTIVE, text(""), RenderType::Integer, Some(NumberFormat::Blank));
            sb.set_display_slot(DisplaySlot::Sidebar, OBJECTIVE);
        }
        Some(sb)
    }

    pub fn sb_sidebar_set_title(&mut self, p: &PlayerRef, title: &str) {
        let Some(sb) = self.sb_ensure(p) else { return };
        SB.with(|s| {
            if let Some(side) = s.borrow_mut().sidebars.get_mut(&p.uuid) {
                side.title = title.to_string();
            }
        });
        sb.update_objective(OBJECTIVE, text(title), RenderType::Integer, Some(NumberFormat::Blank));
    }

    pub fn sb_sidebar_set_line(&mut self, p: &PlayerRef, line: u32, line_text: &str) {
        let Some(sb) = self.sb_ensure(p) else { return };
        let (old, name) = SB.with(|s| {
            let mut s = s.borrow_mut();
            let side = s.sidebars.entry(p.uuid.clone()).or_default();
            let old = side.lines.remove(&line);
            // A score holder name is the line text, so two equal lines need
            // a different invisible tail.
            let mut name = carve_core::colors::colorize(line_text);
            while side.lines.values().any(|n| *n == name) {
                name.push_str("\u{a7}r");
            }
            side.lines.insert(line, name.clone());
            (old, name)
        });
        if let Some(old) = old {
            sb.remove_score(&old, OBJECTIVE);
        }
        // Line 1 has the highest score, so it is on top.
        let score = (MAX_LINES + 1 - line) as i32;
        sb.update_score(&name, OBJECTIVE, score, Some(NumberFormat::Blank));
    }

    pub fn sb_sidebar_clear_line(&mut self, p: &PlayerRef, line: u32) {
        let old = SB.with(|s| s.borrow_mut().sidebars.get_mut(&p.uuid).and_then(|side| side.lines.remove(&line)));
        let Some(old) = old else { return };
        if let Some(java) = self.get(p).and_then(Player::as_java) {
            java.get_scoreboard().remove_score(&old, OBJECTIVE);
        }
    }

    pub fn sb_sidebar_remove(&mut self, p: &PlayerRef) {
        SB.with(|s| s.borrow_mut().sidebars.remove(&p.uuid));
        if let Some(java) = self.get(p).and_then(Player::as_java) {
            java.reset_scoreboard();
        }
    }

    pub fn sb_bossbar_create(&mut self, id: &str, title: &str, c: BarColor, st: BarStyle, progress: f64) {
        let bar = BossBar::new(text(title), color(c), division(st));
        bar.set_health(progress as f32);
        let old = SB.with(|s| {
            let mut s = s.borrow_mut();
            s.viewers.insert(id.to_string(), Vec::new());
            s.bars.insert(id.to_string(), bar)
        });
        if let Some(old) = old {
            old.remove_all();
        }
    }

    fn with_bar(id: &str, f: impl FnOnce(&BossBar)) {
        let found = SB.with(|s| s.borrow().bars.get(id).map(|b| f(b)).is_some());
        if !found {
            unknown_bar(id);
        }
    }

    pub fn sb_bossbar_set_title(&mut self, id: &str, title: &str) {
        Self::with_bar(id, |b| b.set_title(text(title)));
    }

    pub fn sb_bossbar_set_progress(&mut self, id: &str, fraction: f64) {
        Self::with_bar(id, |b| b.set_health(fraction as f32));
    }

    pub fn sb_bossbar_set_color(&mut self, id: &str, c: BarColor) {
        Self::with_bar(id, |b| b.set_color(color(c)));
    }

    pub fn sb_bossbar_set_style(&mut self, id: &str, st: BarStyle) {
        Self::with_bar(id, |b| b.set_division(division(st)));
    }

    pub fn sb_bossbar_set_flag(&mut self, id: &str, flag: BarFlag, on: bool) {
        Self::with_bar(id, |b| {
            let mut m = b.get_metadata();
            match flag {
                BarFlag::DarkenSky => m.darken_sky = on,
                BarFlag::Fog => m.create_fog = on,
                BarFlag::Dragon => m.dragon_bar = on,
            }
            b.set_metadata(m);
        });
    }

    pub fn sb_bossbar_show(&mut self, id: &str, p: &PlayerRef) {
        let Some(pl) = self.fresh(p) else { return };
        let shown = SB.with(|s| {
            let mut s = s.borrow_mut();
            let Some(bar) = s.bars.get(id) else { return false };
            bar.add_player(pl);
            let v = s.viewers.entry(id.to_string()).or_default();
            if !v.contains(&p.uuid) {
                v.push(p.uuid.clone());
            }
            true
        });
        if !shown {
            unknown_bar(id);
        }
    }

    pub fn sb_bossbar_hide(&mut self, id: &str, p: &PlayerRef) {
        let Some(pl) = self.fresh(p) else { return };
        let hidden = SB.with(|s| {
            let mut s = s.borrow_mut();
            let Some(bar) = s.bars.get(id) else { return false };
            bar.remove_player(pl);
            if let Some(v) = s.viewers.get_mut(id) {
                v.retain(|u| *u != p.uuid);
            }
            true
        });
        if !hidden {
            unknown_bar(id);
        }
    }

    pub fn sb_bossbar_remove(&mut self, id: &str) {
        let old = SB.with(|s| {
            let mut s = s.borrow_mut();
            s.viewers.remove(id);
            s.bars.remove(id)
        });
        match old {
            Some(b) => b.remove_all(),
            None => unknown_bar(id),
        }
    }
}

/// The `Host` methods for this area, forwarding to the `sb_` ones above.
macro_rules! sb_host_methods {
    () => {
        fn send_title_timed(
            &mut self,
            p: &PlayerRef,
            title: &str,
            subtitle: Option<&str>,
            fade_in: u32,
            stay: u32,
            fade_out: u32,
        ) {
            self.inner.sb_send_title_timed(p, title, subtitle, fade_in, stay, fade_out)
        }
        fn set_tab_list(&mut self, p: &PlayerRef, header: Option<&str>, footer: Option<&str>) {
            self.inner.sb_set_tab_list(p, header, footer)
        }
        fn sidebar_set_title(&mut self, p: &PlayerRef, title: &str) {
            self.inner.sb_sidebar_set_title(p, title)
        }
        fn sidebar_set_line(&mut self, p: &PlayerRef, line: u32, text: &str) {
            self.inner.sb_sidebar_set_line(p, line, text)
        }
        fn sidebar_clear_line(&mut self, p: &PlayerRef, line: u32) {
            self.inner.sb_sidebar_clear_line(p, line)
        }
        fn sidebar_remove(&mut self, p: &PlayerRef) {
            self.inner.sb_sidebar_remove(p)
        }
        fn bossbar_create(&mut self, id: &str, title: &str, c: carve_core::scoreboards_bossbars::BarColor, s: carve_core::scoreboards_bossbars::BarStyle, progress: f64) {
            self.inner.sb_bossbar_create(id, title, c, s, progress)
        }
        fn bossbar_set_title(&mut self, id: &str, title: &str) {
            self.inner.sb_bossbar_set_title(id, title)
        }
        fn bossbar_set_progress(&mut self, id: &str, fraction: f64) {
            self.inner.sb_bossbar_set_progress(id, fraction)
        }
        fn bossbar_set_color(&mut self, id: &str, c: carve_core::scoreboards_bossbars::BarColor) {
            self.inner.sb_bossbar_set_color(id, c)
        }
        fn bossbar_set_style(&mut self, id: &str, s: carve_core::scoreboards_bossbars::BarStyle) {
            self.inner.sb_bossbar_set_style(id, s)
        }
        fn bossbar_set_flag(&mut self, id: &str, f: carve_core::scoreboards_bossbars::BarFlag, on: bool) {
            self.inner.sb_bossbar_set_flag(id, f, on)
        }
        fn bossbar_show(&mut self, id: &str, p: &PlayerRef) {
            self.inner.sb_bossbar_show(id, p)
        }
        fn bossbar_hide(&mut self, id: &str, p: &PlayerRef) {
            self.inner.sb_bossbar_hide(id, p)
        }
        fn bossbar_remove(&mut self, id: &str) {
            self.inner.sb_bossbar_remove(id)
        }
    };
}
pub(crate) use sb_host_methods;
