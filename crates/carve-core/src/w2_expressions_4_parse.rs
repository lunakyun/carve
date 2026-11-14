//! Parsing of the `w2_expressions_4` expressions. This file is a child module
//! of `expr`, so it can use the parser's private state.

use super::{Expr, P, Ty};
use crate::w2_expressions_4::{W4, W4Op};

fn mk(op: W4Op, args: Vec<Expr>) -> Result<Option<Expr>, String> {
    Ok(Some(Expr::W4(Box::new(W4 { op, args }))))
}

impl P<'_> {
    /// Words at `pos..` equal `words` in order. Marks the end as hit when the
    /// tokens ran out, so the pattern matcher can try a longer slice.
    fn words(&self, words: &[&str]) -> bool {
        words.iter().enumerate().all(|(i, w)| self.is_word(i, w))
    }

    /// `of <expr>` of the given type.
    fn of_arg(&mut self, ty: Ty, what: &str) -> Result<Expr, String> {
        if !self.is_word(0, "of") {
            return Err(format!("expected 'of' after '{what}'"));
        }
        self.pos += 1;
        let e = self.primary()?;
        if !ty.accepts(e.ty(self.env)) {
            return Err(format!("'{what}' needs a {}, got {}", ty.name(), e.ty(self.env).name()));
        }
        Ok(e)
    }

    /// The expressions of this batch. Returns `None` when the words are not
    /// one of them. `lw` is the lowercase word just read.
    pub(super) fn w4(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        if !matches!(
            lw,
            "not"
                | "maximum"
                | "max"
                | "item"
                | "last"
                | "key"
                | "namespaced"
                | "language"
                | "locale"
                | "cooldown"
                | "join"
                | "resource"
                | "highest"
                | "lowest"
                | "light"
                | "location"
                | "block"
                | "sky"
        ) {
            return Ok(None);
        }
        match lw {
            "not" => {
                if self.pos >= self.toks.len() {
                    self.hit_end.set(true);
                }
                let e = self.primary()?;
                mk(W4Op::Not, vec![e])
            }
            "maximum" | "max" if self.words(&["double", "value"]) => {
                self.pos += 2;
                mk(W4Op::MaxDouble, vec![])
            }
            "maximum" | "max" if self.words(&["float", "value"]) => {
                self.pos += 2;
                mk(W4Op::MaxFloat, vec![])
            }
            "maximum" | "max" if self.words(&["durability", "of"]) => {
                self.pos += 1;
                let e = self.of_arg(Ty::Item, "max durability")?;
                mk(W4Op::MaxDurability, vec![e])
            }
            "maximum" | "max" if self.words(&["item", "use", "time", "of"]) => {
                self.pos += 3;
                let e = self.of_arg(Ty::Item, "max item use time")?;
                mk(W4Op::MaxUseTime, vec![e])
            }
            "maximum" | "max" if self.words(&["freeze", "time", "of"]) => {
                self.pos += 2;
                let e = self.of_arg(Ty::Object, "max freeze time")?;
                mk(W4Op::MaxFreeze, vec![e])
            }
            "item" if self.words(&["amount", "of"]) || self.words(&["amounts", "of"]) => {
                self.pos += 1;
                let e = self.of_arg(Ty::Item, "item amount")?;
                mk(W4Op::ItemAmount, vec![e])
            }
            "item" if self.words(&["cooldown", "of"]) => {
                self.pos += 1;
                self.cooldown()
            }
            "cooldown" if self.words(&["of"]) => self.cooldown(),
            "key" if self.words(&["of"]) => {
                let e = self.of_arg(Ty::Item, "key")?;
                mk(W4Op::Key, vec![e])
            }
            "namespaced" if self.words(&["key", "of"]) => {
                self.pos += 1;
                let e = self.of_arg(Ty::Item, "namespaced key")?;
                mk(W4Op::Key, vec![e])
            }
            "language" | "locale" if self.words(&["of"]) => {
                let e = self.of_arg(Ty::Player, lw)?;
                mk(W4Op::Language, vec![e])
            }
            "last" if self.words(&["color", "of"]) || self.words(&["colour", "of"]) => {
                self.pos += 1;
                let e = self.of_arg(Ty::Text, "last color")?;
                mk(W4Op::LastColor, vec![e])
            }
            "last" if self.words(&["colors", "of"]) || self.words(&["colours", "of"]) => {
                self.pos += 1;
                let e = self.of_arg(Ty::Text, "last colors")?;
                mk(W4Op::LastColor, vec![e])
            }
            "last" if self.words(&["death", "location", "of"]) => {
                self.pos += 2;
                let e = self.of_arg(Ty::Player, "last death location")?;
                mk(W4Op::LastDeathLoc, vec![e])
            }
            "last" if self.words(&["attacker", "of"]) => {
                self.pos += 1;
                let e = self.of_arg(Ty::Player, "last attacker")?;
                mk(W4Op::LastAttacker, vec![e])
            }
            "last" if self.words(&["damage", "cause", "of"]) => {
                self.pos += 2;
                let e = self.of_arg(Ty::Player, "last damage cause")?;
                mk(W4Op::LastDamageCause, vec![e])
            }
            "last" if self.words(&["damage", "of"]) => {
                self.pos += 1;
                let e = self.of_arg(Ty::Player, "last damage")?;
                mk(W4Op::LastDamage, vec![e])
            }
            "last" if self.words(&["resource", "pack", "response"]) => {
                self.pos += 3;
                self.pack_response()
            }
            "resource" if self.words(&["pack", "response"]) => {
                self.pos += 2;
                self.pack_response()
            }
            "join" if self.words(&["message"]) => {
                self.pos += 1;
                mk(W4Op::JoinMessage, vec![])
            }
            "location" if self.is_word(0, "at") => {
                self.pos += 1;
                self.location_at()
            }
            "light" | "block" | "sky" if self.light_words(lw) => {
                self.pos += if lw == "light" { 1 } else { 2 };
                let e = self.at_location("light level")?;
                mk(
                    match lw {
                        "block" => W4Op::BlockLight,
                        "sky" => W4Op::SkyLight,
                        _ => W4Op::Light,
                    },
                    vec![e],
                )
            }
            "highest" | "lowest" => {
                let save = self.pos;
                if self.is_word(0, "solid") {
                    self.pos += 1;
                }
                if !self.is_word(0, "block") || !(self.is_word(1, "at") || self.is_word(1, "of")) {
                    self.pos = save;
                    return Ok(None);
                }
                self.pos += 1;
                let e = self.at_location("highest block")?;
                mk(if lw == "highest" { W4Op::Highest } else { W4Op::Lowest }, vec![e])
            }
            _ => Ok(None),
        }
    }

    /// `light level` / `block light level` / `sky light level` ahead.
    fn light_words(&self, lw: &str) -> bool {
        match lw {
            "light" => self.words(&["level"]) && (self.is_word(1, "at") || self.is_word(1, "of")),
            _ => self.words(&["light", "level"]) && (self.is_word(2, "at") || self.is_word(2, "of")),
        }
    }

    /// `at <location>` or `of <location>`.
    fn at_location(&mut self, what: &str) -> Result<Expr, String> {
        if !(self.is_word(0, "at") || self.is_word(0, "of")) {
            return Err(format!("expected 'at <location>' after '{what}'"));
        }
        self.pos += 1;
        let e = self.primary()?;
        if !Ty::Location.accepts(e.ty(self.env)) {
            return Err(format!("'{what}' needs a location"));
        }
        Ok(e)
    }

    /// After `location at`: `(x, y, z)` and an optional `in <world>`.
    fn location_at(&mut self) -> Result<Option<Expr>, String> {
        const HELP: &str = "use 'location at (x, y, z)' or 'location at (x, y, z) in world \"name\"'";
        if !self.eat_sym("(") {
            return Err(HELP.into());
        }
        let mut c = Vec::with_capacity(3);
        for i in 0..3 {
            let e = self.num_arg("a coordinate")?;
            c.push(Box::new(e));
            if i < 2 && !self.eat_sym(",") {
                return Err(HELP.into());
            }
        }
        if !self.eat_sym(")") {
            return Err(HELP.into());
        }
        let mut world = None;
        if self.is_word(0, "in") {
            self.pos += 1;
            let w = self.primary()?;
            if !matches!(w.ty(self.env), Ty::World | Ty::Object | Ty::Text) {
                return Err("'in' needs a world".into());
            }
            world = Some(Box::new(w));
        }
        let mut it = c.into_iter();
        let (Some(x), Some(y), Some(z)) = (it.next(), it.next(), it.next()) else {
            return Err(HELP.into());
        };
        Ok(Some(Expr::Loc(x, y, z, world)))
    }

    /// After `cooldown`: `of <item> for <player>`.
    fn cooldown(&mut self) -> Result<Option<Expr>, String> {
        let item = self.of_arg(Ty::Item, "cooldown")?;
        if !self.is_word(0, "for") {
            return Err("expected 'for <player>', like 'cooldown of ender pearl for player'".into());
        }
        self.pos += 1;
        let p = self.primary()?;
        if !Ty::Player.accepts(p.ty(self.env)) {
            return Err("'cooldown of <item> for' needs a player".into());
        }
        mk(W4Op::Cooldown, vec![item, p])
    }

    /// After `resource pack response`: an optional `of <player>`.
    fn pack_response(&mut self) -> Result<Option<Expr>, String> {
        let p = if self.is_word(0, "of") {
            self.of_arg(Ty::Player, "resource pack response")?
        } else {
            if self.env.in_function || self.env.no_player {
                return Err("'resource pack response' needs a player: use 'resource pack response of <player>'".into());
            }
            Expr::Player
        };
        mk(W4Op::PackResponse, vec![p])
    }
}
