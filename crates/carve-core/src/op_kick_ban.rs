//! Admin syntax: op/deop, kick, ban/unban, `is op`, `is banned`, display and
//! tab list names, ping and IP. The parser tables live in `compile.rs` and
//! `expr.rs`; this module holds the kinds and what each one does to the host.

use crate::compile::{CondKind, EffKind};
use crate::expr::Prop;
use crate::host::Host;
use crate::value::{PlayerRef, Value};

/// An admin effect. All take a target (a player or a name) first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminEff {
    Op,
    Deop,
    Kick,
    Ban,
    BanIp,
    Unban,
    UnbanIp,
    SetDisplayName,
    ResetDisplayName,
    SetTabName,
    ResetTabName,
}

/// An admin condition on a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminCond {
    Op,
    Banned,
}

/// Effect patterns, tried before the core ones.
pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    ("op %object%", EffKind::Admin(AdminEff::Op)),
    ("deop %object%", EffKind::Admin(AdminEff::Deop)),
    (
        "kick %object% [(by reason of|because of|because|on account of|due to|for|with reason) %text%]",
        EffKind::Admin(AdminEff::Kick),
    ),
    (
        "ban ip %object% [(by reason of|because of|because|on account of|due to|for|with reason) %text%]",
        EffKind::Admin(AdminEff::BanIp),
    ),
    (
        "ban %object% by ip [(by reason of|because of|because|on account of|due to|for|with reason) %text%]",
        EffKind::Admin(AdminEff::BanIp),
    ),
    (
        "ban %object% [(by reason of|because of|because|on account of|due to|for|with reason) %text%]",
        EffKind::Admin(AdminEff::Ban),
    ),
    ("(unban|pardon) ip %object%", EffKind::Admin(AdminEff::UnbanIp)),
    ("(unban|pardon) %object%", EffKind::Admin(AdminEff::Unban)),
    (
        "set (display name|nickname) of %object% to %text%",
        EffKind::Admin(AdminEff::SetDisplayName),
    ),
    (
        "set %object%'s (display name|nickname) to %text%",
        EffKind::Admin(AdminEff::SetDisplayName),
    ),
    (
        "set tab list name of %object% to %text%",
        EffKind::Admin(AdminEff::SetTabName),
    ),
    (
        "set %object%'s tab list name to %text%",
        EffKind::Admin(AdminEff::SetTabName),
    ),
    (
        "reset (display name|nickname) of %object%",
        EffKind::Admin(AdminEff::ResetDisplayName),
    ),
    (
        "reset %object%'s (display name|nickname)",
        EffKind::Admin(AdminEff::ResetDisplayName),
    ),
    (
        "reset tab list name of %object%",
        EffKind::Admin(AdminEff::ResetTabName),
    ),
    (
        "reset %object%'s tab list name",
        EffKind::Admin(AdminEff::ResetTabName),
    ),
];

/// Condition patterns, tried before the core ones.
pub const CONDITION_PATTERNS: &[(&str, CondKind)] = &[
    (
        "%object% is [an] (op|operator)",
        CondKind::Admin(AdminCond::Op, false),
    ),
    (
        "%object% (is not|isn't) [an] (op|operator)",
        CondKind::Admin(AdminCond::Op, true),
    ),
    ("%object% is banned", CondKind::Admin(AdminCond::Banned, false)),
    (
        "%object% (is not|isn't) banned",
        CondKind::Admin(AdminCond::Banned, true),
    ),
];

/// Run an effect. `online` is the target when it is on the server; `name` is
/// always set (offline players can be banned, unbanned and opped by name).
/// `text` is the reason or the new name.
pub fn run_effect(
    host: &mut dyn Host,
    kind: AdminEff,
    name: &str,
    online: Option<&PlayerRef>,
    text: Option<&str>,
) {
    match kind {
        AdminEff::Op => host.set_op(name, true),
        AdminEff::Deop => host.set_op(name, false),
        AdminEff::Kick => {
            // Kicking someone who is not here does nothing, but say so.
            if online.is_some() {
                host.kick(name, text);
            } else {
                host.fail(&format!("kick: {name} is not online"));
            }
        }
        AdminEff::Ban => host.ban(name, text, false),
        AdminEff::BanIp => host.ban(name, text, true),
        AdminEff::Unban => host.unban(name, false),
        AdminEff::UnbanIp => host.unban(name, true),
        AdminEff::SetDisplayName => {
            if let Some(p) = online {
                host.set_display_name(p, text);
            }
        }
        AdminEff::ResetDisplayName => {
            if let Some(p) = online {
                host.set_display_name(p, None);
            }
        }
        AdminEff::SetTabName => {
            if let Some(p) = online {
                host.set_tab_list_name(p, text);
            }
        }
        AdminEff::ResetTabName => {
            if let Some(p) = online {
                host.set_tab_list_name(p, None);
            }
        }
    }
}

/// Evaluate an admin condition (before `negate`).
pub fn check(host: &mut dyn Host, kind: AdminCond, name: &str, online: Option<&PlayerRef>) -> bool {
    match kind {
        AdminCond::Op => online.is_some_and(|p| host.is_op(p)),
        AdminCond::Banned => host.is_banned(name),
    }
}

/// Read a property added by this module. `None` when it is unknown.
pub fn read_prop(host: &mut dyn Host, prop: Prop, p: &PlayerRef) -> Option<Value> {
    Some(match prop {
        Prop::DisplayName => Value::Text(host.display_name(p).unwrap_or_else(|| p.name.clone())),
        Prop::TabName => Value::Text(host.tab_list_name(p).unwrap_or_else(|| p.name.clone())),
        Prop::Ping => Value::Num(host.ping(p)?),
        Prop::Ip => Value::Text(host.ip(p)?),
        _ => return None,
    })
}
