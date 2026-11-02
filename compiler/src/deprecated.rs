//! Forms on their way out, as `docs/stability.md` promises them: a settled
//! method or function is never changed without a release that warns first.
//! While a form is listed here it still works — a call of it is the call
//! of its new form — and every use of it is warned about, with what to
//! write instead and the release it goes in.
//!
//! To retire a form (`CONTRIBUTING.md` says when): add it here, add an
//! example of its warning under `tests/deprecated/`, and say so in an item
//! of `CHANGELOG.md` beginning **Changing:**. The release after may take
//! the form, and its line here, away.

pub struct Going {
    /// A built-in type (`Str`, `Int`, `Float`, `Bool`, `Char`, `List`,
    /// `Map`, `Set`) or a namespace (`File`, `Dir`, `Path`, `Env`, `Time`,
    /// `Process`) the method or function belongs to
    pub on: &'static str,
    pub old: &'static str,
    pub new: &'static str,
    /// the release the old form goes in
    pub until: &'static str,
}

/// Nothing is going yet.
const GOING: &[Going] = &[];

/// For the suite only: with `LUME_DEPRECATION_TEST=1`, a method and a
/// namespace's function are forms on their way out, so the warning is
/// checked while the table above is empty.
const TEST: &[Going] = &[
    Going { on: "Str", old: "shout", new: "upcase", until: "9.9" },
    Going { on: "File", old: "slurp", new: "read", until: "9.9" },
];

fn testing() -> bool {
    std::env::var("LUME_DEPRECATION_TEST").is_ok()
}

/// The listed form `old` on `on`, if it is going.
pub fn going(on: &str, old: &str) -> Option<&'static Going> {
    if let Some(g) = GOING.iter().find(|g| g.on == on && g.old == old) {
        return Some(g);
    }
    if testing() {
        return TEST.iter().find(|g| g.on == on && g.old == old);
    }
    None
}

/// Whether any form named `old` is going, on anything: asked first, as it
/// costs nothing, before the type a call is on is worked out.
pub fn named(old: &str) -> bool {
    GOING.iter().any(|g| g.old == old) || (testing() && TEST.iter().any(|g| g.old == old))
}
