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
                let e = if self.is_word(0, "at") {
                    self.pos += 1;
                    let e = self.primary()?;
                    if !Ty::Location.accepts(e.ty(self.env)) {
                        return Err("'highest block at' needs a location".into());
                    }
                    e
                } else {
                    self.of_arg(Ty::Location, "highest block")?
                };
                mk(if lw == "highest" { W4Op::Highest } else { W4Op::Lowest }, vec![e])
            }
            _ => Ok(None),
        }
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
