//! Code generator: Lume AST -> Rust source.
//!
//! Holds the type table (structs, enums, signatures), infers expression
//! types, decides borrowing, and emits Rust. The checks that must produce
//! *Lume* errors (rather than rustc errors) live here: unknown names,
//! assignment to an immutable binding, changing a field without `var self`,
//! wrong argument counts and keywords, `if` used as a value without `else`,
//! non-exhaustive `match`, `?` in a function that cannot return `None`.

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::ast::*;
use crate::error::{LumeError, Result};

pub const PRELUDE: &str = r#"#![allow(unused, non_snake_case, non_camel_case_types, unused_parens, unused_mut, arithmetic_overflow, unconditional_panic, clippy::all)]
// ---- Lume prelude ----
trait LumePow { fn lume_pow(self, e: Self) -> Self; }
impl LumePow for i64 { fn lume_pow(self, e: Self) -> Self { self.pow(e as u32) } }
impl LumePow for f64 { fn lume_pow(self, e: Self) -> Self { self.powf(e) } }
/// A string however it is held: `String`, `&str`, `&&String`, ... Comparisons
/// go through this so closure parameters need no deref guessing.
trait LumeAsStr { fn lume_as_str(&self) -> &str; }
impl LumeAsStr for str { fn lume_as_str(&self) -> &str { self } }
impl LumeAsStr for String { fn lume_as_str(&self) -> &str { self.as_str() } }
impl<T: LumeAsStr + ?std::marker::Sized> LumeAsStr for &T { fn lume_as_str(&self) -> &str { (**self).lume_as_str() } }
trait LumeLen { fn lume_len(&self) -> i64; }
impl<T> LumeLen for Vec<T> { fn lume_len(&self) -> i64 { self.len() as i64 } }
impl<T> LumeLen for [T] { fn lume_len(&self) -> i64 { self.len() as i64 } }
impl LumeLen for String { fn lume_len(&self) -> i64 { self.chars().count() as i64 } }
impl LumeLen for str { fn lume_len(&self) -> i64 { self.chars().count() as i64 } }
impl<T: LumeLen + ?std::marker::Sized> LumeLen for &T { fn lume_len(&self) -> i64 { (**self).lume_len() } }
trait LumeEmpty { fn lume_empty(&self) -> bool; }
impl<T> LumeEmpty for Vec<T> { fn lume_empty(&self) -> bool { self.is_empty() } }
impl<T> LumeEmpty for [T] { fn lume_empty(&self) -> bool { self.is_empty() } }
impl LumeEmpty for String { fn lume_empty(&self) -> bool { self.is_empty() } }
impl LumeEmpty for str { fn lume_empty(&self) -> bool { self.is_empty() } }
impl<T: LumeEmpty + ?std::marker::Sized> LumeEmpty for &T { fn lume_empty(&self) -> bool { (**self).lume_empty() } }
trait LumeSum<T> { fn lume_sum(&self) -> T; }
impl LumeSum<i64> for Vec<i64> { fn lume_sum(&self) -> i64 { self.iter().sum() } }
impl LumeSum<f64> for Vec<f64> { fn lume_sum(&self) -> f64 { self.iter().sum() } }
impl<T, U: LumeSum<T>> LumeSum<T> for &U { fn lume_sum(&self) -> T { (**self).lume_sum() } }
// A value has two forms: `lume_str`, the value as itself, which is what
// `puts` and interpolation show; and `lume_in`, the value as you would
// write it, which is what a value *inside* another one shows. They differ
// only for text: `puts name` is `Ada`, and `puts [name]` is `["Ada"]`, so
// one item and two can always be told apart.
trait LumeShow {
    fn lume_str(&self) -> String;
    fn lume_in(&self) -> String { self.lume_str() }
}
fn lume_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}
impl LumeShow for i64 { fn lume_str(&self) -> String { self.to_string() } }
impl LumeShow for f64 { fn lume_str(&self) -> String { format!("{:?}", if *self == 0.0 { 0.0 } else { *self }) } }
impl LumeShow for bool { fn lume_str(&self) -> String { self.to_string() } }
impl LumeShow for char {
    fn lume_str(&self) -> String { self.to_string() }
    fn lume_in(&self) -> String { lume_quote(&self.to_string()) }
}
fn lume_char_eq_str(c: char, s: &str) -> bool { let mut it = s.chars(); it.next() == Some(c) && it.next().is_none() }
fn lume_join_chars(cs: &[char], sep: &str) -> String {
    let mut out = String::with_capacity(cs.len());
    for (i, c) in cs.iter().enumerate() { if i > 0 { out.push_str(sep); } out.push(*c); }
    out
}
impl LumeShow for String {
    fn lume_str(&self) -> String { self.clone() }
    fn lume_in(&self) -> String { lume_quote(self) }
}
impl LumeShow for str {
    fn lume_str(&self) -> String { self.to_string() }
    fn lume_in(&self) -> String { lume_quote(self) }
}
impl LumeShow for () { fn lume_str(&self) -> String { String::from("()") } }
impl<T: LumeShow> LumeShow for Vec<T> {
    fn lume_str(&self) -> String { format!("[{}]", self.iter().map(|x| x.lume_in()).collect::<Vec<_>>().join(", ")) }
}
impl<T: LumeShow> LumeShow for [T] {
    fn lume_str(&self) -> String { format!("[{}]", self.iter().map(|x| x.lume_in()).collect::<Vec<_>>().join(", ")) }
}
impl<T: LumeShow> LumeShow for Option<T> {
    fn lume_str(&self) -> String { match self { Some(x) => format!("Some({})", x.lume_in()), None => String::from("None") } }
}
impl<A: LumeShow, B: LumeShow> LumeShow for (A, B) {
    fn lume_str(&self) -> String { format!("({}, {})", self.0.lume_in(), self.1.lume_in()) }
}
impl<A: LumeShow, B: LumeShow, C: LumeShow> LumeShow for (A, B, C) {
    fn lume_str(&self) -> String { format!("({}, {}, {})", self.0.lume_in(), self.1.lume_in(), self.2.lume_in()) }
}
impl<T: LumeShow + ?std::marker::Sized> LumeShow for &T {
    fn lume_str(&self) -> String { (**self).lume_str() }
    fn lume_in(&self) -> String { (**self).lume_in() }
}
impl<T: LumeShow + ?std::marker::Sized> LumeShow for ::std::boxed::Box<T> {
    fn lume_str(&self) -> String { (**self).lume_str() }
    fn lume_in(&self) -> String { (**self).lume_in() }
}
impl<T: LumeShow + ?std::marker::Sized> LumeShow for std::sync::Arc<T> { fn lume_str(&self) -> String { (**self).lume_str() } }
impl<T: LumeShow> LumeShow for std::sync::Mutex<T> { fn lume_str(&self) -> String { self.lock().unwrap().lume_str() } }
#[allow(dead_code)]
fn lume_assert_failed(line: usize, text: &str, sides: Option<(String, String)>) -> ! {
    let mut msg = format!(":{}: assert {}", line, text);
    if let Some((l, r)) = sides {
        msg.push_str(&format!("\n  left:  {}\n  right: {}", l, r));
    }
    std::panic::panic_any(LumeAssert(msg));
}
#[allow(dead_code)]
struct LumeAssert(String);
#[derive(Debug, Clone, PartialEq)]
struct Error { message: String }
impl LumeShow for Error { fn lume_str(&self) -> String { format!("Error({})", lume_quote(&self.message)) } }
impl<T: LumeShow, E: LumeShow> LumeShow for ::std::result::Result<T, E> {
    fn lume_str(&self) -> String { match self { Ok(x) => format!("Ok({})", x.lume_in()), Err(e) => e.lume_str() } }
}
fn lume_to_int(s: &str) -> ::std::result::Result<i64, Error> {
    s.trim().parse::<i64>().map_err(|_| Error { message: format!("`{}` is not an integer", s) })
}
fn lume_to_float(s: &str) -> ::std::result::Result<f64, Error> {
    s.trim().parse::<f64>().map_err(|_| Error { message: format!("`{}` is not a number", s) })
}
fn lume_read_file(path: &str) -> ::std::result::Result<String, Error> {
    std::fs::read_to_string(path).map_err(|e| Error { message: format!("cannot read `{}`: {}", path, e) })
}
fn lume_write_file(path: &str, text: &str) -> ::std::result::Result<(), Error> {
    std::fs::write(path, text).map_err(|e| Error { message: format!("cannot write `{}`: {}", path, e) })
}
fn lume_args() -> Vec<String> { std::env::args().skip(1).collect() }
/// Everything on standard input, to the end.
fn lume_stdin() -> ::std::result::Result<String, Error> {
    use std::io::Read;
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s).map_err(|e| Error { message: format!("cannot read input: {}", e) })?;
    Ok(s)
}
fn lume_append_file(path: &str, text: &str) -> ::std::result::Result<(), Error> {
    use std::io::Write;
    std::fs::OpenOptions::new().create(true).append(true).open(path)
        .and_then(|mut f| f.write_all(text.as_bytes()))
        .map_err(|e| Error { message: format!("cannot append to `{}`: {}", path, e) })
}
fn lume_remove_file(path: &str) -> ::std::result::Result<(), Error> {
    std::fs::remove_file(path).map_err(|e| Error { message: format!("cannot remove `{}`: {}", path, e) })
}
fn lume_file_size(path: &str) -> ::std::result::Result<i64, Error> {
    std::fs::metadata(path).map(|m| m.len() as i64).map_err(|e| Error { message: format!("cannot read `{}`: {}", path, e) })
}
/// Seconds since the epoch, as `File.modified` reports them.
fn lume_file_modified(path: &str) -> ::std::result::Result<i64, Error> {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map_err(|e| Error { message: format!("cannot read `{}`: {}", path, e) })
        .map(|t| t.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0))
}
fn lume_dir_make(path: &str) -> ::std::result::Result<(), Error> {
    std::fs::create_dir_all(path).map_err(|e| Error { message: format!("cannot make `{}`: {}", path, e) })
}
/// The names inside a directory, sorted, without `.` and `..`.
fn lume_dir_list(path: &str) -> ::std::result::Result<Vec<String>, Error> {
    let mut out = Vec::new();
    let entries = std::fs::read_dir(path).map_err(|e| Error { message: format!("cannot list `{}`: {}", path, e) })?;
    for e in entries {
        let e = e.map_err(|e| Error { message: format!("cannot list `{}`: {}", path, e) })?;
        out.push(e.file_name().to_string_lossy().into_owned());
    }
    out.sort();
    Ok(out)
}
/// Every file under a directory, sorted, directories walked in order.
fn lume_dir_walk(path: &str) -> ::std::result::Result<Vec<String>, Error> {
    let mut out = Vec::new();
    let mut stack = vec![path.to_string()];
    while let Some(dir) = stack.pop() {
        for name in lume_dir_list(&dir)? {
            let full = lume_path_join(&dir, &name);
            if std::path::Path::new(&full).is_dir() { stack.push(full); } else { out.push(full); }
        }
    }
    out.sort();
    Ok(out)
}
fn lume_dir_remove(path: &str) -> ::std::result::Result<(), Error> {
    std::fs::remove_dir_all(path).map_err(|e| Error { message: format!("cannot remove `{}`: {}", path, e) })
}
fn lume_path_join(a: &str, b: &str) -> String {
    if a.is_empty() { return b.to_string(); }
    if b.starts_with('/') || a.ends_with('/') { format!("{}{}", a, b) } else { format!("{}/{}", a, b) }
}
/// The directory part of a path, "" when there is none.
fn lume_path_dir(p: &str) -> String {
    match p.rfind('/') { Some(0) => "/".into(), Some(i) => p[..i].to_string(), None => String::new() }
}
fn lume_path_base(p: &str) -> String {
    match p.rfind('/') { Some(i) => p[i + 1..].to_string(), None => p.to_string() }
}
/// The extension without the dot, "" when there is none.
fn lume_path_ext(p: &str) -> String {
    let base = lume_path_base(p);
    match base.rfind('.') { Some(i) if i > 0 => base[i + 1..].to_string(), _ => String::new() }
}
/// The file name without its extension.
fn lume_path_stem(p: &str) -> String {
    let base = lume_path_base(p);
    match base.rfind('.') { Some(i) if i > 0 => base[..i].to_string(), _ => base }
}
/// Lume's map: insertion-ordered (as in Ruby), hash lookups, one copy of each key.
/// Entries live in a Vec, which alone carries the order; a key -> position index
/// finds them in one hash and one compare. Removal leaves a tombstone; the Vec is
/// compacted when tombstones outnumber live entries.
#[derive(Clone, Debug)]
struct LumeMap<K, V> {
    entries: Vec<Option<(K, V)>>,
    /// key -> its position in `entries`. One hash and one compare per
    /// lookup; `entries` alone carries the insertion order.
    index: std::collections::HashMap<K, usize>,
    live: usize,
}
impl<K: std::hash::Hash + Eq + Clone, V: Clone> LumeMap<K, V> {
    fn new() -> Self { LumeMap { entries: Vec::new(), index: std::collections::HashMap::new(), live: 0 } }
    fn position<Q>(&self, k: &Q) -> Option<usize> where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?std::marker::Sized {
        self.index.get(k).copied()
    }
    fn get<Q>(&self, k: &Q) -> Option<&V> where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?std::marker::Sized {
        self.position(k).and_then(|i| self.entries[i].as_ref().map(|(_, v)| v))
    }
    fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V> where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?std::marker::Sized {
        let i = self.position(k)?;
        self.entries[i].as_mut().map(|(_, v)| v)
    }
    fn contains_key<Q>(&self, k: &Q) -> bool where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?std::marker::Sized { self.position(k).is_some() }
    fn insert(&mut self, k: K, v: V) -> Option<V> {
        if let Some(i) = self.position(&k) {
            return self.entries[i].as_mut().map(|e| std::mem::replace(&mut e.1, v));
        }
        self.index.insert(k.clone(), self.entries.len());
        self.entries.push(Some((k, v)));
        self.live += 1;
        None
    }
    /// The value for `k`, starting from an empty one if absent: `m[k].push(x)`.
    fn slot(&mut self, k: K) -> &mut V where V: Default { self.entry_or_insert(k, V::default()) }
    /// The value for a key we only have on loan, inserting `default` first
    /// if absent. A key is only made when one is actually kept, so
    /// `counts[w] = counts[w].or(0) + 1` over repeating words allocates
    /// once per distinct word rather than once per word.
    fn entry_or_insert_ref<Q>(&mut self, k: &Q, default: V) -> &mut V
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq + ?std::marker::Sized + ToOwned<Owned = K>,
    {
        let i = match self.position(k) {
            Some(i) => i,
            None => {
                self.index.insert(k.to_owned(), self.entries.len());
                self.entries.push(Some((k.to_owned(), default)));
                self.live += 1;
                self.entries.len() - 1
            }
        };
        &mut self.entries[i].as_mut().unwrap().1
    }
    /// The value for `k`, inserting `default` first if absent.
    fn entry_or_insert(&mut self, k: K, default: V) -> &mut V {
        let i = match self.position(&k) {
            Some(i) => i,
            None => {
                self.index.insert(k.clone(), self.entries.len());
                self.entries.push(Some((k, default)));
                self.live += 1;
                self.entries.len() - 1
            }
        };
        &mut self.entries[i].as_mut().unwrap().1
    }
    fn remove<Q>(&mut self, k: &Q) -> Option<V> where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?std::marker::Sized {
        let i = self.index.remove(k)?;
        let (_, v) = self.entries[i].take()?;
        self.live -= 1;
        if self.entries.len() > 8 && self.live * 2 < self.entries.len() { self.compact(); }
        Some(v)
    }
    fn compact(&mut self) {
        let old = std::mem::take(&mut self.entries);
        self.index.clear();
        self.live = 0;
        for e in old.into_iter().flatten() { self.insert(e.0, e.1); }
    }
    fn len(&self) -> usize { self.live }
    fn is_empty(&self) -> bool { self.live == 0 }
    fn iter(&self) -> impl Iterator<Item = (&K, &V)> { self.entries.iter().flatten().map(|(k, v)| (k, v)) }
    fn keys(&self) -> impl Iterator<Item = &K> { self.iter().map(|(k, _)| k) }
    fn values(&self) -> impl Iterator<Item = &V> { self.iter().map(|(_, v)| v) }
    fn from<const N: usize>(pairs: [(K, V); N]) -> Self {
        let mut m = Self::new();
        for (k, v) in pairs { m.insert(k, v); }
        m
    }
}
impl<K: std::hash::Hash + Eq + Clone, V: Clone + PartialEq> PartialEq for LumeMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().all(|(k, v)| other.get(k) == Some(v))
    }
}
impl<K: LumeShow + std::hash::Hash + Eq + Clone, V: LumeShow + Clone> LumeShow for LumeMap<K, V> {
    fn lume_str(&self) -> String {
        format!("{{{}}}", self.iter().map(|(k, v)| format!("{}: {}", k.lume_in(), v.lume_in())).collect::<Vec<_>>().join(", "))
    }
}
impl<K: std::hash::Hash + Eq + Clone, V: Clone> LumeLen for LumeMap<K, V> { fn lume_len(&self) -> i64 { self.len() as i64 } }
impl<K: std::hash::Hash + Eq + Clone, V: Clone> LumeEmpty for LumeMap<K, V> { fn lume_empty(&self) -> bool { self.is_empty() } }
// `split(sep)` keeps every piece, so `s.split(sep).join(sep)` is `s` again.
// Rust's own `split` is exactly that rule, so there is nothing to wrap.
#[derive(Clone, Debug)]
struct LumeSet<T> { m: LumeMap<T, ()> }
impl<T: std::hash::Hash + Eq + Clone> LumeSet<T> {
    fn new() -> Self { LumeSet { m: LumeMap::new() } }
    fn from<const N: usize>(items: [T; N]) -> Self { let mut s = Self::new(); for i in items { s.insert(i); } s }
    fn insert(&mut self, x: T) -> bool { if self.m.contains_key(&x) { false } else { self.m.insert(x, ()); true } }
    fn remove<Q>(&mut self, x: &Q) -> bool where T: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?std::marker::Sized { self.m.remove(x).is_some() }
    fn contains<Q>(&self, x: &Q) -> bool where T: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?std::marker::Sized { self.m.contains_key(x) }
    fn len(&self) -> usize { self.m.len() }
    fn is_empty(&self) -> bool { self.m.is_empty() }
    fn iter(&self) -> impl Iterator<Item = &T> { self.m.keys() }
    fn union(&self, o: &Self) -> Self { let mut s = self.clone(); for x in o.iter() { s.insert(x.clone()); } s }
    fn intersect(&self, o: &Self) -> Self { self.iter().filter(|x| o.contains(*x)).cloned().collect() }
    fn diff(&self, o: &Self) -> Self { self.iter().filter(|x| !o.contains(*x)).cloned().collect() }
    fn is_subset(&self, o: &Self) -> bool { self.iter().all(|x| o.contains(x)) }
    fn is_superset(&self, o: &Self) -> bool { o.is_subset(self) }
}
impl<T: std::hash::Hash + Eq + Clone> Default for LumeSet<T> { fn default() -> Self { Self::new() } }
impl<K: std::hash::Hash + Eq + Clone, V: Clone> Default for LumeMap<K, V> { fn default() -> Self { Self::new() } }
impl<T: std::hash::Hash + Eq + Clone> FromIterator<T> for LumeSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(it: I) -> Self { let mut s = Self::new(); for x in it { s.insert(x); } s }
}
impl<T: std::hash::Hash + Eq + Clone> PartialEq for LumeSet<T> {
    fn eq(&self, o: &Self) -> bool { self.len() == o.len() && self.is_subset(o) }
}
impl<T: LumeShow + std::hash::Hash + Eq + Clone> LumeShow for LumeSet<T> {
    fn lume_str(&self) -> String { format!("{{{}}}", self.iter().map(|x| x.lume_in()).collect::<Vec<_>>().join(", ")) }
}
impl<T: std::hash::Hash + Eq + Clone> LumeLen for LumeSet<T> { fn lume_len(&self) -> i64 { self.len() as i64 } }
impl<T: std::hash::Hash + Eq + Clone> LumeEmpty for LumeSet<T> { fn lume_empty(&self) -> bool { self.is_empty() } }
/// `s[i]`: the character at a position, `None` when out of range.
fn lume_char_at(s: &str, i: i64) -> Option<char> {
    if i < 0 { return None; }
    s.chars().nth(i as usize)
}
/// `s[a..b]` in characters; positions are clamped to the string, an
/// empty result when they cross.
fn lume_slice_str(s: &str, a: i64, b: i64) -> String {
    let n = s.chars().count() as i64;
    let (a, b) = (a.clamp(0, n), b.clamp(0, n));
    if b <= a { return String::new(); }
    s.chars().skip(a as usize).take((b - a) as usize).collect()
}
/// `xs[a..b]`: positions are clamped to the list.
fn lume_slice_list<T: Clone>(xs: &[T], a: i64, b: i64) -> Vec<T> {
    let n = xs.len() as i64;
    let (a, b) = (a.clamp(0, n), b.clamp(0, n));
    if b <= a { return Vec::new(); }
    xs[a as usize..b as usize].to_vec()
}
fn lume_pad<T: LumeShow + ?std::marker::Sized>(x: &T, width: i64) -> String { format!("{:>w$}", x.lume_str(), w = width.max(0) as usize) }
fn lume_pad_right<T: LumeShow + ?std::marker::Sized>(x: &T, width: i64) -> String { format!("{:<w$}", x.lume_str(), w = width.max(0) as usize) }
fn lume_capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() { Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(), None => String::new() }
}
fn lume_clamp<T: PartialOrd + LumeShow + Copy>(x: T, lo: T, hi: T) -> T {
    if lo > hi { panic!("clamp: the low bound {} is above the high bound {}", lo.lume_str(), hi.lume_str()); }
    if x < lo { lo } else if x > hi { hi } else { x }
}
/// `xs.at(i)`: the item, when the caller has already checked the bound.
fn lume_at<T: Clone>(xs: &[T], i: i64) -> T {
    match usize::try_from(i).ok().and_then(|u| xs.get(u)) {
        Some(x) => x.clone(),
        None => panic!("no item at {} — the list has {}", i, xs.len()),
    }
}
/// `x.decimals(n)`: the number as text, to that many places.
fn lume_decimals(x: f64, n: i64) -> String {
    if n < 0 { panic!("decimals: {} is not a number of places", n); }
    // half goes away from zero, as people expect of money
    let f = 10f64.powi(n as i32);
    let scaled = x * f;
    let r = if scaled.is_finite() { scaled.round() / f } else { x };
    format!("{:.*}", n as usize, r)
}
/// Run-time failures speak Lume: no Rust file paths, no "attempt to".
#[allow(dead_code)]
fn lume_install_panic_hook() {
    std::panic::set_hook(::std::boxed::Box::new(|info| {
        let raw = if let Some(a) = info.payload().downcast_ref::<LumeAssert>() {
            a.0.clone()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else {
            String::from("the program stopped")
        };
        let msg = match raw.as_str() {
            "attempt to add with overflow" => "Int overflow in `+`".to_string(),
            "attempt to subtract with overflow" => "Int overflow in `-`".to_string(),
            "attempt to multiply with overflow" => "Int overflow in `*`".to_string(),
            "attempt to negate with overflow" => "Int overflow in `-`".to_string(),
            "attempt to divide by zero" => "division by zero".to_string(),
            "attempt to calculate the remainder with a divisor of zero" => "`%` by zero".to_string(),
            "attempt to divide with overflow" => "Int overflow in `/`".to_string(),
            m if m.starts_with("index out of bounds") || m.starts_with("insertion index") || m.starts_with("removal index") => "list position out of range".to_string(),
            m => m.to_string(),
        };
        eprintln!("error: {}", msg);
        if std::env::var("LUME_BACKTRACE").is_ok() {
            if let Some(loc) = info.location() {
                eprintln!("  at {}:{} in the generated Rust", loc.file(), loc.line());
            }
        } else {
            eprintln!("  (set LUME_BACKTRACE=1 to see where in the generated Rust)");
        }
    }));
}

fn lume_now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}
fn lume_now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}
fn lume_split(s: &str, sep: &str) -> Vec<String> {
    let mut v: Vec<String> = s.split(sep).map(|x| x.to_string()).collect();
    while v.last().map(|x| x.is_empty()).unwrap_or(false) { v.pop(); }
    v
}
// ---- end prelude ----
"#;

const RUST_RESERVED: &[&str] = &[
    "as", "async", "await", "box", "crate", "dyn", "extern", "fn", "impl", "let", "loop", "macro",
    "mod", "move", "mut", "ref", "static", "super", "trait", "type", "union", "unsafe", "use",
    "yield", "try", "Self", "self", "abstract", "become", "final", "override", "priv", "typeof",
    "unsized", "virtual", "where", "continue", "match", "enum", "struct", "pub", "const", "if",
    "else", "while", "for", "in", "return", "break", "true", "false", "String", "Vec", "Option",
    "Result", "Box", "Some", "None", "Ok", "Err",
];

#[derive(Clone)]
struct Binding {
    mutable: bool,
    borrowed: bool,
    ty: Type,
    line: usize,
}

#[derive(Clone)]
struct Sig {
    params: Vec<(String, Type)>,
    /// Which parameters are `var` (passed as `&mut`).
    var_params: Vec<bool>,
    ret: Type,
    self_kind: SelfKind,
    /// `async def`: a call gives a `Future` until it is awaited.
    is_async: bool,
    /// `def first[T](...)` — filled in from the arguments at every call.
    generics: Vec<TypeParam>,
}

#[derive(Clone)]
struct StructInfo {
    generics: Vec<TypeParam>,
    fields: Vec<(String, Type)>,
    methods: HashMap<String, Sig>,
}

#[derive(Clone)]
struct EnumInfo {
    generics: Vec<TypeParam>,
    variants: Vec<(String, Vec<(String, Type)>)>,
    methods: HashMap<String, Sig>,
}

#[derive(Clone)]
struct IfaceInfo {
    /// `interface Comparable[T]:` — what the interface is generic over.
    generics: Vec<TypeParam>,
    /// Ordered by name, so the generated Rust is the same on every run.
    methods: BTreeMap<String, Sig>,
    required: Vec<String>,
    /// Methods with a body, emitted inside the trait.
    defaults: Vec<FnDef>,
    /// Which module owns the interface (for the emit-once rule).
    local: bool,
}

/// Why a type does or does not satisfy an interface.
#[derive(Clone)]
enum Conformance {
    Yes,
    Missing(Vec<String>),
    Mismatch { method: String, expected: String, actual: String },
}

/// A compiled pattern: the Rust pattern text, extra guard conditions (for
/// string and float literals, which Rust cannot match directly), and the
/// names it binds with their types.
struct CompiledPat {
    text: String,
    guards: Vec<String>,
    binds: Vec<(String, Type, BindKind)>,
    /// Rust `let` lines to run at the top of the arm: how a pattern that
    /// reaches inside a `Box` gives its names their values.
    lets: Vec<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum BindKind {
    /// Copy value reached through a reference: rebind with `let x = *x;`
    Deref,
    /// A recursive field (`Box<T>`) reached through a reference: `let x = &**x;`
    Boxed,
    /// Reference into the scrutinee
    Ref,
    /// Slice from `..rest`: rebind with `let rest = rest.to_vec();`
    Slice,
    /// Owned value (scrutinee matched by value)
    Owned,
}

/// What a module offers to the files that import it.
#[derive(Clone, Default)]
pub struct Exports {
    pub rust_mod: String,
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    fns: HashMap<String, Sig>,
    consts: HashMap<String, Type>,
    interfaces: HashMap<String, IfaceInfo>,
    /// Methods types gained through `extend` in that module: type key -> methods.
    ext_methods: HashMap<String, HashMap<String, Sig>>,
    /// What makes those methods findable from another file: an `extend`
    /// travels with the import, so everything that says where it applies
    /// travels with it.
    ext_targets: HashMap<String, Type>,
    ext_generics: HashMap<String, Vec<TypeParam>>,
    ext_method_target: HashMap<(String, String), Type>,
    ext_impl_generics: HashMap<(String, String), Vec<TypeParam>>,
    /// Where each `extend` was written, for the message when two of them
    /// claim the same type and interface.
    ext_where: HashMap<(String, String), (String, usize)>,
    /// The interfaces those extends name, already qualified, with their
    /// Rust paths. An `extend` is not a name, so it keeps travelling past
    /// the file that imported it — but it can only be honoured where the
    /// interface behind it is known, so that comes along. The name itself
    /// stays unusable: only a file that imports the module can write it.
    ext_ifaces: HashMap<String, (IfaceInfo, String)>,
    /// Types this module reached from its own imports and then named in
    /// something it exports — a field, a return type, a parameter. Without
    /// them an importer can hold such a value but not use it.
    carried_structs: HashMap<String, StructInfo>,
    carried_enums: HashMap<String, EnumInfo>,
    carried_ifaces: HashMap<String, IfaceInfo>,
    carried_paths: HashMap<String, String>,
    private: Vec<String>,
}

/// One import of this module, already resolved to a file by the loader.
pub enum Dep<'a> {
    /// `import users.model [as m]`: the whole module under an alias
    Module { alias: String, id: String, exports: &'a Exports, line: usize, col: usize },
    /// `import users.model.User [as U]`: one public item under a local name
    Single { local: String, id: String, item: String, exports: &'a Exports, line: usize, col: usize },
    /// `import rust.regex [as re]`: a crate's public surface under an alias
    Rust { alias: String, info: &'a crate::bridge::CrateInfo },
    /// A crate some other module imports: its types only, so the entry file
    /// can emit the printing glue once for the whole program.
    RustTypes { info: &'a crate::bridge::CrateInfo },
}

/// What a dotted path such as `regex.Regex.new` names in a crate.
enum ForeignRef<'a> {
    Fn(&'a crate::bridge::ForeignFn),
    Assoc(String, &'a crate::bridge::ForeignFn),
    Type(String),
}

pub struct Output {
    pub rust: String,
    pub warnings: Vec<LumeError>,
    /// Cargo dependencies from `import rust.<crate>`: (crate, version).
    pub deps: Vec<(String, String)>,
    /// The program contains `rust:` blocks, so rustc errors may be the user's.
    pub has_rust_blocks: bool,
}

pub struct Gen {
    out: String,
    indent: usize,
    scopes: Vec<HashMap<String, Binding>>,
    fns: HashMap<String, Sig>,
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    interfaces: HashMap<String, IfaceInfo>,
    /// Methods a type gained through `extend`: type key -> name -> signature.
    ext_methods: HashMap<String, HashMap<String, Sig>>,
    /// `extend` targets that are built-in types: key ("Str") -> the type.
    ext_targets: HashMap<String, Type>,
    /// A generic `extend` — `extend [T] with Container[T]:` — keyed by the
    /// shape of its target, with the type parameters the target introduced.
    /// Several extends may share a shape, so this holds their union: it
    /// says only that the shape is generic.
    ext_generics: HashMap<String, Vec<TypeParam>>,
    /// The target one `extend` was written against, per method it gave the
    /// type. A method of `[T]` is read through `[T]`, whatever another
    /// `extend` of the same shape called its own parameter.
    ext_method_target: HashMap<(String, String), Type>,
    /// One `extend`'s own parameters, keyed by (type, interface): the
    /// bounds of `extend [T: Ordered] with Ranked[T]` belong to that impl
    /// alone, not to every impl for a list.
    ext_impl_generics: HashMap<(String, String), Vec<TypeParam>>,
    /// Which file and line each `extend` was written on, so two that claim
    /// the same type and interface can name each other.
    ext_where: HashMap<(String, String), (String, usize)>,
    /// This module's own file, as a program would name it.
    this_file: String,
    /// Traits an imported module owns. Rust only offers a trait's methods
    /// where the trait is in scope, and an `extend` that travels with the
    /// import has to be callable here.
    trait_uses: Vec<String>,
    /// Local types (struct/enum names) — interfaces are emitted for these.
    local_types: HashSet<String>,
    /// The struct or enum whose method is being generated, if any.
    current_type: Option<String>,
    /// The type of `self` when it is a built-in (inside `extend Str with ...`).
    current_self_ty: Option<Type>,
    /// Set while emitting methods that live inside a trait or trait impl (no `pub`).
    in_trait_impl: bool,
    current_self: SelfKind,
    current_ret: Type,
    current_fn: String,
    loop_depth: usize,
    in_block: bool,
    /// True while emitting statements whose value is the function's result
    /// (so `T or E` returns get their implied `Ok`/`Err`).
    tail_of_fn: bool,
    /// Set just before emitting an if/match that sits in tail position.
    at_tail: bool,
    tmp: usize,
    pub warnings: Vec<LumeError>,
    has_rust_blocks: bool,
    /// Statements that follow the one being emitted, in the same block.
    /// For each enclosing block, the statements after the one being emitted
    /// (innermost last): the liveness pass reads these.
    rest_stack: Vec<Vec<Stmt>>,
    /// Scope depths at which a loop or a block body began: a binding from
    /// outside is used again on the next iteration or call.
    barriers: Vec<usize>,
    /// Rust paths of imported items, keyed by canonical id ("users.model.User").
    paths: HashMap<String, String>,
    /// How this module spells imported items -> canonical id
    /// ("model.User" and a single-imported "User" -> "users.model.User").
    canon: HashMap<String, String>,
    /// Module aliases in scope (`model` for `import users.model`).
    module_aliases: HashMap<String, String>,
    /// Private names of imported modules, for good error messages: alias -> names.
    module_private: HashMap<String, Vec<String>>,
    /// True for the file that holds `main`.
    is_entry: bool,
    /// `lume test`: `test` blocks are compiled, `main` is renamed, `!` is silent inside tests.
    test_mode: bool,
    in_test: bool,
    src_lines: Vec<String>,
    /// Inside an `async def` or a `spawn:` block: `await` is allowed.
    in_async: bool,
    /// The program uses async somewhere: tokio goes in the dependencies.
    uses_async: bool,
    /// Set by `iter_base` for a map: its items are `(key, value)` pairs of
    /// references, not a reference to a pair. Read once by the next lambda.
    map_items: bool,
    /// `c.or("")` on a `Char?`: the default is a string, so the result is one.
    or_default_is_str: bool,
    /// Top-level constants: name -> type, for the root scope and for exports.
    consts: HashMap<String, Type>,
    /// Type parameters of the definition being compiled, innermost last.
    type_params: Vec<TypeParam>,
    /// While emitting a method inside a trait impl: how the interface
    /// declared its parameters. Where it said `T`, Rust lends the value
    /// whatever this type turned out to be.
    trait_decl: Option<Vec<Type>>,
    /// Parameters of the method being emitted that are copied types
    /// arriving behind a reference, because the trait said `T`.
    lent_names: HashSet<String>,
    /// Parameters whose declared type is an interface. Rust takes those as
    /// a generic, while a stored one is a pointer to the trait, so putting
    /// one into a field or a list needs the pointer made here.
    iface_params: HashSet<String>,
    /// Inside a block the program itself declared: the result that block
    /// promises. A built-in collection block leaves this None, because its
    /// closure gives back a plain value.
    block_ret: Option<Type>,
    /// The type the position being compiled expects, innermost last. A bare
    /// variant name (`Num(v)`) is resolved against it when it is ambiguous.
    want: Vec<Type>,
    /// Names copied into the current `spawn:` block.
    spawn_captured: HashSet<String>,
    /// While set, a `shared var` name means the handle, not the value inside.
    want_handle: bool,
    /// `import rust.<crate>` namespaces, by alias.
    crates: HashMap<String, crate::bridge::Namespace>,
    /// Opaque crate types, by Lume key (`regex.Regex`).
    foreign_types: HashMap<String, crate::bridge::ForeignType>,
}


/// Compiles one module. `rust_mod` is `Some(name)` for an imported file,
/// which is emitted as `mod name { ... }`; `None` for the entry file.
pub fn generate_module(program: &[Item], rust_mod: Option<&str>, deps: &[Dep], test_mode: bool, src: &str, file: &str) -> Result<(Output, Exports)> {
    let mut g = Gen {
        out: String::new(),
        indent: 0,
        scopes: Vec::new(),
        fns: HashMap::new(),
        structs: HashMap::new(),
        enums: HashMap::new(),
        interfaces: HashMap::new(),
        type_params: Vec::new(),
        trait_decl: None,
        lent_names: HashSet::new(),
        iface_params: HashSet::new(),
        block_ret: None,
        ext_methods: HashMap::new(),
        ext_targets: HashMap::new(),
        ext_generics: HashMap::new(),
        ext_method_target: HashMap::new(),
        ext_impl_generics: HashMap::new(),
        ext_where: HashMap::new(),
        this_file: file.to_string(),
        trait_uses: Vec::new(),
        local_types: HashSet::new(),
        current_type: None,
        current_self_ty: None,
        in_trait_impl: false,
        current_self: SelfKind::Read,
        current_ret: Type::Unit,
        current_fn: String::new(),
        loop_depth: 0,
        in_block: false,
        tail_of_fn: false,
        at_tail: false,
        tmp: 0,
        warnings: Vec::new(),
        has_rust_blocks: false,
        rest_stack: Vec::new(),
        barriers: Vec::new(),
        paths: HashMap::new(),
        canon: HashMap::new(),
        module_aliases: HashMap::new(),
        module_private: HashMap::new(),
        is_entry: true,
        test_mode,
        in_test: false,
        src_lines: src.lines().map(|l| l.to_string()).collect(),
        in_async: false,
        uses_async: false,
        map_items: false,
        or_default_is_str: false,
        consts: HashMap::new(),
        want: Vec::new(),
        spawn_captured: HashSet::new(),
        want_handle: false,
        crates: HashMap::new(),
        foreign_types: HashMap::new(),
    };
    // The built-in Error type: a struct with one field, defined in the prelude.
    g.structs.insert("Error".into(), StructInfo { generics: Vec::new(), fields: vec![("message".into(), Type::Str)], methods: HashMap::new() });
    g.is_entry = rust_mod.is_none();
    g.register_deps(deps)?;
    g.program(program)?;
    let mut rust_deps = Vec::new();
    for item in program {
        if let Item::Import(imp) = item {
            if imp.is_rust {
                rust_deps.push((imp.krate().to_string(), imp.version.clone().unwrap_or_else(|| "*".into())));
            }
        }
    }
    if g.uses_async {
        rust_deps.push(("tokio".to_string(), "{ version = \"1\", features = [\"rt-multi-thread\", \"macros\", \"time\", \"sync\"] }".to_string()));
    }
    let exports = g.exports(program, rust_mod.unwrap_or("main"));
    let uses = if g.trait_uses.is_empty() {
        String::new()
    } else {
        let mut u = g.trait_uses.clone();
        u.sort();
        format!("#[allow(unused_imports)]\n{}\n", u.join("\n"))
    };
    let rust = match rust_mod {
        Some(m) => {
            let body = format!("{}{}", uses, g.out);
            let body = body.lines().map(|l| if l.is_empty() { String::new() } else { format!("    {}", l) }).collect::<Vec<_>>().join("\n");
            format!("pub mod {} {{\n    use super::*;\n{}\n}}\n", m, body)
        }
        None => {
            // inner attributes must come first in a file, so the uses go
            // just after them
            let mut head = String::new();
            let mut rest = g.out.as_str();
            while let Some(nl) = rest.find('\n') {
                let line = &rest[..nl];
                if line.starts_with("#!") || line.is_empty() {
                    head.push_str(&rest[..=nl]);
                    rest = &rest[nl + 1..];
                } else {
                    break;
                }
            }
            format!("{}{}{}", head, uses, rest)
        }
    };
    Ok((Output { rust, warnings: g.warnings, deps: rust_deps, has_rust_blocks: g.has_rust_blocks }, exports))
}

/// Rewrites a type from module `id` so its named types are keyed the way the
/// importing module sees them (`User` -> `users.model.User`).
/// A type parameter whose bound names a type of the module it came from.
fn qualify_param(p: &TypeParam, id: &str, ex: &Exports) -> TypeParam {
    TypeParam { name: p.name.clone(), bound: p.bound.as_ref().map(|b| qualify_type(b, id, ex)), line: p.line, col: p.col }
}

fn qualify_type(t: &Type, id: &str, ex: &Exports) -> Type {
    match t {
        // an interface too: a bound or a parameter may name one
        Type::Named(n) if n != "Error" && (ex.structs.contains_key(n) || ex.enums.contains_key(n) || ex.interfaces.contains_key(n)) => Type::Named(format!("{}.{}", id, n)),
        Type::App(n, args) => {
            let args = args.iter().map(|x| qualify_type(x, id, ex)).collect();
            if ex.structs.contains_key(n) || ex.enums.contains_key(n) || ex.interfaces.contains_key(n) {
                Type::App(format!("{}.{}", id, n), args)
            } else {
                Type::App(n.clone(), args)
            }
        }
        Type::List(i) => Type::List(Box::new(qualify_type(i, id, ex))),
        Type::Option(i) => Type::Option(Box::new(qualify_type(i, id, ex))),
        Type::Iter(i, b) => Type::Iter(Box::new(qualify_type(i, id, ex)), *b),
        Type::Tuple(ts) => Type::Tuple(ts.iter().map(|x| qualify_type(x, id, ex)).collect()),
        Type::Result(a, b) => Type::Result(Box::new(qualify_type(a, id, ex)), Box::new(qualify_type(b, id, ex))),
        Type::Map(a, b) => Type::Map(Box::new(qualify_type(a, id, ex)), Box::new(qualify_type(b, id, ex))),
        Type::Set(t) => Type::Set(Box::new(qualify_type(t, id, ex))),
        other => other.clone(),
    }
}

fn qualify_sig(sig: &Sig, id: &str, ex: &Exports) -> Sig {
    Sig {
        params: sig.params.iter().map(|(n, t)| (n.clone(), qualify_type(t, id, ex))).collect(),
        var_params: sig.var_params.clone(),
        ret: qualify_type(&sig.ret, id, ex),
        self_kind: sig.self_kind,
        is_async: sig.is_async,
        // a bound names an interface of the module it came from
        generics: sig.generics.iter().map(|g| qualify_param(g, id, ex)).collect(),
    }
}

fn rust_name(name: &str) -> String {
    if name == "_" {
        return "lume_it".into();
    }
    if let Some(op) = op_method_name(name) {
        return op.into();
    }
    let base = name.replace('?', "_q");
    if RUST_RESERVED.contains(&base.as_str()) {
        if base == "self" || base == "Self" || base == "crate" || base == "super" {
            format!("lume_{}", base)
        } else {
            format!("r#{}", base)
        }
    } else {
        base
    }
}

pub fn rust_type(t: &Type) -> String {
    match t {
        Type::Int => "i64".into(),
        Type::Float => "f64".into(),
        Type::Bool => "bool".into(),
        Type::Char => "char".into(),
        Type::Str => "String".into(),
        Type::Unit => "()".into(),
        Type::List(inner) => format!("Vec<{}>", rust_type(inner)),
        Type::Named(n) => n.clone(),
        Type::Option(inner) => format!("Option<{}>", rust_type(inner)),
        Type::Tuple(ts) => format!("({})", ts.iter().map(rust_type).collect::<Vec<_>>().join(", ")),
        Type::Result(t, e) => format!("::std::result::Result<{}, {}>", rust_type(t), rust_type(e)),
        Type::Map(k, v) => format!("LumeMap<{}, {}>", rust_type(k), rust_type(v)),
        Type::Set(t) => format!("LumeSet<{}>", rust_type(t)),
        Type::Iter(inner, _) => format!("Vec<{}>", rust_type(inner)),
        Type::Task(inner) => format!("tokio::task::JoinHandle<{}>", rust_type(inner)),
        Type::Future(inner) => format!("impl std::future::Future<Output = {}>", rust_type(inner)),
        Type::Shared(inner, true) => format!("std::sync::Arc<std::sync::Mutex<{}>>", rust_type(inner)),
        Type::Shared(inner, false) => format!("std::sync::Arc<{}>", rust_type(inner)),
        Type::App(n, args) => format!("{}<{}>", n, args.iter().map(rust_type).collect::<Vec<_>>().join(", ")),
        Type::Fn(ps, r) => format!("impl FnMut({}) -> {}", ps.iter().map(rust_type).collect::<Vec<_>>().join(", "), rust_type(r)),
        Type::Var(n) => n.clone(),
        Type::Unknown => "_".into(),
    }
}

pub fn type_name(t: &Type) -> String {
    match t {
        Type::Int => "Int".into(),
        Type::Float => "Float".into(),
        Type::Bool => "Bool".into(),
        Type::Char => "Char".into(),
        Type::Str => "Str".into(),
        Type::Unit => "()".into(),
        Type::List(i) => format!("[{}]", type_name(i)),
        Type::Named(n) => n.clone(),
        Type::Option(i) => format!("{}?", type_name(i)),
        Type::Tuple(ts) => format!("({})", ts.iter().map(type_name).collect::<Vec<_>>().join(", ")),
        Type::Result(t, e) => format!("{} or {}", type_name(t), type_name(e)),
        Type::Map(k, v) => format!("{{{}: {}}}", type_name(k), type_name(v)),
        Type::Set(t) => format!("{{{}}}", type_name(t)),
        Type::Iter(i, _) => format!("[{}]", type_name(i)),
        Type::Task(i) => format!("Task[{}]", type_name(i)),
        Type::Future(i) => format!("async {}", type_name(i)),
        Type::Shared(i, true) => format!("shared var {}", type_name(i)),
        Type::Shared(i, false) => format!("shared {}", type_name(i)),
        Type::App(n, args) => format!("{}[{}]", n, args.iter().map(type_name).collect::<Vec<_>>().join(", ")),
        Type::Fn(ps, r) => format!("({}) -> {}", ps.iter().map(type_name).collect::<Vec<_>>().join(", "), type_name(r)),
        Type::Var(n) => n.clone(),
        Type::Unknown => "?".into(),
    }
}

fn escape_rust_str(s: &str, for_format: bool) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            '{' if for_format => out.push_str("{{"),
            '}' if for_format => out.push_str("}}"),
            // Rust source must not hold a bare CR or other control character
            c if (c as u32) < 0x20 || c == '\u{7f}' => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            other => out.push(other),
        }
    }
    out
}

fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 { one.to_string() } else { many.to_string() }
}

impl Gen {
    /// Rust spelling of a type; names of imported items go through their module path.
    fn rt(&self, t: &Type) -> String {
        match t {
            _ if self.is_interface(t) => format!("::std::boxed::Box<dyn {}>", self.rust_iface(t)),
            Type::Named(n) if self.foreign_types.contains_key(n) => {
                let ft = &self.foreign_types[n];
                if ft.lifetimes == 0 { ft.rust_path.clone() } else { format!("{}<{}>", ft.rust_path, vec!["'_"; ft.lifetimes].join(", ")) }
            }
            Type::Named(n) => self.path_of(n),
            Type::App(n, args) => format!("{}<{}>", self.path_of(n), args.iter().map(|a| self.rt(a)).collect::<Vec<_>>().join(", ")),
            Type::List(inner) => format!("Vec<{}>", self.rt(inner)),
            Type::Option(inner) => format!("Option<{}>", self.rt(inner)),
            Type::Tuple(ts) => format!("({})", ts.iter().map(|x| self.rt(x)).collect::<Vec<_>>().join(", ")),
            Type::Result(a, b) => format!("::std::result::Result<{}, {}>", self.rt(a), self.rt(b)),
            Type::Map(k, v) => format!("LumeMap<{}, {}>", self.rt(k), self.rt(v)),
            Type::Set(t) => format!("LumeSet<{}>", self.rt(t)),
            Type::Iter(inner, _) => format!("Vec<{}>", self.rt(inner)),
            Type::Task(inner) => format!("tokio::task::JoinHandle<{}>", self.rt(inner)),
            Type::Shared(inner, true) => format!("std::sync::Arc<std::sync::Mutex<{}>>", self.rt(inner)),
            Type::Shared(inner, false) => format!("std::sync::Arc<{}>", self.rt(inner)),
            other => rust_type(other),
        }
    }

    /// The Rust path of a struct/enum/function key as seen from this module.
    fn path_of(&self, key: &str) -> String {
        let key = self.canon(key);
        match self.paths.get(&key) {
            Some(p) => p.clone(),
            None => key,
        }
    }

    /// Canonical id of a name as written in this module.
    fn canon(&self, name: &str) -> String {
        self.canon.get(name).cloned().unwrap_or_else(|| name.to_string())
    }

    /// A type with its named parts canonicalised.
    fn ct(&self, t: &Type) -> Type {
        match t {
            Type::Named(n) if self.type_param(n).is_some() => Type::Var(n.clone()),
            Type::Named(n) => Type::Named(self.canon(n)),
            Type::App(n, args) => Type::App(self.canon(n), args.iter().map(|x| self.ct(x)).collect()),
            Type::List(i) => Type::List(Box::new(self.ct(i))),
            Type::Option(i) => Type::Option(Box::new(self.ct(i))),
            Type::Iter(i, b) => Type::Iter(Box::new(self.ct(i)), *b),
            Type::Tuple(ts) => Type::Tuple(ts.iter().map(|x| self.ct(x)).collect()),
            Type::Result(a, b) => Type::Result(Box::new(self.ct(a)), Box::new(self.ct(b))),
            Type::Map(a, b) => Type::Map(Box::new(self.ct(a)), Box::new(self.ct(b))),
            Type::Set(t) => Type::Set(Box::new(self.ct(t))),
            Type::Task(i) => Type::Task(Box::new(self.ct(i))),
            Type::Future(i) => Type::Future(Box::new(self.ct(i))),
            Type::Shared(i, m) => Type::Shared(Box::new(self.ct(i)), *m),
            Type::Fn(ps, r) => Type::Fn(ps.iter().map(|x| self.ct(x)).collect(), Box::new(self.ct(r))),
            other => other.clone(),
        }
    }

    /// Names a definition declares as type parameters stop being ordinary
    /// type names inside its own signature.
    fn as_vars(t: &Type, gs: &[TypeParam]) -> Type {
        if gs.is_empty() {
            return t.clone();
        }
        let go = |x: &Type| Box::new(Self::as_vars(x, gs));
        match t {
            Type::Named(n) if gs.iter().any(|p| p.name == *n) => Type::Var(n.clone()),
            Type::App(n, args) => Type::App(n.clone(), args.iter().map(|a| Self::as_vars(a, gs)).collect()),
            Type::List(i) => Type::List(go(i)),
            Type::Option(i) => Type::Option(go(i)),
            Type::Set(i) => Type::Set(go(i)),
            Type::Task(i) => Type::Task(go(i)),
            Type::Future(i) => Type::Future(go(i)),
            Type::Iter(i, b) => Type::Iter(go(i), *b),
            Type::Shared(i, b) => Type::Shared(go(i), *b),
            Type::Tuple(ts) => Type::Tuple(ts.iter().map(|x| Self::as_vars(x, gs)).collect()),
            Type::Result(a, b) => Type::Result(go(a), go(b)),
            Type::Map(a, b) => Type::Map(go(a), go(b)),
            Type::Fn(ps, r) => Type::Fn(ps.iter().map(|x| Self::as_vars(x, gs)).collect(), Box::new(Self::as_vars(r, gs))),
            other => other.clone(),
        }
    }

    /// A definition's type parameters, with each bound canonicalised and
    /// with the definition's own names read as parameters: the bound of
    /// `[T: Comparable[T]]` becomes `Comparable[Var T]`, so filling `T` in
    /// at a call reaches it.
    fn norm_generics(&self, gs: &[TypeParam]) -> Vec<TypeParam> {
        gs.iter()
            .map(|p| TypeParam { bound: p.bound.as_ref().map(|b| self.ct(&Self::as_vars(b, gs))), ..p.clone() })
            .collect()
    }

    fn sig_of(&self, f: &FnDef) -> Sig {
        Sig {
            params: f.params.iter().map(|p| (p.name.clone(), Self::as_vars(&self.ct(&p.ty), &f.generics))).collect(),
            var_params: f.params.iter().map(|p| p.mutable).collect(),
            ret: f.ret.as_ref().map(|t| Self::as_vars(&self.ct(t), &f.generics)).unwrap_or(Type::Unknown),
            self_kind: f.self_kind,
            is_async: f.is_async,
            generics: self.norm_generics(&f.generics),
        }
    }

    // ----- generics -------------------------------------------------------

    /// Compiles the definition's own type parameters into scope: inside it,
    /// `T` means the parameter, not a type of that name.
    fn push_generics(&mut self, gs: &[TypeParam]) -> Vec<TypeParam> {
        let gs = &self.norm_generics(gs);
        let mut all = self.type_params.clone();
        for g in gs {
            all.retain(|p| p.name != g.name);
            all.push(g.clone());
        }
        std::mem::replace(&mut self.type_params, all)
    }

    fn pop_generics(&mut self, saved: Vec<TypeParam>) {
        self.type_params = saved;
    }

    fn type_param(&self, name: &str) -> Option<&TypeParam> {
        self.type_params.iter().find(|p| p.name == name)
    }

    /// The type parameters a struct or enum declares.
    fn generics_of(&self, key: &str) -> Vec<TypeParam> {
        if let Some(s) = self.structs.get(key) {
            return s.generics.clone();
        }
        if let Some(e) = self.enums.get(key) {
            return e.generics.clone();
        }
        if let Some(i) = self.interfaces.get(key) {
            return i.generics.clone();
        }
        Vec::new()
    }

    /// `Stack[Int]` seen against `struct Stack[T]` gives `T -> Int`.
    fn subst_for(&self, t: &Type) -> HashMap<String, Type> {
        let mut m = HashMap::new();
        match t {
            Type::App(n, args) => {
                for (p, a) in self.generics_of(&self.canon(n)).iter().zip(args) {
                    m.insert(p.name.clone(), a.clone());
                }
            }
            // `T` bounded by `Sized[Str]`: the interface's own parameters
            // stand for what the bound gave them
            Type::Var(n) => {
                if let Some(Type::App(iname, args)) = self.type_param(n).and_then(|p| p.bound.clone()) {
                    for (p, a) in self.generics_of(&self.canon(&iname)).iter().zip(&args) {
                        m.insert(p.name.clone(), a.clone());
                    }
                }
            }
            _ => {}
        }
        m
    }

    /// A type with its parameters replaced. Unbound parameters stay as they
    /// are, which is what a definition compiling itself wants.
    fn subst(t: &Type, m: &HashMap<String, Type>) -> Type {
        if m.is_empty() {
            return t.clone();
        }
        let go = |x: &Type| Box::new(Self::subst(x, m));
        match t {
            Type::Var(n) => m.get(n).cloned().unwrap_or_else(|| t.clone()),
            Type::App(n, args) => Type::App(n.clone(), args.iter().map(|a| Self::subst(a, m)).collect()),
            Type::List(i) => Type::List(go(i)),
            Type::Option(i) => Type::Option(go(i)),
            Type::Set(i) => Type::Set(go(i)),
            Type::Task(i) => Type::Task(go(i)),
            Type::Future(i) => Type::Future(go(i)),
            Type::Iter(i, b) => Type::Iter(go(i), *b),
            Type::Shared(i, b) => Type::Shared(go(i), *b),
            Type::Tuple(ts) => Type::Tuple(ts.iter().map(|x| Self::subst(x, m)).collect()),
            Type::Result(a, b) => Type::Result(go(a), go(b)),
            Type::Map(a, b) => Type::Map(go(a), go(b)),
            Type::Fn(ps, r) => Type::Fn(ps.iter().map(|x| Self::subst(x, m)).collect(), Box::new(Self::subst(r, m))),
            other => other.clone(),
        }
    }

    /// Reads the type arguments off a value: `[Int]` against the declared
    /// `[T]` says `T` is `Int`. Silent on a mismatch — the argument check
    /// that follows reports it in the user's own terms.
    fn unify(decl: &Type, actual: &Type, m: &mut HashMap<String, Type>) {
        let actual = actual.materialized();
        match (decl, &actual) {
            (Type::Var(n), a) => {
                if *a != Type::Unknown && !m.contains_key(n) {
                    m.insert(n.clone(), a.clone());
                }
            }
            (Type::App(_, ds), Type::App(_, as_)) if ds.len() == as_.len() => {
                for (d, a) in ds.iter().zip(as_) {
                    Self::unify(d, a, m);
                }
            }
            (Type::List(d), Type::List(a))
            | (Type::Option(d), Type::Option(a))
            | (Type::Set(d), Type::Set(a))
            | (Type::Task(d), Type::Task(a))
            | (Type::Shared(d, _), Type::Shared(a, _)) => Self::unify(d, a, m),
            (Type::Result(d1, d2), Type::Result(a1, a2)) | (Type::Map(d1, d2), Type::Map(a1, a2)) => {
                Self::unify(d1, a1, m);
                Self::unify(d2, a2, m);
            }
            (Type::Tuple(ds), Type::Tuple(as_)) if ds.len() == as_.len() => {
                for (d, a) in ds.iter().zip(as_) {
                    Self::unify(d, a, m);
                }
            }
            (Type::Fn(ds, dr), Type::Fn(as_, ar)) if ds.len() == as_.len() => {
                for (d, a) in ds.iter().zip(as_) {
                    Self::unify(d, a, m);
                }
                Self::unify(dr, ar, m);
            }
            // a block whose result is a `T or E` given a plain value: the
            // value is the `Ok` part, as it is at the end of a function —
            // unless it is the error itself, which says nothing about `T`
            (Type::Result(_, e), a) if *a == Type::Named("Error".to_string()) => Self::unify(e, a, m),
            (Type::Result(d, _), a) if !matches!(a, Type::Result(..)) => Self::unify(d, a, m),
            (Type::Option(d), a) if !matches!(a, Type::Option(_)) => Self::unify(d, a, m),
            // `xs: [T]` given an empty list says nothing about `T`
            _ => {}
        }
    }

    fn mentions_var(t: &Type, name: &str) -> bool {
        match t {
            Type::Var(n) => n == name,
            Type::App(_, args) | Type::Tuple(args) => args.iter().any(|a| Self::mentions_var(a, name)),
            Type::List(i) | Type::Option(i) | Type::Set(i) | Type::Task(i) | Type::Future(i) | Type::Iter(i, _) | Type::Shared(i, _) => Self::mentions_var(i, name),
            Type::Result(a, b) | Type::Map(a, b) => Self::mentions_var(a, name) || Self::mentions_var(b, name),
            Type::Fn(ps, r) => ps.iter().any(|x| Self::mentions_var(x, name)) || Self::mentions_var(r, name),
            _ => false,
        }
    }

    /// The name at the head of a bound: `Ordered`, or the interface.
    fn bound_name(t: &Type) -> &str {
        match t {
            Type::Named(n) | Type::App(n, _) => n,
            _ => "",
        }
    }

    /// Does `t` satisfy `bound`? Built-in bounds first, then interfaces.
    fn meets_bound(&self, t: &Type, bound: &Type) -> bool {
        // a type parameter carries only what its own bound promised
        if let Type::Var(n) = t {
            return match self.type_param(n).and_then(|p| p.bound.clone()) {
                Some(b) => self.type_key(&b) == self.type_key(bound) && b == *bound,
                None => false,
            };
        }
        match Self::bound_name(bound) {
            "Ordered" => match t {
                Type::Int | Type::Float | Type::Str | Type::Char | Type::Bool => true,
                Type::List(i) | Type::Option(i) => self.meets_bound(i, bound),
                Type::Tuple(ts) => ts.iter().all(|x| self.meets_bound(x, bound)),
                Type::Named(n) => self.methods_of(&self.canon(n)).map(|m| m.contains_key("<")).unwrap_or(false),
                _ => false,
            },
            "Hashable" => self.key_ok(t, &mut Vec::new()) && *t != Type::Unknown,
            _ => matches!(self.conformance(t, bound), Conformance::Yes),
        }
    }

    fn is_builtin_bound(b: &str) -> bool {
        matches!(b, "Ordered" | "Hashable")
    }

    /// The Rust bounds a type parameter carries. Every Lume value can be
    /// copied, compared and printed; the declared bound adds to that.
    fn rust_bounds(&self, p: &TypeParam) -> String {
        let bs: Vec<Type> = p.bound.iter().cloned().collect();
        self.rust_bounds_many(&p.name, &bs)
    }

    /// One parameter with every bound it has to satisfy at once: what it
    /// was declared with, and what an interface asks of it.
    fn rust_bounds_many(&self, name: &str, bounds: &[Type]) -> String {
        // `'static` because a value held as an interface lives behind a
        // pointer to a trait; every Lume value owns what it holds, so this
        // is always true and never constrains a program.
        let mut parts = vec!["Clone".to_string(), "std::fmt::Debug".to_string(), "PartialEq".to_string(), "LumeShow".to_string(), "'static".to_string()];
        let mut add = |p: String, parts: &mut Vec<String>| {
            if !parts.contains(&p) {
                parts.push(p);
            }
        };
        for b in bounds {
            match Self::bound_name(b) {
                "Ordered" => add("PartialOrd".into(), &mut parts),
                "Hashable" => {
                    add("std::hash::Hash".into(), &mut parts);
                    add("Eq".into(), &mut parts);
                }
                _ => add(self.rust_iface(b), &mut parts),
            }
        }
        format!("{}: {}", name, parts.join(" + "))
    }

    /// An interface as a Rust trait, with its arguments when it takes any.
    fn rust_iface(&self, t: &Type) -> String {
        match t {
            Type::App(n, args) => format!("{}<{}>", self.path_of(n), args.iter().map(|a| self.rt(a)).collect::<Vec<_>>().join(", ")),
            Type::Named(n) => self.path_of(n),
            other => self.rt(other),
        }
    }

    fn rust_fn_bound(&self, ins: &[Type], out: &Type) -> String {
        let ps: Vec<String> = ins.iter().map(|t| self.rust_block_param(t)).collect();
        let ret = if *out == Type::Unit { String::new() } else { format!(" -> {}", self.rt(out)) };
        format!("FnMut({}){}", ps.join(", "), ret)
    }

    /// The Rust type one of a block's parameters arrives as.
    fn rust_block_param(&self, t: &Type) -> String {
        if *t == Type::Str {
            "&str".to_string()
        } else {
            // everything else is lent, whether or not it is copied, so a
            // block declared `(Int) -> Int` and one declared `(T) -> U`
            // have the same shape and can be handed to each other
            format!("&{}", self.rt(t))
        }
    }

    /// `<T: ...>` for a definition's header, empty when it has none.
    fn rust_generics(&self, gs: &[TypeParam]) -> String {
        if gs.is_empty() {
            String::new()
        } else {
            format!("<{}>", gs.iter().map(|p| self.rust_bounds(p)).collect::<Vec<_>>().join(", "))
        }
    }

    /// `<T, K>` — the parameters again, without their bounds.
    fn rust_generic_args(gs: &[TypeParam]) -> String {
        if gs.is_empty() {
            String::new()
        } else {
            format!("<{}>", gs.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", "))
        }
    }

    /// Checks the arguments a generic definition was given: one per
    /// parameter, each satisfying its bound, none of them an interface.
    fn check_type_args(&self, gs: &[TypeParam], args: &[Type], what: &str, line: usize, col: usize) -> Result<()> {
        for (p, a) in gs.iter().zip(args) {
            if self.is_interface(a) {
                return Err(LumeError::new(line, col, format!("`{}` is an interface, so it cannot fill the `{}` of {}", type_name(a), p.name, what))
                    .with_help("a type argument is one concrete type; a list of mixed values is written `[Interface]` instead"));
            }
            if let Some(b) = &p.bound {
                // `[T: Comparable[T]]` — the bound talks about the arguments
                // this call filled in, so fill them in there too
                let filled: HashMap<String, Type> = gs.iter().map(|q| q.name.clone()).zip(args.iter().cloned()).collect();
                let b = Self::subst(b, &filled);
                if !self.meets_bound(a, &b) {
                    let bn = type_name(&b);
                    let e = LumeError::new(line, col, format!("{} needs `{}: {}`, and `{}` is not {}", what, p.name, bn, type_name(a), bn));
                    return Err(match Self::bound_name(&b) {
                        "Ordered" => e.with_help("`Ordered` means `<` works: `Int`, `Float`, `Str`, `Char`, or a type with its own `def <`"),
                        "Hashable" => e.with_help("`Hashable` means the value can be a map key: `Int`, `Str`, `Bool`, `Char`, tuples of those, or a struct or enum made of them"),
                        _ => match self.conformance(a, &b) {
                            Conformance::Missing(m) if !m.is_empty() => e.with_help(format!("`{}` has no `{}` method; add it, or `extend {} with {}:`", type_name(a), m.join("`, `"), type_name(a), bn)),
                            Conformance::Mismatch { method, expected, actual } => e.with_help(format!("its `{}` is `{}`, and `{}` asks for `{}`", method, actual, bn, expected)),
                            _ => e.with_help(format!("give `{}` the methods `{}` asks for", type_name(a), bn)),
                        },
                    });
                }
            }
        }
        Ok(())
    }

    /// Works out a call's type arguments: first from what the arguments
    /// are, then from the type the surrounding position expects.
    fn infer_call(&mut self, gs: &[TypeParam], params: &[(String, Type)], args: &[Arg], ret: &Type) -> HashMap<String, Type> {
        let mut m: HashMap<String, Type> = HashMap::new();
        if let Ok(bound) = self.bind_args("", params, args, 0, 0) {
            for (a, (_, pty)) in bound.iter().zip(params) {
                if gs.iter().any(|p| Self::mentions_var(pty, &p.name)) {
                    // a block says what its result is only once its own
                    // arguments are known, so fill those in first
                    let at = match pty {
                        Type::Fn(ins, _) => {
                            let ins: Vec<Type> = ins.iter().map(|t| Self::subst(t, &m)).collect();
                            self.block_type(a, &ins)
                        }
                        _ => self.ty_of(a),
                    };
                    Self::unify(pty, &at, &mut m);
                }
            }
        }
        if gs.iter().any(|p| !m.contains_key(&p.name)) {
            if let Some(w) = self.want.last().cloned() {
                Self::unify(ret, &w, &mut m);
            }
        }
        // `[K, V, T: Keyed[K, V]]` — K and V appear nowhere but the bound,
        // so they come from how `T` conforms
        if gs.iter().any(|p| !m.contains_key(&p.name)) {
            for p in gs {
                let (known, bound) = (m.get(&p.name).cloned(), p.bound.clone());
                if let (Some(t), Some(Type::App(iname, args))) = (known, bound) {
                    if args.iter().any(|a| gs.iter().any(|q| Self::mentions_var(a, &q.name))) {
                        let (verdict, got) = self.conformance_with(&t, &Type::Named(self.canon(&iname)));
                        if matches!(verdict, Conformance::Yes) {
                            for (a, g) in args.iter().zip(got) {
                                Self::unify(a, &g, &mut m);
                            }
                        }
                    }
                }
            }
        }
        m
    }

    /// What a block given as an argument turns out to be, once its own
    /// argument types are settled: `{ |n| n * 2 }` over an `Int` is
    /// `(Int) -> Int`.
    fn block_type(&mut self, e: &Expr, ins: &[Type]) -> Type {
        match &e.kind {
            ExprKind::Lambda { params, body } if params.len() == ins.len() => {
                self.push_scope();
                for (p, t) in params.iter().zip(ins) {
                    self.declare(p, false, !t.is_copy(), t.clone(), e.line);
                }
                let r = self.tail_type(body).materialized();
                self.pop_scope();
                Type::Fn(ins.to_vec(), Box::new(r))
            }
            // a block this function was handed, passed straight on
            ExprKind::Ident(n) if matches!(self.lookup(n).map(|b| b.ty.clone()), Some(Type::Fn(..))) => {
                self.lookup(n).map(|b| b.ty.clone()).unwrap_or(Type::Unknown)
            }
            // the name of a function used as behaviour
            ExprKind::Ident(n) if self.lookup(n).is_none() => {
                let sg = self.fns.get(&self.canon(n)).cloned().or_else(|| self.bare_method(n));
                match sg {
                    Some(s) => Type::Fn(s.params.iter().map(|(_, t)| t.clone()).collect(), Box::new(s.ret)),
                    None => Type::Unknown,
                }
            }
            _ => Type::Unknown,
        }
    }

    /// What a call to `sig` gives back, with its type parameters filled in.
    fn call_ret(&mut self, sig: &Sig, args: &[Arg]) -> Type {
        if sig.generics.is_empty() {
            return sig.ret.clone();
        }
        let m = self.infer_call(&sig.generics, &sig.params, args, &sig.ret);
        Self::subst(&sig.ret, &m)
    }

    /// The type `Stack(items: [1])` builds.
    fn ctor_type(&mut self, name: &str, info: &StructInfo, args: &[Arg]) -> Type {
        if info.generics.is_empty() {
            return Type::Named(name.to_string());
        }
        let whole = Type::App(name.to_string(), info.generics.iter().map(|p| Type::Var(p.name.clone())).collect());
        let m = self.infer_call(&info.generics, &info.fields, args, &whole);
        Self::subst(&whole, &m)
    }

    /// The type `Node(left, right)` builds for a generic enum.
    fn variant_type(&mut self, en: &str, vname: &str, args: &[Arg]) -> Type {
        let info = match self.enums.get(en) {
            Some(i) => i.clone(),
            None => return Type::Named(en.to_string()),
        };
        if info.generics.is_empty() {
            return Type::Named(en.to_string());
        }
        let fields = info.variants.iter().find(|(n, _)| n == vname).map(|(_, f)| f.clone()).unwrap_or_default();
        let whole = Type::App(en.to_string(), info.generics.iter().map(|p| Type::Var(p.name.clone())).collect());
        let m = self.infer_call(&info.generics, &fields, args, &whole);
        Self::subst(&whole, &m)
    }

    /// `f(x)` where `f` names a block the caller handed in.
    fn call_block(&mut self, name: &str, ins: &[Type], args: &[Arg], e: &Expr) -> Result<String> {
        if let Some(a) = args.iter().find(|a| a.name.is_some()) {
            return Err(LumeError::new(e.line, e.col, format!("a block's arguments have no names, so `{}:` does not fit here", a.name.clone().unwrap()))
                .with_help(format!("`{}` takes {} by position", name, plural(ins.len(), "one value", "its values"))));
        }
        if args.len() != ins.len() {
            return Err(LumeError::new(e.line, e.col, format!(
                "`{}` takes {} {}, but {} {} given",
                name,
                ins.len(),
                plural(ins.len(), "value", "values"),
                args.len(),
                plural(args.len(), "was", "were")
            )));
        }
        // `f(f(x))`: running a block inside its own argument would hold it
        // twice at once, so the inner result is taken first
        let nested = args.iter().any(|a| expr_mentions(&a.value, name));
        let mut parts = Vec::new();
        let mut lets = String::new();
        for (i, (a, t)) in args.iter().zip(ins).enumerate() {
            self.check_assign(&a.value, t, &format!("`{}` takes a `{}`", name, type_name(t)))?;
            let v = self.expr_arg(&a.value, t)?;
            // a block is handed its arguments lent, copied types included
            let v = if *t != Type::Str && t.is_copy() { format!("&({})", v) } else { v };
            if nested {
                let tmp = format!("lume_arg{}", i);
                lets.push_str(&format!("let {} = {}; ", tmp, v));
                parts.push(tmp);
            } else {
                parts.push(v);
            }
        }
        if nested {
            return Ok(format!("({{ {}{}({}) }})", lets, rust_name(name), parts.join(", ")));
        }
        Ok(format!("{}({})", rust_name(name), parts.join(", ")))
    }

    /// A generic signature with this call's type arguments put in, after
    /// checking each one against its bound. A plain signature passes through.
    fn instantiate(&mut self, sig: &Sig, args: &[Arg], what: &str, line: usize, col: usize) -> Result<Sig> {
        if sig.generics.is_empty() {
            return Ok(sig.clone());
        }
        let m = self.infer_call(&sig.generics, &sig.params, args, &sig.ret);
        let targs = self.all_bound(&sig.generics, &m, what, line, col)?;
        self.check_type_args(&sig.generics, &targs, what, line, col)?;
        Ok(Sig {
            params: sig.params.iter().map(|(n, t)| (n.clone(), Self::subst(t, &m))).collect(),
            var_params: sig.var_params.clone(),
            ret: Self::subst(&sig.ret, &m),
            self_kind: sig.self_kind,
            is_async: sig.is_async,
            generics: Vec::new(),
        })
    }

    /// The same for a struct constructor: its fields with `T` filled in.
    fn instantiate_struct(&mut self, name: &str, info: &StructInfo, args: &[Arg], line: usize, col: usize) -> Result<StructInfo> {
        if info.generics.is_empty() {
            return Ok(info.clone());
        }
        let what = format!("`{}`", name);
        let whole = Type::App(name.to_string(), info.generics.iter().map(|p| Type::Var(p.name.clone())).collect());
        let m = self.infer_call(&info.generics, &info.fields, args, &whole);
        let targs = self.all_bound(&info.generics, &m, &what, line, col)?;
        self.check_type_args(&info.generics, &targs, &what, line, col)?;
        Ok(StructInfo {
            generics: Vec::new(),
            fields: info.fields.iter().map(|(n, t)| (n.clone(), Self::subst(t, &m))).collect(),
            methods: info.methods.clone(),
        })
    }

    /// Every parameter the call could not work out, reported once.
    fn all_bound(&self, gs: &[TypeParam], m: &HashMap<String, Type>, what: &str, line: usize, col: usize) -> Result<Vec<Type>> {
        let mut out = Vec::new();
        for p in gs {
            match m.get(&p.name) {
                Some(t) if type_is_known(t) => out.push(t.clone()),
                _ => {
                    return Err(LumeError::new(line, col, format!("cannot tell what `{}` is in this call to {}", p.name, what))
                        .with_help("say it in the binding's type, as in `xs: [Int] = ...`, or pass a value that fixes it"))
                }
            }
        }
        Ok(out)
    }

    // ----- output helpers -------------------------------------------------

    fn line(&mut self, s: &str) {
        for _ in 0..self.indent {
            self.out.push_str("    ");
        }
        self.out.push_str(s);
        self.out.push('\n');
    }

    fn fresh(&mut self, base: &str) -> String {
        self.tmp += 1;
        format!("__{}{}", base, self.tmp)
    }

    // ----- scopes -----------------------------------------------------------

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn lookup(&self, name: &str) -> Option<&Binding> {
        for s in self.scopes.iter().rev() {
            if let Some(b) = s.get(name) {
                return Some(b);
            }
        }
        None
    }

    fn declare(&mut self, name: &str, mutable: bool, borrowed: bool, ty: Type, line: usize) {
        self.scopes.last_mut().unwrap().insert(name.to_string(), Binding { mutable, borrowed, ty, line });
    }

    fn field_type(&self, name: &str) -> Option<Type> {
        let s = self.current_type.as_ref()?;
        let info = self.structs.get(s)?;
        info.fields.iter().find(|(n, _)| n == name).map(|(_, t)| t.clone())
    }

    /// A zero-argument method of the current type, callable bare inside
    /// another method (`total` for `def total`).
    fn bare_method(&self, name: &str) -> Option<Sig> {
        let t = self.current_type.as_ref()?;
        let ms = self.methods_of(t)?;
        let m = ms.get(name)?;
        if m.params.is_empty() { Some(m.clone()) } else { None }
    }

    /// Methods callable on a type: inherent ones plus those from `extend`;
    /// for an interface, its methods; for a built-in, only `extend` methods.
    /// The methods of a concrete type, with a generic `extend`'s names
    /// filled in by what this receiver holds: the methods of `[T]` seen by
    /// a `[Int]` talk about `Int`.
    fn methods_for(&self, t: &Type) -> Option<HashMap<String, Sig>> {
        let key = self.type_key(t);
        let m = self.methods_of(&key)?;
        // reached through a bound that carries arguments — `P: Pairish[Int,
        // Int]` — the interface's own names stand for those arguments
        if let Type::Var(n) = t {
            if let Some(Type::App(iname, iargs)) = self.type_param(n).and_then(|p| p.bound.clone()) {
                if let Some(info) = self.interfaces.get(&self.canon(&iname)) {
                    let sub: HashMap<String, Type> = info.generics.iter().map(|g| g.name.clone()).zip(iargs.iter().cloned()).collect();
                    return Some(m.into_iter().map(|(k, v)| (k, Sig { params: v.params.iter().map(|(pn, x)| (pn.clone(), Self::subst(x, &sub))).collect(), ret: Self::subst(&v.ret, &sub), ..v })).collect());
                }
            }
        }
        if !self.ext_generics.contains_key(&key) {
            return Some(m);
        }
        Some(
            m.into_iter()
                .map(|(k, v)| {
                    // Each method is read through the target its own
                    // `extend` was written against. A name this receiver
                    // does not pin stays as written: a `[Int]` says what
                    // `T` is, the shape `[T]` itself does not.
                    let mut sub: HashMap<String, Type> = HashMap::new();
                    if let Some(shape) = self.ext_method_target.get(&(key.clone(), k.clone())) {
                        Self::unify(shape, t, &mut sub);
                        sub.retain(|_, x| *x != Type::Unknown);
                    }
                    let v = Sig { params: v.params.iter().map(|(n, x)| (n.clone(), Self::subst(x, &sub))).collect(), ret: Self::subst(&v.ret, &sub), ..v };
                    (k, v)
                })
                .collect(),
        )
    }

    fn methods_of(&self, tname: &str) -> Option<HashMap<String, Sig>> {
        let mut m: HashMap<String, Sig> = if let Some(s) = self.structs.get(tname) {
            s.methods.clone()
        } else if let Some(e) = self.enums.get(tname) {
            e.methods.clone()
        } else if let Some(i) = self.interfaces.get(tname) {
            i.methods.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        } else if self.ext_methods.contains_key(tname) {
            HashMap::new()
        } else {
            return None;
        };
        if let Some(ext) = self.ext_methods.get(tname) {
            for (k, v) in ext {
                m.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
        Some(m)
    }

    fn is_type(&self, name: &str) -> bool {
        self.structs.contains_key(name) || self.enums.contains_key(name) || self.interfaces.contains_key(name) || self.foreign_types.contains_key(name)
    }

    fn foreign_type(&self, t: &Type) -> Option<&crate::bridge::ForeignType> {
        match t {
            Type::Named(n) => self.foreign_types.get(n),
            _ => None,
        }
    }

    /// Resolves `alias.path.to.item` through a crate's namespaces. `e` is
    /// the whole chain (the call's receiver, or the call itself when its
    /// last segment is a function).
    fn foreign_ref(&self, e: &Expr) -> Option<ForeignRef<'_>> {
        // collect segments from the root outwards
        let mut segs: Vec<&str> = Vec::new();
        let mut cur = e;
        loop {
            match &cur.kind {
                ExprKind::Method { recv, name, args } if args.is_empty() || std::ptr::eq(cur, e) => {
                    segs.push(name);
                    cur = recv;
                }
                ExprKind::Ident(n) => {
                    segs.push(n);
                    break;
                }
                _ => return None,
            }
        }
        segs.reverse();
        let alias = segs[0];
        if self.lookup(alias).is_some() || self.field_type(alias).is_some() {
            return None;
        }
        let mut ns = self.crates.get(alias)?;
        let mut prefix = alias.to_string();
        let mut i = 1;
        while i < segs.len() {
            let seg = segs[i];
            let last = i + 1 == segs.len();
            if let Some(sub) = ns.modules.get(seg) {
                ns = sub;
                prefix = format!("{}.{}", prefix, seg);
                i += 1;
                continue;
            }
            if let Some(f) = ns.fns.get(seg) {
                return if last { Some(ForeignRef::Fn(f)) } else { None };
            }
            if let Some(key) = ns.types.get(seg) {
                if last {
                    return Some(ForeignRef::Type(key.clone()));
                }
                let ft = self.foreign_types.get(key)?;
                let f = ft.assoc.get(segs[i + 1])?;
                return if i + 2 == segs.len() { Some(ForeignRef::Assoc(key.clone(), f)) } else { None };
            }
            return None;
        }
        None
    }

    /// A statement whose value is dropped on the floor. Most values may be:
    /// `xs.push(1)` gives a `()` nobody wants. A failure may not — an `Error`
    /// written without `return` used to evaporate, and a call that can fail
    /// used to carry on as if it had not.
    fn check_not_dropped(&mut self, e: &Expr, t: &Type) -> Result<()> {
        let t = t.materialized();
        // A bare error value: the failure path, written and then not taken.
        let err_here = match &self.current_ret {
            Type::Result(_, err_t) => t == **err_t && t != Type::Unknown,
            _ => false,
        };
        if err_here {
            return Err(LumeError::new(e.line, e.col, "this failure is thrown away, because nothing returns it")
                .with_help(format!("write `return {}` to stop here with it", snippet(e))));
        }
        if let Type::Result(ok, _) = &t {
            let what = match &e.kind {
                ExprKind::Call { name, .. } => format!("`{}` can fail", name),
                ExprKind::Method { name, .. } => format!("`{}` can fail", name),
                _ => "this can fail".to_string(),
            };
            let seen = if **ok == Type::Unit { "whether it did" } else { "the result" };
            return Err(LumeError::new(e.line, e.col, format!("{}, and nothing here looks at {}", what, seen))
                .with_help("pass the failure on with `?`, handle it with `match`, or say you mean to drop it: `_ = ...`"));
        }
        Ok(())
    }

    /// A value written into a `format!` or a `println!`: the value-as-itself
    /// form of milestone 32. For the four types whose `lume_str` is exactly
    /// Rust's `Display`, the value goes in as it stands, so `puts "n is #{n}"`
    /// builds one string instead of three. A `Float` is not one of them — it
    /// keeps its point — and neither is anything that renders its insides.
    fn shown(&mut self, e: &Expr) -> Result<String> {
        // `puts None` with nothing to say what it would have held. Every
        // `T?` prints the same word here, so pick a `T` rather than leak
        // rustc's complaint about a type parameter the program never named.
        if matches!(e.kind, ExprKind::None) && matches!(self.ty_of(e).materialized(), Type::Option(i) if *i == Type::Unknown) {
            return Ok("(None::<i64>).lume_str()".to_string());
        }
        let v = self.expr_val(e)?;
        Ok(match self.ty_of(e).materialized() {
            Type::Str | Type::Int | Type::Bool | Type::Char => v,
            _ => format!("({}).lume_str()", v),
        })
    }

    /// `puts` and `warn`. An interpolated line goes straight to the terminal:
    /// `puts "n is #{n}"` writes its pieces out rather than building a string
    /// first, which is the shape most Lume programs print in.
    fn print_call(&mut self, macro_name: &str, arg: &Expr) -> Result<String> {
        if let ExprKind::Str(pieces) = &arg.kind {
            if pieces.iter().any(|p| matches!(p, StrPiece::Expr(_))) {
                let (fmt, args) = self.interp(pieces)?;
                return Ok(format!("{}!(\"{}\", {})", macro_name, fmt, args.join(", ")));
            }
        }
        self.check_printable(arg)?;
        let a = self.shown(arg)?;
        Ok(format!("{}!(\"{{}}\", {})", macro_name, a))
    }

    /// The Rust format string and arguments behind `"a #{b} c"`.
    fn interp(&mut self, pieces: &[StrPiece]) -> Result<(String, Vec<String>)> {
        let mut fmt = String::new();
        let mut args = Vec::new();
        for p in pieces {
            match p {
                StrPiece::Lit(s) => fmt.push_str(&escape_rust_str(s, true)),
                StrPiece::Expr(x) => {
                    self.check_printable(x)?;
                    fmt.push_str("{}");
                    args.push(self.shown(x)?);
                }
            }
        }
        Ok((fmt, args))
    }

    /// A crate type can be printed only when the crate says how (`Display`).
    fn check_printable(&mut self, e: &Expr) -> Result<()> {
        let t = self.ty_of(e).materialized();
        let inner = match &t {
            Type::List(i) | Type::Option(i) => (**i).clone(),
            other => other.clone(),
        };
        if let Some(ft) = self.foreign_type(&inner) {
            if !ft.display && ft.iter_item.is_none() {
                let key = match &inner { Type::Named(n) => n.clone(), _ => String::new() };
                let mut err = LumeError::new(e.line, e.col, format!("`{}` values cannot be printed: the crate gives them no text form", key));
                let names: Vec<&String> = ft.methods.iter().filter(|(_, f)| f.ret == Type::Str && f.params.is_empty()).map(|(n, _)| n).collect();
                if !names.is_empty() {
                    err = err.with_help(format!("print one of its parts instead: {}", names.iter().map(|n| format!(".{}", n)).collect::<Vec<_>>().join(", ")));
                }
                return Err(err);
            }
        }
        Ok(())
    }

    /// Is `e` a dotted crate path whose root is a crate alias? (for messages)
    fn foreign_root(&self, e: &Expr) -> Option<String> {
        let mut cur = e;
        loop {
            match &cur.kind {
                ExprKind::Method { recv, .. } => cur = recv,
                ExprKind::Ident(n) if self.crates.contains_key(n) && self.lookup(n).is_none() => return Some(n.clone()),
                _ => return None,
            }
        }
    }

    /// One argument for a crate function, converted the way its parameter wants.
    fn foreign_arg(&mut self, a: &Expr, p: &crate::bridge::ForeignParam, callee: &str) -> Result<String> {
        use crate::bridge::Pass;
        let at = self.ty_of(a).materialized();
        if at != Type::Unknown && p.ty != Type::Unknown && !types_compatible(&at, &p.ty) {
            return Err(LumeError::new(a.line, a.col, format!("`{}` takes `{}: {}`, but this is a `{}`", callee, p.name, type_name(&p.ty), type_name(&at))));
        }
        Ok(match &p.pass {
            Pass::Borrow => {
                if let Type::Iter(elem, by_ref) = self.ty_of(a) {
                    let s = self.expr(a)?;
                    let c = self.collect_iter_t(&s, by_ref, &elem);
                    format!("&{}", c)
                } else {
                    let s = self.expr(a)?;
                    if self.is_borrowed_ident(a) { format!("&*{}", s) } else { format!("&{}", s) }
                }
            }
            Pass::Owned => self.expr_owned(a)?,
            Pass::IntCast(w) => format!("(({}) as {})", self.expr_val(a)?, w),
            Pass::F32 => format!("(({}) as f32)", self.expr_val(a)?),
            Pass::Option(inner) => {
                let v = self.expr_owned(a)?;
                match &**inner {
                    Pass::IntCast(w) => format!("({}).map(|lume_x| lume_x as {})", v, w),
                    Pass::F32 => format!("({}).map(|lume_x| lume_x as f32)", v),
                    _ => v,
                }
            }
            Pass::MutBorrow => self.expr_var_arg(a, &p.name, callee)?,
        })
    }

    /// Emits a call to a crate function or associated function.
    fn foreign_call(&mut self, f: &crate::bridge::ForeignFn, display: &str, args: &[Arg], e: &Expr) -> Result<String> {
        if let Some(why) = &f.unsupported {
            return Err(LumeError::new(e.line, e.col, format!("`{}` cannot be called from Lume yet: {}", display, why))
                .with_help("call it from a `rust:` block, which can use any Rust"));
        }
        let params: Vec<(String, Type)> = f.params.iter().map(|p| (p.name.clone(), p.ty.clone())).collect();
        let bound = self.bind_args(&format!("`{}`", display), &params, args, e.line, e.col)?;
        let mut parts = Vec::new();
        for (a, p) in bound.iter().zip(&f.params) {
            parts.push(self.foreign_arg(a, p, display)?);
        }
        Ok(f.ret_conv.apply(&format!("{}({})", f.rust_path, parts.join(", "))))
    }

    /// Emits a method call on a value of a crate type.
    fn foreign_method(&mut self, recv: &Expr, key: &str, name: &str, args: &[Arg], e: &Expr) -> Result<String> {
        let ft = self.foreign_types[key].clone();
        let f = match ft.methods.get(name) {
            Some(f) => f.clone(),
            None => {
                let err = LumeError::new(e.line, e.col, format!("`{}` has no method `{}`", key, name));
                let names = ft.methods.keys().cloned();
                return Err(match self.suggest_from(name, names) {
                    Some(sug) => err.with_help(format!("did you mean `{}`?", sug)),
                    None => err.with_help(format!("`lume crate <file> {}` lists what the crate offers", key.split('.').next().unwrap_or(key))),
                });
            }
        };
        let display = format!("{}.{}", key, name);
        if let Some(why) = &f.unsupported {
            return Err(LumeError::new(e.line, e.col, format!("`{}` cannot be called from Lume yet: {}", display, why))
                .with_help("call it from a `rust:` block, which can use any Rust"));
        }
        if f.self_kind == Some(SelfKind::Mutate) {
            self.check_receiver_mutable(recv, name, e.line, e.col)?;
        }
        let params: Vec<(String, Type)> = f.params.iter().map(|p| (p.name.clone(), p.ty.clone())).collect();
        let bound = self.bind_args(&format!("`{}`", display), &params, args, e.line, e.col)?;
        let mut parts = Vec::new();
        for (a, p) in bound.iter().zip(&f.params) {
            parts.push(self.foreign_arg(a, p, &display)?);
        }
        let r = self.expr(recv)?;
        Ok(f.ret_conv.apply(&format!("({}).{}({})", r, name, parts.join(", "))))
    }

    fn is_interface(&self, t: &Type) -> bool {
        matches!(t, Type::Named(n) | Type::App(n, _) if self.interfaces.contains_key(&self.canon(n)))
    }

    /// The key under which `extend` methods and conformance are recorded
    /// for a type: its name for structs/enums, its spelling for built-ins.
    fn type_key(&self, t: &Type) -> String {
        match t {
            Type::Named(n) | Type::App(n, _) => self.canon(n),
            // a type parameter answers with the methods its bound promises
            Type::Var(n) => match self.type_param(n).and_then(|p| p.bound.clone()) {
                Some(b) if !Self::is_builtin_bound(Self::bound_name(&b)) => self.type_key(&b),
                _ => type_name(t),
            },
            other => {
                // a built-in container that a generic `extend` covers answers
                // under its shape, so `[Int]` finds what `[T]` was given
                if let Some(shape) = Self::shape_key(other) {
                    if self.ext_generics.contains_key(&shape) {
                        return shape;
                    }
                }
                type_name(other)
            }
        }
    }

    /// One `extend` block, with its type parameters already in scope.
    fn register_extend(&mut self, x: &ExtendDef, gens: &[TypeParam]) -> Result<()> {
        let target = self.ct(&x.target);
        self.check_type(&target, x.line, x.col)?;
        let iface_ty = self.ct(&x.iface);
        let iface = self.type_key(&iface_ty);
        if !self.interfaces.contains_key(&iface) {
            return Err(LumeError::new(x.line, x.col, format!("unknown interface `{}`", type_name(&x.iface))));
        }
        if self.is_interface(&target) {
            return Err(LumeError::new(x.line, x.col, "an interface cannot be extended with another; extend the concrete types"));
        }
        // A generic extend on a built-in has no name to register under, so
        // it registers under the container's shape; a user's own generic
        // type already keys by its name.
        let key = match (gens.is_empty(), Self::shape_key(&target)) {
            (false, Some(shape)) => shape,
            _ => self.type_key(&target),
        };
        // An `extend` travels with the import, so two of them claiming the
        // same type and interface would be two answers to one question.
        if let Some((other_file, other_line)) = self.ext_where.get(&(key.clone(), iface.clone())) {
            let here = format!("{}:{}", self.this_file, x.line);
            let there = format!("{}:{}", other_file, other_line);
            if here != there {
                let _ = here;
                return Err(LumeError::new(x.line, x.col, format!("`{}` is already a `{}`", type_name(&target), type_name(&x.iface)))
                    .with_help(format!("{} extends it too, and an `extend` travels with the import, so only one of them can hold: remove one, or give one of them an interface of its own", there)));
            }
        }
        self.ext_where.insert((key.clone(), iface.clone()), (self.this_file.clone(), x.line));
        if !gens.is_empty() {
            let all = self.ext_generics.entry(key.clone()).or_default();
            for g in gens {
                if !all.iter().any(|p| p.name == g.name) {
                    all.push(g.clone());
                }
            }
            self.ext_impl_generics.insert((key.clone(), iface.clone()), gens.to_vec());
        }
        if !matches!(target, Type::Named(_)) {
            self.ext_targets.insert(key.clone(), target.clone());
        }
        let existing = self.methods_of(&key).unwrap_or_default();
        for m in &x.methods {
            if m.self_kind == SelfKind::Mutate {
                return Err(LumeError::new(m.line, m.col, "methods in an `extend` block cannot take `var self`"));
            }
            if !self.interfaces[&iface].methods.contains_key(&m.name) {
                return Err(LumeError::new(m.line, m.col, format!("`{}` is not a method of `{}`", m.name, type_name(&x.iface)))
                    .with_help(format!("`{}` has: {}", type_name(&x.iface), self.interfaces[&iface].methods.keys().cloned().collect::<Vec<_>>().join(", "))));
            }
            if existing.contains_key(&m.name) {
                return Err(LumeError::new(m.line, m.col, format!("`{}` already has a `{}` method", type_name(&target), m.name)));
            }
            let sg = self.sig_of(m);
            if !gens.is_empty() {
                self.ext_method_target.insert((key.clone(), m.name.clone()), target.clone());
            }
            self.ext_methods.entry(key.clone()).or_default().insert(m.name.clone(), sg);
        }
        // A generic `extend` covers a built-in, whose own methods are not
        // in any table, so say here what is missing rather than letting a
        // call site report a method the type "does not have".
        if !gens.is_empty() {
            let info = self.interfaces[&iface].clone();
            let missing: Vec<String> = info
                .required
                .iter()
                .filter(|r| {
                    !x.methods.iter().any(|m| m.name == **r)
                        && !existing.contains_key(*r)
                        && builtin_method_type(&target, r) == Type::Unknown
                        && !is_builtin_name(r)
                })
                .cloned()
                .collect();
            if !missing.is_empty() {
                let needs: Vec<String> = missing.iter().filter_map(|n| info.methods.get(n).map(|s| format!("def {} {}", n, self.describe_sig(s)))).collect();
                return Err(LumeError::new(x.line, x.col, format!("`extend {} with {}` is missing `{}`", type_name(&target), type_name(&x.iface), missing.join("`, `")))
                    .with_help(format!("`{}` needs: {}. Add {} inside this `extend` block", type_name(&x.iface), needs.join("; "), if missing.len() == 1 { "it" } else { "them" })));
            }
        }
        Ok(())
    }

    /// The key an `extend` block registers under, generic or not.
    fn ext_key_of(&self, x: &ExtendDef) -> String {
        let mut names = Vec::new();
        self.target_params(&x.target, false, &mut names);
        if !names.is_empty() {
            // the shape does not depend on what the target holds, so it can
            // be read straight off the written type
            if let Some(shape) = Self::shape_key(&x.target) {
                return shape;
            }
        }
        self.type_key(&self.ct(&x.target))
    }

    /// `Chain<T>` names a type; `Chain::<T>` names it in a path, which is
    /// what calling one of its own methods needs.
    fn turbofish(rust_ty: &str) -> String {
        match rust_ty.find('<') {
            Some(i) => format!("{}::{}", &rust_ty[..i], &rust_ty[i..]),
            None => rust_ty.to_string(),
        }
    }

    /// The shape of a built-in container, ignoring what it holds: `[Int]`
    /// and `[T]` are both `[]`. A generic `extend` on a built-in registers
    /// under this, because there is no name to register under. A user's own
    /// generic type needs none — `Stack[Int]` already keys as `Stack`.
    fn shape_key(t: &Type) -> Option<String> {
        Some(match t {
            Type::List(_) => "[]".to_string(),
            Type::Set(_) => "{}".to_string(),
            Type::Map(..) => "{:}".to_string(),
            Type::Option(_) => "?".to_string(),
            // a tuple's shape is how many parts it has
            Type::Tuple(parts) => format!("({})", ",".repeat(parts.len().saturating_sub(1))),
            _ => return None,
        })
    }

    /// The names a generic `extend` target introduces: a name inside the
    /// target that is not a type this program knows. Only names nested in a
    /// container count, so `extend Poimt with Named` is still an unknown
    /// type rather than a silent type parameter.
    fn target_params(&self, t: &Type, nested: bool, out: &mut Vec<String>) {
        let mut pairs = Vec::new();
        self.target_params_ex(t, nested, &mut pairs);
        for (n, _) in pairs {
            if !out.contains(&n) {
                out.push(n);
            }
        }
    }

    /// The same, with the bound the container itself implies: a set's item
    /// and a map's key must be `Hashable`, so an `extend` on one need not
    /// say so (and, inside `{...}`, could not).
    fn target_params_ex(&self, t: &Type, nested: bool, out: &mut Vec<(String, Option<Type>)>) {
        let implied = |n: &String, b: Option<&str>, out: &mut Vec<(String, Option<Type>)>| {
            if !out.iter().any(|(x, _)| x == n) {
                out.push((n.clone(), b.map(|s| Type::Named(s.to_string()))));
            }
        };
        match t {
            Type::Named(n) => {
                if nested && !self.is_type(&self.canon(n)) {
                    implied(n, None, out);
                }
            }
            Type::Set(i) | Type::Map(i, _) => {
                if let Type::Named(n) = &**i {
                    if !self.is_type(&self.canon(n)) {
                        implied(n, Some("Hashable"), out);
                    }
                } else {
                    self.target_params_ex(i, true, out);
                }
                if let Type::Map(_, v) = t {
                    self.target_params_ex(v, true, out);
                }
            }
            Type::List(i) | Type::Option(i) => self.target_params_ex(i, true, out),
            Type::Tuple(parts) => parts.iter().for_each(|x| self.target_params_ex(x, true, out)),
            Type::App(_, args) => args.iter().for_each(|x| self.target_params_ex(x, true, out)),
            _ => {}
        }
    }

    /// Does `t` satisfy interface `iface`? Structural: every required
    /// method exists with the same parameters and result; defaults that the
    /// type also defines must match too.
    /// A default method of an interface this type conforms to: a conforming
    /// type has the defaults too, without naming the interface anywhere.
    fn iface_default(&self, t: &Type, name: &str) -> Option<Sig> {
        self.iface_default_ex(t, name).map(|(s, _)| s)
    }

    /// The default's signature as this type sees it, and the types the
    /// interface declared: where it said `T`, Rust lends the value.
    fn iface_default_ex(&self, t: &Type, name: &str) -> Option<(Sig, Vec<Type>)> {
        if self.is_interface(t) {
            return None;
        }
        let mut names: Vec<&String> = self.interfaces.keys().collect();
        names.sort();
        for iname in names {
            let info = &self.interfaces[iname];
            if info.required.contains(&name.to_string()) {
                continue;
            }
            if let Some(sig) = info.methods.get(name) {
                let (verdict, args) = self.conformance_with(t, &Type::Named(iname.clone()));
                if matches!(verdict, Conformance::Yes) {
                    // a generic interface's default speaks of the arguments
                    // this type conforms with
                    let sub: HashMap<String, Type> = info.generics.iter().map(|g| g.name.clone()).zip(args).collect();
                    let filled = Sig { params: sig.params.iter().map(|(n, x)| (n.clone(), Self::subst(x, &sub))).collect(), ret: Self::subst(&sig.ret, &sub), ..sig.clone() };
                    return Some((filled, sig.params.iter().map(|(_, x)| x.clone()).collect()));
                }
            }
        }
        None
    }

    /// Which interface supplies `name` to `t` as a default.
    fn default_owner(&self, t: &Type, name: &str) -> Option<String> {
        if self.is_interface(t) {
            return None;
        }
        let mut names: Vec<&String> = self.interfaces.keys().collect();
        names.sort();
        for iname in names {
            let info = &self.interfaces[iname];
            if info.required.contains(&name.to_string()) {
                continue;
            }
            if info.methods.contains_key(name) && matches!(self.conformance_with(t, &Type::Named(iname.clone())).0, Conformance::Yes) {
                return Some(iname.clone());
            }
        }
        None
    }

    /// An interface default whose body calls the very method it defines.
    /// A generic `extend` makes this easy to write by accident: once
    /// `extend [T] with Bag[T]` exists, a list has `Bag`'s methods, so a
    /// default written as `items.empty?` no longer means the built-in.
    fn check_default_recursion(&self, rt: &Type, name: &str, line: usize, col: usize) -> Result<()> {
        if !self.in_trait_impl || name != self.current_fn {
            return Ok(());
        }
        let owner = match self.default_owner(rt, name) {
            Some(o) => o,
            None => return Ok(()),
        };
        if Some(&owner) != self.current_type.as_ref() {
            return Ok(());
        }
        Err(LumeError::new(line, col, format!("`{}` here calls itself, and would never stop", name))
            .with_help(format!(
                "`{}` conforms to `{}`, so inside this default `{}` means this very method. Give the interface's method a name of its own, or write the body with a method the type already had",
                type_name(rt),
                owner,
                name
            )))
    }

    /// How the interface that owns `method` for type `key` declared its
    /// parameters. An `extend` method is reached through the trait, so its
    /// arguments travel the way the trait says, not the way the `extend`
    /// block spelled them.
    fn iface_decl_params(&self, key: &str, method: &str) -> Vec<Type> {
        let mut names: Vec<&String> = self.interfaces.keys().collect();
        names.sort();
        for iname in names {
            let info = &self.interfaces[iname];
            if info.generics.is_empty() {
                continue;
            }
            if let Some(sig) = info.methods.get(method) {
                if self.ext_methods.get(key).map(|m| m.contains_key(method)).unwrap_or(false) {
                    return sig.params.iter().map(|(_, t)| t.clone()).collect();
                }
            }
        }
        Vec::new()
    }

    fn conformance(&self, t: &Type, iface: &Type) -> Conformance {
        self.conformance_with(t, iface).0
    }

    /// Conformance, plus the arguments the interface ends up with: for a
    /// generic interface given no arguments, they are read off the type's
    /// own methods (`Version` has `def compare(o: Version)`, so it is a
    /// `Comparable[Version]`).
    fn conformance_with(&self, t: &Type, iface: &Type) -> (Conformance, Vec<Type>) {
        let key = self.type_key(iface);
        let info = match self.interfaces.get(&key) {
            Some(i) => i,
            None => return (Conformance::Missing(vec![]), Vec::new()),
        };
        if self.is_interface(t) {
            let same = self.type_key(t) == key;
            return (if same { Conformance::Yes } else { Conformance::Missing(info.required.clone()) }, Vec::new());
        }
        let have = self.methods_for(t).unwrap_or_default();
        // what the interface's own parameters stand for here
        let mut sub: HashMap<String, Type> = HashMap::new();
        if let Type::App(_, args) = iface {
            for (p, a) in info.generics.iter().zip(args) {
                sub.insert(p.name.clone(), a.clone());
            }
        } else if !info.generics.is_empty() {
            // no arguments written: read them off the methods the type has
            for (name, want) in &info.methods {
                if let Some(got) = have.get(name) {
                    for ((_, w), (_, g)) in want.params.iter().zip(&got.params) {
                        Self::unify(w, g, &mut sub);
                    }
                    Self::unify(&want.ret, &got.ret, &mut sub);
                }
            }
        }
        let mut missing = Vec::new();
        for (name, want) in &info.methods {
            let want = Sig { params: want.params.iter().map(|(n, x)| (n.clone(), Self::subst(x, &sub))).collect(), ret: Self::subst(&want.ret, &sub), ..want.clone() };
            match have.get(name) {
                None => {
                    if info.required.contains(name) {
                        missing.push(name.clone());
                    }
                }
                Some(got) => {
                    let same = got.params.len() == want.params.len()
                        && got.params.iter().zip(&want.params).all(|((_, a), (_, b))| a == b || *a == Type::Unknown || *b == Type::Unknown)
                        && (got.ret == want.ret || got.ret == Type::Unknown || want.ret == Type::Unknown);
                    if !same {
                        return (Conformance::Mismatch { method: name.clone(), expected: self.describe_sig(&want), actual: self.describe_sig(got) }, Vec::new());
                    }
                }
            }
        }
        missing.sort();
        // An interface that asks for nothing — every method has a default —
        // would otherwise be satisfied by every type in the program, which is
        // not a claim anybody means to make. Conformance has to rest on at
        // least one method the type really has; an `extend` says so outright
        // and is registered before this runs.
        if !info.methods.is_empty() && !info.methods.keys().any(|n| have.contains_key(n)) {
            return (Conformance::Missing(info.methods.keys().cloned().collect()), Vec::new());
        }
        let args: Vec<Type> = info.generics.iter().map(|p| sub.get(&p.name).cloned().unwrap_or(Type::Unknown)).collect();
        if !missing.is_empty() {
            return (Conformance::Missing(missing), args);
        }
        // a parameter no method mentions cannot be worked out
        if args.iter().any(|a| !type_is_known(a)) {
            return (Conformance::Missing(Vec::new()), args);
        }
        (Conformance::Yes, args)
    }

    fn describe_sig(&self, s: &Sig) -> String {
        let ps: Vec<String> = s.params.iter().map(|(n, t)| format!("{}: {}", n, type_name(t))).collect();
        if ps.is_empty() { format!("-> {}", type_name(&s.ret)) } else { format!("({}) -> {}", ps.join(", "), type_name(&s.ret)) }
    }

    /// Error for a value of type `t` used where interface `iface` is needed.
    fn require_conforms(&self, t: &Type, iface: &Type, line: usize, col: usize) -> Result<()> {
        let key = self.type_key(iface);
        // Check against the interface as it was asked for, arguments and
        // all: re-checking against the bare name lets the arguments be
        // inferred again, which accepts whatever the type happens to say.
        let asked = self.ct(iface);
        let iface = &type_name(iface).clone();
        match self.conformance(t, &asked) {
            Conformance::Yes => Ok(()),
            Conformance::Missing(m) => {
                let info = match self.interfaces.get(&key) {
                    Some(i) => i,
                    None => return Err(LumeError::new(line, col, format!("unknown interface `{}`", iface))),
                };
                let needs: Vec<String> = m.iter().filter_map(|n| info.methods.get(n).map(|s| format!("def {} {}", n, self.describe_sig(s)))).collect();
                Err(LumeError::new(line, col, format!("`{}` is used as a `{}` here, but it has no `{}` method", type_name(t), iface, m.join("`, `")))
                    .with_help(format!("`{}` needs: {}. Add {} to `{}`, or `extend {} with {}:`", iface, needs.join("; "), if m.len() == 1 { "it" } else { "them" }, type_name(t), type_name(t), iface)))
            }
            Conformance::Mismatch { method, expected, actual } => Err(LumeError::new(
                line,
                col,
                format!("`{}` is used as a `{}`, but its `{}` is `{}` and `{}` needs `{}`", type_name(t), iface, method, actual, iface, expected),
            )),
        }
    }

    /// Boxes a concrete value where an interface value is stored.
    fn coerce(&self, text: String, from: &Type, to: &Type, line: usize, col: usize) -> Result<String> {
        if *from == Type::Char && *to == Type::Str {
            return Ok(format!("({}).to_string()", text));
        }
        if let Type::Named(_) | Type::App(..) = to {
            if self.is_interface(to) && !self.is_interface(from) && *from != Type::Unknown {
                self.require_conforms(from, to, line, col)?;
                return Ok(format!("(::std::boxed::Box::new({}) as ::std::boxed::Box<dyn {}>)", text, self.rust_iface(to)));
            }
        }
        Ok(text)
    }

    /// Emits the one-time note for a declaration whose type holds interface
    /// values behind a pointer.
    fn note_dynamic(&mut self, name: &str, t: &Type, line: usize, col: usize) {
        fn boxed_iface<'a>(g: &'a Gen, t: &'a Type) -> Option<&'a str> {
            match t {
                Type::List(i) | Type::Option(i) | Type::Iter(i, _) => match &**i {
                    Type::Named(n) if g.is_interface(i) => Some(n.as_str()),
                    other => boxed_iface(g, other),
                },
                Type::Map(_, v) => boxed_iface(g, v),
                Type::Tuple(ts) => ts.iter().find_map(|x| boxed_iface(g, x)),
                _ => None,
            }
        }
        if let Some(iface) = boxed_iface(self, t) {
            self.warnings.push(
                LumeError::new(line, col, format!("`{}` holds values of different types behind `{}`, so calls on its items go through a pointer", name, iface))
                    .with_help("fine for most code; use a single concrete type or an enum if this is a hot loop"),
            );
        }
    }

    /// Enums that have a variant with this name.
    fn enums_with_variant(&self, v: &str) -> Vec<String> {
        let mut out: Vec<String> = self.enums.iter().filter(|(_, e)| e.variants.iter().any(|(n, _)| n == v)).map(|(n, _)| n.clone()).collect();
        out.sort_by_key(|k| (k.matches('.').count(), k.clone()));
        let mut seen = HashSet::new();
        out.retain(|k| seen.insert(self.path_of(k)));
        out
    }

    /// Resolves a bare variant name: the current enum's variant first, then
    /// any enum that has it uniquely.
    fn resolve_variant(&self, v: &str, line: usize, col: usize) -> Result<Option<String>> {
        if let Some(t) = &self.current_type {
            if let Some(e) = self.enums.get(t) {
                if e.variants.iter().any(|(n, _)| n == v) {
                    return Ok(Some(t.clone()));
                }
            }
        }
        let owners = self.enums_with_variant(v);
        match owners.len() {
            0 => Ok(None),
            1 => Ok(Some(owners[0].clone())),
            _ => {
                // the position says which enum is meant: a return type, an
                // argument, a field, a typed binding, a list element
                if let Some(en) = self.wanted_enum() {
                    if owners.contains(&en) {
                        return Ok(Some(en));
                    }
                }
                Err(LumeError::new(line, col, format!("`{}` is a variant of more than one enum: {}", v, owners.join(", ")))
                    .with_help(format!("write `{}.{}`", owners[0], v)))
            }
        }
    }

    /// The enum the current position expects, looking through `T?`, `T or E`,
    /// `[T]` and `{T}`.
    fn wanted_enum(&self) -> Option<String> {
        fn inner(t: &Type) -> Option<String> {
            match t {
                Type::Named(n) => Some(n.clone()),
                Type::Option(i) | Type::List(i) | Type::Set(i) | Type::Iter(i, _) => inner(i),
                Type::Result(ok, _) => inner(ok),
                _ => None,
            }
        }
        let t = self.want.last()?;
        let en = inner(t)?;
        if self.enums.contains_key(&en) { Some(en) } else { None }
    }

    fn all_names(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        for s in &self.scopes {
            v.extend(s.keys().cloned());
        }
        v.extend(self.fns.keys().cloned());
        v.extend(self.structs.keys().cloned());
        v.extend(self.enums.keys().cloned());
        if let Some(s) = &self.current_type {
            if let Some(info) = self.structs.get(s) {
                v.extend(info.fields.iter().map(|(n, _)| n.clone()));
            }
            if let Some(m) = self.methods_of(s) {
                v.extend(m.keys().cloned());
            }
        }
        v
    }

    fn suggest_from(&self, name: &str, cands: impl Iterator<Item = String>) -> Option<String> {
        let mut best: Option<(usize, String)> = None;
        for cand in cands {
            let d = edit_distance(name, &cand);
            if d <= 2 && d < name.len() && best.as_ref().map(|b| d < b.0).unwrap_or(true) {
                best = Some((d, cand));
            }
        }
        best.map(|b| b.1)
    }

    fn suggest(&self, name: &str) -> Option<String> {
        self.suggest_from(name, self.all_names().into_iter())
    }

    /// Makes an imported module's public items visible under `key_prefix`
    /// ("model" for an alias, "users.model" for the full id).
    fn register_module_items(&mut self, key_prefix: &str, id: &str, ex: &Exports, line: usize, col: usize) -> Result<()> {
        for (n, info) in &ex.structs {
            let key = format!("{}.{}", key_prefix, n);
            let qualified = StructInfo {
                generics: info.generics.clone(),
                fields: info.fields.iter().map(|(f, t)| (f.clone(), qualify_type(t, id, ex))).collect(),
                methods: info.methods.iter().map(|(m, sg)| (m.clone(), qualify_sig(sg, id, ex))).collect(),
            };
            self.structs.insert(key.clone(), qualified);
            self.paths.insert(key, format!("{}::{}", ex.rust_mod, n));
        }
        for (n, info) in &ex.enums {
            let key = format!("{}.{}", key_prefix, n);
            let qualified = EnumInfo {
                generics: info.generics.clone(),
                variants: info.variants.iter().map(|(v, fs)| (v.clone(), fs.iter().map(|(f, t)| (f.clone(), qualify_type(t, id, ex))).collect())).collect(),
                methods: info.methods.iter().map(|(m, sg)| (m.clone(), qualify_sig(sg, id, ex))).collect(),
            };
            self.enums.insert(key.clone(), qualified);
            self.paths.insert(key, format!("{}::{}", ex.rust_mod, n));
        }
        for (n, sig) in &ex.fns {
            let key = format!("{}.{}", key_prefix, n);
            self.fns.insert(key.clone(), qualify_sig(sig, id, ex));
            self.paths.insert(key, format!("{}::{}", ex.rust_mod, n));
        }
        for (n, t) in &ex.consts {
            let key = format!("{}.{}", key_prefix, n);
            self.consts.insert(key.clone(), qualify_type(t, id, ex));
            self.paths.insert(key, format!("{}::{}", ex.rust_mod, n));
        }
        for (n, info) in &ex.interfaces {
            let key = format!("{}.{}", key_prefix, n);
            let qualified = IfaceInfo {
                generics: info.generics.clone(),
                methods: info.methods.iter().map(|(m, sg)| (m.clone(), qualify_sig(sg, id, ex))).collect(),
                required: info.required.clone(),
                defaults: Vec::new(),
                local: false,
            };
            self.interfaces.insert(key.clone(), qualified);
            self.paths.insert(key, format!("{}::{}", ex.rust_mod, n));
            let u = format!("use crate::{}::{};", ex.rust_mod, n);
            if !self.trait_uses.contains(&u) {
                self.trait_uses.push(u);
            }
        }
        let requalify = |tkey: &String| -> String {
            if ex.structs.contains_key(tkey) || ex.enums.contains_key(tkey) {
                format!("{}.{}", key_prefix, tkey)
            } else {
                tkey.clone()
            }
        };
        for (tkey, ms) in &ex.ext_methods {
            let tkey = requalify(tkey);
            let entry = self.ext_methods.entry(tkey).or_default();
            for (m, sg) in ms {
                entry.insert(m.clone(), qualify_sig(sg, id, ex));
            }
        }
        // An `extend` travels with the import: what it applies to, what its
        // names stand for and where it was written all come across, or the
        // methods above could never be found.
        for (tkey, t) in &ex.ext_targets {
            self.ext_targets.insert(requalify(tkey), qualify_type(t, id, ex));
        }
        for (tkey, gs) in &ex.ext_generics {
            let entry = self.ext_generics.entry(requalify(tkey)).or_default();
            for g in gs {
                if !entry.iter().any(|p| p.name == g.name) {
                    entry.push(qualify_param(g, id, ex));
                }
            }
        }
        for ((tkey, m), t) in &ex.ext_method_target {
            self.ext_method_target.insert((requalify(tkey), m.clone()), qualify_type(t, id, ex));
        }
        for ((tkey, iname), gs) in &ex.ext_impl_generics {
            let ik = if ex.interfaces.contains_key(iname) { format!("{}.{}", key_prefix, iname) } else { iname.clone() };
            self.ext_impl_generics.insert((requalify(tkey), ik), gs.iter().map(|g| qualify_param(g, id, ex)).collect());
        }
        for ((tkey, iname), w) in &ex.ext_where {
            let ik = if ex.interfaces.contains_key(iname) { format!("{}.{}", key_prefix, iname) } else { iname.clone() };
            let k = (requalify(tkey), ik);
            // Two modules can each write an `extend` without ever seeing
            // the other; where they meet is here.
            if let Some(other) = self.ext_where.get(&k) {
                if other.0 != w.0 {
                    return Err(LumeError::new(line, col, format!("`{}` is extended with `{}` twice", k.0, k.1))
                        .with_help(format!("{}:{} and {}:{} both extend it, and an `extend` travels with the import, so this program would have two answers: remove one, or give one of them an interface of its own", other.0, other.1, w.0, w.1)));
                }
            }
            self.ext_where.insert(k, w.clone());
        }
        // Types this module reached through its own imports, kept under the
        // ids they already have: a value handed across two boundaries is
        // still a value you can use.
        for (k, v) in &ex.carried_structs {
            self.structs.entry(k.clone()).or_insert_with(|| v.clone());
        }
        for (k, v) in &ex.carried_enums {
            self.enums.entry(k.clone()).or_insert_with(|| v.clone());
        }
        for (k, v) in &ex.carried_ifaces {
            self.interfaces.entry(k.clone()).or_insert_with(|| IfaceInfo { local: false, ..v.clone() });
        }
        for (k, v) in &ex.carried_paths {
            self.paths.entry(k.clone()).or_insert_with(|| v.clone());
        }
        // The interfaces those extends name. An `extend` is not a name, so
        // it keeps travelling past the file that imported it; the interface
        // behind it comes along so the conformance can still be honoured,
        // under a key no file can write unless it imports that module.
        for (iname, (info, path)) in &ex.ext_ifaces {
            let ik = if ex.interfaces.contains_key(iname) { format!("{}.{}", key_prefix, iname) } else { iname.clone() };
            if !self.interfaces.contains_key(&ik) {
                let carried = IfaceInfo { local: false, ..info.clone() };
                self.interfaces.insert(ik.clone(), carried);
                self.paths.insert(ik.clone(), path.clone());
            }
            // the `use` is needed either way: Rust offers a trait's methods
            // only where the trait is in scope
            let u = format!("use crate::{};", path);
            if !self.trait_uses.contains(&u) {
                self.trait_uses.push(u);
            }
        }
        Ok(())
    }

    fn register_deps(&mut self, deps: &[Dep]) -> Result<()> {
        for d in deps {
            match d {
                Dep::Rust { alias, info } => {
                    self.crates.insert(alias.clone(), info.root.clone());
                    for (k, v) in &info.types {
                        self.foreign_types.insert(k.clone(), v.clone());
                    }
                }
                Dep::RustTypes { info } => {
                    for (k, v) in &info.types {
                        self.foreign_types.entry(k.clone()).or_insert_with(|| v.clone());
                    }
                }
                Dep::Module { alias, id, exports, line, col } => {
                    if self.module_aliases.contains_key(alias) {
                        return Err(LumeError::new(*line, *col, format!("`{}` is imported twice", alias)));
                    }
                    self.module_aliases.insert(alias.clone(), id.clone());
                    self.module_private.insert(alias.clone(), exports.private.clone());
                    self.register_module_items(id, id, exports, *line, *col)?;
                    for n in exports.structs.keys().chain(exports.enums.keys()).chain(exports.fns.keys()).chain(exports.interfaces.keys()).chain(exports.consts.keys()) {
                        self.canon.insert(format!("{}.{}", alias, n), format!("{}.{}", id, n));
                    }
                }
                Dep::Single { local, id, item, exports, line, col } => {
                    // the module's types must resolve for signatures that mention them
                    self.register_module_items(id, id, exports, *line, *col)?;
                    let full = format!("{}.{}", id, item);
                    // a constant is as importable by name as a type or a
                    // function: `import geo.units.METRES_PER_MILE`
                    if self.structs.contains_key(&full)
                        || self.enums.contains_key(&full)
                        || self.fns.contains_key(&full)
                        || self.interfaces.contains_key(&full)
                        || self.consts.contains_key(&full)
                    {
                        self.canon.insert(local.clone(), full.clone());
                    } else if exports.private.contains(item) {
                        return Err(LumeError::new(*line, *col, format!("`{}` exists in module `{}` but is not `pub`", item, id))
                            .with_help(format!("add `pub` in front of its definition in {}.lume", id.replace('.', "/"))));
                    } else {
                        let e = LumeError::new(*line, *col, format!("module `{}` has no `{}`", id, item));
                        let names: Vec<String> = exports.structs.keys().chain(exports.enums.keys()).chain(exports.fns.keys()).chain(exports.consts.keys()).chain(exports.interfaces.keys()).cloned().collect();
                        return Err(match self.suggest_from(item, names.iter().cloned()) {
                            Some(sug) => e.with_help(format!("did you mean `{}`?", sug)),
                            None => e.with_help(format!("its public items are: {}", names.join(", "))),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// This module's public items, for files that import it.
    fn exports(&self, program: &[Item], rust_mod: &str) -> Exports {
        let mut ex = Exports { rust_mod: rust_mod.to_string(), ..Default::default() };
        for item in program {
            match item {
                Item::Fn(f) if f.public => {
                    ex.fns.insert(f.name.clone(), self.fns[&f.name].clone());
                }
                Item::Fn(f) => ex.private.push(f.name.clone()),
                Item::Const(c) if c.public => {
                    ex.consts.insert(c.name.clone(), self.consts.get(&c.name).cloned().unwrap_or(Type::Unknown));
                }
                Item::Const(c) => ex.private.push(c.name.clone()),
                Item::Struct(st) if st.public => {
                    ex.structs.insert(st.name.clone(), self.structs[&st.name].clone());
                }
                Item::Struct(st) => ex.private.push(st.name.clone()),
                Item::Enum(en) if en.public => {
                    ex.enums.insert(en.name.clone(), self.enums[&en.name].clone());
                }
                Item::Enum(en) => ex.private.push(en.name.clone()),
                Item::Interface(i) if i.public => {
                    ex.interfaces.insert(i.name.clone(), self.interfaces[&i.name].clone());
                }
                Item::Interface(i) => ex.private.push(i.name.clone()),
                Item::Import(_) | Item::Extend(_) | Item::Test(_) => {}
            }
        }
        ex.ext_methods = self.ext_methods.clone();
        ex.ext_targets = self.ext_targets.clone();
        ex.ext_generics = self.ext_generics.clone();
        ex.ext_method_target = self.ext_method_target.clone();
        ex.ext_impl_generics = self.ext_impl_generics.clone();
        ex.ext_where = self.ext_where.clone();
        // Types from this module's own imports: an importer that never
        // imported them still has to be able to use what it is handed.
        for (k, v) in &self.structs {
            if k.contains('.') {
                ex.carried_structs.insert(k.clone(), v.clone());
            }
        }
        for (k, v) in &self.enums {
            if k.contains('.') {
                ex.carried_enums.insert(k.clone(), v.clone());
            }
        }
        for (k, v) in &self.interfaces {
            if k.contains('.') {
                ex.carried_ifaces.insert(k.clone(), v.clone());
            }
        }
        for (k, v) in &self.paths {
            if k.contains('.') {
                ex.carried_paths.insert(k.clone(), v.clone());
            }
        }
        for (_, iname) in self.ext_where.keys() {
            if let (Some(info), Some(path)) = (self.interfaces.get(iname), self.paths.get(iname).cloned().or_else(|| Some(iname.clone()))) {
                let path = if info.local { format!("{}::{}", ex.rust_mod, iname) } else { path };
                ex.ext_ifaces.insert(iname.clone(), (info.clone(), path));
            }
        }
        ex
    }

    /// A `module.name` reference: the item key if `module` is an imported
    /// module alias, plus a good error when the name is private or missing.
    fn module_item(&self, alias: &str, name: &str, line: usize, col: usize) -> Result<Option<String>> {
        let id = match self.module_aliases.get(alias) {
            Some(id) => id.clone(),
            None => return Ok(None),
        };
        let key = self.canon(&format!("{}.{}", alias, name));
        if self.structs.contains_key(&key) || self.enums.contains_key(&key) || self.fns.contains_key(&key) || self.interfaces.contains_key(&key) || self.consts.contains_key(&key) {
            return Ok(Some(key));
        }
        if self.module_private.get(alias).map(|p| p.contains(&name.to_string())).unwrap_or(false) {
            return Err(LumeError::new(line, col, format!("`{}` exists in module `{}` but is not `pub`", name, id))
                .with_help(format!("add `pub` in front of its definition in {}.lume", id.replace('.', "/"))));
        }
        let e = LumeError::new(line, col, format!("module `{}` has no `{}`", id, name));
        let prefix = format!("{}.", id);
        let names: Vec<String> = self
            .structs
            .keys()
            .chain(self.enums.keys())
            .chain(self.fns.keys())
            .filter_map(|k| k.strip_prefix(&prefix).map(|s| s.to_string()))
            .filter(|s| !s.contains('.'))
            .collect();
        Err(match self.suggest_from(name, names.iter().cloned()) {
            Some(sug) => e.with_help(format!("did you mean `{}`?", sug)),
            None => e.with_help(format!("its public items are: {}", names.join(", "))),
        })
    }

    // ----- program ----------------------------------------------------------

    fn program(&mut self, program: &[Item]) -> Result<()> {
        // Pass 1: collect types and signatures.
        let mut seen = HashSet::new();
        let mut imported: HashSet<String> = HashSet::new();
        for item in program {
            match item {
                Item::Import(imp) if imp.is_rust => {
                    let key = imp.alias.clone().unwrap_or_else(|| imp.krate().to_string());
                    if !imported.insert(key.clone()) {
                        return Err(LumeError::new(imp.line, imp.col, format!("`{}` is imported twice", key)));
                    }
                }
                Item::Import(_) | Item::Test(_) => {}
                Item::Const(c) => {
                    if !seen.insert(c.name.clone()) {
                        return Err(LumeError::new(c.line, c.col, format!("`{}` is defined twice", c.name)));
                    }
                    reserved_type_name(&c.name, c.line, c.col)?;
                }
                Item::Fn(f) => {
                    if !seen.insert(f.name.clone()) {
                        return Err(LumeError::new(f.line, f.col, format!("function `{}` is defined twice", f.name)));
                    }
                    let saved = self.push_generics(&f.generics);
                    let sg = self.sig_of(f);
                    self.pop_generics(saved);
                    self.fns.insert(f.name.clone(), sg);
                }
                Item::Struct(s) => {
                    if s.name == "Error" {
                        return Err(LumeError::new(s.line, s.col, "`Error` is the built-in error type").with_help("name yours differently, or define an `enum` of error kinds and return `T or MyError`"));
                    }
                    reserved_type_name(&s.name, s.line, s.col)?;
                    if !seen.insert(s.name.clone()) {
                        return Err(LumeError::new(s.line, s.col, format!("`{}` is defined twice", s.name)));
                    }
                    let mut fnames = HashSet::new();
                    for fld in &s.fields {
                        if !fnames.insert(fld.name.clone()) {
                            return Err(LumeError::new(fld.line, fld.col, format!("field `{}` is listed twice in `{}`", fld.name, s.name)));
                        }
                    }
                    let saved = self.push_generics(&s.generics);
                    let methods = self.collect_methods(&s.methods, &s.name, &fnames)?;
                    let fields = s.fields.iter().map(|p| (p.name.clone(), self.ct(&p.ty))).collect();
                    self.pop_generics(saved);
                    self.structs.insert(s.name.clone(), StructInfo { generics: self.norm_generics(&s.generics), fields, methods });
                }
                Item::Enum(e) => {
                    reserved_type_name(&e.name, e.line, e.col)?;
                    if !seen.insert(e.name.clone()) {
                        return Err(LumeError::new(e.line, e.col, format!("`{}` is defined twice", e.name)));
                    }
                    let saved = self.push_generics(&e.generics);
                    let methods = self.collect_methods(&e.methods, &e.name, &HashSet::new())?;
                    let variants = e
                        .variants
                        .iter()
                        .map(|v| (v.name.clone(), v.fields.iter().map(|p| (p.name.clone(), self.ct(&p.ty))).collect()))
                        .collect();
                    self.pop_generics(saved);
                    self.enums.insert(e.name.clone(), EnumInfo { generics: self.norm_generics(&e.generics), variants, methods });
                    self.local_types.insert(e.name.clone());
                }
                Item::Interface(i) => {
                    reserved_type_name(&i.name, i.line, i.col)?;
                    if !seen.insert(i.name.clone()) {
                        return Err(LumeError::new(i.line, i.col, format!("`{}` is defined twice", i.name)));
                    }
                    let saved = self.push_generics(&i.generics);
                    let mut methods = HashMap::new();
                    for m in i.required.iter().chain(&i.defaults) {
                        if m.self_kind == SelfKind::Mutate {
                            return Err(LumeError::new(m.line, m.col, format!("interface method `{}` cannot take `var self`", m.name))
                                .with_help("interfaces describe reading behaviour; mutation stays on the concrete type"));
                        }
                        if methods.insert(m.name.clone(), self.sig_of(m)).is_some() {
                            return Err(LumeError::new(m.line, m.col, format!("method `{}` is listed twice in `{}`", m.name, i.name)));
                        }
                    }
                    self.pop_generics(saved);
                    self.interfaces.insert(
                        i.name.clone(),
                        IfaceInfo { generics: self.norm_generics(&i.generics), methods: methods.into_iter().collect(), required: i.required.iter().map(|m| m.name.clone()).collect(), defaults: i.defaults.clone(), local: true },
                    );
                }
                Item::Extend(_) => {}
            }
        }
        for item in program {
            if let Item::Struct(s) = item {
                self.local_types.insert(s.name.clone());
            }
        }
        // `extend Type with Iface:` adds methods to the type.
        for item in program {
            if let Item::Extend(x) = item {
                // `extend [T] with Container[T]:` — a name inside the target
                // that is no type of this program's is a type parameter.
                let mut pairs = Vec::new();
                self.target_params_ex(&x.target, false, &mut pairs);
                let names: Vec<String> = pairs.iter().map(|(n, _)| n.clone()).collect();
                let gens: Vec<TypeParam> = pairs
                    .iter()
                    .map(|(n, implied)| match x.bounds.iter().find(|b| b.name == *n) {
                        Some(b) => b.clone(),
                        None => TypeParam { name: n.clone(), bound: implied.clone(), line: x.line, col: x.col },
                    })
                    .collect();
                if let Some(b) = x.bounds.iter().find(|b| !names.contains(&b.name)) {
                    return Err(LumeError::new(b.line, b.col, format!("`{}` is a type, so it takes no bound here", b.name))
                        .with_help("a bound belongs on a name the `extend` introduces, like `extend [T: Ordered] with Sortable[T]:`"));
                }
                self.check_generics(&gens, "this `extend`")?;
                let saved_g = self.push_generics(&gens);
                let r = self.register_extend(x, &gens);
                self.pop_generics(saved_g);
                r?;
            }
        }
        // Check that every named type exists.
        for item in program {
            match item {
                Item::Import(_) | Item::Test(_) => {}
                Item::Fn(f) => self.check_sig_types(f, &[])?,
                Item::Const(c) => {
                    if let Some(t) = &c.ty {
                        self.check_type(t, c.line, c.col)?;
                    }
                }
                Item::Struct(s) => {
                    self.check_generics(&s.generics, &format!("`{}`", s.name))?;
                    let saved = self.push_generics(&s.generics);
                    for fld in &s.fields {
                        self.check_type(&Self::as_vars(&fld.ty, &s.generics), fld.line, fld.col)?;
                        Self::no_fn_outside_params(&Self::as_vars(&fld.ty, &s.generics), "a field", fld.line, fld.col)?;
                    }
                    self.pop_generics(saved);
                    for m in &s.methods {
                        self.check_sig_types(m, &s.generics)?;
                    }
                    self.check_generics_used(&s.generics, s.fields.iter().map(|f| &f.ty), &format!("struct `{}`", s.name))?;
                }
                Item::Enum(e) => {
                    self.check_generics(&e.generics, &format!("`{}`", e.name))?;
                    let saved = self.push_generics(&e.generics);
                    for v in &e.variants {
                        for fld in &v.fields {
                            self.check_type(&Self::as_vars(&fld.ty, &e.generics), fld.line, fld.col)?;
                            Self::no_fn_outside_params(&Self::as_vars(&fld.ty, &e.generics), "a field", fld.line, fld.col)?;
                        }
                    }
                    self.pop_generics(saved);
                    for m in &e.methods {
                        self.check_sig_types(m, &e.generics)?;
                    }
                    self.check_generics_used(&e.generics, e.variants.iter().flat_map(|v| v.fields.iter().map(|f| &f.ty)), &format!("enum `{}`", e.name))?;
                }
                Item::Interface(i) => {
                    if !i.required.iter().chain(&i.defaults).all(|m| m.generics.is_empty()) {
                        return Err(LumeError::new(i.line, i.col, format!("interface `{}` has a method with type parameters, which is not supported yet", i.name))
                            .with_help("an interface method takes concrete types; a generic function can take the interface instead"));
                    }
                    self.check_generics(&i.generics, &format!("`{}`", i.name))?;
                    let sig_types: Vec<Type> = i
                        .required
                        .iter()
                        .chain(&i.defaults)
                        .flat_map(|m| m.params.iter().map(|p| p.ty.clone()).chain(m.ret.clone()))
                        .collect();
                    self.check_generics_used(&i.generics, sig_types.iter(), &format!("interface `{}`", i.name))?;
                    for m in i.required.iter().chain(&i.defaults) {
                        self.check_sig_types(m, &i.generics)?;
                        if m.params.iter().any(|p| self.is_interface(&self.ct(&p.ty))) || m.ret.as_ref().map(|r| self.is_interface(&self.ct(r))).unwrap_or(false) {
                            return Err(LumeError::new(m.line, m.col, format!("interface method `{}` mentions an interface in its signature, which is not supported yet", m.name))
                                .with_help("use concrete types in interface signatures for now"));
                        }
                    }
                }
                Item::Extend(x) => {
                    let ikey = self.type_key(&self.ct(&x.iface));
                    let gens = self.ext_impl_generics.get(&(self.ext_key_of(x), ikey)).cloned().unwrap_or_default();
                    let saved = self.push_generics(&gens);
                    let r = (|g: &mut Self| -> Result<()> {
                        for m in &x.methods {
                            g.check_sig_types(m, &[])?;
                        }
                        Ok(())
                    })(self);
                    self.pop_generics(saved);
                    r?;
                }
            }
        }
        // Pass 2: infer missing return types (a few rounds so dependencies settle).
        for _round in 0..4 {
            let mut progressed = false;
            for item in program {
                let ext_key;
                let (fns, owner): (Vec<&FnDef>, Option<&String>) = match item {
                    Item::Const(_) => continue,
                    Item::Fn(f) => (vec![f], None),
                    Item::Struct(s) => (s.methods.iter().collect(), Some(&s.name)),
                    Item::Enum(e) => (e.methods.iter().collect(), Some(&e.name)),
                    Item::Interface(i) => (i.defaults.iter().collect(), Some(&i.name)),
                    Item::Extend(x) => {
                        ext_key = self.ext_key_of(x);
                        (x.methods.iter().collect(), Some(&ext_key))
                    }
                    Item::Import(_) | Item::Test(_) => (vec![], None),
                };
                for f in fns {
                    if f.ret.is_some() || self.sig_ret(&f.name, owner) != Type::Unknown {
                        continue;
                    }
                    let t = self.infer_ret(f, owner);
                    if t != Type::Unknown {
                        self.set_sig_ret(&f.name, owner, t);
                        progressed = true;
                    }
                }
            }
            if !progressed {
                break;
            }
        }
        for item in program {
            let ext_key;
            let (fns, owner): (Vec<&FnDef>, Option<&String>) = match item {
                Item::Const(_) => continue,
                Item::Fn(f) => (vec![f], None),
                Item::Struct(s) => (s.methods.iter().collect(), Some(&s.name)),
                Item::Enum(e) => (e.methods.iter().collect(), Some(&e.name)),
                Item::Interface(i) => (i.defaults.iter().collect(), Some(&i.name)),
                Item::Extend(x) => {
                    ext_key = self.ext_key_of(x);
                    (x.methods.iter().collect(), Some(&ext_key))
                }
                Item::Import(_) | Item::Test(_) => (vec![], None),
            };
            for f in fns {
                if self.sig_ret(&f.name, owner) == Type::Unknown {
                    return Err(LumeError::new(f.line, f.col, format!("cannot work out what `{}` returns", f.name))
                        .with_help(format!("add the return type to the signature: `def {}(...) -> Type`", f.name)));
                }
            }
        }

        // Pass 3: emit. The prelude is shared: only the entry file carries it.
        if self.is_entry {
            self.out.push_str(PRELUDE);
            self.out.push('\n');
        }
        for item in program {
            if let Item::Import(imp) = item {
                if !imp.is_rust {
                    continue;
                }
                match &imp.alias {
                    Some(a) => self.line(&format!("use {} as {};", imp.krate(), a)),
                    None => self.line(&format!("use {};", imp.krate())),
                }
            }
        }
        self.out.push('\n');
        // constants: one value each, computed the first time it is used
        for item in program {
            if let Item::Const(c) = item {
                self.const_def(c)?;
            }
        }
        self.out.push('\n');
        let mut test_index = 0usize;
        for item in program {
            match item {
                Item::Const(_) => continue,
                Item::Fn(f) => self.fn_def(f, None)?,
                Item::Struct(s) => self.struct_def(s)?,
                Item::Enum(e) => self.enum_def(e)?,
                Item::Interface(i) => self.interface_def(i)?,
                Item::Import(_) | Item::Extend(_) => continue,
                Item::Test(t) => {
                    if !self.test_mode {
                        continue;
                    }
                    let i = test_index;
                    test_index += 1;
                    self.test_def(t, i)?;
                }
            }
            self.out.push('\n');
        }
        self.emit_conformances(program)?;
        if self.is_entry {
            // printing for crate types that implement Display, once per program
            let mut seen = HashSet::new();
            let mut fts: Vec<&crate::bridge::ForeignType> = self.foreign_types.values().filter(|ft| ft.display && ft.iter_item.is_none()).collect();
            fts.sort_by(|a, b| a.rust_path.cmp(&b.rust_path));
            for ft in fts {
                if !seen.insert(ft.rust_path.clone()) {
                    continue;
                }
                let lts: Vec<String> = (0..ft.lifetimes).map(|i| format!("'l{}", i)).collect();
                let (gen, args) = if lts.is_empty() { (String::new(), String::new()) } else { (format!("<{}>", lts.join(", ")), format!("<{}>", lts.join(", "))) };
                self.out.push_str(&format!("impl{} LumeShow for {}{} {{ fn lume_str(&self) -> String {{ self.to_string() }} }}\n", gen, ft.rust_path, args));
            }
        }
        if self.is_entry && !self.test_mode && !self.fns.contains_key("main") {
            return Err(LumeError::new(1, 1, "no `main` function").with_help("a program starts at `def main:`"));
        }
        Ok(())
    }

    fn sig_ret(&self, name: &str, owner: Option<&String>) -> Type {
        match owner {
            None => self.fns[name].ret.clone(),
            Some(o) => self.methods_of(o).and_then(|m| m.get(name).map(|s| s.ret.clone())).unwrap_or(Type::Unknown),
        }
    }

    fn set_sig_ret(&mut self, name: &str, owner: Option<&String>, t: Type) {
        match owner {
            None => self.fns.get_mut(name).unwrap().ret = t,
            Some(o) => {
                if let Some(s) = self.structs.get_mut(o).and_then(|s| s.methods.get_mut(name)) {
                    s.ret = t;
                } else if let Some(s) = self.enums.get_mut(o).and_then(|e| e.methods.get_mut(name)) {
                    s.ret = t;
                } else if let Some(s) = self.interfaces.get_mut(o).and_then(|i| i.methods.get_mut(name)) {
                    s.ret = t;
                } else if let Some(s) = self.ext_methods.get_mut(o).and_then(|m| m.get_mut(name)) {
                    s.ret = t;
                }
            }
        }
    }

    fn check_type(&self, t: &Type, line: usize, col: usize) -> Result<()> {
        match t {
            Type::Var(_) => Ok(()),
            Type::Named(n) if self.type_param(n).is_some() => Ok(()),
            Type::App(n, args) => {
                let key = self.canon(n);
                if !self.is_type(&key) {
                    return self.check_type(&Type::Named(n.clone()), line, col);
                }
                let gs = self.generics_of(&key);
                if gs.is_empty() {
                    return Err(LumeError::new(line, col, format!("`{}` takes no type arguments", n))
                        .with_help(format!("write it as `{}`; only a `struct` or `enum` declared `{}[T]` takes them", n, n)));
                }
                if gs.len() != args.len() {
                    return Err(LumeError::new(line, col, format!("`{}` takes {} {}, but {} {} given", n, gs.len(), plural(gs.len(), "type argument", "type arguments"), args.len(), plural(args.len(), "was", "were")))
                        .with_help(format!("it is declared `{}[{}]`", n, gs.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", "))));
                }
                for a in args {
                    self.check_type(a, line, col)?;
                }
                self.check_type_args(&gs, args, &format!("`{}`", n), line, col)
            }
            Type::Named(n) if !self.is_type(&self.canon(n)) => {
                // `ledger.Signed` where `Signed` is there but not `pub`:
                // say so, rather than calling it unknown
                if let Some((alias, name)) = n.split_once('.') {
                    if let Some(id) = self.module_aliases.get(alias).cloned() {
                        if self.module_private.get(alias).map(|p| p.contains(&name.to_string())).unwrap_or(false) {
                            return Err(LumeError::new(line, col, format!("`{}` exists in module `{}` but is not `pub`", name, id))
                                .with_help(format!("add `pub` in front of its definition in {}.lume", id.replace('.', "/"))));
                        }
                    }
                }
                let e = LumeError::new(line, col, format!("unknown type `{}`", n));
                let cands = self
                    .structs
                    .keys()
                    .cloned()
                    .chain(self.enums.keys().cloned())
                    .chain(self.interfaces.keys().cloned())
                    .chain(["Int", "Float", "Bool", "Str"].iter().map(|s| s.to_string()));
                Err(match self.suggest_from(n, cands) {
                    Some(s) => e.with_help(format!("did you mean `{}`?", s)),
                    None => e.with_help("built-in types are Int, Float, Bool, Str, [T], T? and tuples; others must be a `struct` or `enum`"),
                })
            }
            Type::Fn(ps, r) => {
                for p in ps {
                    self.check_type(p, line, col)?;
                    if matches!(p, Type::Fn(..)) {
                        return Err(LumeError::new(line, col, "a block cannot take another block as an argument")
                            .with_help("pass the values it needs instead, and call the second block yourself"));
                    }
                }
                self.check_type(r, line, col)
            }
            Type::List(inner) | Type::Option(inner) | Type::Task(inner) | Type::Shared(inner, _) => self.check_type(inner, line, col),
            Type::Set(inner) => {
                self.check_type(inner, line, col)?;
                self.check_key_type(inner, "set", line, col)
            }
            Type::Tuple(ts) => ts.iter().try_for_each(|t| self.check_type(t, line, col)),
            Type::Result(a, b) | Type::Map(a, b) => {
                self.check_type(a, line, col)?;
                if matches!(t, Type::Map(..)) {
                    self.check_key_type(a, "map", line, col)?;
                }
                self.check_type(b, line, col)
            }
            _ => Ok(()),
        }
    }

    fn check_sig_types(&mut self, f: &FnDef, outer: &[TypeParam]) -> Result<()> {
        let mut gs = outer.to_vec();
        gs.extend(f.generics.iter().cloned());
        let saved = self.push_generics(&gs);
        let r = self.check_sig_types_inner(f, &gs);
        self.pop_generics(saved);
        r
    }

    fn check_sig_types_inner(&self, f: &FnDef, gs: &[TypeParam]) -> Result<()> {
        self.check_generics(&f.generics, &format!("`{}`", f.name))?;
        for p in &f.params {
            self.check_type(&Self::as_vars(&p.ty, gs), p.line, p.col)?;
            if matches!(p.ty, Type::Fn(..)) && p.mutable {
                return Err(LumeError::new(p.line, p.col, format!("`{}` is behaviour, so it cannot be a `var` parameter", p.name))
                    .with_help("a block is run, not changed; drop the `var`"));
            }
        }
        if let Some(r) = &f.ret {
            self.check_type(&Self::as_vars(r, gs), f.line, f.col)?;
            Self::no_fn_outside_params(&Self::as_vars(r, gs), "a result", f.line, f.col)?;
        }
        Ok(())
    }

    /// A function value is a parameter and nothing else: it is run during the
    /// call it was handed to, so there is nowhere else for it to live yet.
    fn no_fn_outside_params(t: &Type, what: &str, line: usize, col: usize) -> Result<()> {
        let holds = match t {
            Type::Fn(..) => true,
            Type::List(i) | Type::Option(i) | Type::Set(i) | Type::Task(i) | Type::Shared(i, _) | Type::Iter(i, _) => matches!(**i, Type::Fn(..)),
            Type::Tuple(ts) => ts.iter().any(|x| matches!(x, Type::Fn(..))),
            Type::Result(a, b) | Type::Map(a, b) => matches!(**a, Type::Fn(..)) || matches!(**b, Type::Fn(..)),
            Type::App(_, args) => args.iter().any(|x| matches!(x, Type::Fn(..))),
            _ => false,
        };
        if holds {
            return Err(LumeError::new(line, col, format!("a block cannot be {}", what))
                .with_help("behaviour is passed to a function and run there; it cannot be stored or handed back yet"));
        }
        Ok(())
    }

    /// A type a struct or enum never stores cannot be worked out from a
    /// value, so Rust would refuse the definition outright.
    fn check_generics_used<'a>(&self, gs: &[TypeParam], types: impl Iterator<Item = &'a Type>, what: &str) -> Result<()> {
        let all: Vec<Type> = types.map(|t| Self::as_vars(t, gs)).collect();
        for p in gs {
            if !all.iter().any(|t| Self::mentions_var(t, &p.name)) {
                return Err(LumeError::new(p.line, p.col, format!("{} never uses `{}`", what, p.name))
                    .with_help(format!("mention `{}` in a field or a method signature, or drop it from the type parameters", p.name)));
            }
        }
        Ok(())
    }

    /// A type parameter's bound must be a built-in one or a real interface.
    fn check_generics(&self, gs: &[TypeParam], what: &str) -> Result<()> {
        for p in gs {
            let b = match &p.bound {
                Some(b) => b,
                None => continue,
            };
            let head = Self::bound_name(b);
            if Self::is_builtin_bound(head) {
                if matches!(b, Type::App(..)) {
                    return Err(LumeError::new(p.line, p.col, format!("`{}` takes no type arguments", head)));
                }
                continue;
            }
            let key = self.canon(head);
            if !self.interfaces.contains_key(&key) {
                let e = LumeError::new(p.line, p.col, format!("unknown bound `{}` on `{}` of {}", type_name(b), p.name, what));
                let cands = self.interfaces.keys().cloned().chain(["Ordered", "Hashable"].iter().map(|s| s.to_string()));
                return Err(match self.suggest_from(head, cands) {
                    Some(s) => e.with_help(format!("did you mean `{}`?", s)),
                    None => e.with_help("a bound is `Ordered`, `Hashable`, or the name of an `interface`"),
                });
            }
            let want = self.interfaces[&key].generics.len();
            let got = match b { Type::App(_, a) => a.len(), _ => 0 };
            if want != got {
                return Err(LumeError::new(p.line, p.col, format!("`{}` takes {} {}, but {} {} given", head, want, plural(want, "type argument", "type arguments"), got, plural(got, "was", "were")))
                    .with_help(if want == 0 { format!("write it as `{}`", head) } else { format!("it is declared `{}[{}]`", head, self.interfaces[&key].generics.iter().map(|q| q.name.clone()).collect::<Vec<_>>().join(", ")) }));
            }
        }
        Ok(())
    }

    /// Type of the function's tail expression, evaluated in a scope holding
    /// only its parameters (and, for a method, its fields).
    fn infer_ret(&mut self, f: &FnDef, owner: Option<&String>) -> Type {
        if f.name == "main" && owner.is_none() {
            return Type::Unit;
        }
        self.push_scope();
        let saved = self.current_type.clone();
        let saved_self = self.current_self_ty.clone();
        let saved_gs = self.push_generics(&self.scope_generics(f, owner));
        self.current_type = owner.cloned();
        self.current_self_ty = owner.and_then(|o| self.ext_targets.get(o).cloned());
        for p in &f.params {
            let pty = self.ct(&p.ty);
            self.declare(&p.name, false, !pty.is_copy(), pty, p.line);
        }
        let t = self.tail_type(&f.body);
        self.current_type = saved;
        self.current_self_ty = saved_self;
        self.pop_generics(saved_gs);
        self.pop_scope();
        t
    }

    /// The type parameters in scope inside a function: the owning type's,
    /// then its own.
    fn scope_generics(&self, f: &FnDef, owner: Option<&String>) -> Vec<TypeParam> {
        let mut gs = owner.map(|o| self.generics_of(&self.canon(o))).unwrap_or_default();
        gs.extend(f.generics.iter().cloned());
        gs
    }

    fn tail_type(&mut self, b: &Block) -> Type {
        self.push_scope();
        let mut t = Type::Unit;
        let n = b.stmts.len();
        for (i, s) in b.stmts.iter().enumerate() {
            let last = i + 1 == n;
            match s {
                Stmt::Destructure { names, value, line, .. } => {
                    if let Type::Tuple(ts) = self.ty_of(value).materialized() {
                        for (n, t) in names.iter().zip(ts) {
                            if n != "_" {
                                self.declare(n, false, false, t, *line);
                            }
                        }
                    }
                    if last {
                        t = Type::Unit;
                    }
                }
                Stmt::Bind { name, ty, value, line, .. } | Stmt::Var { name, ty, value, line, .. } => {
                    let vt = ty.as_ref().map(|t| self.ct(t)).unwrap_or_else(|| self.ty_of(value).materialized());
                    let mutable = matches!(s, Stmt::Var { .. });
                    if self.lookup(name).is_none() || mutable {
                        self.declare(name, mutable, false, vt, *line);
                    }
                    if last {
                        t = Type::Unit;
                    }
                }
                Stmt::Expr(e) if last => {
                    t = match &e.kind {
                        ExprKind::If { branches, else_block } => {
                            let mut bt = self.tail_type(&branches[0].1);
                            if bt == Type::Unknown {
                                if let Some(eb) = else_block {
                                    bt = self.tail_type(eb);
                                }
                            }
                            if else_block.is_none() { Type::Unit } else { bt }
                        }
                        ExprKind::Match { scrutinee, arms } => self.match_type(scrutinee, arms),
                        _ => self.ty_of(e).materialized(),
                    };
                }
                _ => {
                    if last {
                        t = Type::Unit;
                    }
                }
            }
        }
        self.pop_scope();
        t
    }

    /// Type of a `match`: the first arm whose body has a known type, with
    /// the arm's pattern bindings in scope.
    fn match_type(&mut self, scrutinee: &Expr, arms: &[MatchArm]) -> Type {
        let st = self.ty_of(scrutinee);
        for arm in arms {
            self.push_scope();
            let _ = self.declare_pattern_types(&arm.pat, &st);
            let t = self.tail_type(&arm.body);
            self.pop_scope();
            if t != Type::Unknown {
                return t;
            }
        }
        Type::Unknown
    }

    /// Declares the names a pattern binds, with their types, in the current
    /// scope (used for inference only; codegen does its own declaring).
    fn declare_pattern_types(&mut self, p: &Pattern, t: &Type) -> Result<()> {
        match &p.kind {
            PatKind::Or(alts) => {
                if let Some(a) = alts.first() {
                    self.declare_pattern_types(a, t)?;
                }
            }
            PatKind::Bind(n) => {
                self.declare(n, false, !t.is_copy(), t.clone(), p.line);
            }
            PatKind::Variant { enum_name, name, args, rest: _ } => {
                match t {
                    Type::Option(inner) => {
                        if let Some(a) = args.first() {
                            self.declare_pattern_types(a, inner)?;
                        }
                    }
                    Type::Result(ok_t, err_t) => {
                        if let Some(a) = args.first() {
                            let inner = if name == "Ok" { ok_t } else { err_t };
                            self.declare_pattern_types(a, inner)?;
                        }
                    }
                    Type::Named(en) => {
                        let _ = enum_name;
                        if let Some(info) = self.enums.get(en).cloned() {
                            if let Some((_, fields)) = info.variants.iter().find(|(n, _)| n == name) {
                                for (a, (_, ft)) in args.iter().zip(fields) {
                                    self.declare_pattern_types(a, ft)?;
                                }
                            }
                        }
                    }
                    _ => {
                        for a in args {
                            self.declare_pattern_types(a, &Type::Unknown)?;
                        }
                    }
                }
            }
            PatKind::Tuple(items) => {
                if let Type::Tuple(ts) = t {
                    for (a, at) in items.iter().zip(ts) {
                        self.declare_pattern_types(a, at)?;
                    }
                } else {
                    for a in items {
                        self.declare_pattern_types(a, &Type::Unknown)?;
                    }
                }
            }
            PatKind::List { items, rest } => {
                let elem = match t {
                    Type::List(e) => (**e).clone(),
                    _ => Type::Unknown,
                };
                for a in items {
                    self.declare_pattern_types(a, &elem)?;
                }
                if let Some(Some(r)) = rest {
                    self.declare(r, false, false, Type::List(Box::new(elem)), p.line);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Rewrites `model.f(x)` to a call of the key `model.f`, `model.User(..)`
    /// to that constructor, and `model.Shape.Circle(..)` to a variant of the
    /// key `model.Shape`, when `model` is an imported module alias.
    fn module_ref(&mut self, e: &Expr) -> Result<Option<Expr>> {
        let (recv, name, args) = match &e.kind {
            ExprKind::Method { recv, name, args } => (recv, name, args),
            _ => return Ok(None),
        };
        // model.Enum.Variant
        if let ExprKind::Method { recv: r2, name: tname, args: a2 } = &recv.kind {
            if a2.is_empty() {
                if let ExprKind::Ident(alias) = &r2.kind {
                    if self.lookup(alias).is_none() && self.module_aliases.contains_key(alias) {
                        let key = self.canon(&format!("{}.{}", alias, tname));
                        if self.enums.contains_key(&key) {
                            let en = Expr::new(ExprKind::Ident(key), recv.line, recv.col);
                            return Ok(Some(Expr::new(ExprKind::Method { recv: Box::new(en), name: name.clone(), args: args.clone() }, e.line, e.col)));
                        }
                    }
                }
            }
        }
        if let ExprKind::Ident(alias) = &recv.kind {
            if self.lookup(alias).is_none() && self.module_aliases.contains_key(alias) {
                let key = match self.module_item(alias, name, e.line, e.col)? {
                    Some(k) => k,
                    None => return Ok(None),
                };
                if self.enums.contains_key(&key) {
                    return Err(LumeError::new(e.line, e.col, format!("`{}` is an enum; pick a variant like `{}.{}`", key, key, self.enums[&key].variants[0].0)));
                }
                if self.consts.contains_key(&key) {
                    if !args.is_empty() {
                        return Err(LumeError::new(e.line, e.col, format!("`{}` is a constant, not a function", key)));
                    }
                    return Ok(Some(Expr::new(ExprKind::Ident(key), e.line, e.col)));
                }
                return Ok(Some(Expr::new(ExprKind::Call { name: key, args: args.clone() }, e.line, e.col)));
            }
        }
        Ok(None)
    }

    /// `xs.map(f)` with `f` a function name: the same as `xs.map { |x| f(x) }`.
    fn fn_ref_as_block(&self, e: &Expr) -> Option<Expr> {
        let (recv, name, args) = match &e.kind {
            ExprKind::Method { recv, name, args } => (recv, name, args),
            _ => return None,
        };
        const BLOCK_METHODS: &[&str] = &[
            "map", "filter", "reject", "each", "sum", "count", "any?", "all?", "find", "take_while", "sort_by", "min_by", "max_by", "group_by", "partition", "flat_map",
            "map_values",
        ];
        if !BLOCK_METHODS.contains(&name.as_str()) || args.len() != 1 || args[0].name.is_some() {
            return None;
        }
        let (l, c) = (args[0].value.line, args[0].value.col);
        let it = Arg { name: None, value: Expr::new(ExprKind::Ident("_".into()), l, c) };
        let call = match &args[0].value.kind {
            ExprKind::Ident(n) if self.lookup(n).is_none() && self.fns.contains_key(n) => Expr::new(ExprKind::Call { name: n.clone(), args: vec![it] }, l, c),
            // a block the function was given, handed straight on to a
            // built-in: `def total(xs, f) = xs.sum(f)`
            ExprKind::Ident(n) if matches!(self.lookup(n).map(|b| &b.ty), Some(Type::Fn(..))) => Expr::new(ExprKind::Call { name: n.clone(), args: vec![it] }, l, c),
            // a crate function named by its path: `xs.map(urlencoding.encode)`
            ExprKind::Method { recv: fr, name: fname, args: fargs } if fargs.is_empty() && matches!(self.foreign_ref(&args[0].value), Some(ForeignRef::Fn(_)) | Some(ForeignRef::Assoc(..))) => {
                Expr::new(ExprKind::Method { recv: fr.clone(), name: fname.clone(), args: vec![it] }, l, c)
            }
            _ => return None,
        };
        let lam = Expr::new(ExprKind::Lambda { params: vec!["_".into()], body: Block { stmts: vec![Stmt::Expr(call)] } }, l, c);
        Some(Expr::new(ExprKind::Method { recv: recv.clone(), name: name.clone(), args: vec![Arg { name: None, value: lam }] }, e.line, e.col))
    }

    /// `total?` where nothing is called `total?` but `total` is a value
    /// here: the `?` is propagation, not part of the name. The lexer folds
    /// a trailing `?` into an identifier, so only this can tell them apart.
    fn split_ident_try(&mut self, e: &Expr) -> Option<Expr> {
        let n = match &e.kind {
            ExprKind::Ident(n) => n,
            // a zero-argument function called bare parses as a call
            ExprKind::Call { name, args } if args.is_empty() => name,
            _ => return None,
        };
        if !n.ends_with('?') || n.len() < 2 || self.lookup(n).is_some() {
            return None;
        }
        let base = &n[..n.len() - 1];
        let known = self.lookup(base).is_some() || self.field_type(base).is_some() || self.consts.contains_key(&self.canon(base));
        if !known {
            return None;
        }
        let inner = Expr::new(ExprKind::Ident(base.to_string()), e.line, e.col);
        Some(Expr::new(ExprKind::Try(Box::new(inner)), e.line, e.col))
    }

    /// `x.name?` where no method `name?` exists but `name` does means
    /// `x.name` followed by `?` (propagation). Returns the rewritten
    /// expression in that case.
    fn split_trailing_try(&mut self, e: &Expr) -> Option<Expr> {
        let (recv, name, args) = match &e.kind {
            ExprKind::Method { recv, name, args } => (recv, name, args),
            _ => return None,
        };
        if !name.ends_with('?') || name.len() < 2 {
            return None;
        }
        let base = &name[..name.len() - 1];
        // built-in namespaces: File.read?
        if let ExprKind::Ident(tn) = &recv.kind {
            if self.lookup(tn).is_none() && !self.is_type(&self.canon(tn)) && builtin_namespace_type(tn, name).is_none() && builtin_namespace_type(tn, base).is_some() {
                let inner = Expr::new(ExprKind::Method { recv: recv.clone(), name: base.to_string(), args: args.clone() }, e.line, e.col);
                return Some(Expr::new(ExprKind::Try(Box::new(inner)), e.line, e.col));
            }
        }
        let rt = self.ty_of(recv);
        let exists = |g: &Self, n: &str| -> bool {
            match &rt {
                Type::Named(tn) => {
                    g.methods_of(tn).map(|m| m.contains_key(n)).unwrap_or(false)
                        || g.structs.get(tn).map(|s| s.fields.iter().any(|(f, _)| f == n)).unwrap_or(false)
                }
                Type::Unknown => false,
                other => is_builtin_name(n) || builtin_method_type(other, n) != Type::Unknown,
            }
        };
        if exists(self, name) || !exists(self, base) {
            return None;
        }
        let inner = Expr::new(ExprKind::Method { recv: recv.clone(), name: base.to_string(), args: args.clone() }, e.line, e.col);
        Some(Expr::new(ExprKind::Try(Box::new(inner)), e.line, e.col))
    }

    // ----- type inference ---------------------------------------------------

    fn ty_of(&mut self, e: &Expr) -> Type {
        if matches!(&e.kind, ExprKind::Ident(_) | ExprKind::Call { .. }) {
            if let Some(ne) = self.split_ident_try(e) {
                return self.ty_of(&ne);
            }
        }
        if let ExprKind::Method { .. } = &e.kind {
            if let Some(ne) = self.split_trailing_try(e) {
                return self.ty_of(&ne);
            }
            if let Some(ne) = self.fn_ref_as_block(e) {
                return self.ty_of(&ne);
            }
            if let Ok(Some(ne)) = self.module_ref(e) {
                return self.ty_of(&ne);
            }
        }
        match &e.kind {
            ExprKind::Int(_) => Type::Int,
            ExprKind::Float(_) => Type::Float,
            ExprKind::Bool(_) => Type::Bool,
            ExprKind::Str(_) => Type::Str,
            ExprKind::Ident(n) => {
                if let Some(b) = self.lookup(n) {
                    b.ty.clone()
                } else if let Some(t) = self.consts.get(&self.canon(n)).cloned() {
                    t
                } else if let Some(t) = self.field_type(n) {
                    t
                } else if let Some(m) = self.bare_method(n) {
                    m.ret
                } else if let Some(st) = self.current_self_ty.clone() {
                    if builtin_method_type(&st, n) != Type::Unknown || is_builtin_name(n) {
                        builtin_method_type(&st, n)
                    } else {
                        Type::Unknown
                    }
                } else if let Ok(Some(en)) = self.resolve_variant(n, e.line, e.col) {
                    self.variant_type(&en, n, &[])
                } else {
                    Type::Unknown
                }
            }
            ExprKind::SelfRef => self.current_self_ty.clone().or_else(|| self.current_type.clone().map(Type::Named)).unwrap_or(Type::Unknown),
            ExprKind::List(items) => {
                let mut t = Type::Unknown;
                for i in items {
                    let it = self.ty_of(i).materialized();
                    if it != Type::Unknown {
                        t = it;
                        break;
                    }
                }
                Type::List(Box::new(t))
            }
            ExprKind::Range { .. } => Type::Iter(Box::new(Type::Int), false),
            ExprKind::Unary { op, expr } => match *op {
                "not" => Type::Bool,
                _ => self.ty_of(expr),
            },
            ExprKind::Binary { op, lhs, rhs } => {
                if let Type::Named(tn) = self.ty_of(lhs).materialized() {
                    if let Some(sg) = self.methods_of(&tn).and_then(|m| m.get(*op).cloned()) {
                        return sg.ret;
                    }
                }
                match *op {
                    "==" | "!=" | "<" | "<=" | ">" | ">=" | "and" | "or" => Type::Bool,
                    _ => {
                        let l = self.ty_of(lhs);
                        if l == Type::Unknown { self.ty_of(rhs) } else { l }
                    }
                }
            }
            ExprKind::Call { name: raw_name, args } => {
                let cname = self.canon(raw_name);
                let name = &cname;
                // `f(x)` where `f` is a block this function was handed
                if let Some(Type::Fn(_, out)) = self.lookup(raw_name).map(|b| b.ty.clone()) {
                    return *out;
                }
                if let Some(m) = self.current_type.as_ref().and_then(|t| self.methods_of(t)).and_then(|m| m.get(name).cloned()) {
                    // inside a type, its own method wins over a free function
                    let ret = self.call_ret(&m, args);
                    if m.is_async { Type::Future(Box::new(ret)) } else { ret }
                } else if let Some(s) = self.fns.get(name).cloned() {
                    let ret = self.call_ret(&s, args);
                    if s.is_async { Type::Future(Box::new(ret)) } else { ret }
                } else if let Some(info) = self.structs.get(name).cloned() {
                    self.ctor_type(name, &info, args)
                } else if let Ok(Some(en)) = self.resolve_variant(name, e.line, e.col) {
                    self.variant_type(&en, name, args)
                } else {
                    Type::Unknown
                }
            }
            ExprKind::Method { recv, name, args } => {
                // Enum variant constructor: Shape.Circle(...)
                if let ExprKind::Ident(tn) = &recv.kind {
                    let ctn = self.canon(tn);
                    if self.lookup(tn).is_none() && self.enums.contains_key(&ctn) {
                        return Type::Named(ctn);
                    }
                    if self.lookup(tn).is_none() && !self.is_type(&ctn) {
                        if let Some(t) = builtin_namespace_type(tn, name) {
                            if tn == "Time" && name == "sleep" && self.in_async {
                                return Type::Future(Box::new(Type::Unit));
                            }
                            return t;
                        }
                    }
                }
                if let Some(fr) = self.foreign_ref(e) {
                    return match fr {
                        ForeignRef::Fn(f) | ForeignRef::Assoc(_, f) => f.ret.clone(),
                        ForeignRef::Type(_) => Type::Unknown,
                    };
                }
                // `m[k].push(x)` / `grid[i].push(x)`: the stored collection, changed in place
                if is_mutating_builtin(name) {
                    if let Some(el) = self.indexed_collection(recv) {
                        return builtin_method_type(&el, name);
                    }
                }
                // `xs[i].field` / `xs[i].method(...)`: the element itself, as a place
                if let Some(elem) = self.indexed_element(recv, name) {
                    let inner = Expr::new(ExprKind::Method { recv: Box::new(Expr::new(ExprKind::Ident("lume_elem".into()), recv.line, recv.col)), name: name.clone(), args: args.clone() }, e.line, e.col);
                    self.push_scope();
                    self.declare("lume_elem", true, true, elem, recv.line);
                    let t = self.ty_of(&inner);
                    self.pop_scope();
                    return t;
                }
                let rt = match self.ty_of(recv) {
                    // a shared value behaves as the value it holds
                    Type::Shared(inner, _) => *inner,
                    other => other,
                };
                if let Some(ft) = self.foreign_type(&rt) {
                    return ft.methods.get(name).map(|f| f.ret.clone()).unwrap_or(Type::Unknown);
                }
                if let Some(Arg { value: lam, .. }) = args.last() {
                    if matches!(lam.kind, ExprKind::Lambda { .. }) && self.user_block_method(&rt, name) {
                        // the receiver's own type declares this one
                    } else if let ExprKind::Lambda { params, body } = &lam.kind {
                        let init = if args.len() > 1 { Some(self.ty_of(&args[0].value)) } else { None };
                        let t = self.block_method_type(&rt, name, params, body, init);
                        // a chain that starts at a `shared var` is materialised inside the lock
                        if let Type::Iter(el, _) = &t {
                            if self.shared_root(recv).is_some() {
                                return Type::List(el.clone());
                            }
                        }
                        return t;
                    }
                }
                if let Type::Named(_) | Type::App(..) | Type::Var(_) = &rt {
                    let sn = &self.type_key(&rt);
                    let sub = self.subst_for(&rt);
                    if let Some(info) = self.structs.get(sn) {
                        if let Some((_, ft)) = info.fields.iter().find(|(n, _)| n == name) {
                            return Self::subst(ft, &sub);
                        }
                    }
                    if let Some(m) = self.methods_for(&rt).and_then(|m| m.get(name).cloned()) {
                        let ret = Self::subst(&self.call_ret(&m, args), &sub);
                        return if m.is_async { Type::Future(Box::new(ret)) } else { ret };
                    }
                    if let Some(m) = self.iface_default(&rt, name) {
                        return m.ret;
                    }
                    if name == "to_s" || name == "to_str" {
                        return Type::Str;
                    }
                }
                if self.ext_methods.get(&self.type_key(&rt)).map(|m| m.contains_key(name)).unwrap_or(false) {
                    // through `methods_for`, so a generic `extend`'s names
                    // are read as what this receiver holds
                    if let Some(m) = self.methods_for(&rt).and_then(|m| m.get(name).cloned()) {
                        return m.ret;
                    }
                }
                if let Some(m) = self.iface_default(&rt, name) {
                    return m.ret;
                }
                if name == "or" && args.len() == 1 && matches!(&rt, Type::Option(i) if **i == Type::Char) && char_literal(&args[0].value).is_none() {
                    if self.ty_of(&args[0].value).materialized() == Type::Str {
                        return Type::Str;
                    }
                }
                if name == "zip" && args.len() == 1 {
                    let other = self.ty_of(&args[0].value);
                    if let (Some((a, _)), Some((b, _))) = (self.elem_of(&rt), self.elem_of(&other)) {
                        return Type::List(Box::new(Type::Tuple(vec![a, b])));
                    }
                }
                builtin_method_type(&rt, name)
            }
            ExprKind::If { branches, else_block } => {
                if else_block.is_none() {
                    return Type::Unit;
                }
                for (_, b) in branches {
                    let t = self.tail_type(b);
                    if t != Type::Unknown {
                        return t;
                    }
                }
                if let Some(eb) = else_block {
                    return self.tail_type(eb);
                }
                Type::Unknown
            }
            ExprKind::Puts(_) | ExprKind::Warn(_) => Type::Unit,
            ExprKind::Placeholder | ExprKind::Lambda { .. } => Type::Unknown,
            ExprKind::Match { scrutinee, arms } => self.match_type(scrutinee, arms),
            ExprKind::Tuple(items) if items.is_empty() => Type::Unit,
            ExprKind::Tuple(items) => Type::Tuple(items.iter().map(|i| self.ty_of(i).materialized()).collect()),
            ExprKind::TupleIndex { recv, index } => match self.ty_of(recv) {
                Type::Tuple(ts) => ts.get(*index).cloned().unwrap_or(Type::Unknown),
                _ => Type::Unknown,
            },
            ExprKind::Some(x) => Type::Option(Box::new(self.ty_of(x).materialized())),
            ExprKind::None => Type::Option(Box::new(Type::Unknown)),
            ExprKind::Try(x) | ExprKind::Unwrap(x) => match self.ty_of(x) {
                Type::Option(inner) => *inner,
                Type::Result(t, _) => *t,
                _ => Type::Unknown,
            },
            ExprKind::Rust(_) => Type::Unknown,
            ExprKind::Index { recv, index } => {
                let slice = matches!(index.kind, ExprKind::Range { .. });
                match self.ty_of(recv).materialized() {
                    Type::List(e) if slice => Type::List(e),
                    Type::List(e) => Type::Option(e),
                    Type::Map(_, v) => Type::Option(v),
                    Type::Str if slice => Type::Str,
                    Type::Str => Type::Option(Box::new(Type::Char)),
                    _ => Type::Unknown,
                }
            }
            ExprKind::SetLit(items) => {
                let mut t = Type::Unknown;
                for i in items {
                    if t == Type::Unknown {
                        t = self.ty_of(i).materialized();
                    }
                }
                Type::Set(Box::new(t))
            }
            ExprKind::MapLit(pairs) => {
                let mut k = Type::Unknown;
                let mut v = Type::Unknown;
                for (pk, pv) in pairs {
                    if k == Type::Unknown { k = self.ty_of(pk).materialized(); }
                    if v == Type::Unknown { v = self.ty_of(pv).materialized(); }
                }
                Type::Map(Box::new(k), Box::new(v))
            }
            ExprKind::Ok(x) => {
                let t = self.ty_of(x).materialized();
                match &self.current_ret {
                    Type::Result(_, e) => Type::Result(Box::new(t), e.clone()),
                    _ => Type::Result(Box::new(t), Box::new(Type::Named("Error".into()))),
                }
            }
            ExprKind::Await(x) => match self.ty_of(x).materialized() {
                Type::Future(t) | Type::Task(t) => *t,
                Type::List(inner) => match *inner {
                    Type::Task(t) => Type::List(t),
                    _ => Type::Unknown,
                },
                _ => Type::Unknown,
            },
            ExprKind::Spawn(body) => Type::Task(Box::new(self.spawn_body_type(body))),
        }
    }

    /// The value a `spawn:` block produces: its last expression, else `()`.
    fn spawn_body_type(&mut self, body: &Block) -> Type {
        match body.stmts.last() {
            Some(Stmt::Expr(e)) => {
                self.push_scope();
                // bindings made earlier in the block shape the tail's type
                for st in &body.stmts[..body.stmts.len() - 1] {
                    if let Stmt::Bind { name, ty, value, line, .. } | Stmt::Var { name, ty, value, line, .. } = st {
                        let t = ty.as_ref().map(|t| self.ct(t)).unwrap_or_else(|| self.ty_of(value).materialized());
                        self.declare(name, true, false, t, *line);
                    }
                }
                let t = self.ty_of(e).materialized();
                self.pop_scope();
                match t {
                    Type::Future(_) => Type::Unknown,
                    other => other,
                }
            }
            _ => Type::Unit,
        }
    }

    /// The rules of the binary operators: arithmetic needs two `Int`s or two
    /// `Float`s, `+` also joins two `Str`s, ordering needs numbers or strings
    /// of one kind, `==` needs one type on both sides, `and`/`or` need `Bool`.
    fn check_operands(&mut self, op: &str, lt: &Type, rhs: &Expr, line: usize, col: usize) -> Result<()> {
        let rt = self.ty_of(rhs).materialized();
        if rt == Type::Unknown {
            return Ok(());
        }
        let op = op.trim_end_matches('=').to_string();
        let op = if op.is_empty() { "=".to_string() } else { op };
        let op = op.as_str();
        let numeric = |t: &Type| matches!(t, Type::Int | Type::Float);
        let same = self.assignable(&rt, lt) && self.assignable(lt, &rt);
        let ok = match op {
            "and" | "or" => *lt == Type::Bool && rt == Type::Bool,
            "+" => (numeric(lt) && same) || (*lt == Type::Str && rt == Type::Str) || (matches!(lt, Type::List(_) | Type::Iter(..)) && self.assignable(lt, &rt)) || text_like(lt) && text_like(&rt),
            "-" | "*" | "/" | "%" | "**" => numeric(lt) && same,
            // a type parameter compares when its bound says it is `Ordered`
            "<" | "<=" | ">" | ">=" => (same && (numeric(lt) || *lt == Type::Str || (matches!(lt, Type::Var(_)) && self.meets_bound(lt, &Type::Named("Ordered".into()))))) || (text_like(lt) && text_like(&rt)),
            "==" | "!=" => same || (text_like(lt) && text_like(&rt)),
            _ => true,
        };
        if ok {
            return Ok(());
        }
        // a type parameter with no bound: say what the bound would have to be
        if let (Type::Var(a), Type::Var(b)) = (lt, &rt) {
            if a == b && matches!(op, "<" | "<=" | ">" | ">=") {
                return Err(LumeError::new(line, col, format!("`{}` could be any type, so `{}` has nothing to compare", a, op))
                    .with_help(format!("declare it `[{}: Ordered]`, which promises every type it is given can be compared", a)));
            }
        }
        let mut err = LumeError::new(line, col, format!("`{}` cannot combine a `{}` and a `{}`", op, type_name(lt), type_name(&rt)));
        let help = match (op, lt, &rt) {
            ("+", Type::Str, _) | ("+", _, Type::Str) => Some(format!("to build a string, interpolate: `\"...#{{{}}}...\"`; `+` joins two strings or adds two numbers", snippet(rhs))),
            (_, Type::Int, Type::Float) => Some("convert one side: `.to_float` on the Int, or `.round`/`.floor` on the Float".to_string()),
            (_, Type::Float, Type::Int) => Some(format!("write `{}.to_float`, or a float literal like `2.0`", snippet(rhs))),
            (_, Type::Option(inner), _) if self.assignable(inner, &rt) => Some("the left side may be absent: unwrap it with `match`, `?` or `.or(default)`".to_string()),
            (_, _, Type::Option(inner)) if self.assignable(lt, inner) => Some("the right side may be absent: unwrap it with `match`, `?` or `.or(default)`".to_string()),
            ("and", _, _) | ("or", _, _) => Some("both sides must be `Bool`; compare first, as in `x > 0 and y > 0`".to_string()),
            ("+", Type::Set(_), Type::Set(_)) => Some("sets combine by name: `a.union(b)`; the others are `.intersect(b)` and `.diff(b)`".to_string()),
            ("-", Type::Set(_), Type::Set(_)) => Some("write `a.diff(b)` for the items of `a` that are not in `b`".to_string()),
            ("+", Type::Map(..), Type::Map(..)) => Some("write `a.merge(b)`: the entries of `b` win where the keys are the same".to_string()),
            _ => None,
        };
        if let Some(h) = help {
            err = err.with_help(h);
        }
        Err(err)
    }

    /// `xs[i]` where `xs` is a list of structs/enums: the element type, when
    /// the expression is used as a place (`xs[i].field`, `xs[i].method`).
    fn indexed_element(&mut self, recv: &Expr, member: &str) -> Option<Type> {
        if let ExprKind::Index { recv: lst, .. } = &recv.kind {
            if let Type::List(elem) = self.ty_of(lst).materialized() {
                if let Type::Named(n) = &*elem {
                    let key = self.canon(n);
                    let has_member = self.structs.get(&key).map(|s| s.fields.iter().any(|(f, _)| f == member)).unwrap_or(false)
                        || self.methods_of(&key).map(|m| m.contains_key(member)).unwrap_or(false);
                    if has_member {
                        return Some(*elem);
                    }
                }
            }
        }
        None
    }

    /// The Rust text of a map key in a read position, borrowed as the map
    /// wants it. A one-character literal is a `Char` where the keys are.
    fn map_key_text(&mut self, index: &Expr, key_ty: &Type, map_ty: &Type) -> Result<String> {
        if *key_ty == Type::Char {
            if let Some(c) = char_literal(index) {
                return Ok(format!("&'{}'", rust_char(c)));
            }
        }
        let k = self.expr_val(index)?;
        Ok(map_key(map_ty, &k))
    }

    /// `xs[i]` / `m[k]` whose stored value is itself a list, set or map.
    fn indexed_collection(&mut self, recv: &Expr) -> Option<Type> {
        if let ExprKind::Index { recv: cont, index } = &recv.kind {
            if matches!(index.kind, ExprKind::Range { .. }) {
                return None;
            }
            match self.ty_of(cont).materialized() {
                Type::List(el) | Type::Map(_, el) if matches!(*el, Type::List(_) | Type::Set(_) | Type::Map(..)) => return Some(*el),
                _ => {}
            }
        }
        None
    }

    /// `m[k].add(x)`, `grid[i].push(x)`: a mutating built-in on a collection
    /// stored in a list or map changes it where it is stored. A missing map
    /// key starts from an empty value; a list position out of range stops
    /// the program, like any index write.
    fn indexed_collection_mutation(&mut self, recv: &Expr, name: &str, args: &[Arg], e: &Expr) -> Result<Option<String>> {
        if !is_mutating_builtin(name) {
            return Ok(None);
        }
        let el = match self.indexed_collection(recv) {
            Some(el) => el,
            None => return Ok(None),
        };
        let (cont, index) = match &recv.kind {
            ExprKind::Index { recv, index } => (recv, index),
            _ => unreachable!(),
        };
        let ct = self.ty_of(cont).materialized();
        let place = match &ct {
            Type::Map(k, _) => {
                let m = self.mutable_place(cont, "this map", e.line, e.col)?;
                let kt = (**k).clone();
                self.check_assign(index, &kt, &format!("this map's keys are `{}`", type_name(&kt)))?;
                let key = self.expr_owned_as(index, &kt)?;
                format!("{}.slot({})", m, key)
            }
            _ => {
                let it = self.ty_of(index).materialized();
                if it != Type::Int && it != Type::Unknown {
                    return Err(self.type_mismatch(index, &it, &Type::Int, "a list position is an `Int`"));
                }
                self.mutable_place(recv, "this position", e.line, e.col)?
            }
        };
        // emit the call against a synthetic `var` binding standing for the place
        let tmp = self.fresh("slot");
        self.push_scope();
        self.declare(&tmp, true, false, el, recv.line);
        let inner = Expr::new(ExprKind::Method { recv: Box::new(Expr::new(ExprKind::Ident(tmp.clone()), recv.line, recv.col)), name: name.to_string(), args: args.to_vec() }, e.line, e.col);
        let text = self.expr(&inner);
        self.pop_scope();
        Ok(Some(text?.replacen(&tmp, &format!("({})", place), 1)))
    }

    /// A value stored somewhere a change can reach: a name, `self`, a field,
    /// a position in a stored list or map.
    fn is_place(&mut self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Ident(_) | ExprKind::SelfRef => true,
            ExprKind::Index { recv, index } => !matches!(index.kind, ExprKind::Range { .. }) && self.is_place(recv),
            ExprKind::TupleIndex { recv, .. } => self.is_place(recv),
            ExprKind::Method { recv, name, args } if args.is_empty() => {
                let rt = self.ty_of(recv).materialized();
                let is_field = match &rt {
                    // a generic struct reached at an instantiation is the
                    // same struct: `Stack[Int]` has `Stack`'s fields
                    Type::Named(n) | Type::App(n, _) => self.structs.get(&self.canon(n)).map(|s| s.fields.iter().any(|(f, _)| f == name)).unwrap_or(false),
                    _ => false,
                };
                is_field && self.is_place(recv)
            }
            _ => false,
        }
    }

    /// Emits `xs[i].name(args)` against the element in place: a mutating
    /// method needs `xs` to be a `var`; an out-of-range position stops the
    /// program like any other index write.
    fn indexed_member(&mut self, recv: &Expr, elem: Type, name: &str, args: &[Arg], e: &Expr) -> Result<String> {
        let (lst, index) = match &recv.kind {
            ExprKind::Index { recv, index } => (recv, index),
            _ => unreachable!(),
        };
        let mutates = match &elem {
            Type::Named(tn) => self.methods_of(tn).and_then(|m| m.get(name).map(|s| s.self_kind == SelfKind::Mutate)).unwrap_or(false),
            _ => false,
        };
        if mutates {
            self.check_receiver_mutable(lst, name, e.line, e.col)?;
        }
        let l = self.expr(lst)?;
        let i = self.expr_val(index)?;
        let it = self.ty_of(index).materialized();
        if it != Type::Int && it != Type::Unknown {
            return Err(self.type_mismatch(index, &it, &Type::Int, "a list position is an `Int`"));
        }
        // emit the member call against a synthetic place binding
        let tmp = self.fresh("el");
        self.push_scope();
        self.declare(&tmp, true, true, elem, recv.line);
        let inner = Expr::new(ExprKind::Method { recv: Box::new(Expr::new(ExprKind::Ident(tmp.clone()), recv.line, recv.col)), name: name.to_string(), args: args.to_vec() }, e.line, e.col);
        let text = self.expr(&inner);
        self.pop_scope();
        let text = text?;
        // `tmp` stands for the place `xs[i]`
        Ok(text.replacen(&tmp, &format!("{}[({}) as usize]", l, i), 1))
    }

    /// A variant field whose type is the enum itself lives behind a `Box`.
    fn boxed_field(&self, en: &str, fty: &Type) -> bool {
        matches!(fty, Type::Named(n) | Type::App(n, _) if self.canon(n) == self.canon(en))
    }

    /// Can a value of type `have` be used where `want` is expected?
    /// Unknown on either side is trusted (inference did not reach it).
    /// Can a value of type `have` be used where `want` is expected?
    fn assignable(&self, have: &Type, want: &Type) -> bool {
        self.assign_ok(have, want, true)
    }

    fn assign_ok_n(&self, have: &Type, want: &Type) -> bool {
        self.assign_ok(have, want, false)
    }

    fn assign_ok(&self, have: &Type, want: &Type, top: bool) -> bool {
        match (have, want) {
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            (Type::Future(_), _) | (_, Type::Future(_)) => true,
            (Type::Shared(h, _), w) => self.assign_ok_n(h, w),
            (h, Type::Shared(w, _)) => self.assign_ok_n(h, w),
            (Type::Iter(a, _), Type::List(b)) | (Type::List(a), Type::Iter(b, _)) | (Type::List(a), Type::List(b)) | (Type::Iter(a, _), Type::Iter(b, _)) => self.assign_ok_n(a, b),
            (Type::Option(a), Type::Option(b)) | (Type::Task(a), Type::Task(b)) => self.assign_ok_n(a, b),
            (Type::Result(a, b), Type::Result(c, d)) | (Type::Map(a, b), Type::Map(c, d)) => self.assign_ok_n(a, c) && self.assign_ok_n(b, d),
            (Type::Set(a), Type::Set(b)) => self.assign_ok_n(a, b),
            // a character is a one-character string wherever a string is wanted
            (Type::Char, Type::Str) => top,
            // `{}` where a set is wanted is the empty set
            (Type::Map(k, v), Type::Set(_)) if **k == Type::Unknown && **v == Type::Unknown => true,
            (Type::Tuple(xs), Type::Tuple(ys)) => xs.len() == ys.len() && xs.iter().zip(ys).all(|(x, y)| self.assign_ok_n(x, y)),
            (Type::Named(a), Type::Named(b)) if self.canon(a) == self.canon(b) => true,
            (Type::Var(a), Type::Var(b)) => a == b,
            (Type::Fn(ps, r), Type::Fn(qs, s)) => ps.len() == qs.len() && ps.iter().zip(qs).all(|(x, y)| self.assign_ok_n(x, y)) && self.assign_ok_n(r, s),
            (Type::Fn(..), _) | (_, Type::Fn(..)) => false,
            (Type::App(a, xs), Type::App(b, ys)) if self.canon(a) == self.canon(b) => xs.len() == ys.len() && xs.iter().zip(ys).all(|(x, y)| self.assign_ok_n(x, y)),
            // a value where an interface is wanted: it must have the methods
            (_, Type::Named(_) | Type::App(..)) if self.is_interface(want) => matches!(self.conformance(have, want), Conformance::Yes),
            (Type::Var(_), _) | (_, Type::Var(_)) | (Type::App(..), _) | (_, Type::App(..)) => false,
            (Type::Named(_), _) | (_, Type::Named(_)) => false,
            (a, b) => a == b,
        }
    }

    /// The error for a value of the wrong type. `what` names the slot:
    /// "`add` takes `b: Int`", "`User` field `age` is `Int`", "`f` returns `Str`".
    /// `x.ok?` where `x.ok` followed by `?` was meant: the predicate and the
    /// early return share a spelling, and only parentheses tell them apart.
    fn predicate_try_hint(&self, e: &Expr, have: &Type, want: &Type) -> Option<String> {
        if *have != Type::Bool || *want == Type::Bool {
            return None;
        }
        if let ExprKind::Method { recv, name, args } = &e.kind {
            if args.is_empty() && name.ends_with('?') && name.len() > 1 {
                let base = &name[..name.len() - 1];
                return Some(format!(
                    "`.{}` asks a yes/no question; for `.{}` followed by `?` (the early return) write `({}.{})?`",
                    name, base, snippet(recv), base
                ));
            }
        }
        None
    }

    fn type_mismatch(&self, e: &Expr, have: &Type, want: &Type, what: &str) -> LumeError {
        let err = LumeError::new(e.line, e.col, format!("{}, but this is a `{}`", what, type_name(have)));
        if let Some(h) = self.predicate_try_hint(e, have, want) {
            return err.with_help(h);
        }
        let help = match (have, want) {
            (Type::Str, Type::Char) => Some("a `Char` is one character: use a one-character literal like `\"a\"`, or `.chars` and take one".to_string()),
            (Type::Char, Type::Str) => Some("write `.to_s` to make it a string".to_string()),
            (Type::List(h), Type::List(w)) | (Type::Set(h), Type::Set(w)) if **h == Type::Char && **w == Type::Str => {
                Some("the characters of a string are `Char` values: declare the type with `Char`, or turn them into strings with `.map(_.to_s)`".to_string())
            }
            (Type::Str, Type::Int) => Some("parse it: `.to_int` gives `Int or Error`, so `x.to_int?` or `x.to_int.or(0)`".to_string()),
            (Type::Str, Type::Float) => Some("parse it: `.to_float` gives `Float or Error`".to_string()),
            (Type::Int, Type::Str) | (Type::Float, Type::Str) | (Type::Bool, Type::Str) => Some(format!("write `\"#{{{}}}\"` or `{}.to_s`", snippet(e), snippet(e))),
            (Type::Int, Type::Float) => Some(format!("write `{}.to_float`, or a float literal like `2.0`", snippet(e))),
            (Type::Float, Type::Int) => Some(format!("write `{}.round`, `.floor` or `.ceil` to get an `Int`", snippet(e))),
            (Type::Option(inner), w) if self.assignable(inner, w) => Some("this may be absent: unwrap it with `match`, `?` or `.or(default)`".to_string()),
            (Type::Result(inner, _), w) if self.assignable(inner, w) => Some("this may be an error: `?` passes it up, `match` handles it, `.or(default)` ignores it".to_string()),
            (Type::List(a), Type::List(b)) if **a != Type::Unknown && **b != Type::Unknown => Some(format!("the items are `{}`; `.map` them into `{}` first", type_name(a), type_name(b))),
            _ => None,
        };
        match help {
            Some(h) => err.with_help(h),
            None => err,
        }
    }

    /// Checks `e` against `want`; implied `Some`/`Ok` wrapping is decided by
    /// the caller, so a plain `T` where `T?` is wanted passes here.
    fn check_assign(&mut self, e: &Expr, want: &Type, what: &str) -> Result<()> {
        // a one-character literal is a `Char` where one is wanted
        if *want == Type::Char && char_literal(e).is_some() {
            return Ok(());
        }
        self.want.push(want.clone());
        let have = self.ty_of(e);
        self.want.pop();
        if let Type::Named(_) | Type::App(..) = want {
            // an interface slot: the detailed conformance error says what is missing
            if self.is_interface(want) && have != Type::Unknown && !self.is_interface(&have.materialized()) {
                let want = want.clone();
                return self.require_conforms(&have.materialized(), &want, e.line, e.col);
            }
        }
        if self.assignable(&have, want) {
            return Ok(());
        }
        // a bare value where an optional or a result is wanted is wrapped, not rejected
        match want {
            Type::Option(inner) if self.assignable(&have, inner) && !matches!(have, Type::Option(_)) => return Ok(()),
            Type::Result(ok, _) if self.assignable(&have, ok) && !matches!(have, Type::Result(..)) => return Ok(()),
            _ => {}
        }
        Err(self.type_mismatch(e, &have.materialized(), want, what))
    }

    /// `expr_arg` with the type check and the message naming the parameter.
    /// The type a generic parameter was *declared* with decides how the
    /// value is passed (a `T` is always lent), while the type this call
    /// filled in decides what is allowed.
    fn expr_arg_named_as(&mut self, a: &Expr, check: &Type, decl: &Type, callee: &str, pname: &str) -> Result<String> {
        // a block: the declared types set the Rust shape, this call's set
        // what the body is allowed to do
        if let (Type::Fn(dins, _), Type::Fn(cins, cout)) = (decl, check) {
            let (dins, cins, cout) = (dins.clone(), cins.clone(), cout.clone());
            let recursive = callee == self.current_fn;
            return self.block_arg(a, &cins, &dins, &cout, recursive);
        }
        if check == decl {
            return self.expr_arg_named(a, check, callee, pname);
        }
        self.check_assign(a, check, &format!("`{}` takes `{}: {}`", callee, pname, type_name(check)))?;
        self.expr_arg(a, decl)
    }

    fn expr_arg_named(&mut self, a: &Expr, t: &Type, callee: &str, pname: &str) -> Result<String> {
        self.check_assign(a, t, &format!("`{}` takes `{}: {}`", callee, pname, type_name(t)))?;
        let have = self.ty_of(a);
        // a plain value where `T?` / `T or E` is expected: wrap it
        match t {
            Type::Option(inner) if !matches!(have, Type::Option(_) | Type::Unknown) && self.assignable(&have, inner) => {
                let v = self.expr_owned(a)?;
                return Ok(format!("Some({})", v));
            }
            Type::Result(ok, _) if !matches!(have, Type::Result(..) | Type::Unknown) && self.assignable(&have, ok) => {
                let v = self.expr_owned(a)?;
                return Ok(format!("Ok({})", v));
            }
            _ => {}
        }
        self.expr_arg(a, t)
    }

    /// Is `t` a value that must be awaited before use?
    fn no_future(&self, t: &Type, e: &Expr) -> Result<()> {
        if let Type::Future(_) = t {
            let what = match &e.kind {
                ExprKind::Call { name, .. } => format!("`{}` is an `async def`", name),
                ExprKind::Method { name, .. } => format!("`{}` is an `async def`", name),
                _ => "this is an async value".to_string(),
            };
            return Err(LumeError::new(e.line, e.col, format!("{}: its result arrives later", what))
                .with_help("write `await` in front of the call to wait for it, or `spawn:` to run it alongside other work"));
        }
        Ok(())
    }

    /// If the receiver chain of a place (`x`, `x.a`, `x.a[i].b`) starts at a
    /// `shared var` value, returns (root expression, inner type).
    fn shared_root<'a>(&mut self, e: &'a Expr) -> Option<(&'a Expr, Type)> {
        self.shared_root_ex(e).and_then(|(r, t, m)| if m { Some((r, t)) } else { None })
    }

    /// Like `shared_root`, for `shared` and `shared var` alike; the bool is
    /// true for `shared var`.
    fn shared_root_ex<'a>(&mut self, e: &'a Expr) -> Option<(&'a Expr, Type, bool)> {
        let mut cur = e;
        loop {
            if let Type::Shared(inner, m) = self.ty_of(cur) {
                return Some((cur, *inner, m));
            }
            match &cur.kind {
                ExprKind::Method { recv, args, .. } if args.is_empty() => cur = recv,
                ExprKind::Index { recv, .. } => cur = recv,
                ExprKind::TupleIndex { recv, .. } => cur = recv,
                _ => return None,
            }
        }
    }

    /// The name a shared root is known by, for "mentions" checks.
    fn root_name(e: &Expr) -> Option<String> {
        match &e.kind {
            ExprKind::Ident(n) => Some(n.clone()),
            ExprKind::Method { name, args, .. } if args.is_empty() => Some(name.clone()),
            _ => None,
        }
    }

    /// `e` with the sub-expression `root` (by pointer) replaced by `new`.
    fn replace_root(e: &Expr, root: &Expr, new: &Expr) -> Expr {
        if e.line == root.line && e.col == root.col && same_expr(e, root) {
            return new.clone();
        }
        let kind = match &e.kind {
            ExprKind::Method { recv, name, args } => ExprKind::Method { recv: Box::new(Self::replace_root(recv, root, new)), name: name.clone(), args: args.clone() },
            ExprKind::Index { recv, index } => ExprKind::Index { recv: Box::new(Self::replace_root(recv, root, new)), index: index.clone() },
            ExprKind::TupleIndex { recv, index } => ExprKind::TupleIndex { recv: Box::new(Self::replace_root(recv, root, new)), index: *index },
            other => other.clone(),
        };
        Expr { kind, line: e.line, col: e.col }
    }

    /// The handle (`Arc`) of a shared place, not the value inside it.
    fn handle_expr(&mut self, e: &Expr) -> Result<String> {
        let saved = self.want_handle;
        self.want_handle = true;
        let r = self.expr(e);
        self.want_handle = saved;
        r
    }

    /// Emits `e`, whose receiver chain starts at a `shared var`, as one
    /// locked block: `({ let mut g = root.lock().unwrap(); ...g... })`.
    /// Arguments that mention the shared value are computed first, so the
    /// lock is never taken twice at once.
    fn shared_access(&mut self, e: &Expr, root: &Expr, inner: Type, args_of: Option<&[Arg]>) -> Result<String> {
        let rname = Self::root_name(root);
        let root_text = self.handle_expr(root)?;
        let g = self.fresh("g");
        let mut prelude = String::new();
        let mut e2 = e.clone();
        if let (Some(args), Some(rn)) = (args_of, &rname) {
            let mentions: Vec<bool> = args.iter().map(|a| expr_mentions(&a.value, rn)).collect();
            if mentions.iter().any(|m| *m) {
                if let ExprKind::Method { args: new_args, .. } = &mut e2.kind {
                    for (i, a) in new_args.iter_mut().enumerate() {
                        if matches!(a.value.kind, ExprKind::Lambda { .. }) {
                            if mentions[i] {
                                return Err(LumeError::new(a.value.line, a.value.col, format!("`{}` is used inside a block while `{}` is locked, which would wait forever", rn, rn))
                                    .with_help(format!("bind what the block needs from `{}` to a name before this line", rn)));
                            }
                            continue;
                        }
                        let t = self.ty_of(&a.value).materialized();
                        let v = self.expr_owned(&a.value)?;
                        let tmp = self.fresh("a");
                        prelude.push_str(&format!("let {} = {}; ", tmp, v));
                        self.declare(&tmp, false, false, t, a.value.line);
                        a.value = Expr::new(ExprKind::Ident(tmp), a.value.line, a.value.col);
                    }
                }
            }
        }
        self.push_scope();
        self.declare(&g, true, false, inner, e.line);
        let ge = Expr::new(ExprKind::Ident(g.clone()), root.line, root.col);
        let rewritten = Self::replace_root(&e2, root, &ge);
        let body = self.expr_owned(&rewritten);
        self.pop_scope();
        let body = body?;
        Ok(format!("({{ {}let mut {} = {}.lock().unwrap(); {} }})", prelude, g, root_text, body))
    }

    /// A field or index assignment whose place starts at a `shared var`:
    /// the right-hand side is computed first, then the change happens
    /// under one lock.
    fn shared_stmt(&mut self, s: &Stmt, root: &Expr, inner: Type, value: &Expr, is_tail: bool) -> Result<()> {
        let root_text = self.handle_expr(root)?;
        let vt = self.ty_of(value).materialized();
        let v = self.expr_owned(value)?;
        let tmp = self.fresh("v");
        let g = self.fresh("g");
        self.line("{");
        self.indent += 1;
        self.line(&format!("let {} = {};", tmp, v));
        self.line(&format!("let mut {} = {}.lock().unwrap();", g, root_text));
        self.push_scope();
        self.declare(&tmp, false, false, vt, value.line);
        self.declare(&g, true, false, inner, root.line);
        let ge = Expr::new(ExprKind::Ident(g.clone()), root.line, root.col);
        let ve = Expr::new(ExprKind::Ident(tmp.clone()), value.line, value.col);
        let ns = match s {
            Stmt::FieldAssign { recv, field, op, line, col, .. } => Stmt::FieldAssign { recv: Self::replace_root(recv, root, &ge), field: field.clone(), op: *op, value: ve, line: *line, col: *col },
            Stmt::IndexAssign { recv, index, op, line, col, .. } => Stmt::IndexAssign { recv: Self::replace_root(recv, root, &ge), index: index.clone(), op: *op, value: ve, line: *line, col: *col },
            _ => unreachable!(),
        };
        let r = self.stmt(&ns, false);
        self.pop_scope();
        self.indent -= 1;
        self.line("}");
        r?;
        if is_tail {
            let (line, col) = parser_pos(s);
            return self.tail_unit(line, col);
        }
        Ok(())
    }

    /// `spawn:` — the block runs as its own task. Every local it mentions is
    /// copied in (a `shared` handle is cloned, which is the point of it).
    fn spawn_expr(&mut self, body: &Block, e: &Expr) -> Result<String> {
        self.uses_async = true;
        if self.current_type.is_some() && (block_mentions_self(body) || self.field_names().iter().any(|f| body.stmts.iter().any(|st| stmt_mentions(st, f)))) {
            return Err(LumeError::new(e.line, e.col, "a `spawn:` block inside a method cannot use `self` or its fields")
                .with_help("bind the fields the task needs to names before `spawn:`; a `shared var` field can be bound and passed in"));
        }
        let ret = self.spawn_body_type(body);
        // captured locals
        let mut captured: Vec<(String, Binding)> = Vec::new();
        let mut seen = HashSet::new();
        for scope in self.scopes.iter().rev() {
            for (n, b) in scope {
                if seen.insert(n.clone()) && body.stmts.iter().any(|st| stmt_uses(st, n)) {
                    captured.push((n.clone(), b.clone()));
                }
            }
        }
        captured.sort_by(|a, b| a.0.cmp(&b.0));
        let mut prelude = String::new();
        for (n, b) in &captured {
            let rn = rust_name(n);
            let copy = if matches!(b.ty, Type::Shared(..)) {
                format!("{}.clone()", rn)
            } else if b.ty.is_copy() {
                if b.borrowed { format!("*{}", rn) } else { rn.clone() }
            } else if b.ty == Type::Str {
                format!("{}.to_string()", rn)
            } else {
                format!("{}.clone()", rn)
            };
            prelude.push_str(&format!("let {} = {}; ", rn, copy));
        }
        let saved_loop = self.loop_depth;
        let saved_in_block = self.in_block;
        let saved_tail = self.tail_of_fn;
        let saved_ret = std::mem::replace(&mut self.current_ret, ret.clone());
        let saved_async = self.in_async;
        let saved_captured = std::mem::take(&mut self.spawn_captured);
        self.loop_depth = 0;
        self.in_block = false;
        self.tail_of_fn = true;
        self.in_async = true;
        self.push_scope();
        for (n, b) in &captured {
            self.declare(n, false, false, b.ty.clone(), b.line);
            self.spawn_captured.insert(n.clone());
        }
        let saved_out = std::mem::take(&mut self.out);
        let base = self.indent;
        self.out.push_str("tokio::spawn(async move {\n");
        let r = self.nested_block(body, ret != Type::Unit);
        self.out.push_str(&"    ".repeat(base));
        self.out.push_str("})");
        let text = std::mem::take(&mut self.out);
        self.out = saved_out;
        self.pop_scope();
        self.spawn_captured = saved_captured;
        self.in_async = saved_async;
        self.current_ret = saved_ret;
        self.loop_depth = saved_loop;
        self.in_block = saved_in_block;
        self.tail_of_fn = saved_tail;
        r?;
        Ok(format!("{{ {}{} }}", prelude, text))
    }

    fn field_names(&self) -> Vec<String> {
        match &self.current_type {
            Some(t) => self.structs.get(t).map(|s| s.fields.iter().map(|(n, _)| n.clone()).collect()).unwrap_or_default(),
            None => Vec::new(),
        }
    }

    /// Element type and by-reference flag of a list, range or lazy chain.
    fn elem_of(&self, t: &Type) -> Option<(Type, bool)> {
        match t {
            Type::List(e) => Some(((**e).clone(), !e.is_copy())),
            Type::Iter(e, by_ref) => Some(((**e).clone(), *by_ref)),
            Type::Map(k, v) => Some((Type::Tuple(vec![(**k).clone(), (**v).clone()]), true)),
            Type::Set(e) => Some(((**e).clone(), !e.is_copy())),
            _ => None,
        }
    }

    /// Declares block parameters for an item of type `elem`. Two parameters
    /// destructure a pair; with `acc` set, the first parameter is the fold
    /// accumulator.
    fn declare_block_params(&mut self, params: &[String], elem: &Type, by_ref: bool, acc: Option<&Type>, line: usize) {
        match (acc, params.len(), elem) {
            (Some(at), 2, _) => {
                self.declare(&params[0], false, false, at.clone(), line);
                self.declare(&params[1], false, by_ref && !elem.is_copy(), elem.clone(), line);
            }
            (None, 2, Type::Tuple(ts)) if ts.len() == 2 => {
                self.declare(&params[0], false, by_ref && !ts[0].is_copy(), ts[0].clone(), line);
                self.declare(&params[1], false, by_ref && !ts[1].is_copy(), ts[1].clone(), line);
            }
            _ => {
                if let Some(p) = params.first() {
                    self.declare(p, false, by_ref && !elem.is_copy(), elem.clone(), line);
                }
            }
        }
    }

    fn lambda_body_type(&mut self, params: &[String], elem: &Type, by_ref: bool, acc: Option<&Type>, body: &Block) -> Type {
        self.push_scope();
        self.declare_block_params(params, elem, by_ref, acc, 0);
        let t = self.tail_type(body);
        self.pop_scope();
        t
    }

    /// An empty seed still unknown after `fold_acc_type`: the block's result
    /// `acc.union(x.tags)` / `acc.merge(m)` / `acc + [x]` gives the type.
    fn fold_seed_from_body(&mut self, seed: Type, params: &[String], body: &Block, elem: &Type, by_ref: bool) -> Type {
        if type_is_known(&seed) || params.len() != 2 {
            return seed;
        }
        let tail = match body.stmts.last() {
            Some(Stmt::Expr(e)) => e.clone(),
            _ => return seed,
        };
        let arg = match &tail.kind {
            ExprKind::Method { recv, name, args } if matches!(&recv.kind, ExprKind::Ident(n) if *n == params[0])
                && matches!(name.as_str(), "union" | "intersect" | "diff" | "merge") && args.len() == 1 => args[0].value.clone(),
            ExprKind::Binary { op: "+", lhs, rhs } if matches!(&lhs.kind, ExprKind::Ident(n) if *n == params[0]) => (**rhs).clone(),
            _ => return seed,
        };
        self.push_scope();
        self.declare_block_params(params, elem, by_ref, Some(&seed), 0);
        let t = self.ty_of(&arg).materialized();
        self.pop_scope();
        if type_is_known(&t) && self.assignable(&seed, &t) { t } else { seed }
    }

    fn block_method_type(&mut self, recv: &Type, name: &str, params: &[String], body: &Block, init: Option<Type>) -> Type {
        match (recv, name) {
            (Type::Option(inner), "map") => {
                let bt = self.lambda_body_type(params, inner, false, None, body).materialized();
                return Type::Option(Box::new(bt));
            }
            (Type::Result(ok, err), "map") => {
                let bt = self.lambda_body_type(params, ok, false, None, body).materialized();
                return Type::Result(Box::new(bt), err.clone());
            }
            (Type::Map(..), "filter" | "reject") => return recv.clone(),
            (Type::Set(_), "filter" | "reject") => return recv.clone(),
            (Type::Map(k, v), "map_values") => {
                let bt = self.lambda_body_type(params, v, true, None, body).materialized();
                return Type::Map(k.clone(), Box::new(bt));
            }
            _ => {}
        }
        let (elem, by_ref) = match self.elem_of(recv) {
            Some(x) => x,
            None => return Type::Unknown,
        };
        match name {
            "group_by" => {
                let kt = self.lambda_body_type(params, &elem, by_ref, None, body).materialized();
                Type::Map(Box::new(kt), Box::new(Type::List(Box::new(elem))))
            }
            "partition" => {
                let l = Type::List(Box::new(elem));
                Type::Tuple(vec![l.clone(), l])
            }
            "flat_map" => match self.lambda_body_type(params, &elem, by_ref, None, body).materialized() {
                Type::List(inner) => Type::List(inner),
                _ => Type::Unknown,
            },
            "map" => {
                let bt = self.lambda_body_type(params, &elem, by_ref, None, body).materialized();
                Type::Iter(Box::new(bt), false)
            }
            // map entries come back owned (see `block_method`)
            "take_while" if matches!(recv, Type::Map(..)) => Type::Iter(Box::new(elem), false),
            "filter" | "reject" | "take_while" => Type::Iter(Box::new(elem), by_ref),
            "each" => Type::Unit,
            "sum" => self.lambda_body_type(params, &elem, by_ref, None, body),
            "count" => Type::Int,
            "any?" | "all?" => Type::Bool,
            "sort_by" => Type::List(Box::new(elem)),
            "find" => Type::Option(Box::new(elem)),
            "min_by" | "max_by" => Type::Option(Box::new(elem)),
            "fold" => match init {
                Some(t) => {
                    let seed = fold_acc_type(t.materialized(), &elem);
                    self.fold_seed_from_body(seed, params, body, &elem, by_ref)
                }
                None => Type::Unknown,
            },
            _ => Type::Unknown,
        }
    }

    // ----- items ------------------------------------------------------------

    fn interface_def(&mut self, i: &InterfaceDef) -> Result<()> {
        let saved_gs = self.push_generics(&i.generics);
        let gen = self.rust_generics(&i.generics);
        let gargs = Self::rust_generic_args(&i.generics);
        // `Debug` as well, because a value held as an interface still ends
        // up inside a struct that derives it.
        self.line(&format!("pub trait {}{}: LumeShow + std::fmt::Debug {{", i.name, gen));
        self.indent += 1;
        let info = self.interfaces[&i.name].clone();
        // A value held as an interface is behind a pointer, and a pointer to
        // a trait cannot be cloned on its own: the value inside makes the
        // copy, through this.
        self.line(&format!("fn lume_box(&self) -> ::std::boxed::Box<dyn {}{}>;", i.name, gargs));
        for m in &i.required {
            let sg = &info.methods[&m.name];
            self.line(&format!("fn {}({}) -> {};", rust_name(&m.name), self.sig_params_rust(sg, true), self.rt(&sg.ret)));
        }
        self.in_trait_impl = true;
        for m in &i.defaults {
            self.fn_def(m, Some(&i.name))?;
        }
        self.in_trait_impl = false;
        self.indent -= 1;
        self.line("}");
        // Boxed values forward to the value inside.
        let box_gen = if i.generics.is_empty() {
            format!("<Inner: {} + ?std::marker::Sized>", i.name)
        } else {
            format!("<{}, Inner: {}{} + ?std::marker::Sized>", i.generics.iter().map(|p| self.rust_bounds(p)).collect::<Vec<_>>().join(", "), i.name, gargs)
        };
        self.line(&format!("impl{} {}{} for ::std::boxed::Box<Inner> {{", box_gen, i.name, gargs));
        self.indent += 1;
        self.line(&format!("fn lume_box(&self) -> ::std::boxed::Box<dyn {}{}> {{ (**self).lume_box() }}", i.name, gargs));
        for (name, sg) in &info.methods {
            let args: Vec<String> = sg.params.iter().map(|(n, _)| rust_name(n)).collect();
            self.line(&format!(
                "fn {}({}) -> {} {{ (**self).{}({}) }}",
                rust_name(name),
                self.sig_params_rust(sg, true),
                self.rt(&sg.ret),
                rust_name(name),
                args.join(", ")
            ));
        }
        self.indent -= 1;
        self.line("}");
        // A list or a field of interface values copies element by element.
        let clone_gen = if i.generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", i.generics.iter().map(|p| self.rust_bounds(p)).collect::<Vec<_>>().join(", "))
        };
        self.line(&format!(
            "impl{} Clone for ::std::boxed::Box<dyn {}{}> {{ fn clone(&self) -> Self {{ (**self).lume_box() }} }}",
            clone_gen, i.name, gargs
        ));
        self.line(&format!(
            "impl{} PartialEq for ::std::boxed::Box<dyn {}{}> {{ fn eq(&self, o: &Self) -> bool {{ self.lume_str() == o.lume_str() }} }}",
            clone_gen, i.name, gargs
        ));
        self.pop_generics(saved_gs);
        Ok(())
    }

    /// `&self, a: &A, b: i64` for a method signature.
    fn sig_params_rust(&self, sg: &Sig, with_self: bool) -> String {
        self.sig_params_rust_as(sg, None, with_self)
    }

    fn sig_params_rust_as(&self, sg: &Sig, decl: Option<&[Type]>, with_self: bool) -> String {
        let mut parts: Vec<String> = Vec::new();
        if with_self {
            parts.push("&self".into());
        }
        for (i, (n, t)) in sg.params.iter().enumerate() {
            let rt = self.rt(t);
            let lent = decl.and_then(|d| d.get(i)).map(|d| matches!(d, Type::Var(_))).unwrap_or(false);
            let rt = if sg.var_params.get(i).copied().unwrap_or(false) {
                format!("&mut {}", rt)
            } else if lent {
                format!("&{}", rt)
            } else if t.is_copy() {
                rt
            } else if *t == Type::Str {
                "&str".into()
            } else {
                format!("&{}", rt)
            };
            parts.push(format!("{}: {}", rust_name(n), rt));
        }
        parts.join(", ")
    }

    /// For every (type, interface) pair this module is responsible for, emit
    /// the trait impl: inherent methods delegate, `extend` methods bring
    /// their bodies, defaults are left to the trait.
    fn emit_conformances(&mut self, program: &[Item]) -> Result<()> {
        // extend blocks by (type key, iface)
        let mut ext_bodies: HashMap<(String, String), Vec<FnDef>> = HashMap::new();
        for item in program {
            if let Item::Extend(x) = item {
                let key = self.ext_key_of(x);
                ext_bodies.entry((key, self.type_key(&self.ct(&x.iface)))).or_default().extend(x.methods.iter().cloned());
            }
        }
        // candidate types: local structs/enums, extend targets, imported types (when the interface is local)
        let mut types: Vec<(String, Type)> = Vec::new();
        for n in &self.local_types {
            // a generic `extend` on this type carries its parameters, so
            // that spelling of it is the one to emit against
            if self.ext_generics.contains_key(n) && self.ext_targets.contains_key(n) {
                continue;
            }
            // a generic type conforms at its own parameters: `Chain[T]`,
            // not the bare name, or the impl would speak of a `T` it never
            // introduced
            let gs = self.generics_of(n);
            let t = if gs.is_empty() { Type::Named(n.clone()) } else { Type::App(n.clone(), gs.iter().map(|g| Type::Var(g.name.clone())).collect()) };
            types.push((n.clone(), t));
        }
        for (k, t) in &self.ext_targets {
            types.push((k.clone(), t.clone()));
        }
        for n in self.structs.keys().chain(self.enums.keys()) {
            if n != "Error" && !self.local_types.contains(n) && n.contains('.') {
                types.push((n.clone(), Type::Named(n.clone())));
            }
        }
        // deterministic output: sort by name
        types.sort_by(|a, b| a.0.cmp(&b.0));
        types.dedup_by(|a, b| a.0 == b.0);
        let mut ifaces: Vec<(String, IfaceInfo)> = self.interfaces.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        ifaces.sort_by(|a, b| a.0.cmp(&b.0));
        let mut done: HashSet<(String, String)> = HashSet::new();
        for (tkey, t) in &types {
            for (iname, info) in &ifaces {
                let type_local = self.local_types.contains(tkey) || self.ext_targets.contains_key(tkey);
                let owns = info.local || (type_local && !tkey.contains('.'));
                let has_extend = ext_bodies.contains_key(&(tkey.clone(), iname.clone()));
                // An `extend` travels with the import, but its impl is
                // written once, in the file that wrote the `extend`.
                if let Some((f, _)) = self.ext_where.get(&(tkey.clone(), iname.clone())) {
                    if *f != self.this_file {
                        continue;
                    }
                }
                if !(owns || has_extend) {
                    continue;
                }
                if !done.insert((tkey.clone(), iname.clone())) {
                    continue;
                }
                // a generic `extend` wrote everything below in terms of the
                // names its target introduced, so they are in scope here
                // an `extend` brings its own parameters; a type that
                // conforms by having the methods brings its own
                let egens = match self.ext_impl_generics.get(&(tkey.clone(), iname.clone())) {
                    Some(g) => g.clone(),
                    None => self.generics_of(tkey),
                };
                let saved_eg = self.push_generics(&egens);
                let r = self.emit_one_conformance(program, tkey, t, iname, info, &ext_bodies, &egens);
                self.pop_generics(saved_eg);
                r?;
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_one_conformance(
        &mut self,
        program: &[Item],
        tkey: &String,
        t: &Type,
        iname: &String,
        info: &IfaceInfo,
        ext_bodies: &HashMap<(String, String), Vec<FnDef>>,
        egens: &[TypeParam],
    ) -> Result<()> {
        {
            {
                // an `extend` may name the interface's arguments; otherwise
                // they are read off the type's own methods
                let written = program.iter().find_map(|it| match it {
                    Item::Extend(x) if self.ext_key_of(x) == *tkey && self.type_key(&self.ct(&x.iface)) == *iname => Some(self.ct(&x.iface)),
                    _ => None,
                });
                let asked = written.clone().unwrap_or_else(|| Type::Named(iname.clone()));
                let (verdict, targs) = self.conformance_with(t, &asked);
                if !matches!(verdict, Conformance::Yes) {
                    if let Some(x) = program.iter().find_map(|it| match it {
                        Item::Extend(x) if self.ext_key_of(x) == *tkey && self.type_key(&self.ct(&x.iface)) == *iname => Some(x),
                        _ => None,
                    }) {
                        // an extend that still leaves methods missing is an error here
                        if let Conformance::Missing(m) = verdict {
                            if !m.is_empty() {
                                let info = &self.interfaces[iname];
                                let needs: Vec<String> = m.iter().filter_map(|n| info.methods.get(n).map(|s| format!("def {} {}", n, self.describe_sig(s)))).collect();
                                return Err(LumeError::new(x.line, x.col, format!("`extend {} with {}` is missing `{}`", type_name(t), type_name(&x.iface), m.join("`, `")))
                                    .with_help(format!("`{}` needs: {}. Add {} inside this `extend` block", type_name(&x.iface), needs.join("; "), if m.len() == 1 { "it" } else { "them" })));
                            }
                            return Err(LumeError::new(x.line, x.col, format!("cannot tell what `{}` stands for in `{}`", info.generics.iter().map(|g| g.name.clone()).collect::<Vec<_>>().join("`, `"), iname))
                                .with_help(format!("name it: `extend {} with {}[{}]:`", type_name(t), iname, type_name(t))));
                        }
                        self.require_conforms(t, &asked, x.line, x.col)?;
                    }
                    return Ok(());
                }
                let iface_rust = if info.generics.is_empty() {
                    self.path_of(iname)
                } else {
                    format!("{}<{}>", self.path_of(iname), targs.iter().map(|a| self.rt(a)).collect::<Vec<_>>().join(", "))
                };
                let isub: HashMap<String, Type> = info.generics.iter().map(|g| g.name.clone()).zip(targs.iter().cloned()).collect();
                let bodies = ext_bodies.get(&(tkey.clone(), iname.clone())).cloned().unwrap_or_default();
                let targets: Vec<String> = match t {
                    Type::Str => vec!["str".into(), "String".into()],
                    other => vec![self.rt(other)],
                };
                // A generic `extend` becomes a generic impl: the names its
                // target introduced are the impl's own parameters, and they
                // inherit what the interface asks of its own — an
                // `interface Sortable[T: Ordered]` means this `T` is Ordered.
                let impl_gen = if egens.is_empty() {
                    String::new()
                } else {
                    let parts: Vec<String> = egens
                        .iter()
                        .map(|g| {
                            let mut bs: Vec<Type> = g.bound.iter().cloned().collect();
                            for (ip, a) in info.generics.iter().zip(&targs) {
                                if let (Some(b), Type::Var(n) | Type::Named(n)) = (&ip.bound, a) {
                                    if *n == g.name && !bs.contains(b) {
                                        bs.push(b.clone());
                                    }
                                }
                            }
                            self.rust_bounds_many(&g.name, &bs)
                        })
                        .collect();
                    format!("<{}>", parts.join(", "))
                };
                for target in targets {
                    self.line(&format!("impl{} {} for {} {{", impl_gen, iface_rust, target));
                    self.indent += 1;
                    // how this type makes the copy a pointer-to-interface needs
                    let boxed_self = if target == "str" { "self.to_string()" } else { "self.clone()" };
                    self.line(&format!("fn lume_box(&self) -> ::std::boxed::Box<dyn {}> {{ ::std::boxed::Box::new({}) }}", iface_rust, boxed_self));
                    let inherent = self.methods_of(tkey).unwrap_or_default();
                    for (mname, sg0) in &info.methods {
                        let sg = &Sig { params: sg0.params.iter().map(|(n, x)| (n.clone(), Self::subst(x, &isub))).collect(), ret: Self::subst(&sg0.ret, &isub), ..sg0.clone() };
                        let decl_tys: Vec<Type> = sg0.params.iter().map(|(_, x)| x.clone()).collect();
                        // the trait lends what it declared as `T`; the method
                        // underneath takes a copied type by value
                        let pass: Vec<String> = sg
                            .params
                            .iter()
                            .enumerate()
                            .map(|(i, (n, t))| {
                                let lent = matches!(decl_tys.get(i), Some(Type::Var(_)));
                                if lent && t.is_copy() { format!("*{}", rust_name(n)) } else { rust_name(n) }
                            })
                            .collect();
                        if let Some(body) = bodies.iter().find(|b| b.name == *mname) {
                            if target == "String" {
                                // delegate to the str impl
                                // this one delegates to the `str` impl of the
                                // same trait, so its arguments travel as the
                                // trait declared them
                                let same: Vec<String> = sg.params.iter().map(|(n, _)| rust_name(n)).collect();
                                self.line(&format!("fn {}({}) -> {} {{ self.as_str().{}({}) }}", rust_name(mname), self.sig_params_rust_as(sg, Some(&decl_tys), true), self.rt(&sg.ret), rust_name(mname), same.join(", ")));
                            } else {
                                self.in_trait_impl = true;
                                self.trait_decl = Some(decl_tys.clone());
                                self.fn_def(body, Some(tkey))?;
                                self.trait_decl = None;
                                self.in_trait_impl = false;
                            }
                        } else if inherent.contains_key(mname) && !self.ext_methods.get(tkey).map(|m| m.contains_key(mname)).unwrap_or(false) {
                            let args: Vec<String> = pass.clone();
                            self.line(&format!(
                                "fn {}({}) -> {} {{ {}::{}(self{}{}) }}",
                                rust_name(mname),
                                self.sig_params_rust_as(sg, Some(&decl_tys), true),
                                self.rt(&sg.ret),
                                Self::turbofish(&self.rt(t)),
                                rust_name(mname),
                                if args.is_empty() { "" } else { ", " },
                                args.join(", ")
                            ));
                        }
                    }
                    self.indent -= 1;
                    self.line("}");
                }
            }
        }
        Ok(())
    }

    /// `#[derive(...)]` for a user type: `PartialEq` is derived unless the
    /// type defines `==` itself.
    fn derive_line(&mut self, name: &str, methods: &[FnDef]) {
        if methods.iter().any(|m| m.name == "==") {
            self.line("#[derive(Debug, Clone)]");
        } else if self.hashable_named(name) {
            // usable as a map key or set item
            self.line("#[derive(Debug, Clone, PartialEq, Eq, Hash)]");
        } else {
            self.line("#[derive(Debug, Clone, PartialEq)]");
        }
    }

    /// `def ==` becomes `PartialEq`, `def <` becomes `PartialOrd`, so the
    /// type works with `contains?`, `sort`, `max`, `min` and `==` on lists.
    fn op_impls(&mut self, name: &str, methods: &[FnDef], gen: &str, args: &str) {
        if methods.iter().any(|m| m.name == "==") {
            self.line(&format!("impl{} PartialEq for {}{} {{ fn eq(&self, o: &Self) -> bool {{ self.op_eq(o) }} }}", gen, name, args));
        }
        if methods.iter().any(|m| m.name == "<") {
            self.line(&format!(
                "impl{} PartialOrd for {}{} {{ fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {{ Some(if self.op_lt(o) {{ std::cmp::Ordering::Less }} else if o.op_lt(self) {{ std::cmp::Ordering::Greater }} else {{ std::cmp::Ordering::Equal }}) }} }}",
                gen, name, args
            ));
        }
    }

    /// `NAME = value` at the top level: a Rust `static` computed on first use.
    /// Inside functions the name reads as a borrowed value of that type.
    fn const_def(&mut self, c: &ConstDef) -> Result<()> {
        self.push_scope();
        let inferred = self.ty_of(&c.value).materialized();
        let t = match &c.ty {
            Some(t) => {
                let ct = self.ct(t);
                self.check_assign(&c.value, &ct, &format!("`{}` is declared `{}`", c.name, type_name(&ct)))?;
                ct
            }
            None => inferred,
        };
        if !type_is_known(&t) {
            self.pop_scope();
            return Err(LumeError::new(c.line, c.col, format!("cannot tell the type of `{}` from `{}` alone", c.name, describe_value(&c.value)))
                .with_help(type_hint(&c.name, &c.value, &t)));
        }
        if matches!(t, Type::Shared(..) | Type::Task(_) | Type::Future(_)) {
            self.pop_scope();
            return Err(LumeError::new(c.line, c.col, format!("a constant cannot hold a `{}`", type_name(&t))));
        }
        let v = self.expr_owned_as(&c.value, &t)?;
        self.pop_scope();
        self.consts.insert(c.name.clone(), t.clone());
        let rt = self.rt(&t);
        let vis = if c.public { "pub " } else { "" };
        self.line(&format!("{}static {}: std::sync::LazyLock<{}> = std::sync::LazyLock::new(|| {});", vis, rust_name(&c.name), rt, v));
        Ok(())
    }

    fn struct_def(&mut self, s: &StructDef) -> Result<()> {
        let saved_gs = self.push_generics(&s.generics);
        let gen = self.rust_generics(&s.generics);
        let gargs = Self::rust_generic_args(&s.generics);
        let shared_fields: Vec<&Param> = s.fields.iter().filter(|f| matches!(self.ct(&f.ty), Type::Shared(_, true))).collect();
        // a crate type in a field: it must be storable (Clone, no borrowed lifetime)
        let mut foreign_no_eq = false;
        for f in &s.fields {
            if let Some(ft) = self.foreign_type(&self.ct(&f.ty)).cloned() {
                if ft.lifetimes > 0 || ft.iter_item.is_some() {
                    return Err(LumeError::new(f.line, f.col, format!("`{}` borrows from something else, so it cannot be kept in a field", type_name(&f.ty)))
                        .with_help("keep what it points at instead (a `Str`, a list) and rebuild it when needed"));
                }
                if !ft.clone {
                    return Err(LumeError::new(f.line, f.col, format!("`{}` cannot be copied, so it cannot be a struct field", type_name(&f.ty)))
                        .with_help("Lume structs are copied when stored or returned; keep this value in a local instead"));
                }
                if !ft.partial_eq {
                    foreign_no_eq = true;
                }
            }
        }
        // a field held as an interface is behind a pointer to a trait, which
        // `derive` cannot compare: it is compared by what it shows
        let iface_fields: Vec<&Param> = s.fields.iter().filter(|f| self.is_interface(&self.ct(&f.ty))).collect();
        let manual_eq = (!shared_fields.is_empty() || foreign_no_eq || !iface_fields.is_empty()) && !s.methods.iter().any(|m| m.name == "==");
        if manual_eq {
            self.line("#[derive(Debug, Clone)]");
        } else {
            self.derive_line(&s.name, &s.methods);
        }
        self.line(&format!("pub struct {}{} {{", s.name, gen));
        self.indent += 1;
        for f in &s.fields {
            let ft = self.rt(&self.ct(&f.ty));
            self.line(&format!("pub {}: {},", rust_name(&f.name), ft));
        }
        self.indent -= 1;
        self.line("}");
        self.line(&format!("impl{} LumeShow for {}{} {{", gen, s.name, gargs));
        self.indent += 1;
        let fmt: Vec<String> = s.fields.iter().map(|f| format!("{}: {{}}", f.name)).collect();
        let args: Vec<String> = s.fields.iter().map(|f| format!("self.{}.lume_in()", rust_name(&f.name))).collect();
        self.line(&format!("fn lume_str(&self) -> String {{ format!(\"{}({})\", {}) }}", s.name, fmt.join(", "), args.join(", ")));
        self.indent -= 1;
        self.line("}");
        if manual_eq {
            // a lock has no `==`: compare what is behind it (the same handle is trivially equal)
            let parts: Vec<String> = s
                .fields
                .iter()
                .map(|f| {
                    let n = rust_name(&f.name);
                    if matches!(self.ct(&f.ty), Type::Shared(_, true)) {
                        format!("(std::sync::Arc::ptr_eq(&self.{n}, &o.{n}) || *self.{n}.lock().unwrap() == *o.{n}.lock().unwrap())", n = n)
                    } else if self.is_interface(&self.ct(&f.ty)) {
                        format!("self.{n}.lume_str() == o.{n}.lume_str()", n = n)
                    } else if self.foreign_type(&self.ct(&f.ty)).map(|ft| !ft.partial_eq).unwrap_or(false) {
                        // a crate type without `==`: it does not take part in the comparison
                        "true".to_string()
                    } else {
                        format!("self.{n} == o.{n}", n = n)
                    }
                })
                .collect();
            self.line(&format!("impl{} PartialEq for {}{} {{ fn eq(&self, o: &Self) -> bool {{ {} }} }}", gen, s.name, gargs, parts.join(" && ")));
        }
        self.op_impls(&s.name, &s.methods, &gen, &gargs);
        if !s.methods.is_empty() {
            self.line(&format!("impl{} {}{} {{", gen, s.name, gargs));
            self.indent += 1;
            for m in &s.methods {
                self.fn_def(m, Some(&s.name))?;
            }
            self.indent -= 1;
            self.line("}");
        }
        self.pop_generics(saved_gs);
        Ok(())
    }

    fn enum_def(&mut self, e: &EnumDef) -> Result<()> {
        let saved_gs = self.push_generics(&e.generics);
        let gen = self.rust_generics(&e.generics);
        let gargs = Self::rust_generic_args(&e.generics);
        self.derive_line(&e.name, &e.methods);
        self.line(&format!("pub enum {}{} {{", e.name, gen));
        self.indent += 1;
        for v in &e.variants {
            if v.fields.is_empty() {
                self.line(&format!("{},", v.name));
            } else {
                let fs: Vec<String> = v
                    .fields
                    .iter()
                    .map(|f| {
                        let t = self.rt(&self.ct(&f.ty));
                        // a variant that holds its own enum: `Node(left: Tree, ...)` — boxed for Rust, invisible in Lume
                        if self.boxed_field(&e.name, &self.ct(&f.ty)) { format!("{}: ::std::boxed::Box<{}>", rust_name(&f.name), t) } else { format!("{}: {}", rust_name(&f.name), t) }
                    })
                    .collect();
                self.line(&format!("{} {{ {} }},", v.name, fs.join(", ")));
            }
        }
        self.indent -= 1;
        self.line("}");
        self.line(&format!("impl{} LumeShow for {}{} {{", gen, e.name, gargs));
        self.indent += 1;
        self.line("fn lume_str(&self) -> String {");
        self.indent += 1;
        self.line("match self {");
        self.indent += 1;
        for v in &e.variants {
            if v.fields.is_empty() {
                self.line(&format!("{}::{} => String::from(\"{}\"),", e.name, v.name, v.name));
            } else {
                let names: Vec<String> = v.fields.iter().map(|f| rust_name(&f.name)).collect();
                let fmt: Vec<String> = v.fields.iter().map(|f| format!("{}: {{}}", f.name)).collect();
                let args: Vec<String> = names.iter().map(|n| format!("{}.lume_in()", n)).collect();
                self.line(&format!(
                    "{}::{} {{ {} }} => format!(\"{}({})\", {}),",
                    e.name,
                    v.name,
                    names.join(", "),
                    v.name,
                    fmt.join(", "),
                    args.join(", ")
                ));
            }
        }
        self.indent -= 1;
        self.line("}");
        self.indent -= 1;
        self.line("}");
        self.indent -= 1;
        self.line("}");
        self.op_impls(&e.name, &e.methods, &gen, &gargs);
        if !e.methods.is_empty() {
            self.line(&format!("impl{} {}{} {{", gen, e.name, gargs));
            self.indent += 1;
            for m in &e.methods {
                self.fn_def(m, Some(&e.name))?;
            }
            self.indent -= 1;
            self.line("}");
        }
        self.pop_generics(saved_gs);
        Ok(())
    }

    fn fn_def(&mut self, f: &FnDef, owner: Option<&String>) -> Result<()> {
        let sig = match owner {
            None => self.fns[&f.name].clone(),
            Some(s) => self.methods_of(s).unwrap()[&f.name].clone(),
        };
        let is_main = f.name == "main" && owner.is_none() && self.is_entry;
        let main_result = is_main && matches!(sig.ret, Type::Result(..));
        // `-> () or E`: the body ends with statements, and finishes with Ok(()).
        let unit_result = matches!(&sig.ret, Type::Result(t, _) if **t == Type::Unit);
        if is_main {
            let ok = f.params.is_empty()
                && match &f.ret {
                    None => true,
                    Some(Type::Unit) => true,
                    Some(Type::Result(t, _)) => **t == Type::Unit,
                    _ => false,
                };
            if !ok {
                return Err(LumeError::new(f.line, f.col, "`main` takes no parameters and returns nothing, or `() or Error`")
                    .with_help("write `def main:` or `def main -> () or Error:`"));
            }
            if !f.generics.is_empty() {
                return Err(LumeError::new(f.line, f.col, "`main` cannot take type parameters")
                    .with_help("nothing calls `main`, so there is nowhere for them to come from"));
            }
        }
        let fn_name = if is_main && self.test_mode {
            "lume_program_main".to_string()
        } else if main_result {
            "lume_main".to_string()
        } else {
            rust_name(&f.name)
        };
        if is_main && self.test_mode {
            self.line("#[allow(dead_code)]");
        }
        let own_generics = self.scope_generics(f, owner);
        let saved_gs = self.push_generics(&own_generics);
        // the owning type's parameters belong to the `impl` block, not here
        let mut generics: Vec<String> = f.generics.iter().map(|p| self.rust_bounds(p)).collect();
        let mut parts: Vec<String> = Vec::new();
        if owner.is_some() {
            parts.push(match f.self_kind {
                SelfKind::Read => "&self".into(),
                SelfKind::Mutate => "&mut self".into(),
            });
        }
        let mut names = HashSet::new();
        let saved_lent = std::mem::take(&mut self.lent_names);
        let mut lent_params: HashMap<String, bool> = HashMap::new();
        for p in &f.params {
            if !names.insert(p.name.clone()) {
                return Err(LumeError::new(p.line, p.col, format!("parameter `{}` is listed twice", p.name)));
            }
            let pty = self.ct(&p.ty);
            if self.is_interface(&pty) && !self.in_trait_impl {
                // `s: Shape` — one generic parameter per interface-typed parameter (static dispatch)
                let g = format!("Iface{}", generics.len());
                let iface = self.rust_iface(&pty);
                generics.push(format!("{}: {}", g, iface));
                parts.push(format!("{}: &{}", rust_name(&p.name), g));
                continue;
            }
            if let Type::Fn(ins, out) = &pty {
                // `f: (T) -> U` — the block is its own generic parameter, so
                // the caller's closure is compiled straight into this function
                let g = format!("Blk{}", generics.len());
                generics.push(format!("{}: {}", g, self.rust_fn_bound(ins, out)));
                parts.push(format!("mut {}: {}", rust_name(&p.name), g));
                continue;
            }
            let rt = self.rt(&pty);
            let lent = self.trait_decl.as_ref().and_then(|d| d.get(parts.len() - usize::from(owner.is_some()))).map(|d| matches!(d, Type::Var(_))).unwrap_or(false);
            lent_params.insert(p.name.clone(), lent);
            if lent && pty.is_copy() {
                self.lent_names.insert(p.name.clone());
            }
            let rt = if p.mutable {
                format!("&mut {}", rt)
            } else if lent {
                format!("&{}", rt)
            } else if pty.is_copy() {
                rt
            } else if pty == Type::Str {
                "&str".to_string()
            } else {
                format!("&{}", rt)
            };
            parts.push(format!("{}: {}", rust_name(&p.name), rt));
        }
        let vis = if self.in_trait_impl { "" } else { "pub " };
        let gen = if generics.is_empty() { String::new() } else { format!("<{}>", generics.join(", ")) };
        if f.is_async {
            self.uses_async = true;
            if is_main && !main_result && !self.test_mode {
                self.line("#[tokio::main]");
            }
        }
        let saved_async = self.in_async;
        self.in_async = f.is_async;
        let mut header = format!("{}{}fn {}{}({})", vis, if f.is_async { "async " } else { "" }, fn_name, gen, parts.join(", "));
        if sig.ret != Type::Unit {
            header.push_str(&format!(" -> {}", self.rt(&sig.ret)));
        }
        header.push_str(" {");
        self.line(&header);
        self.indent += 1;
        if is_main && !self.test_mode {
            self.line("lume_install_panic_hook();");
        }
        self.push_scope();
        for p in &f.params {
            let pty = self.ct(&p.ty);
            let borrowed = !pty.is_copy() || lent_params.get(&p.name).copied().unwrap_or(false);
            if self.is_interface(&pty) {
                self.iface_params.insert(p.name.clone());
            }
            self.declare(&p.name, p.mutable, borrowed, pty, p.line);
        }
        self.current_ret = sig.ret.clone();
        self.current_type = owner.cloned();
        self.current_self_ty = owner.and_then(|o| self.ext_targets.get(o).cloned());
        self.current_self = f.self_kind;
        self.current_fn = f.name.clone();
        let want_value = sig.ret != Type::Unit;
        self.tail_of_fn = true;
        if unit_result {
            self.block_body(&f.body, false)?;
            self.line("Ok(())");
        } else {
            self.block_body(&f.body, want_value)?;
        }
        self.tail_of_fn = false;
        self.in_async = saved_async;
        self.pop_scope();
        self.pop_generics(saved_gs);
        self.lent_names = saved_lent;
        self.current_type = None;
        self.current_self_ty = None;
        self.indent -= 1;
        self.line("}");
        if main_result && !self.test_mode {
            if f.is_async {
                self.line("#[tokio::main]");
                self.line("async fn main() {");
                self.line("    if let Err(e) = lume_main().await { eprintln!(\"error: {}\", e.message); std::process::exit(1); }");
            } else {
                self.line("fn main() {");
                self.line("    if let Err(e) = lume_main() { eprintln!(\"error: {}\", e.message); std::process::exit(1); }");
            }
            self.line("}");
        }
        Ok(())
    }

    /// A `test "name":` block becomes `pub fn lume_test_N()`; the runner in
    /// the entry file calls it under `catch_unwind`.
    fn test_def(&mut self, t: &TestDef, index: usize) -> Result<()> {
        self.line(&format!("pub fn lume_test_{}() {{", index));
        self.indent += 1;
        self.push_scope();
        self.current_fn = format!("test \"{}\"", t.name);
        self.current_ret = Type::Unit;
        self.current_self = SelfKind::Read;
        self.in_test = true;
        self.tail_of_fn = false;
        self.block_body(&t.body, false)?;
        self.in_test = false;
        self.pop_scope();
        self.indent -= 1;
        self.line("}");
        Ok(())
    }

    /// The source text of an `assert`, for its failure message.
    fn assert_text(&self, line: usize, col: usize) -> String {
        let l = match self.src_lines.get(line.wrapping_sub(1)) {
            Some(l) => l,
            None => return String::new(),
        };
        let rest: String = l.chars().skip(col - 1).collect();
        let rest = rest.trim_start_matches("assert").trim();
        // drop a trailing comment, honouring quotes
        let mut out = String::new();
        let mut in_str = false;
        let mut prev = ' ';
        for ch in rest.chars() {
            if ch == '"' && prev != '\\' {
                in_str = !in_str;
            }
            if ch == '#' && !in_str && prev.is_whitespace() {
                break;
            }
            out.push(ch);
            prev = ch;
        }
        out.trim().to_string()
    }

    /// In a function returning `T or E`, a result value of type `T` is
    /// wrapped in `Ok`, and one of type `E` in `Err`.
    fn coerce_result(&mut self, text: String, e: &Expr) -> Result<String> {
        // A block declared `(A) -> B?` implies its `Some` the same way one
        // declared `(A) -> B or E` implies its `Ok`.
        if !self.tail_of_fn || (self.in_block && !matches!(self.block_ret, Some(Type::Result(..)) | Some(Type::Option(_)))) {
            return Ok(text);
        }
        if matches!(e.kind, ExprKind::Rust(_)) {
            return Ok(text);
        }
        let ret = self.current_ret.clone();
        if ret != Type::Unknown && ret != Type::Unit {
            let what = if self.current_fn.starts_with("test ") {
                format!("{} produces nothing", self.current_fn)
            } else if self.current_fn == "this block" {
                format!("this block gives back a `{}`", type_name(&ret))
            } else {
                format!("`{}` returns `{}`", self.current_fn, type_name(&ret))
            };
            // a bare error value in a `T or E` function is the implied `Err`
            let have = self.ty_of(e).materialized();
            let is_err_value = matches!(&ret, Type::Result(_, err_t) if have == **err_t);
            // the value part is still open: this value is what fills it
            let open_ok = matches!(&ret, Type::Result(ok, _) if matches!(**ok, Type::Var(_) | Type::Unknown));
            if !is_err_value && !open_ok {
                self.check_assign(e, &ret, &what)?;
            }
        }
        if self.is_interface(&ret) || ret == Type::Str {
            let et = self.ty_of(e).materialized();
            return self.coerce(text, &et, &ret, e.line, e.col);
        }
        // a bare value where a `T?` is returned is `Some(value)`
        if let Type::Option(inner) = &ret {
            let et = self.ty_of(e).materialized();
            if !matches!(et, Type::Option(_)) && et != Type::Unknown && !matches!(e.kind, ExprKind::None) && (et == **inner || **inner == Type::Unknown) {
                return Ok(format!("Some({})", text));
            }
            return Ok(text);
        }
        let (ok_t, err_t) = match &ret {
            Type::Result(t, e) => ((**t).clone(), (**e).clone()),
            _ => return Ok(text),
        };
        let et = self.ty_of(e).materialized();
        if matches!(et, Type::Result(..)) {
            return Ok(text);
        }
        if et == err_t && et != ok_t {
            return Ok(format!("Err({})", text));
        }
        Ok(format!("Ok({})", text))
    }

    fn block_body(&mut self, b: &Block, want_value: bool) -> Result<()> {
        let n = b.stmts.len();
        if want_value && n == 0 {
            return Err(LumeError::new(1, 1, "a block that produces a value cannot be empty"));
        }
        for (i, s) in b.stmts.iter().enumerate() {
            let last = i + 1 == n;
            self.rest_stack.push(b.stmts[i + 1..].to_vec());
            let r = self.stmt(s, want_value && last);
            self.rest_stack.pop();
            r?;
        }
        Ok(())
    }

    /// Liveness: is the local `name` used again after the statement being
    /// emitted? True when a later statement of this block or of an enclosing
    /// block mentions it, or when it was bound outside a loop or block body
    /// that is being emitted (the next iteration or call uses it again).
    fn used_after(&self, name: &str) -> bool {
        let idx = match self.scopes.iter().rposition(|sc| sc.contains_key(name)) {
            Some(i) => i,
            None => return true,
        };
        if self.barriers.iter().any(|b| idx < *b) {
            return true;
        }
        self.rest_stack.iter().any(|rest| rest.iter().any(|st| stmt_uses(st, name)))
    }

    /// Emits one argument of a call while the arguments still to come count
    /// as later uses: `Heading(level: n, title: t, slug: slug(t))` must copy
    /// `t` into the field rather than move it, because `slug(t)` follows.
    fn arg_with_rest(&mut self, a: &Expr, want: &Type, rest: &[&Expr]) -> Result<String> {
        let later: Vec<Stmt> = rest.iter().map(|x| Stmt::Expr((*x).clone())).collect();
        self.rest_stack.push(later);
        let r = self.expr_owned_as(a, want);
        self.rest_stack.pop();
        r
    }

    /// True when a name bound in the current block is not used by any later
    /// statement, so its value may be moved instead of cloned.
    fn dead_after_this(&self, name: &str) -> bool {
        self.lookup(name).is_some() && !self.used_after(name)
    }

    fn nested_block(&mut self, b: &Block, want_value: bool) -> Result<()> {
        self.indent += 1;
        self.push_scope();
        self.at_tail = false;
        self.block_body(b, want_value)?;
        self.pop_scope();
        self.indent -= 1;
        Ok(())
    }

    // ----- statements -------------------------------------------------------

    fn stmt(&mut self, s: &Stmt, is_tail: bool) -> Result<()> {
        // A `shared var` place on the left: do the change inside one lock.
        match s {
            Stmt::FieldAssign { recv, value, .. } | Stmt::IndexAssign { recv, value, .. } => {
                if let Some((root, inner)) = self.shared_root(recv) {
                    let root = root.clone();
                    return self.shared_stmt(s, &root, inner, value, is_tail);
                }
            }
            _ => {}
        }
        match s {
            Stmt::Shared { name, mutable, ty, value, line, col } => {
                if let Some(t) = ty {
                    self.check_type(&self.ct(t), *line, *col)?;
                    Self::no_fn_outside_params(&self.ct(t), "a binding", *line, *col)?;
                }
                let vt0 = self.ty_of(value).materialized();
                self.no_future(&vt0, value)?;
                if let Type::Shared(..) = vt0 {
                    return Err(LumeError::new(*line, *col, format!("`{}` is already shared", describe_value(value)))
                        .with_help(format!("another handle to the same value is just `{} = {}`", name, describe_value(value))));
                }
                let inner = ty.as_ref().map(|t| self.ct(t)).unwrap_or_else(|| vt0.clone());
                if ty.is_none() && !type_is_known(&inner) {
                    // a private or missing module item explains the unknown type better
                    self.module_ref(value)?;
                    return Err(LumeError::new(*line, *col, format!("cannot tell the type of `{}` from `{}` alone", name, describe_value(value)))
                        .with_help(type_hint(&format!("shared var {}", name), value, &inner)));
                }
                if self.scopes.last().unwrap().contains_key(name) {
                    return Err(LumeError::new(*line, *col, format!("`{}` is already declared in this block", name)));
                }
                let v = self.expr_owned_as(value, &inner)?;
                let st = Type::Shared(Box::new(inner.clone()), *mutable);
                if *mutable {
                    self.warnings.push(
                        LumeError::new(*line, *col, format!("`{}` is `shared var`: every use of it takes a lock, so tasks change it one at a time", name))
                            .with_help("fine for a store or a counter; keep the work done while it is locked short"),
                    );
                }
                self.declare(name, false, false, st, *line);
                let wrapped = if *mutable { format!("std::sync::Arc::new(std::sync::Mutex::new({}))", v) } else { format!("std::sync::Arc::new({})", v) };
                self.line(&format!("let {} = {};", rust_name(name), wrapped));
                if is_tail {
                    return self.tail_unit(*line, *col);
                }
            }
            Stmt::Var { name, ty, value, line, col } => {
                if let Some(t) = ty {
                    self.check_type(&self.ct(t), *line, *col)?;
                    Self::no_fn_outside_params(&self.ct(t), "a binding", *line, *col)?;
                    let ct = self.ct(t);
                    self.check_assign(value, &ct, &format!("`{}` is declared `{}`", name, type_name(&ct)))?;
                }
                let inferred = self.ty_of(value).materialized();
                self.no_future(&inferred, value)?;
                let vt = ty.as_ref().map(|t| self.ct(t)).unwrap_or_else(|| inferred.clone());
                if ty.is_none() && !type_is_known(&inferred) {
                    // a private or missing module item explains the unknown type better
                    self.module_ref(value)?;
                    return Err(LumeError::new(*line, *col, format!("cannot tell the type of `{}` from `{}` alone", name, describe_value(value)))
                        .with_help(type_hint(&format!("var {}", name), value, &inferred)));
                }
                let v = self.expr_owned_as(value, &vt)?;
                if ty.is_some() {
                    self.note_dynamic(name, &vt, *line, *col);
                }
                if self.scopes.last().unwrap().contains_key(name) {
                    return Err(LumeError::new(*line, *col, format!("`{}` is already declared in this block", name))
                        .with_help(format!("to change it, write `{} = ...`", name)));
                }
                self.declare(name, true, false, vt.clone(), *line);
                let ann = if ty.is_some() { format!(": {}", self.rt(&vt)) } else { String::new() };
                self.line(&format!("let mut {}{} = {};", rust_name(name), ann, v));
                if is_tail {
                    return self.tail_unit(*line, *col);
                }
            }
            Stmt::Destructure { names, value, line, col } => {
                let vt = self.ty_of(value).materialized();
                let ts = match &vt {
                    Type::Tuple(ts) => ts.clone(),
                    Type::Unknown => vec![Type::Unknown; names.len()],
                    other => {
                        return Err(LumeError::new(value.line, value.col, format!("`({}) = ...` takes a tuple apart, but this is a `{}`", names.join(", "), type_name(other))));
                    }
                };
                if ts.len() != names.len() {
                    return Err(LumeError::new(*line, *col, format!("this tuple has {} parts, but {} {} named", ts.len(), names.len(), plural(names.len(), "is", "are")))
                        .with_help("name every part; use `_` for a part you do not need"));
                }
                let mut seen = HashSet::new();
                for n in names {
                    if n == "_" {
                        continue;
                    }
                    if !seen.insert(n.clone()) {
                        return Err(LumeError::new(*line, *col, format!("`{}` is named twice", n)));
                    }
                    if let Some(b) = self.lookup(n) {
                        return Err(LumeError::new(*line, *col, format!("`{}` already exists (line {}); `(a, b) = ...` makes new names", n, b.line))
                            .with_help("pick new names, or assign the parts one by one: `x = pair.0`"));
                    }
                }
                let v = self.expr_owned(value)?;
                let pat: Vec<String> = names.iter().map(|n| if n == "_" { "_".to_string() } else { rust_name(n) }).collect();
                for (n, t) in names.iter().zip(ts) {
                    if n != "_" {
                        self.declare(n, false, false, t, *line);
                    }
                }
                self.line(&format!("let ({}) = {};", pat.join(", "), v));
                if is_tail {
                    return self.tail_unit(*line, *col);
                }
            }
            Stmt::Bind { name, ty, value, line, col } => {
                let vt0 = self.ty_of(value);
                self.no_future(&vt0, value)?;
                if let Some(t) = ty {
                    let ct = self.ct(t);
                    self.check_type(&ct, *line, *col)?;
                    self.check_assign(value, &ct, &format!("`{}` is declared `{}`", name, type_name(&ct)))?;
                } else if let Some(b) = self.lookup(name).cloned() {
                    if b.mutable && leftmost_ident(value) != Some(name.as_str()) {
                        self.check_assign(value, &b.ty, &format!("`{}` is a `{}`", name, type_name(&b.ty)))?;
                    } else if b.mutable {
                        self.check_assign(value, &b.ty, &format!("`{}` is a `{}`", name, type_name(&b.ty)))?;
                    }
                }
                if self.spawn_captured.contains(name) && self.lookup(name).map(|b| !b.mutable).unwrap_or(false) {
                    return Err(LumeError::new(*line, *col, format!("`{}` inside `spawn:` is a copy, so changing it here would not be seen outside", name))
                        .with_help(format!("declare it `shared var {}` before the `spawn:` if tasks are meant to change it, or bind a new name here", name)));
                }
                if let Some(t) = ty {
                    self.check_type(&self.ct(t), *line, *col)?;
                    Self::no_fn_outside_params(&self.ct(t), "a binding", *line, *col)?;
                    if self.lookup(name).is_some() {
                        return Err(LumeError::new(*line, *col, format!("`{}` already exists; a type goes only on a new binding", name)));
                    }
                }
                let inferred = self.ty_of(value).materialized();
                let vt = ty.as_ref().map(|t| self.ct(t)).unwrap_or_else(|| inferred.clone());
                if ty.is_none() && self.lookup(name).is_none() && self.field_type(name).is_none() && !type_is_known(&inferred) {
                    // a private or missing module item explains the unknown type better
                    self.module_ref(value)?;
                    return Err(LumeError::new(*line, *col, format!("cannot tell the type of `{}` from `{}` alone", name, describe_value(value)))
                        .with_help(type_hint(name, value, &inferred)));
                }
                let ann = if ty.is_some() { format!(": {}", self.rt(&vt)) } else { String::new() };
                match self.lookup(name).cloned() {
                    Some(Binding { ty: Type::Shared(_, true), .. }) => {
                        // replace the value behind the lock; the new value is computed first
                        let v = self.expr_owned(value)?;
                        self.line(&format!("{{ let lume_v = {}; *{}.lock().unwrap() = lume_v; }}", v, rust_name(name)));
                    }
                    Some(Binding { ty: Type::Shared(_, false), line: bl, .. }) => {
                        return Err(LumeError::new(*line, *col, format!("`{}` is `shared` and read-only", name))
                            .with_help(format!("declare it `shared var {}` on line {} if tasks change it", name, bl)));
                    }
                    Some(b) if b.mutable => {
                        let v = self.expr_owned(value)?;
                        self.line(&format!("{} = {};", rust_name(name), v));
                    }
                    Some(b) => {
                        let same_scope = self.scopes.last().map(|s| s.contains_key(name)).unwrap_or(false);
                        if leftmost_ident(value) == Some(name.as_str()) && !same_scope {
                            // `total = total + i` inside a loop or branch: a new binding here
                            // would hide the outer one and vanish at the end of the block
                            return Err(LumeError::new(*line, *col, format!("`{}` is declared on line {}, outside this block; a new `{}` here would hide it and be lost when the block ends", name, b.line, name))
                                .with_help(format!("to change it from here, declare it `var {}` on line {}; to make a separate value, give it another name", name, b.line)));
                        }
                        if leftmost_ident(value) == Some(name.as_str()) {
                            let v = self.expr_owned(value)?;
                            self.declare(name, false, false, vt, *line);
                            self.line(&format!("let {} = {};", rust_name(name), v));
                        } else {
                            return Err(LumeError::new(*line, *col, format!("`{}` is immutable and cannot be reassigned", name))
                                .with_help(format!(
                                    "declare it with `var {} = ...` on line {} if it needs to change; `{} = {}.something` is allowed as a transform of the same value",
                                    name, b.line, name, name
                                )));
                        }
                    }
                    None => {
                        if self.field_type(name).is_some() {
                            self.require_var_self(name, *line, *col)?;
                            let v = self.expr_owned(value)?;
                            self.line(&format!("self.{} = {};", rust_name(name), v));
                        } else {
                            let v = self.expr_owned_as(value, &vt)?;
                            if ty.is_some() {
                                self.note_dynamic(name, &vt, *line, *col);
                            }
                            self.declare(name, false, false, vt, *line);
                            self.line(&format!("let {}{} = {};", rust_name(name), ann, v));
                        }
                    }
                }
                if is_tail {
                    return self.tail_unit(*line, *col);
                }
            }
            Stmt::OpAssign { name, op, value, line, col } => {
                if let Some(b) = self.lookup(name).cloned() {
                    let bt = b.ty.materialized();
                    if bt != Type::Unknown {
                        self.check_operands(*op, &bt, value, *line, *col)?;
                    }
                }
                let v = self.expr(value)?;
                let is_str = matches!(self.lookup(name).map(|b| b.ty.materialized()), Some(Type::Str));
                let rhs_char = self.ty_of(value).materialized() == Type::Char;
                // `text += x`: append in place, whatever form `x` has
                if is_str && *op == "+=" && rhs_char && matches!(self.lookup(name), Some(b) if b.mutable && !matches!(b.ty, Type::Shared(..))) {
                    self.line(&format!("{}.push({});", rust_name(name), v));
                    if is_tail {
                        return self.tail_unit(*line, *col);
                    }
                    return Ok(());
                }
                let v = if is_str && *op == "+=" && rhs_char { format!("&({}).to_string()", v) } else if is_str && *op == "+=" { format!("&({})", v) } else { v };
                match self.lookup(name).cloned() {
                    Some(Binding { ty: Type::Shared(_, true), .. }) => {
                        self.line(&format!("{{ let lume_v = {}; *{}.lock().unwrap() {} lume_v; }}", v, rust_name(name), op));
                    }
                    Some(Binding { ty: Type::Shared(_, false), line: bl, .. }) => {
                        return Err(LumeError::new(*line, *col, format!("`{}` is `shared` and read-only", name))
                            .with_help(format!("declare it `shared var {}` on line {} if tasks change it", name, bl)));
                    }
                    Some(b) if b.mutable => {
                        self.line(&format!("{} {} {};", rust_name(name), op, v));
                    }
                    Some(b) => {
                        return Err(LumeError::new(*line, *col, format!("`{}` is immutable and cannot be changed with `{}`", name, op))
                            .with_help(format!("declare it with `var {} = ...` on line {}", name, b.line)));
                    }
                    None => {
                        if self.field_type(name).is_some() {
                            self.require_var_self(name, *line, *col)?;
                            self.line(&format!("self.{} {} {};", rust_name(name), op, v));
                        } else {
                            return Err(self.unknown_name(name, *line, *col));
                        }
                    }
                }
                if is_tail {
                    return self.tail_unit(*line, *col);
                }
            }
            Stmt::IndexAssign { recv, index, op, value, line, col } => {
                // `grid[i][j] = v`: the row is reached in place, not as a `T?`
                let rt = match self.indexed_collection(recv) {
                    Some(el) => el,
                    None => self.ty_of(recv).materialized(),
                };
                match &rt {
                    Type::List(_) | Type::Map(..) => {}
                    Type::Unknown => {}
                    other => return Err(LumeError::new(*line, *col, format!("`{}` values cannot be indexed", type_name(other)))),
                }
                let target = Expr::new(ExprKind::Index { recv: Box::new(recv.clone()), index: Box::new(index.clone()) }, *line, *col);
                if let (Type::Map(k_ty, v_ty), None) = (&rt, op) {
                    // `m[k] = m[k].or(d) op x`: one lookup through the entry API.
                    if let ExprKind::Binary { op: bop, lhs, rhs } = &value.kind {
                        if matches!(*bop, "+" | "-" | "*" | "/" | "%") {
                            if let ExprKind::Method { recv: orecv, name: oname, args: oargs } = &lhs.kind {
                                if oname == "or" && oargs.len() == 1 {
                                    if let ExprKind::Index { recv: irecv, index: iidx } = &orecv.kind {
                                        if same_expr(irecv, recv) && same_expr(iidx, index) {
                                            let place = self.mutable_place(recv, "this map", *line, *col)?;
                                            // The map only keeps a key when it actually stores one,
                                            // so the key is lent and copied on a miss. Over repeating
                                            // words that is one copy per distinct word, not per word.
                                            let (entry, k) = match &**k_ty {
                                                Type::Str => ("entry_or_insert_ref", format!("({}).lume_as_str()", self.expr(index)?)),
                                                _ => ("entry_or_insert", self.expr_owned(index)?),
                                            };
                                            let d = self.expr_owned(&oargs[0].value)?;
                                            let x = self.expr_val(rhs)?;
                                            let tmp = self.fresh("e");
                                            // a list grows by extending; Rust has no `+` for it
                                            let step = match (&**v_ty, *bop) {
                                                (Type::List(_), "+") | (Type::Iter(..), "+") => format!("{}.extend(({}).iter().cloned());", tmp, x),
                                                (Type::Str, "+") => format!("{}.push_str(({}).lume_as_str());", tmp, x),
                                                _ => format!("*{} = *{} {} {};", tmp, tmp, bop, x),
                                            };
                                            self.line(&format!("{{ let {} = {}.{}({}, {}); {} }}", tmp, place, entry, k, d, step));
                                            if is_tail {
                                                return self.tail_unit(*line, *col);
                                            }
                                            return Ok(());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // insert or replace: the value is computed first (it may read the
                    // same key), and a non-Copy key is cloned so it stays usable after.
                    let place = self.mutable_place(recv, "this map", *line, *col)?;
                    let k = self.expr_val(index)?;
                    let movable = matches!(&index.kind, ExprKind::Ident(n) if self.dead_after_this(n) && !self.is_borrowed_ident(index));
                    let k = if k_ty.is_copy() || matches!(index.kind, ExprKind::Str(_) | ExprKind::Int(_)) || movable { k } else if **k_ty == Type::Str { format!("({}).to_string()", k) } else { format!("({}).clone()", k) };
                    let v = self.expr_owned(value)?;
                    let tmp = self.fresh("v");
                    self.line(&format!("{{ let {} = {}; {}.insert({}, {}); }}", tmp, v, place, k, tmp));
                } else {
                    let place = self.mutable_place(&target, "this position", *line, *col)?;
                    let v = if op.is_some() { self.expr(value)? } else { self.expr_owned(value)? };
                    self.line(&format!("{} {} {};", place, op.unwrap_or("="), v));
                }
                if is_tail {
                    return self.tail_unit(*line, *col);
                }
            }
            Stmt::FieldAssign { recv, field, op, value, line, col } if matches!(recv.kind, ExprKind::Index { .. }) => {
                let rt = self.ty_of(recv).materialized();
                let inner = match &rt {
                    Type::Option(i) => (**i).clone(),
                    other => other.clone(),
                };
                let sname = match &inner {
                    Type::Named(s) if self.structs.contains_key(s) => s.clone(),
                    other => return Err(LumeError::new(*line, *col, format!("`{}` values have no fields to assign", type_name(other)))),
                };
                if !self.structs[&sname].fields.iter().any(|(n, _)| n == field) {
                    return Err(self.no_such_member(&sname, field, *line, *col));
                }
                let place = self.mutable_place(recv, &format!("the field `{}`", field), *line, *col)?;
                let v = if op.is_some() { self.expr(value)? } else { self.expr_owned(value)? };
                self.line(&format!("{}.{} {} {};", place, rust_name(field), op.unwrap_or("="), v));
                if is_tail {
                    return self.tail_unit(*line, *col);
                }
            }
            Stmt::FieldAssign { recv, field, op, value, line, col } => {
                let rt = self.ty_of(recv);
                let sname = match &rt {
                    Type::Named(s) if self.structs.contains_key(s) => s.clone(),
                    Type::Named(s) => return Err(LumeError::new(*line, *col, format!("`{}` is an enum; its values have no assignable fields", s))),
                    Type::Unknown => return Err(LumeError::new(*line, *col, format!("cannot tell what `.{}` belongs to here", field))),
                    other => return Err(LumeError::new(*line, *col, format!("`{}` values have no fields to assign", type_name(other)))),
                };
                let info = self.structs[&sname].clone();
                if !info.fields.iter().any(|(n, _)| n == field) {
                    return Err(self.no_such_member(&sname, field, *line, *col));
                }
                match &recv.kind {
                    ExprKind::SelfRef => self.require_var_self(field, *line, *col)?,
                    ExprKind::Ident(n) => match self.lookup(n).cloned() {
                        Some(b) if b.mutable => {}
                        Some(b) => {
                            return Err(LumeError::new(*line, *col, format!("`{}` is immutable, so its field `{}` cannot be changed", n, field))
                                .with_help(format!("declare it with `var {} = ...` on line {}", n, b.line)));
                        }
                        None => {
                            if self.field_type(n).is_some() {
                                self.require_var_self(n, *line, *col)?;
                            } else {
                                return Err(self.unknown_name(n, *line, *col));
                            }
                        }
                    },
                    _ => {
                        return Err(LumeError::new(*line, *col, "only a named value's field can be assigned").with_help("bind the value to a `var` first"));
                    }
                }
                let r = self.expr(recv)?;
                let v = if op.is_some() { self.expr(value)? } else { self.expr_owned(value)? };
                self.line(&format!("{}.{} {} {};", r, rust_name(field), op.unwrap_or("="), v));
                if is_tail {
                    return self.tail_unit(*line, *col);
                }
            }
            Stmt::Expr(e) => {
                if is_tail {
                    if let ExprKind::If { else_block: None, .. } = &e.kind {
                        return Err(LumeError::new(e.line, e.col, "this `if` is the function's result but has no `else`")
                            .with_help("add an `else` branch, or add `return` before it if it is not the result"));
                    }
                    // the function's result type says which enum a bare
                    // variant name means
                    self.want.push(self.current_ret.clone());
                    let v = match &e.kind {
                        ExprKind::If { .. } | ExprKind::Match { .. } => {
                            self.at_tail = self.tail_of_fn;
                            self.expr_owned(e)
                        }
                        _ => self.expr_owned(e).and_then(|v| self.coerce_result(v, e)),
                    };
                    self.want.pop();
                    let v = v?;
                    self.line(&v);
                } else {
                    let et0 = self.ty_of(e);
                    self.no_future(&et0, e)?;
                    // In a `() or E` function a bare error value is an early return.
                    if !self.in_block {
                        if let Type::Result(t, err_t) = self.current_ret.clone() {
                            if *t == Type::Unit {
                                let et = self.ty_of(e).materialized();
                                if et == *err_t && et != Type::Unknown {
                                    let v = self.expr_owned(e)?;
                                    self.line(&format!("return Err({});", v));
                                    return Ok(());
                                }
                            }
                        }
                    }
                    self.check_not_dropped(e, &et0)?;
                    let v = self.expr_stmt(e)?;
                    self.line(&format!("{};", v));
                }
            }
            Stmt::Return { value, line, col } => {
                if self.in_block {
                    self.warnings.push(
                        LumeError::new(*line, *col, "`return` inside a block ends this item's block, not the function")
                            .with_help("the last expression is already the block's value; to stop early, use `find`, `take_while` or a `for` loop"),
                    );
                    match value {
                        Some(e) => {
                            let v = self.expr_owned(e)?;
                            self.line(&format!("return {};", v));
                        }
                        None => self.line("return;"),
                    }
                    return Ok(());
                }
                match value {
                    Some(e) => {
                        if self.current_ret == Type::Unit {
                            return Err(LumeError::new(*line, *col, "this function returns nothing, but `return` has a value")
                                .with_help("add `-> Type` to the function signature"));
                        }
                        // `return x` is a tail: the branches of an `if`/`match`
                        // in it agree the same way the last expression's do
                        let saved = self.tail_of_fn;
                        self.tail_of_fn = true;
                        // an `if`/`match` coerces its own branches, like a tail one
                        let branching = matches!(e.kind, ExprKind::If { .. } | ExprKind::Match { .. });
                        self.at_tail = branching;
                        self.want.push(self.current_ret.clone());
                        let v = if branching { self.expr_owned(e) } else { self.expr_owned(e).and_then(|v| self.coerce_result(v, e)) };
                        self.want.pop();
                        self.tail_of_fn = saved;
                        let v = v?;
                        self.line(&format!("return {};", v));
                    }
                    None => {
                        match &self.current_ret {
                            Type::Unit => self.line("return;"),
                            Type::Result(t, _) if **t == Type::Unit => self.line("return Ok(());"),
                            other => {
                                return Err(LumeError::new(*line, *col, format!("this function returns `{}`, so `return` needs a value", type_name(other))));
                            }
                        }
                    }
                }
            }
            Stmt::While { cond, body } => {
                let c = self.expr(cond)?;
                if matches!(cond.kind, ExprKind::Bool(true)) {
                    self.line("loop {");
                } else {
                    self.line(&format!("while {} {{", c));
                }
                self.loop_depth += 1;
                self.barriers.push(self.scopes.len());
                let r = self.nested_block(body, false);
                self.barriers.pop();
                self.loop_depth -= 1;
                r?;
                self.line("}");
            }
            Stmt::For { vars, mutable, iter, filter, body, line, col } => {
                let it_ty = self.ty_of(iter);
                if *mutable {
                    // `for var a in xs`: each element is changed in place
                    let elem = match it_ty.materialized() {
                        Type::List(e) => *e,
                        other => {
                            return Err(LumeError::new(iter.line, iter.col, format!("`for var` walks a list and changes its items, but this is a `{}`", type_name(&other))));
                        }
                    };
                    if elem.is_copy() {
                        return Err(LumeError::new(*line, *col, format!("`for var` over `{}` values would change copies", type_name(&elem)))
                            .with_help("build the new list instead: `xs = xs.map { |x| x + 1 }.to_list`"));
                    }
                    if !self.is_place(iter) {
                        return Err(LumeError::new(iter.line, iter.col, format!("`for var` changes items where they are stored, but `{}` is a copy", snippet(iter)))
                            .with_help("loop over the stored list itself (`for var p in xs`), and use `where` or `if` to pick the items to change"));
                    }
                    self.check_receiver_mutable(iter, "for var", iter.line, iter.col)?;
                    let ex = self.expr(iter)?;
                    self.line(&format!("for {} in ({}).iter_mut() {{", rust_name(&vars[0]), ex));
                    self.indent += 1;
                    self.push_scope();
                    self.declare(&vars[0], true, true, elem, *line);
                    if let Some(f) = filter {
                        let fc = self.expr(f)?;
                        self.line(&format!("if !({}) {{ continue; }}", fc));
                    }
                    self.loop_depth += 1;
                    self.barriers.push(self.scopes.len() - 1);
                    let r = self.block_body(body, false);
                    self.barriers.pop();
                    self.loop_depth -= 1;
                    r?;
                    self.pop_scope();
                    self.indent -= 1;
                    self.line("}");
                    return Ok(());
                }
                // Iterating a collection reached through `self` while the body may
                // change `self` would be two borrows at once; iterate a copy instead.
                // A body that touches the collection itself would be a second
                // borrow: walk a copy, so `for x in s: s.add(...)` is fine.
                let body_touches = match self.place_root(iter).map(|r| r.kind.clone()) {
                    Some(ExprKind::Ident(n)) => body.stmts.iter().any(|st| stmt_uses(st, &n)),
                    _ => false,
                };
                let self_rooted = body_touches
                    || self.current_self == SelfKind::Mutate
                    && matches!(self.place_root(iter).map(|r| &r.kind), Some(ExprKind::SelfRef))
                    || (self.current_self == SelfKind::Mutate
                        && matches!(self.place_root(iter).map(|r| &r.kind), Some(ExprKind::Ident(n)) if self.lookup(n).is_none() && self.field_type(n).is_some()));
                let chars_of = if vars.len() == 1 && !*mutable { self.chars_source(iter) } else { None };
                let (it, elem_ty, borrowed) = match &it_ty {
                    _ if chars_of.is_some() => (format!("({}).chars()", self.expr_val(chars_of.as_ref().unwrap())?), Type::Char, false),
                    Type::Iter(elem, by_ref) => (self.expr(iter)?, (**elem).clone(), *by_ref && !elem.is_copy()),
                    Type::Set(elem) if self_rooted => (format!("({}).iter().cloned().collect::<Vec<_>>().into_iter()", self.expr(iter)?), (**elem).clone(), false),
                    Type::List(elem) | Type::Map(_, elem) if self_rooted => {
                        let ex = self.expr(iter)?;
                        match &it_ty {
                            Type::Map(k, v) => (
                                format!("({}).iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>().into_iter()", ex),
                                Type::Tuple(vec![(**k).clone(), (**v).clone()]),
                                false,
                            ),
                            _ => (format!("({}).clone().into_iter()", ex), (**elem).clone(), false),
                        }
                    }
                    // A `shared var` collection: copy it out under one lock and
                    // walk the copy. Holding the lock for the whole body is how
                    // a program deadlocks, and `shared var` is short locks.
                    Type::Shared(inner, _) if matches!(**inner, Type::List(_) | Type::Map(..) | Type::Set(_)) => {
                        let h = self.handle_expr(iter)?;
                        let snap = |what: &str| format!("{{ let lume_g = {}.lock().unwrap(); {} }}.into_iter()", h, what);
                        match &**inner {
                            Type::Map(k, v) => (
                                snap("lume_g.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>()"),
                                Type::Tuple(vec![(**k).clone(), (**v).clone()]),
                                false,
                            ),
                            Type::Set(e) => (snap("lume_g.iter().cloned().collect::<Vec<_>>()"), (**e).clone(), false),
                            Type::List(e) => (snap("lume_g.clone()"), (**e).clone(), false),
                            _ => unreachable!(),
                        }
                    }
                    Type::List(elem) if elem.is_copy() || **elem == Type::Unknown => (format!("({}).iter().cloned()", self.expr(iter)?), (**elem).clone(), false),
                    Type::List(elem) => (format!("({}).iter()", self.expr(iter)?), (**elem).clone(), true),
                    Type::Map(k, v) => (format!("({}).iter()", self.expr(iter)?), Type::Tuple(vec![(**k).clone(), (**v).clone()]), true),
                    Type::Set(elem) if self_rooted => (format!("({}).iter().cloned().collect::<Vec<_>>().into_iter()", self.expr(iter)?), (**elem).clone(), false),
                    Type::Set(elem) if elem.is_copy() => (format!("({}).iter().cloned()", self.expr(iter)?), (**elem).clone(), false),
                    Type::Set(elem) => (format!("({}).iter()", self.expr(iter)?), (**elem).clone(), true),
                    Type::Unknown => (format!("({}).iter().cloned()", self.expr(iter)?), Type::Unknown, false),
                    other => {
                        return Err(LumeError::new(iter.line, iter.col, format!("cannot loop over a `{}`", type_name(other))).with_help("`for` needs a list, a set, a map or a range"));
                    }
                };
                let pattern = if vars.len() == 1 {
                    rust_name(&vars[0])
                } else {
                    match &elem_ty {
                        Type::Tuple(ts) if ts.len() == vars.len() => {
                            // over a borrowed tuple, Copy parts come out by value
                            // and the rest by reference, matching what is declared
                            // a list or set yields `&(a, b)`; a map or a lazy
                            // chain (`enumerate`, `map`) yields `(a, b)` whose
                            // parts are already references
                            let map_pairs = !matches!(it_ty, Type::List(_) | Type::Set(_));
                            let parts: Vec<String> = vars
                                .iter()
                                .zip(ts)
                                .map(|(v, t)| if borrowed && !map_pairs && t.is_copy() { rust_name(v) } else if borrowed && !map_pairs { format!("ref {}", rust_name(v)) } else { rust_name(v) })
                                .collect();
                            let inner = format!("({})", parts.join(", "));
                            if borrowed && !map_pairs { format!("&{}", inner) } else { inner }
                        }
                        Type::Tuple(ts) => {
                            return Err(LumeError::new(*line, *col, format!("`for {}` names {} variables, but each item has {} parts", vars.join(", "), vars.len(), ts.len())));
                        }
                        _ => {
                            return Err(LumeError::new(*line, *col, format!("`for {}` names two variables, but the items are not pairs", vars.join(", ")))
                                .with_help("use `xs.enumerate` for (index, item) pairs, or a single variable"));
                        }
                    }
                };
                self.line(&format!("for {} in {} {{", pattern, it));
                self.indent += 1;
                self.push_scope();
                if vars.len() == 1 {
                    self.declare(&vars[0], false, borrowed, elem_ty.clone(), *line);
                } else if let Type::Tuple(ts) = &elem_ty {
                    for (v, t) in vars.iter().zip(ts) {
                        self.declare(v, false, borrowed && !t.is_copy(), t.clone(), *line);
                    }
                }
                if let Some(f) = filter {
                    let fc = self.expr(f)?;
                    self.line(&format!("if !({}) {{ continue; }}", fc));
                }
                self.loop_depth += 1;
                self.barriers.push(self.scopes.len() - 1);
                let r = self.block_body(body, false);
                self.barriers.pop();
                self.loop_depth -= 1;
                r?;
                self.pop_scope();
                self.indent -= 1;
                self.line("}");
            }
            Stmt::Break { line, col } => {
                self.loop_control("break", "break;", *line, *col)?;
            }
            Stmt::Next { line, col } => {
                self.loop_control("next", "continue;", *line, *col)?;
            }
            Stmt::Assert { cond, line, col } => {
                let ct = self.ty_of(cond).materialized();
                if ct != Type::Bool && ct != Type::Unknown {
                    return Err(LumeError::new(cond.line, cond.col, format!("`assert` needs a `Bool`, but this is a `{}`", type_name(&ct)))
                        .with_help("compare it to something: `assert x == 3`, `assert xs.len > 0`"));
                }
                let text = self.assert_text(*line, *col).replace('\\', "\\\\").replace('"', "\\\"");
                let c = self.expr(cond)?;
                // a comparison prints both sides
                let sides = match &cond.kind {
                    ExprKind::Binary { op, lhs, rhs } if matches!(*op, "==" | "!=" | "<" | "<=" | ">" | ">=") => {
                        // the two sides tell each other what they hold, so an
                        // empty list or map still has a type to print
                        let lt = self.ty_of(lhs).materialized();
                        let rt = self.ty_of(rhs).materialized();
                        let other = |mine: &Type, theirs: &Type| if type_is_known(mine) { mine.clone() } else { theirs.clone() };
                        // a bare `None` has no type of its own to print: use its text
                        let side = |g: &mut Self, x: &Expr, t: Type| -> Result<String> {
                            let empty = matches!(&x.kind, ExprKind::List(i) if i.is_empty()) || matches!(&x.kind, ExprKind::MapLit(p) if p.is_empty());
                            Ok(match &x.kind {
                                ExprKind::None => "String::from(\"None\")".to_string(),
                                // `xs == []`: the other side says what the empty one holds
                                _ if empty && type_is_known(&t) => format!("(<{}>::new()).lume_str()", g.rt(&t)),
                                _ => format!("({}).lume_str()", g.expr_val(x)?),
                            })
                        };
                        let l = side(self, lhs, other(&lt, &rt))?;
                        let r = side(self, rhs, other(&rt, &lt))?;
                        format!("Some(({}, {}))", l, r)
                    }
                    _ => "None".to_string(),
                };
                self.line(&format!("if !({}) {{ lume_assert_failed({}, \"{}\", {}); }}", c, line, text, sides));
            }
        }
        Ok(())
    }

    fn loop_control(&mut self, kw: &str, rs: &str, line: usize, col: usize) -> Result<()> {
        if self.loop_depth == 0 {
            let e = LumeError::new(line, col, format!("`{}` outside of a loop", kw));
            return Err(if self.in_block {
                e.with_help(format!(
                    "a block runs once per item and cannot `{}`; stop early with `find`, `take_while` or `filter` before `each`, or use a `for` loop",
                    kw
                ))
            } else {
                e
            });
        }
        self.line(rs);
        Ok(())
    }

    /// The root name of a place expression (`a.b[c].d` -> `a`), if any.
    fn place_root<'a>(&self, e: &'a Expr) -> Option<&'a Expr> {
        match &e.kind {
            ExprKind::Ident(_) | ExprKind::SelfRef => Some(e),
            ExprKind::Method { recv, args, .. } if args.is_empty() => self.place_root(recv),
            ExprKind::Index { recv, .. } | ExprKind::TupleIndex { recv, .. } => self.place_root(recv),
            _ => None,
        }
    }

    /// Checks that a place can be written through, and returns the Rust
    /// text of the place as an lvalue.
    fn mutable_place(&mut self, e: &Expr, what: &str, line: usize, col: usize) -> Result<String> {
        let root = match self.place_root(e) {
            Some(r) => r.clone(),
            None => {
                return Err(LumeError::new(line, col, format!("only a named value can be changed, not {}", what)).with_help("bind the value to a `var` first"));
            }
        };
        match &root.kind {
            ExprKind::SelfRef => self.require_var_self(what, line, col)?,
            ExprKind::Ident(n) => match self.lookup(n).cloned() {
                Some(b) if b.mutable => {}
                Some(b) => {
                    return Err(LumeError::new(line, col, format!("`{}` is immutable, so {} cannot be changed", n, what))
                        .with_help(format!("declare it with `var {} = ...` on line {}", n, b.line)));
                }
                None if self.field_type(n).is_some() => self.require_var_self(n, line, col)?,
                None => return Err(self.unknown_name(n, line, col)),
            },
            _ => unreachable!(),
        }
        self.lvalue(e)
    }

    /// Rust lvalue text for a place: indexes become `[i as usize]` on lists
    /// and `get_mut` on maps.
    fn lvalue(&mut self, e: &Expr) -> Result<String> {
        match &e.kind {
            ExprKind::Index { recv, index } => {
                let rt = match self.indexed_collection(recv) {
                    Some(el) => el,
                    None => self.ty_of(recv).materialized(),
                };
                let r = self.lvalue(recv)?;
                match rt {
                    Type::List(_) | Type::Unknown => Ok(format!("{}[({}) as usize]", r, self.expr(index)?)),
                    Type::Map(ref kt, ref vt) => {
                        let kt = (**kt).clone();
                        let k = self.map_key_text(index, &kt, &rt)?;
                        // Writing through a missing key creates it, the way a
                        // plain `m[k] = v` does — but only where the value has
                        // an empty form to start from.
                        let makes_itself = matches!(
                            **vt,
                            Type::Map(..) | Type::Set(_) | Type::List(_) | Type::Str | Type::Int | Type::Float | Type::Bool
                        );
                        if makes_itself {
                            // the map keeps the key it creates, so it gets one of its own
                            let owned = self.expr_owned(index)?;
                            Ok(format!("(*{}.slot({}))", r, owned))
                        } else {
                            Ok(format!("(*{}.get_mut({}).expect(\"no such key in map\"))", r, k))
                        }
                    }
                    other => Err(LumeError::new(e.line, e.col, format!("`{}` values cannot be indexed", type_name(&other)))),
                }
            }
            ExprKind::Method { recv, name, args } if args.is_empty() => {
                let r = self.lvalue(recv)?;
                Ok(format!("{}.{}", r, rust_name(name)))
            }
            ExprKind::TupleIndex { recv, index } => Ok(format!("{}.{}", self.lvalue(recv)?, index)),
            _ => self.expr(e),
        }
    }

    fn require_var_self(&self, field: &str, line: usize, col: usize) -> Result<()> {
        if self.current_self != SelfKind::Mutate {
            return Err(LumeError::new(line, col, format!("`{}` changes the field `{}`, but its `self` is read-only", self.current_fn, field))
                .with_help(format!("write `def {}(var self, ...)` to allow it", self.current_fn)));
        }
        Ok(())
    }

    fn tail_unit(&mut self, line: usize, col: usize) -> Result<()> {
        Err(LumeError::new(line, col, format!("the last line of a function returning `{}` must be a value, not a binding", type_name(&self.current_ret)))
            .with_help("put the value on its own line after the binding"))
    }

    fn expr_stmt(&mut self, e: &Expr) -> Result<String> {
        match &e.kind {
            ExprKind::If { branches, else_block } => self.if_chain(branches, else_block.as_ref(), false),
            ExprKind::Match { scrutinee, arms } => self.match_expr(scrutinee, arms, false, e),
            // a value on its own does nothing: `None`, `()`
            ExprKind::None => Ok("()".into()),
            _ => self.expr(e),
        }
    }

    // ----- expressions ------------------------------------------------------

    fn unknown_name(&self, name: &str, line: usize, col: usize) -> LumeError {
        let e = LumeError::new(line, col, format!("unknown name `{}`", name));
        match name {
            "continue" => return e.with_help("Lume spells it `next`: it skips to the next turn of the loop"),
            "null" | "nil" | "None" | "undefined" => return e.with_help("an absent value is `None`, and a value that may be absent has type `T?`"),
            "self" | "this" => return e.with_help("`self` is only available inside a method"),
            "print" | "println" | "console" | "echo" => return e.with_help("Lume prints with `puts`"),
            "len" | "length" => return e.with_help("length is a method: `xs.len`"),
            _ => {}
        }
        match self.suggest(name) {
            Some(s) => e.with_help(format!("did you mean `{}`?", s)),
            None => e.with_help("names must be bound with `name = value` or `var name = value` before use"),
        }
    }

    fn no_such_member(&self, tname: &str, name: &str, line: usize, col: usize) -> LumeError {
        let e = LumeError::new(line, col, format!("`{}` has no field or method named `{}`", tname, name));
        let fields: Vec<String> = self.structs.get(tname).map(|s| s.fields.iter().map(|(n, _)| n.clone()).collect()).unwrap_or_default();
        let methods: Vec<String> = self.methods_of(tname).map(|m| m.keys().cloned().collect()).unwrap_or_default();
        match self.suggest_from(name, fields.iter().cloned().chain(methods.iter().cloned())) {
            Some(s) => e.with_help(format!("did you mean `{}`?", s)),
            None if !fields.is_empty() => e.with_help(format!("`{}` has fields {}", tname, fields.join(", "))),
            None => e,
        }
    }

    /// True when the expression names a place we cannot move out of: a
    /// borrowed parameter, `self`, or a field reached through either.
    /// In an `extend` on a copied built-in — `extend Int with ...` — the
    /// method takes `&self`, so `self` is already read through once and no
    /// other rule should read through it again.
    fn self_is_lent_copy(&self) -> bool {
        self.current_self_ty.as_ref().map(|t| t.materialized().is_copy()).unwrap_or(false)
    }

    fn is_borrowed_place(&mut self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Ident(n) => match self.lookup(n) {
                Some(b) => b.borrowed,
                None => self.field_type(n).is_some() || self.consts.contains_key(&self.canon(n)),
            },
            ExprKind::SelfRef => !self.self_is_lent_copy(),
            ExprKind::Method { recv, name, args }
                if args.is_empty()
                    && matches!(&recv.kind, ExprKind::Ident(a) if self.lookup(a).is_none() && self.module_aliases.contains_key(a))
                    && matches!(&recv.kind, ExprKind::Ident(a) if self.consts.contains_key(&self.canon(&format!("{}.{}", a, name)))) =>
            {
                true
            }
            ExprKind::Method { recv, name, args } if args.is_empty() => {
                // a generic struct at an instantiation keeps its fields:
                // `Stack[Int]`'s `vals` is as borrowed as `Stack`'s
                let rt = self.ty_of(recv);
                if let Type::Named(s) | Type::App(s, _) = &rt {
                    if let Some(info) = self.structs.get(&self.canon(s)) {
                        return info.fields.iter().any(|(n, _)| n == name);
                    }
                }
                false
            }
            ExprKind::TupleIndex { recv, .. } => self.is_borrowed_place(recv) || self.is_borrowed_ident(recv),
            _ => false,
        }
    }

    fn is_borrowed_ident(&self, e: &Expr) -> bool {
        match &e.kind {
            // a lent copy is already read through once when its value is
            // taken, so it is not borrowed by the time anyone asks
            ExprKind::Ident(n) if self.lent_names.contains(n) => false,
            ExprKind::Ident(n) => self.lookup(n).map(|b| b.borrowed).unwrap_or(false),
            ExprKind::SelfRef => !self.self_is_lent_copy(),
            _ => false,
        }
    }

    /// Materialise a lazy chain into a Vec of owned values.
    fn collect_iter(&self, it: &str, by_ref: bool) -> String {
        self.collect_iter_t(it, by_ref, &Type::Unknown)
    }

    /// Like `collect_iter`, but knows the item type: borrowed strings may be
    /// `&str` (from `split`/`lines`), which `to_string` handles and `cloned` does not.
    fn collect_iter_t(&self, it: &str, by_ref: bool, elem: &Type) -> String {
        if by_ref && *elem == Type::Str {
            format!("({}).map(|s| s.to_string()).collect::<Vec<String>>()", it)
        } else if by_ref {
            format!("({}).cloned().collect::<Vec<_>>()", it)
        } else {
            format!("({}).collect::<Vec<_>>()", it)
        }
    }

    /// An expression wherever a plain value is needed (printing, comparing,
    /// interpolating): lazy chains are collected, everything else passes.
    fn expr_val(&mut self, e: &Expr) -> Result<String> {
        let t = self.ty_of(e);
        if let Type::Iter(elem, by_ref) = t {
            let s = self.expr(e)?;
            return Ok(self.collect_iter_t(&s, by_ref, &elem));
        }
        // a copied value that arrives behind a reference because the
        // interface declared it as `T`: read it by value
        if let ExprKind::Ident(n) = &e.kind {
            if self.lent_names.contains(n) && t.materialized().is_copy() {
                let s = self.expr(e)?;
                return Ok(format!("(*{})", s));
            }
        }
        self.expr(e)
    }

    /// An expression in a position that needs an owned value (binding,
    /// return, constructor argument, list item). Places that are borrowed
    /// get cloned; owned locals move.
    fn expr_owned(&mut self, e: &Expr) -> Result<String> {
        let t = self.ty_of(e);
        if let Type::Iter(elem, by_ref) = &t {
            let (by_ref, elem) = (*by_ref, (**elem).clone());
            let s = self.expr(e)?;
            return Ok(self.collect_iter_t(&s, by_ref, &elem));
        }
        let s = self.expr(e)?;
        if !t.is_copy() && self.is_borrowed_place(e) {
            // a borrowed string may be a `&str`: to_string covers both
            if t == Type::Str { Ok(format!("{}.to_string()", s)) } else { Ok(format!("{}.clone()", s)) }
        } else if !t.is_copy() && matches!(&e.kind, ExprKind::Ident(n) if self.lookup(n).is_some() && self.used_after(n)) {
            // an owned local that is used again later: give away a copy, keep the value
            Ok(format!("{}.clone()", s))
        } else {
            Ok(s)
        }
    }

    /// `expr_owned` for a position whose type is known: boxes a concrete
    /// value stored as an interface, element by element for list literals.
    fn expr_owned_as(&mut self, e: &Expr, expected: &Type) -> Result<String> {
        self.want.push(expected.clone());
        let r = self.expr_owned_as_inner(e, expected);
        self.want.pop();
        r
    }

    fn expr_owned_as_inner(&mut self, e: &Expr, expected: &Type) -> Result<String> {
        if let Type::Shared(_, mutable) = expected {
            return match self.ty_of(e) {
                Type::Shared(..) => Ok(format!("{}.clone()", self.handle_expr(e)?)),
                _ => {
                    let v = self.expr_owned(e)?;
                    Ok(if *mutable { format!("std::sync::Arc::new(std::sync::Mutex::new({}))", v) } else { format!("std::sync::Arc::new({})", v) })
                }
            };
        }
        if *expected == Type::Char {
            if let Some(c) = char_literal(e) {
                return Ok(format!("'{}'", rust_char(c)));
            }
        }
        match (&e.kind, expected) {
            // `{}` where the kind is known: say it, so nothing downstream has to infer it
            (ExprKind::MapLit(pairs), Type::Map(..) | Type::Set(_)) if pairs.is_empty() && type_is_known(expected) => {
                Ok(format!("<{}>::new()", self.rt(expected)))
            }
            (ExprKind::List(items), Type::List(elem)) if self.is_interface(elem) => {
                let mut parts = Vec::new();
                for i in items {
                    let it = self.ty_of(i).materialized();
                    let v = self.expr_owned(i)?;
                    parts.push(self.coerce(v, &it, elem, i.line, i.col)?);
                }
                Ok(format!("vec![{}]", parts.join(", ")))
            }
            (ExprKind::Some(inner), Type::Option(t)) if self.is_interface(t) => {
                let it = self.ty_of(inner).materialized();
                let v = self.expr_owned(inner)?;
                Ok(format!("Some({})", self.coerce(v, &it, t, inner.line, inner.col)?))
            }
            _ => {
                let from = self.ty_of(e).materialized();
                let v = self.expr_owned(e)?;
                // an interface parameter arrives as a generic; stored, it
                // has to become the pointer a field or a list holds
                if self.is_interface(expected) {
                    if let ExprKind::Ident(n) = &e.kind {
                        if self.iface_params.contains(n) {
                            // the value inside makes the pointer, which is
                            // what every interface carries `lume_box` for
                            return Ok(format!("{}.lume_box()", rust_name(n)));
                        }
                    }
                }
                self.coerce(v, &from, expected, e.line, e.col)
            }
        }
    }

    /// An argument for a parameter of type `t`: Copy types by value,
    /// everything else by reference.
    fn expr_arg(&mut self, e: &Expr, t: &Type) -> Result<String> {
        self.want.push(t.clone());
        let r = self.expr_arg_inner(e, t);
        self.want.pop();
        r
    }

    fn expr_arg_inner(&mut self, e: &Expr, t: &Type) -> Result<String> {
        // a slot the definition spelled `T`, filled in with `Str`: Rust
        // wants a `&String` there, and Lume's strings travel as `&str`
        if matches!(t, Type::Var(_)) && self.ty_of(e).materialized() == Type::Str {
            let owned = self.expr_owned(e)?;
            return Ok(format!("&{}", owned));
        }
        if let Type::Fn(ins, out) = t {
            let (ins, out) = (ins.clone(), out.clone());
            return self.block_arg(e, &ins, &ins, &out, false);
        }
        if let Type::Shared(..) = t {
            // the callee gets its own handle to the same value
            let at = self.ty_of(e);
            if !matches!(at, Type::Shared(..)) {
                return Err(LumeError::new(e.line, e.col, format!("this parameter is `{}`, but the value is a plain `{}`", type_name(t), type_name(&at.materialized())))
                    .with_help("declare the value with `shared var` (or `shared`) where it is created"));
            }
            let h = self.handle_expr(e)?;
            return Ok(format!("{}.clone()", h));
        }
        if let Type::Shared(inner, true) = self.ty_of(e) {
            // a shared value passed where a plain one is expected: lend it under the lock
            if !inner.is_copy() {
                let h = self.handle_expr(e)?;
                return Ok(format!("&*{}.lock().unwrap()", h));
            }
        }
        if self.is_interface(t) {
            // static dispatch through a generic parameter; the value must conform
            let at = self.ty_of(e).materialized();
            if at != Type::Unknown && !self.is_interface(&at) {
                self.require_conforms(&at, t, e.line, e.col)?;
            }
        }
        if let Type::Iter(elem, by_ref) = self.ty_of(e) {
            let s = self.expr(e)?;
            let c = self.collect_iter_t(&s, by_ref, &elem);
            return Ok(format!("&{}", c));
        }
        if *t == Type::Char {
            if let Some(c) = char_literal(e) {
                return Ok(format!("'{}'", rust_char(c)));
            }
        }
        let s = self.expr(e)?;
        if *t == Type::Str && self.ty_of(e).materialized() == Type::Char {
            return Ok(format!("&({}).to_string()", s));
        }
        if t.is_copy() {
            return Ok(if self.is_borrowed_ident(e) && self.ty_of(e).is_copy() { format!("(*{})", s) } else { s });
        }
        if self.is_borrowed_ident(e) {
            Ok(s)
        } else {
            Ok(format!("&{}", s))
        }
    }

    /// An argument for a `var` parameter: must be a `var` binding (or a `var`
    /// parameter, or a field of a `var self`), passed as `&mut`.
    fn expr_var_arg(&mut self, e: &Expr, pname: &str, callee: &str) -> Result<String> {
        match &e.kind {
            ExprKind::Ident(n) => match self.lookup(n).cloned() {
                Some(b) if b.mutable && b.borrowed => Ok(rust_name(n)), // a var parameter: reborrow
                Some(b) if b.mutable => Ok(format!("&mut {}", rust_name(n))),
                Some(b) => Err(LumeError::new(e.line, e.col, format!("`{}` may change `{}`, but `{}` is immutable", callee, pname, n))
                    .with_help(format!("declare it with `var {} = ...` on line {}", n, b.line))),
                None if self.field_type(n).is_some() => {
                    self.require_var_self(n, e.line, e.col)?;
                    Ok(format!("&mut self.{}", rust_name(n)))
                }
                None => Err(self.unknown_name(n, e.line, e.col)),
            },
            _ => Err(LumeError::new(e.line, e.col, format!("`{}` changes its `{}` argument, so it needs a named `var` value, not an expression", callee, pname))
                .with_help("bind the value to a `var` first")),
        }
    }

    /// The iterator a list, range or chain yields: (rust, elem type, by_ref).
    fn iter_base(&mut self, recv: &Expr, e: &Expr) -> Result<(String, Type, bool)> {
        if let Some(inner) = self.chars_source(recv) {
            let r = self.expr_val(&inner)?;
            return Ok((format!("({}).chars()", r), Type::Char, false));
        }
        let rt = self.ty_of(recv);
        let r = self.expr(recv)?;
        match rt {
            Type::List(elem) => {
                if elem.is_copy() {
                    Ok((format!("({}).iter().cloned()", r), *elem, false))
                } else {
                    Ok((format!("({}).iter()", r), *elem, true))
                }
            }
            Type::Iter(elem, by_ref) => Ok((r, *elem, by_ref)),
            Type::Set(elem) => {
                if elem.is_copy() {
                    Ok((format!("({}).iter().cloned()", r), *elem, false))
                } else {
                    Ok((format!("({}).iter()", r), *elem, true))
                }
            }
            Type::Map(k, v) => {
                // Copy parts travel by value, so a block sees `v > 1` on an `Int`
                self.map_items = true;
                let pair = map_pair(&k, &v);
                Ok((format!("({}).iter().map(|(k, v)| {})", r, pair), Type::Tuple(vec![*k, *v]), true))
            }
            Type::Unknown => Err(LumeError::new(e.line, e.col, "cannot tell what kind of value this block is applied to")
                .with_help("add a type to the binding or parameter it comes from")),
            other => Err(LumeError::new(e.line, e.col, format!("`{}` values cannot take a block; only lists and ranges can", type_name(&other)))),
        }
    }

    /// The pieces of a left-leaning `+` chain over text, in order.
    fn text_leaves<'a>(&mut self, e: &'a Expr, out: &mut Vec<&'a Expr>) {
        if let ExprKind::Binary { op: "+", lhs, rhs } = &e.kind {
            let lt = self.ty_of(lhs).materialized();
            let rt = self.ty_of(rhs).materialized();
            if text_like(&lt) && text_like(&rt) {
                self.text_leaves(lhs, out);
                self.text_leaves(rhs, out);
                return;
            }
        }
        out.push(e);
    }

    /// `s.chars` used only to iterate: the string it came from, so the
    /// characters can be walked without building a list first.
    fn chars_source(&mut self, e: &Expr) -> Option<Expr> {
        if let ExprKind::Method { recv, name, args } = &e.kind {
            if name == "chars" && args.is_empty() && self.ty_of(recv).materialized() == Type::Str && self.shared_root(recv).is_none() {
                return Some((**recv).clone());
            }
        }
        None
    }

    /// Emits a Rust closure for a Lume block over items of type `elem`.
    /// `pattern_ref` is set for predicates (Rust passes `&Item`). `acc` is
    /// the accumulator type for `fold`.
    fn gen_lambda(&mut self, params: &[String], body: &Block, elem: &Type, by_ref: bool, pattern_ref: bool, want_value: bool, acc: Option<&Type>, at: &Expr) -> Result<String> {
        self.gen_lambda_ex(params, body, elem, by_ref, pattern_ref, want_value, acc, false, at)
    }

    fn gen_lambda_ex(&mut self, params: &[String], body: &Block, elem: &Type, by_ref: bool, pattern_ref: bool, want_value: bool, acc: Option<&Type>, negate: bool, at: &Expr) -> Result<String> {
        let expected = if acc.is_some() { 2 } else if matches!(elem, Type::Tuple(ts) if ts.len() == 2) && params.len() == 2 { 2 } else { 1 };
        if params.len() != expected {
            if params.is_empty() {
                return Err(LumeError::new(at.line, at.col, format!("this block names no arguments, but it is given {}", plural(expected, "one", "two")))
                    .with_help("write `{ |x| ... }` or `do |x|`; use `_` only in a bare argument like `.map(_.name)`"));
            }
            let msg = if acc.is_some() {
                "`fold` takes a block with two arguments: the accumulator and the item".to_string()
            } else if params.len() == 2 {
                "this block names two arguments, but the items are not pairs".to_string()
            } else {
                format!("this block takes one argument, but {} were named", params.len())
            };
            return Err(LumeError::new(at.line, at.col, msg));
        }
        let names: Vec<String> = params.iter().map(|p| rust_name(p)).collect();
        let map_items = std::mem::take(&mut self.map_items);
        // The closure pattern spells out every reference layer, so each name
        // binds exactly what `declare_block_params` says it is: a Copy part by
        // value, anything else as one `&`.
        let item_pat = if acc.is_none() && params.len() == 2 && matches!(elem, Type::Tuple(ts) if ts.len() == 2) {
            let ts = match elem {
                Type::Tuple(ts) => ts.clone(),
                _ => unreachable!(),
            };
            if map_items {
                // items are `(k, v)` values holding references already
                let pair = format!("({}, {})", names[0], names[1]);
                if pattern_ref { format!("&{}", pair) } else { pair }
            } else if by_ref || (pattern_ref && !elem.is_copy()) {
                // a reference to a tuple: take Copy parts out, borrow the rest
                let parts: Vec<String> = names.iter().zip(&ts).map(|(n, t)| if t.is_copy() { n.clone() } else { format!("ref {}", n) }).collect();
                let pair = format!("({}, {})", parts[0], parts[1]);
                let depth = (by_ref as usize) + (pattern_ref as usize);
                format!("{}{}", "&".repeat(depth), pair)
            } else {
                let pair = format!("({}, {})", names[0], names[1]);
                if pattern_ref { format!("&{}", pair) } else { pair }
            }
        } else {
            let n = if acc.is_some() { names[1].clone() } else { names[0].clone() };
            if pattern_ref && elem.is_copy() {
                format!("&{}", n)
            } else if pattern_ref && by_ref && !map_items {
                // `.iter().filter(...)` passes `&&T`: bind `n` as `&T`
                format!("&{}", n)
            } else {
                n
            }
        };
        let pattern = if acc.is_some() { format!("{}, {}", names[0], item_pat) } else { item_pat };
        let item_borrowed = if pattern_ref { !elem.is_copy() } else { by_ref && !elem.is_copy() };
        self.push_scope();
        self.declare_block_params(params, elem, item_borrowed, acc, at.line);
        let saved_loop = self.loop_depth;
        let saved_in_block = self.in_block;
        let saved_tail = self.tail_of_fn;
        let saved_block_ret = self.block_ret.take();
        self.loop_depth = 0;
        self.in_block = true;
        self.tail_of_fn = false;
        self.barriers.push(self.scopes.len() - 1);
        let saved_out = std::mem::take(&mut self.out);
        let base = self.indent;
        let inline = body.stmts.len() == 1 && matches!(body.stmts[0], Stmt::Expr(ref x) if !matches!(x.kind, ExprKind::If { .. } | ExprKind::Match { .. }));
        let text = if inline {
            if let Stmt::Expr(x) = &body.stmts[0] {
                let v = if want_value { self.expr_owned(x)? } else { format!("{{ {}; }}", self.expr_stmt(x)?) };
                if negate { format!("|{}| !({})", pattern, v) } else { format!("|{}| {}", pattern, v) }
            } else {
                unreachable!()
            }
        } else {
            self.out.push_str(&format!("|{}| {}{{\n", pattern, if negate { "!" } else { "" }));
            self.nested_block(body, want_value)?;
            self.out.push_str(&"    ".repeat(base));
            self.out.push('}');
            std::mem::take(&mut self.out)
        };
        self.out = saved_out;
        self.barriers.pop();
        self.loop_depth = saved_loop;
        self.in_block = saved_in_block;
        self.tail_of_fn = saved_tail;
        // a lambda inside a declared block does not end that block: what the
        // block promised is still owed after this closure
        self.block_ret = saved_block_ret;
        self.pop_scope();
        Ok(text)
    }

    /// Does the receiver's own type declare `name` with a block parameter?
    /// Then the block belongs to that method, not to the built-in of the
    /// same name.
    fn user_block_method(&self, rt: &Type, name: &str) -> bool {
        if !matches!(rt, Type::Named(_) | Type::App(..) | Type::Var(_)) {
            return false;
        }
        self.methods_of(&self.type_key(rt))
            .and_then(|m| m.get(name).cloned())
            .map(|s| s.params.iter().any(|(_, t)| matches!(t, Type::Fn(..))))
            .unwrap_or(false)
    }

    /// The argument for a `(A, B) -> C` parameter: a block, a `_`
    /// shorthand, or the name of a function that fits.
    fn block_arg(&mut self, e: &Expr, ins: &[Type], decl: &[Type], out: &Type, recursive: bool) -> Result<String> {
        let want = format!("({}) -> {}", ins.iter().map(type_name).collect::<Vec<_>>().join(", "), type_name(out));
        // a block this function was handed, passed straight on
        if let ExprKind::Ident(n) = &e.kind {
            if let Some(have) = self.lookup(n).map(|b| b.ty.clone()) {
                if let Type::Fn(..) = have {
                    if !self.assignable(&have, &Type::Fn(ins.to_vec(), Box::new(out.clone()))) {
                        return Err(LumeError::new(e.line, e.col, format!("`{}` is `{}`, but this parameter takes `{}`", n, type_name(&have), want)));
                    }
                    // A function that hands its own block to itself is moved
                    // in, so every level has the same type; anywhere else it
                    // is lent, and may be handed on again.
                    return Ok(if recursive { rust_name(n) } else { format!("&mut {}", rust_name(n)) });
                }
            }
        }
        // a function named where behaviour is wanted: `map_all(xs, double)`
        let named = match &e.kind {
            ExprKind::Ident(n) if self.lookup(n).is_none() && (self.fns.contains_key(&self.canon(n)) || self.bare_method(n).is_some()) => Some(n.clone()),
            _ => None,
        };
        if let Some(n) = named {
            let names: Vec<String> = (0..ins.len()).map(|i| format!("lume_a{}", i)).collect();
            let args: Vec<Arg> = names.iter().map(|a| Arg { name: None, value: Expr::new(ExprKind::Ident(a.clone()), e.line, e.col) }).collect();
            let call = Expr::new(ExprKind::Call { name: n, args }, e.line, e.col);
            let body = Block { stmts: vec![Stmt::Expr(call)] };
            return self.gen_block_closure(&names, &body, ins, decl, out, e);
        }
        let (params, body) = match &e.kind {
            ExprKind::Lambda { params, body } => (params.clone(), body.clone()),
            _ => {
                let have = self.ty_of(e).materialized();
                let err = LumeError::new(e.line, e.col, format!("this parameter takes behaviour, `{}`, but `{}` is a value", want, snippet(e)));
                return Err(if have == Type::Unknown {
                    err.with_help("pass a block — `{ |x| x * 2 }` after the call, or `do |x|` and an indented body — or the name of a function that fits")
                } else {
                    err.with_help(format!("`{}` is a `{}`; a block is written `{{ |x| ... }}` after the call, or `do |x|` with an indented body", snippet(e), type_name(&have)))
                });
            }
        };
        if params.len() != ins.len() {
            return Err(LumeError::new(e.line, e.col, format!(
                "this parameter takes a block of {} {}, but the block names {}",
                ins.len(),
                plural(ins.len(), "argument", "arguments"),
                params.len()
            ))
            .with_help(format!("it is declared `{}`", want)));
        }
        self.gen_block_closure(&params, &body, ins, decl, out, e)
    }

    /// A Rust closure for a block whose argument types the definition fixed.
    /// `decl` is how the definition spelled them: where it said `T` and this
    /// call filled in a copied type, the closure takes a reference, so the
    /// pattern unwraps it and the name binds a value either way.
    fn gen_block_closure(&mut self, params: &[String], body: &Block, ins: &[Type], decl: &[Type], out: &Type, at: &Expr) -> Result<String> {
        let mut pattern: Vec<String> = Vec::new();
        self.push_scope();
        for (i, (p, t)) in params.iter().zip(ins).enumerate() {
            let lent = *decl.get(i).unwrap_or(t) != Type::Str;
            let unwrap = lent && t.is_copy();
            pattern.push(if unwrap { format!("&{}", rust_name(p)) } else { rust_name(p) });
            self.declare(p, false, lent && !unwrap, t.clone(), at.line);
        }
        let saved_loop = self.loop_depth;
        let saved_in_block = self.in_block;
        let saved_tail = self.tail_of_fn;
        let saved_ret = self.current_ret.clone();
        let saved_fn = self.current_fn.clone();
        let saved_block_ret = std::mem::replace(&mut self.block_ret, Some(out.clone()));
        self.current_fn = "this block".to_string();
        self.loop_depth = 0;
        self.in_block = true;
        // a block that gives back a `T or E` or a `T?` ends the way a
        // function does: a bare value is the implied `Ok` or `Some`, and `?`
        // leaves the block
        self.tail_of_fn = matches!(out, Type::Result(..) | Type::Option(_));
        self.current_ret = out.clone();
        self.barriers.push(self.scopes.len() - 1);
        let want_value = *out != Type::Unit;
        if want_value {
            let have = self.tail_type(body).materialized();
            if have != Type::Unknown && !self.assignable(&have, out) && !matches!(out, Type::Result(..) | Type::Option(_)) {
                self.pop_scope();
                return Err(LumeError::new(at.line, at.col, format!("this block gives back a `{}`, but the parameter asks for a `{}`", type_name(&have), type_name(out)))
                    .with_help(format!("the block's last line must be a `{}`", type_name(out))));
            }
        }
        let saved_out = std::mem::take(&mut self.out);
        let base = self.indent;
        let inline = body.stmts.len() == 1 && matches!(body.stmts[0], Stmt::Expr(ref x) if !matches!(x.kind, ExprKind::If { .. } | ExprKind::Match { .. }));
        let text = if inline {
            match &body.stmts[0] {
                Stmt::Expr(x) => {
                    let v = if want_value {
                        let want = match out {
                            Type::Result(ok, _) => (**ok).clone(),
                            // a bare value is fine where a `T?` is asked for
                            Type::Option(inner) if !matches!(self.tail_type(body).materialized(), Type::Option(_)) => (**inner).clone(),
                            other => other.clone(),
                        };
                        let inner = self.expr_owned_as(x, &want)?;
                        self.coerce_result(inner, x)?
                    } else {
                        format!("{{ {}; }}", self.expr_stmt(x)?)
                    };
                    format!("|{}| {}", pattern.join(", "), v)
                }
                _ => unreachable!(),
            }
        } else {
            self.out.push_str(&format!("|{}| {{\n", pattern.join(", ")));
            self.nested_block(body, want_value)?;
            self.out.push_str(&"    ".repeat(base));
            self.out.push('}');
            std::mem::take(&mut self.out)
        };
        self.out = saved_out;
        self.barriers.pop();
        self.loop_depth = saved_loop;
        self.in_block = saved_in_block;
        self.tail_of_fn = saved_tail;
        self.current_ret = saved_ret;
        self.current_fn = saved_fn;
        self.block_ret = saved_block_ret;
        self.pop_scope();
        Ok(text)
    }

    /// Collection methods that take a block.
    fn block_method(&mut self, recv: &Expr, name: &str, args: &[Arg], lam: &Expr, e: &Expr) -> Result<String> {
        let (params, body) = match &lam.kind {
            ExprKind::Lambda { params, body } => (params, body),
            _ => unreachable!(),
        };
        if name != "fold" && !args.is_empty() {
            return Err(LumeError::new(e.line, e.col, format!("`{}` takes only a block", name)));
        }
        let rt = self.ty_of(recv);
        match (&rt, name) {
            (Type::Option(inner), "map") | (Type::Result(inner, _), "map") => {
                let inner = (**inner).clone();
                let r = self.expr(recv)?;
                let f = self.gen_lambda(params, body, &inner, false, false, true, None, lam)?;
                return Ok(format!("({}).clone().map({})", r, f));
            }
            (Type::Map(k, v), "filter" | "reject") => {
                let elem = Type::Tuple(vec![(**k).clone(), (**v).clone()]);
                let r = self.expr(recv)?;
                // `keep((k, v))` is called with the pair itself
                self.map_items = true;
                let f = self.gen_lambda_ex(params, body, &elem, true, false, true, None, name == "reject", lam)?;
                let pair = map_pair(k, v);
                let pair_ty = format!(
                    "({}{}, {}{})",
                    if k.is_copy() { "" } else { "&" },
                    self.rt(k),
                    if v.is_copy() { "" } else { "&" },
                    self.rt(v)
                );
                return Ok(format!(
                    "{{ let lume_keep = {}; let mut lume_m = LumeMap::new(); for (k, v) in ({}).iter() {{ if lume_keep({}) {{ lume_m.insert(k.clone(), v.clone()); }} }} lume_m }}",
                    annotate_closure(&f, &pair_ty),
                    r,
                    pair
                ));
            }
            (Type::Set(el), "filter" | "reject") => {
                let el = (**el).clone();
                let r = self.expr(recv)?;
                let by_ref = !el.is_copy();
                let f = self.gen_lambda_ex(params, body, &el, by_ref, true, true, None, name == "reject", lam)?;
                let it = if by_ref { format!("({}).iter()", r) } else { format!("({}).iter().cloned()", r) };
                let tail = if by_ref { ".cloned()" } else { "" };
                return Ok(format!("{}.filter({}){}.collect::<LumeSet<_>>()", it, f, tail));
            }
            (Type::Map(_, v), "map_values") => {
                let v = (**v).clone();
                let r = self.expr(recv)?;
                let f = self.gen_lambda(params, body, &v, false, true, true, None, lam)?;
                let f = annotate_closure(&f, &format!("&{}", self.rt(&v)));
                return Ok(format!("{{ let lume_f = {}; let mut lume_m = LumeMap::new(); for (k, v) in ({}).iter() {{ lume_m.insert(k.clone(), lume_f(v)); }} lume_m }}", f, r));
            }
            _ => {}
        }
        let (mut it, elem, mut by_ref) = self.iter_base(recv, e)?;
        // methods that hand map entries back need owned pairs, not borrowed ones
        if matches!(rt, Type::Map(..)) && matches!(name, "find" | "sort_by" | "min_by" | "max_by" | "group_by" | "partition" | "take_while") {
            self.map_items = false;
            it = format!("{}.map(|(k, v)| (k.clone(), v.clone()))", it);
            by_ref = false;
        }
        Ok(match name {
            "group_by" => {
                // the key function is called with `&item` of an owned copy
                let f = self.gen_lambda(params, body, &elem, false, true, true, None, lam)?;
                let owned = self.collect_iter_t(&it, by_ref, &elem);
                format!(
                    "{{ let lume_key = {}; let mut lume_m: LumeMap<_, Vec<_>> = LumeMap::new(); for lume_x in {} {{ let lume_k = lume_key(&lume_x); match lume_m.get_mut(&lume_k) {{ Some(v) => v.push(lume_x), None => {{ lume_m.insert(lume_k, vec![lume_x]); }} }} }} lume_m }}",
                    key_closure(&f, &self.rt(&elem)),
                    owned
                )
            }
            "partition" => {
                let f = self.gen_lambda(params, body, &elem, false, true, true, None, lam)?;
                let owned = self.collect_iter_t(&it, by_ref, &elem);
                format!("{{ let lume_test = {}; {}.into_iter().partition::<Vec<_>, _>(|lume_x| lume_test(lume_x)) }}", key_closure(&f, &self.rt(&elem)), owned)
            }
            "flat_map" => {
                let f = self.gen_lambda(params, body, &elem, by_ref, false, true, None, lam)?;
                format!("{}.flat_map({}).collect::<Vec<_>>()", it, f)
            }
            "map" => {
                let f = self.gen_lambda(params, body, &elem, by_ref, false, true, None, lam)?;
                format!("{}.map({})", it, f)
            }
            "filter" | "reject" | "take_while" => {
                let f = self.gen_lambda_ex(params, body, &elem, by_ref, true, true, None, name == "reject", lam)?;
                format!("{}.{}({})", it, if name == "reject" { "filter" } else { name }, f)
            }
            "each" => {
                let f = self.gen_lambda(params, body, &elem, by_ref, false, false, None, lam)?;
                format!("{}.for_each({})", it, f)
            }
            "sum" => {
                let bt = self.lambda_body_type(params, &elem, by_ref, None, body);
                let f = self.gen_lambda(params, body, &elem, by_ref, false, true, None, lam)?;
                let rt = if bt == Type::Float { "f64" } else { "i64" };
                format!("{}.map({}).sum::<{}>()", it, f, rt)
            }
            "count" => {
                let f = self.gen_lambda(params, body, &elem, by_ref, true, true, None, lam)?;
                format!("({}.filter({}).count() as i64)", it, f)
            }
            "any?" | "all?" => {
                let f = self.gen_lambda(params, body, &elem, by_ref, false, true, None, lam)?;
                format!("{}.{}({})", it, if name == "any?" { "any" } else { "all" }, f)
            }
            "find" => {
                let f = self.gen_lambda(params, body, &elem, by_ref, true, true, None, lam)?;
                if by_ref {
                    format!("{}.find({}).cloned()", it, f)
                } else {
                    format!("{}.find({})", it, f)
                }
            }
            "sort_by" | "min_by" | "max_by" => {
                // the key is called with `&item` of an owned copy
                let f = self.gen_lambda(params, body, &elem, false, true, true, None, lam)?;
                let owned = self.collect_iter_t(&it, by_ref, &elem);
                let key = key_closure(&f, &self.rt(&elem));
                match name {
                    "sort_by" => format!("{{ let mut v = {}; let key = {}; v.sort_by(|a, b| key(a).partial_cmp(&key(b)).unwrap()); v }}", owned, key),
                    "min_by" => format!("{{ let key = {}; {}.into_iter().min_by(|a, b| key(a).partial_cmp(&key(b)).unwrap()) }}", key, owned),
                    _ => format!("{{ let key = {}; {}.into_iter().max_by(|a, b| key(a).partial_cmp(&key(b)).unwrap()) }}", key, owned),
                }
            }
            "fold" => {
                if args.len() != 1 {
                    return Err(LumeError::new(e.line, e.col, "`fold` takes the starting value and a block: `xs.fold(0) { |acc, x| acc + x }`"));
                }
                let seed = fold_acc_type(self.ty_of(&args[0].value).materialized(), &elem);
                let acc_ty = self.fold_seed_from_body(seed, params, body, &elem, by_ref);
                let init = self.expr_owned(&args[0].value)?;
                let init = if type_is_known(&acc_ty) { format!("{{ let lume_seed: {} = {}; lume_seed }}", self.rt(&acc_ty), init) } else { init };
                let f = self.gen_lambda(params, body, &elem, by_ref, false, true, Some(&acc_ty), lam)?;
                format!("{}.fold({}, {})", it, init, f)
            }
            _ => {
                return Err(LumeError::new(e.line, e.col, format!("`{}` does not take a block", name))
                    .with_help("block methods are: map, filter, reject, each, sum, count, any?, all?, find, take_while, sort_by, min_by, max_by, fold, group_by, partition, flat_map"));
            }
        })
    }

    fn if_chain(&mut self, branches: &[(Expr, Block)], else_block: Option<&Block>, want_value: bool) -> Result<String> {
        let saved_tail = self.tail_of_fn;
        if !self.at_tail {
            self.tail_of_fn = false;
        }
        self.at_tail = false;
        let r = self.if_chain_inner(branches, else_block, want_value);
        self.tail_of_fn = saved_tail;
        r
    }

    /// The value type of a block that is one branch of an `if`/`match`, or
    /// None when the branch does not produce one (it returns, breaks, ...).
    fn branch_value(&mut self, b: &Block) -> Option<(Type, usize, usize)> {
        match b.stmts.last() {
            Some(Stmt::Expr(e)) => {
                let t = self.tail_type(b);
                if t == Type::Unknown { None } else { Some((t, e.line, e.col)) }
            }
            _ => None,
        }
    }

    /// Every branch that yields a value must agree with the first one.
    fn check_branches(&mut self, what: &str, values: Vec<(Type, usize, usize)>) -> Result<()> {
        // At a function's tail, a branch may give the plain value, the error, or `None`:
        // each is the implied form of the declared result type.
        // A block the program declared to give back a `T or E` ends the way
        // a function does, so its branches may give the value or the error.
        let block_result = matches!(&self.block_ret, Some(Type::Result(..)) | Some(Type::Option(_)));
        let ret = if block_result { self.block_ret.clone().unwrap_or(Type::Unknown) } else { self.current_ret.clone() };
        let at_tail = self.tail_of_fn && (!self.in_block || block_result);
        let normalize = |g: &Self, t: Type| -> Type {
            if !at_tail {
                return t;
            }
            match &ret {
                // the value part is still open, so any branch value is it
                Type::Result(ok, _) if matches!(**ok, Type::Var(_) | Type::Unknown) => ret.clone(),
                Type::Result(ok, err) if g.assignable(&t, ok) || g.assignable(&t, err) || g.assignable(&t, &ret) => ret.clone(),
                Type::Option(inner) if g.assignable(&t, inner) || g.assignable(&t, &ret) => ret.clone(),
                _ => t,
            }
        };
        let values: Vec<(Type, usize, usize)> = values.into_iter().map(|(t, l, c)| (normalize(self, t), l, c)).collect();
        let mut first: Option<(Type, usize)> = None;
        for (t, line, col) in values {
            match &first {
                None => first = Some((t, line)),
                Some((ft, fl)) => {
                    let str_char = (*ft == Type::Str && t == Type::Char) || (*ft == Type::Char && t == Type::Str);
                    if str_char {
                        return Err(LumeError::new(line, col, format!("the branches of this `{}` give different types: `{}` on line {} and `{}` here", what, type_name(ft), fl, type_name(&t)))
                            .with_help("a `Char` becomes a `Str` with `.to_s`; give every branch the same type"));
                    }
                    if !(self.assignable(&t, ft) || self.assignable(ft, &t)) {
                        return Err(LumeError::new(line, col, format!("the branches of this `{}` give different types: `{}` on line {} and `{}` here", what, type_name(ft), fl, type_name(&t)))
                            .with_help("a value chosen by a branch must have one type; convert one side, or return an `Error`/`None` instead"));
                    }
                }
            }
        }
        Ok(())
    }

    fn if_chain_inner(&mut self, branches: &[(Expr, Block)], else_block: Option<&Block>, want_value: bool) -> Result<String> {
        if want_value {
            let mut values = Vec::new();
            for (_, body) in branches {
                if let Some(v) = self.branch_value(body) {
                    values.push(v);
                }
            }
            if let Some(eb) = else_block {
                if let Some(v) = self.branch_value(eb) {
                    values.push(v);
                }
            }
            self.check_branches("if", values)?;
        }
        let saved = std::mem::take(&mut self.out);
        let base = self.indent;
        let mut first = true;
        for (i, (cond, body)) in branches.iter().enumerate() {
            // the branches still to come use what the condition mentions, so
            // a name the condition consumes must be copied, not moved
            let later: Vec<Stmt> = branches[i..]
                .iter()
                .flat_map(|(_, b)| b.stmts.iter().cloned())
                .chain(branches[i + 1..].iter().map(|(c, _)| Stmt::Expr(c.clone())))
                .chain(else_block.iter().flat_map(|b| b.stmts.iter().cloned()))
                .collect();
            self.rest_stack.push(later);
            let c = self.expr(cond);
            self.rest_stack.pop();
            let c = c?;
            if first {
                self.out.push_str(&format!("if {} {{\n", c));
                first = false;
            } else {
                self.line(&format!("}} else if {} {{", c));
            }
            self.nested_block(body, want_value)?;
        }
        if let Some(eb) = else_block {
            self.line("} else {");
            self.nested_block(eb, want_value)?;
        }
        self.out.push_str(&"    ".repeat(base));
        self.out.push('}');
        Ok(std::mem::replace(&mut self.out, saved))
    }

    // ----- match --------------------------------------------------------------

    fn match_expr(&mut self, scrutinee: &Expr, arms: &[MatchArm], want_value: bool, e: &Expr) -> Result<String> {
        let saved_tail = self.tail_of_fn;
        if !self.at_tail {
            self.tail_of_fn = false;
        }
        self.at_tail = false;
        let r = self.match_expr_inner(scrutinee, arms, want_value, e);
        self.tail_of_fn = saved_tail;
        r
    }

    fn match_expr_inner(&mut self, scrutinee: &Expr, arms: &[MatchArm], want_value: bool, e: &Expr) -> Result<String> {
        let st = self.ty_of(scrutinee).materialized();
        if st == Type::Unknown {
            return Err(LumeError::new(scrutinee.line, scrutinee.col, "cannot tell the type of the value being matched")
                .with_help("bind it to a name with a known type first"));
        }
        if want_value {
            let mut values = Vec::new();
            for arm in arms {
                self.push_scope();
                let _ = self.declare_pattern_types(&arm.pat, &st);
                let v = self.branch_value(&arm.body);
                self.pop_scope();
                if let Some(v) = v {
                    values.push(v);
                }
            }
            self.check_branches("match", values)?;
        }
        let uses_list_pat = arms.iter().any(|a| pat_has_list(&a.pat));
        let s = self.expr_val(scrutinee)?;
        // How the scrutinee is matched: by value for Copy types, otherwise by reference.
        let by_ref = !st.is_copy();
        let scrut = if uses_list_pat {
            if !matches!(st, Type::List(_)) {
                return Err(LumeError::new(e.line, e.col, format!("a list pattern cannot match a `{}`", type_name(&st))));
            }
            format!("({}).as_slice()", s)
        } else if by_ref && !self.is_borrowed_ident(scrutinee) {
            format!("&({})", s)
        } else {
            format!("({})", s)
        };
        self.check_exhaustive(&st, arms, e)?;

        let saved = std::mem::take(&mut self.out);
        let base = self.indent;
        self.out.push_str(&format!("match {} {{\n", scrut));
        self.indent += 1;
        for arm in arms {
            let cp = self.compile_pattern(&arm.pat, &st, by_ref)?;
            self.push_scope();
            let mut rebinds: Vec<String> = Vec::new();
            for (name, ty, kind) in &cp.binds {
                match kind {
                    BindKind::Deref => {
                        rebinds.push(format!("let {} = *{};", rust_name(name), rust_name(name)));
                        self.declare(name, false, false, ty.clone(), arm.line);
                    }
                    BindKind::Slice => {
                        rebinds.push(format!("let {} = {}.to_vec();", rust_name(name), rust_name(name)));
                        self.declare(name, false, false, ty.clone(), arm.line);
                    }
                    BindKind::Boxed => {
                        rebinds.push(format!("let {} = &**{};", rust_name(name), rust_name(name)));
                        self.declare(name, false, true, ty.clone(), arm.line);
                    }
                    BindKind::Ref => self.declare(name, false, true, ty.clone(), arm.line),
                    BindKind::Owned => self.declare(name, false, false, ty.clone(), arm.line),
                }
            }
            let mut guards = cp.guards.clone();
            if let Some(g) = &arm.guard {
                // In the guard the rebinds have not run yet: Copy bindings are still references.
                for (name, ty, kind) in &cp.binds {
                    if *kind == BindKind::Deref {
                        self.declare(name, false, true, ty.clone(), arm.line);
                    }
                }
                guards.push(self.expr(g)?);
                for (name, ty, kind) in &cp.binds {
                    if *kind == BindKind::Deref {
                        self.declare(name, false, false, ty.clone(), arm.line);
                    }
                }
            }
            let guard_text = if guards.is_empty() { String::new() } else { format!(" if {}", guards.join(" && ")) };
            self.line(&format!("{}{} => {{", cp.text, guard_text));
            self.indent += 1;
            for r in rebinds {
                self.line(&r);
            }
            // names a pattern reached through a `Box` get their values here
            for l in &cp.lets {
                self.line(l);
            }
            self.block_body(&arm.body, want_value)?;
            self.indent -= 1;
            self.line("}");
            self.pop_scope();
        }
        self.indent -= 1;
        self.out.push_str(&"    ".repeat(base));
        self.out.push('}');
        let _ = base;
        Ok(std::mem::replace(&mut self.out, saved))
    }

    fn compile_pattern(&mut self, p: &Pattern, t: &Type, by_ref: bool) -> Result<CompiledPat> {
        let mut cp = CompiledPat { text: String::new(), guards: Vec::new(), binds: Vec::new(), lets: Vec::new() };
        self.compile_pat_into(p, t, by_ref, &mut cp)?;
        Ok(cp)
    }

    fn compile_pat_into(&mut self, p: &Pattern, t: &Type, by_ref: bool, cp: &mut CompiledPat) -> Result<()> {
        let text = match &p.kind {
            PatKind::Wild => "_".to_string(),
            PatKind::Or(alts) => {
                // `"a" | "b"` and `1.0 | 2.0` compare against one binding
                if alts.iter().all(|a| matches!(a.kind, PatKind::Str(_))) && *t != Type::Char {
                    self.expect_pat_type(t, &Type::Str, p, "a string")?;
                    let tmp = self.fresh("s");
                    let tests: Vec<String> = alts
                        .iter()
                        .map(|a| match &a.kind {
                            PatKind::Str(s) => format!("{} == \"{}\"", tmp, escape_rust_str(s, false)),
                            _ => unreachable!(),
                        })
                        .collect();
                    cp.guards.push(format!("({})", tests.join(" || ")));
                    return Ok(cp.text = tmp);
                }
                if alts.iter().all(|a| matches!(a.kind, PatKind::Float(_))) {
                    self.expect_pat_type(t, &Type::Float, p, "a float")?;
                    let tmp = self.fresh("f");
                    let deref = if by_ref { "*" } else { "" };
                    let tests: Vec<String> = alts
                        .iter()
                        .map(|a| match &a.kind {
                            PatKind::Float(v) => format!("{}{} == {:?}", deref, tmp, v),
                            _ => unreachable!(),
                        })
                        .collect();
                    cp.guards.push(format!("({})", tests.join(" || ")));
                    return Ok(cp.text = tmp);
                }
                let mut texts = Vec::new();
                let mut alt_guards: Vec<Vec<String>> = Vec::new();
                let mut first: Option<Vec<(String, Type, BindKind)>> = None;
                let tmp_before = self.tmp;
                for a in alts {
                    // every alternative names its temporaries the same way, so
                    // `Some("a") | Some("e")` is one pattern with two tests
                    self.tmp = tmp_before;
                    let mut sub = CompiledPat { text: String::new(), guards: Vec::new(), binds: Vec::new(), lets: Vec::new() };
                    self.compile_pat_into(a, t, by_ref, &mut sub)?;
                    alt_guards.push(std::mem::take(&mut sub.guards));
                    let mut names: Vec<&String> = sub.binds.iter().map(|(n, _, _)| n).collect();
                    names.sort();
                    match &first {
                        None => first = Some(sub.binds.clone()),
                        Some(f) => {
                            let mut fnames: Vec<&String> = f.iter().map(|(n, _, _)| n).collect();
                            fnames.sort();
                            if names != fnames {
                                return Err(LumeError::new(a.line, a.col, "every alternative of `|` must bind the same names")
                                    .with_help(format!(
                                        "the first alternative binds {}, this one binds {}",
                                        describe_names(&fnames),
                                        describe_names(&names)
                                    )));
                            }
                            for (n, ty, kind) in &sub.binds {
                                let (_, fty, fkind) = f.iter().find(|(fnm, _, _)| fnm == n).unwrap();
                                if fty != ty || fkind != kind {
                                    return Err(LumeError::new(a.line, a.col, format!(
                                        "`{}` is a `{}` in one alternative and a `{}` in another",
                                        n,
                                        type_name(fty),
                                        type_name(ty)
                                    )));
                                }
                            }
                        }
                    }
                    texts.push(sub.text);
                }
                cp.binds.extend(first.unwrap_or_default());
                if alt_guards.iter().any(|g| !g.is_empty()) {
                    // literals inside the alternatives: the shapes must agree,
                    // and the tests become one `||` guard
                    if texts.iter().any(|t| *t != texts[0]) {
                        return Err(LumeError::new(p.line, p.col, "alternatives with a string or float inside must otherwise look the same")
                            .with_help("write `Some(\"a\") | Some(\"e\")` (same shape), or separate arms"));
                    }
                    let tests: Vec<String> = alt_guards.iter().map(|g| if g.is_empty() { "true".to_string() } else { g.join(" && ") }).collect();
                    cp.guards.push(format!("({})", tests.join(" || ")));
                    return Ok(cp.text = texts.remove(0));
                }
                format!("({})", texts.join(" | "))
            }
            PatKind::Bind(n) => {
                let kind = if !by_ref {
                    BindKind::Owned
                } else if t.is_copy() {
                    BindKind::Deref
                } else {
                    BindKind::Ref
                };
                cp.binds.push((n.clone(), t.clone(), kind));
                rust_name(n)
            }
            PatKind::Int(v) => {
                self.expect_pat_type(t, &Type::Int, p, "an integer")?;
                format!("{}", v)
            }
            PatKind::Bool(b) => {
                self.expect_pat_type(t, &Type::Bool, p, "a boolean")?;
                b.to_string()
            }
            PatKind::Range { lo, hi, inclusive } => {
                self.expect_pat_type(t, &Type::Int, p, "a range")?;
                if *inclusive { format!("{}..={}", lo, hi) } else { format!("{}..{}", lo, hi) }
            }
            PatKind::Float(v) => {
                self.expect_pat_type(t, &Type::Float, p, "a float")?;
                let tmp = self.fresh("f");
                let deref = if by_ref { "*" } else { "" };
                cp.guards.push(format!("{}{} == {:?}", deref, tmp, v));
                tmp
            }
            PatKind::Str(s) if *t == Type::Char => {
                // a one-character string matches a character directly
                let mut cs = s.chars();
                match (cs.next(), cs.next()) {
                    (Some(c), None) => format!("'{}'", rust_char(c)),
                    _ => {
                        return Err(LumeError::new(p.line, p.col, format!("this pattern is the string \"{}\", but the value is a single `Char`", s))
                            .with_help("match one character, like `\"a\"`, or turn the value into a string first with `.to_s`"));
                    }
                }
            }
            PatKind::Str(s) => {
                self.expect_pat_type(t, &Type::Str, p, "a string")?;
                let tmp = self.fresh("s");
                cp.guards.push(format!("{} == \"{}\"", tmp, escape_rust_str(s, false)));
                tmp
            }
            PatKind::Tuple(items) => {
                let ts = match t {
                    Type::Tuple(ts) => ts.clone(),
                    Type::Unknown => vec![Type::Unknown; items.len()],
                    other => return Err(LumeError::new(p.line, p.col, format!("a tuple pattern cannot match a `{}`", type_name(other)))),
                };
                if ts.len() != items.len() {
                    return Err(LumeError::new(p.line, p.col, format!("this tuple has {} parts, but the pattern names {}", ts.len(), items.len())));
                }
                let mut parts = Vec::new();
                for (it, ty) in items.iter().zip(&ts) {
                    let mut sub = CompiledPat { text: String::new(), guards: Vec::new(), binds: Vec::new(), lets: Vec::new() };
                    self.compile_pat_into(it, ty, by_ref, &mut sub)?;
                    parts.push(sub.text);
                    cp.guards.extend(sub.guards);
                    cp.binds.extend(sub.binds);
                    cp.lets.extend(sub.lets);
                }
                format!("({})", parts.join(", "))
            }
            PatKind::List { items, rest } => {
                let elem = match t {
                    Type::List(e) => (**e).clone(),
                    other => return Err(LumeError::new(p.line, p.col, format!("a list pattern cannot match a `{}`", type_name(other)))),
                };
                let mut parts = Vec::new();
                for it in items {
                    let mut sub = CompiledPat { text: String::new(), guards: Vec::new(), binds: Vec::new(), lets: Vec::new() };
                    // slice elements are always reached by reference
                    self.compile_pat_into(it, &elem, true, &mut sub)?;
                    parts.push(sub.text);
                    cp.guards.extend(sub.guards);
                    cp.binds.extend(sub.binds);
                    cp.lets.extend(sub.lets);
                }
                match rest {
                    Some(Some(r)) => {
                        cp.binds.push((r.clone(), Type::List(Box::new(elem)), BindKind::Slice));
                        parts.push(format!("{} @ ..", rust_name(r)));
                    }
                    Some(None) => parts.push("..".into()),
                    None => {}
                }
                format!("[{}]", parts.join(", "))
            }
            PatKind::Variant { enum_name, name, args, rest } => {
                // Result
                if (name == "Ok" || name == "Error") && matches!(t, Type::Result(..)) {
                    let (ok_t, err_t) = match t {
                        Type::Result(a, b) => ((**a).clone(), (**b).clone()),
                        _ => unreachable!(),
                    };
                    if args.len() != 1 {
                        return Err(LumeError::new(p.line, p.col, format!("`{}` takes exactly one pattern, like `{}(x)`", name, name)));
                    }
                    let inner_t = if name == "Ok" { ok_t } else { err_t };
                    let mut sub = CompiledPat { text: String::new(), guards: Vec::new(), binds: Vec::new(), lets: Vec::new() };
                    self.compile_pat_into(&args[0], &inner_t, by_ref, &mut sub)?;
                    cp.guards.extend(sub.guards);
                    cp.binds.extend(sub.binds);
                    cp.lets.extend(sub.lets);
                    cp.text = format!("{}({})", if name == "Ok" { "Ok" } else { "Err" }, sub.text);
                    return Ok(());
                }
                let enum_has_it = matches!(t, Type::Named(n) if self.enums.get(n).map(|e| e.variants.iter().any(|(v, _)| v == name)).unwrap_or(false));
                if (name == "Ok" || name == "Error") && *t != Type::Unknown && !enum_has_it && !matches!(t, Type::Named(n) if n == "Error") {
                    return Err(LumeError::new(p.line, p.col, format!("`{}` matches a `T or E` value, but this is a `{}`", name, type_name(t))));
                }
                // Option
                if name == "Some" || name == "None" {
                    let inner = match t {
                        Type::Option(i) => (**i).clone(),
                        other => {
                            return Err(LumeError::new(p.line, p.col, format!("`{}` matches an optional value, but this is a `{}`", name, type_name(other))));
                        }
                    };
                    if name == "None" {
                        if !args.is_empty() {
                            return Err(LumeError::new(p.line, p.col, "`None` carries no value"));
                        }
                        "None".to_string()
                    } else {
                        if args.len() != 1 {
                            return Err(LumeError::new(p.line, p.col, "`Some` takes exactly one pattern, like `Some(x)`"));
                        }
                        let mut sub = CompiledPat { text: String::new(), guards: Vec::new(), binds: Vec::new(), lets: Vec::new() };
                        self.compile_pat_into(&args[0], &inner, by_ref, &mut sub)?;
                        cp.guards.extend(sub.guards);
                        cp.binds.extend(sub.binds);
                        format!("Some({})", sub.text)
                    }
                } else {
                    let en = match (enum_name, t) {
                        (Some(en), _) => self.canon(en),
                        (None, Type::Named(en) | Type::App(en, _)) if self.enums.contains_key(&self.canon(en)) => self.canon(en),
                        (None, _) => match self.resolve_variant(name, p.line, p.col)? {
                            Some(en) => en,
                            None => {
                                return Err(LumeError::new(p.line, p.col, format!("unknown variant `{}`", name))
                                    .with_help("variants start with a capital letter and belong to an `enum`; lowercase names are bindings"));
                            }
                        },
                    };
                    if let Type::Named(a0) | Type::App(a0, _) = t {
                        let actual = &self.canon(a0);
                        if *actual != en {
                            return Err(LumeError::new(p.line, p.col, format!("`{}` is a variant of `{}`, but the value is a `{}`", name, en, type_name(t))));
                        }
                        // A bare name that is not a variant of this enum but is one of another
                        if enum_name.is_none() && !self.enums[actual].variants.iter().any(|(n, _)| n == name) {
                            let owners = self.enums_with_variant(name);
                            if !owners.is_empty() {
                                return Err(LumeError::new(p.line, p.col, format!("`{}` is a variant of `{}`, but the value is a `{}`", name, owners.join("` and `"), actual))
                                    .with_help(format!("the variants of `{}` are: {}", actual, self.enums[actual].variants.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", "))));
                            }
                        }
                    } else if *t != Type::Unknown {
                        return Err(LumeError::new(p.line, p.col, format!("`{}` is a variant of `{}`, but the value is a `{}`", name, en, type_name(t))));
                    }
                    let info = self.enums[&en].clone();
                    let vsub = self.subst_for(t);
                    let declared: Vec<Type> = info.variants.iter().find(|(n, _)| n == name).map(|(_, fs)| fs.iter().map(|(_, t)| t.clone()).collect()).unwrap_or_default();
                    let (_, fields) = match info.variants.iter().find(|(n, _)| n == name) {
                        Some((vn, fs)) => (vn.clone(), fs.iter().map(|(fname, ft)| (fname.clone(), Self::subst(ft, &vsub))).collect::<Vec<_>>()),
                        None => {
                            let e = LumeError::new(p.line, p.col, format!("`{}` has no variant `{}`", en, name));
                            let names: Vec<String> = info.variants.iter().map(|(n, _)| n.clone()).collect();
                            return Err(match self.suggest_from(name, names.iter().cloned()) {
                                Some(s) => e.with_help(format!("did you mean `{}`?", s)),
                                None => e.with_help(format!("the variants are: {}", names.join(", "))),
                            });
                        }
                    };
                    let epath = self.path_of(&en);
                    if fields.is_empty() {
                        if !args.is_empty() && !*rest {
                            return Err(LumeError::new(p.line, p.col, format!("`{}` carries no values; write it without parentheses", name)));
                        }
                        format!("{}::{}", epath, name)
                    } else {
                        let mut args: Vec<Pattern> = args.clone();
                        if *rest {
                            while args.len() < fields.len() {
                                args.push(Pattern { kind: PatKind::Wild, line: p.line, col: p.col });
                            }
                        }
                        let args = &args;
                        if args.len() != fields.len() {
                            return Err(LumeError::new(p.line, p.col, format!(
                                "`{}` carries {} {}, but the pattern names {}",
                                name,
                                fields.len(),
                                plural(fields.len(), "value", "values"),
                                args.len()
                            ))
                            .with_help(format!("write `{}({})`; use `_` for values you do not need", name, fields.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", "))));
                        }
                        let mut parts = Vec::new();
                        for (fi, (a, (fname, fty))) in args.iter().zip(&fields).enumerate() {
                            let mut sub = CompiledPat { text: String::new(), guards: Vec::new(), binds: Vec::new(), lets: Vec::new() };
                            if self.boxed_field(&en, declared.get(fi).unwrap_or(fty)) {
                                // a recursive field sits behind a Box: bind it (or ignore it), no nested pattern
                                match &a.kind {
                                    PatKind::Bind(n) => {
                                        sub.text = rust_name(n);
                                        sub.binds.push((n.clone(), fty.clone(), if by_ref { BindKind::Boxed } else { BindKind::Owned }));
                                    }
                                    PatKind::Wild => sub.text = "_".into(),
                                    _ => {
                                        // the field is behind a `Box`: bind it, test
                                        // it in the guard, and take it apart in the arm
                                        let tmp = self.fresh("bx");
                                        let inner = self.compile_pattern(a, fty, true)?;
                                        let cond = if inner.guards.is_empty() { String::new() } else { format!(" if {}", inner.guards.join(" && ")) };
                                        sub.text = rust_name(&tmp);
                                        sub.guards.push(format!("matches!(&**{}, {}{})", rust_name(&tmp), inner.text, cond));
                                        if !inner.binds.is_empty() {
                                            let names: Vec<String> = inner.binds.iter().map(|(n, _, _)| rust_name(n)).collect();
                                            let values: Vec<String> = inner
                                                .binds
                                                .iter()
                                                .map(|(n, _, k)| match k {
                                                    BindKind::Deref => format!("*{}", rust_name(n)),
                                                    BindKind::Boxed => format!("&**{}", rust_name(n)),
                                                    BindKind::Slice => format!("{}.to_vec()", rust_name(n)),
                                                    _ => rust_name(n),
                                                })
                                                .collect();
                                            sub.lets.push(format!(
                                                "let ({},) = match &**{} {{ {}{} => ({},), _ => unreachable!() }};",
                                                names.join(", "),
                                                rust_name(&tmp),
                                                inner.text,
                                                cond,
                                                values.join(", ")
                                            ));
                                            for (n, t, k) in inner.binds {
                                                let kind = match k {
                                                    BindKind::Deref | BindKind::Owned | BindKind::Slice => BindKind::Owned,
                                                    _ => BindKind::Ref,
                                                };
                                                sub.binds.push((n, t, kind));
                                            }
                                        }
                                        inner.lets.into_iter().for_each(|l| sub.lets.push(l));
                                    }
                                }
                            } else {
                                self.compile_pat_into(a, fty, by_ref, &mut sub)?;
                            }
                            parts.push(format!("{}: {}", rust_name(fname), sub.text));
                            cp.guards.extend(sub.guards);
                            cp.binds.extend(sub.binds);
                            cp.lets.extend(sub.lets);
                        }
                        format!("{}::{} {{ {} }}", epath, name, parts.join(", "))
                    }
                }
            }
        };
        cp.text = text;
        Ok(())
    }

    fn expect_pat_type(&self, actual: &Type, want: &Type, p: &Pattern, what: &str) -> Result<()> {
        if actual == want || *actual == Type::Unknown {
            Ok(())
        } else {
            Err(LumeError::new(p.line, p.col, format!("this pattern is {}, but the value is a `{}`", what, type_name(actual))))
        }
    }

    fn check_exhaustive(&mut self, t: &Type, arms: &[MatchArm], e: &Expr) -> Result<()> {
        // Rows of the pattern matrix: the unguarded arms, `|` expanded. An arm
        // that adds nothing to the rows above it can never run.
        let mut rows: Vec<Vec<Pat>> = Vec::new();
        for a in arms {
            let pats = self.flat_pats(&a.pat, t);
            let reachable = pats.iter().any(|p| self.useful(&rows, std::slice::from_ref(p), std::slice::from_ref(t)).is_some());
            if !reachable && !rows.is_empty() {
                self.warnings.push(
                    LumeError::new(a.line, a.col, "this arm can never match: the arms above it already cover these values")
                        .with_help("remove it, or move it above the arm that shadows it"),
                );
            }
            if a.guard.is_none() {
                for p in pats {
                    rows.push(vec![p]);
                }
            }
        }
        // Each witness found becomes a row, so the next search finds a different one.
        let mut missing: Vec<String> = Vec::new();
        while missing.len() < 6 {
            match self.useful(&rows, &[Pat::Wild], &[t.clone()]) {
                None => break,
                Some(w) => {
                    missing.push(pat_text(&w[0]));
                    rows.push(w);
                }
            }
        }
        if missing.is_empty() {
            return Ok(());
        }
        let more = missing.len() == 6;
        if more {
            missing.pop();
        }
        // a recursive type has endless shapes: show the simplest ones
        missing.sort_by_key(|m| (m.chars().count(), m.clone()));
        let more = more || missing.len() > 3;
        missing.truncate(3);
        let listed = if more { format!("{}, ...", missing.join(", ")) } else { missing.join(", ") };
        let err = LumeError::new(e.line, e.col, format!("this `match` on `{}` does not cover: {}", type_name(t), listed));
        Err(if missing == ["_"] {
            err.with_help("values of this type have too many cases to list; add a final `_ ->` arm")
        } else if more {
            err.with_help("add arms for the missing cases, or a final `_ ->` arm")
        } else {
            err.with_help(format!("add {} for {}, or a final `_ ->` arm", plural(missing.len(), "an arm", "arms"), missing.join(" and ")))
        })
    }

    /// The two ends of `lo..hi` / `lo...hi` used as a slice, as exclusive Rust bounds.
    fn slice_bounds(&mut self, lo: &Expr, hi: &Expr, inclusive: bool) -> Result<(String, String)> {
        for x in [lo, hi] {
            let t = self.ty_of(x);
            if t != Type::Int && t != Type::Unknown {
                return Err(LumeError::new(x.line, x.col, format!("a slice position is an `Int`, but this is a `{}`", type_name(&t))));
            }
        }
        let a = self.expr_val(lo)?;
        let b = self.expr_val(hi)?;
        Ok((a, if inclusive { format!("({}).saturating_add(1)", b) } else { b }))
    }

    /// Map keys and set items are compared by hashing, so a `Float` (which
    /// has no total equality) and types that hold one are not allowed.
    fn check_key_type(&self, t: &Type, what: &str, line: usize, col: usize) -> Result<()> {
        if let Type::Var(n) = t {
            if self.type_param(n).and_then(|p| p.bound.clone()).map(|b| Self::bound_name(&b) == "Hashable").unwrap_or(false) {
                return Ok(());
            }
            return Err(LumeError::new(line, col, format!("`{}` could be any type, so it cannot be {} of a {}", n, if what == "map" { "the key" } else { "an item" }, what))
                .with_help(format!("declare it `[{}: Hashable]`, which promises every type it is given can be a key", n)));
        }
        if self.key_ok(t, &mut Vec::new()) {
            Ok(())
        } else {
            let e = LumeError::new(line, col, format!("a `{}` cannot be {} of a {}", type_name(t), if what == "map" { "the key" } else { "an item" }, what));
            Err(match t {
                Type::Float => e.with_help("floats have no exact equality; use an `Int` (cents, thousandths) or a `Str`"),
                Type::Named(_) => e.with_help("only a struct or enum whose fields are all `Int`, `Str`, `Bool`, tuples or other such types can be hashed; and not one with its own `==`"),
                _ => e.with_help("keys and items must be `Int`, `Str`, `Bool`, tuples of those, or a struct/enum made of them"),
            })
        }
    }

    fn key_ok(&self, t: &Type, visiting: &mut Vec<String>) -> bool {
        match t {
            Type::Int | Type::Bool | Type::Str | Type::Char | Type::Unit | Type::Unknown => true,
            Type::Tuple(ts) => ts.iter().all(|x| self.key_ok(x, visiting)),
            Type::Option(i) => self.key_ok(i, visiting),
            Type::Named(n) => {
                let key = self.canon(n);
                // a recursive type refers back to itself: fine, the field is boxed
                if visiting.contains(&key) {
                    return true;
                }
                visiting.push(key.clone());
                let ok = if let Some(s) = self.structs.get(&key) {
                    !s.methods.contains_key("==") && s.fields.iter().all(|(_, t)| self.key_ok(t, visiting))
                } else if let Some(e) = self.enums.get(&key) {
                    !e.methods.contains_key("==") && e.variants.iter().all(|(_, fs)| fs.iter().all(|(_, t)| self.key_ok(t, visiting)))
                } else {
                    false
                };
                visiting.pop();
                ok
            }
            _ => false,
        }
    }

    /// A user type that derives `Hash + Eq`: every field hashable, no custom `==`.
    fn hashable_named(&self, n: &str) -> bool {
        self.key_ok(&Type::Named(n.to_string()), &mut Vec::new())
    }

    /// The constructors of a type, with their argument types; `None` when
    /// there are too many to enumerate (numbers, strings, unknown types).
    fn ctors(&self, t: &Type) -> Option<Vec<(String, Vec<Type>)>> {
        Some(match t {
            Type::Named(en) | Type::App(en, _) => {
                let info = self.enums.get(&self.canon(en))?;
                let sub = self.subst_for(t);
                info.variants.iter().map(|(n, fs)| (n.clone(), fs.iter().map(|(_, ft)| Self::subst(ft, &sub)).collect())).collect()
            }
            Type::Option(i) => vec![("Some".into(), vec![(**i).clone()]), ("None".into(), vec![])],
            Type::Result(o, e) => vec![("Ok".into(), vec![(**o).clone()]), ("Error".into(), vec![(**e).clone()])],
            Type::Bool => vec![("true".into(), vec![]), ("false".into(), vec![])],
            Type::Tuple(ts) => vec![("()".into(), ts.clone())],
            // a list is `[]` or an element followed by a list
            Type::List(i) => vec![("[]".into(), vec![]), ("::".into(), vec![(**i).clone(), t.clone()])],
            _ => return None,
        })
    }

    /// Converts a source pattern into matrix patterns (one per `|` combination).
    fn flat_pats(&self, p: &Pattern, t: &Type) -> Vec<Pat> {
        match &p.kind {
            PatKind::Wild | PatKind::Bind(_) => vec![Pat::Wild],
            PatKind::Int(_) | PatKind::Float(_) | PatKind::Str(_) | PatKind::Range { .. } => vec![Pat::Lit],
            PatKind::Bool(b) => vec![Pat::Ctor(b.to_string(), vec![])],
            PatKind::Or(alts) => alts.iter().flat_map(|a| self.flat_pats(a, t)).collect(),
            PatKind::Tuple(items) => {
                let ts: Vec<Type> = match t {
                    Type::Tuple(ts) => ts.clone(),
                    _ => vec![Type::Unknown; items.len()],
                };
                let subs: Vec<Vec<Pat>> = items.iter().zip(&ts).map(|(i, it)| self.flat_pats(i, it)).collect();
                if !matches!(t, Type::Tuple(_)) && subs.iter().all(|s| s == &[Pat::Wild]) {
                    return vec![Pat::Wild];
                }
                product(subs).into_iter().map(|args| Pat::Ctor("()".into(), args)).collect()
            }
            PatKind::List { items, rest } => {
                let elem = match t {
                    Type::List(e) => (**e).clone(),
                    _ => Type::Unknown,
                };
                let subs: Vec<Vec<Pat>> = items.iter().map(|i| self.flat_pats(i, &elem)).collect();
                product(subs)
                    .into_iter()
                    .map(|args| {
                        let mut tail = if rest.is_some() { Pat::Wild } else { Pat::Ctor("[]".into(), vec![]) };
                        for a in args.into_iter().rev() {
                            tail = Pat::Ctor("::".into(), vec![a, tail]);
                        }
                        tail
                    })
                    .collect()
            }
            PatKind::Variant { name, args, rest, .. } => {
                let _ = rest;
                let arg_tys: Vec<Type> = match self.ctors(t) {
                    Some(cs) => match cs.into_iter().find(|(n, _)| n == name) {
                        Some((_, tys)) => tys,
                        None => return vec![Pat::Lit],
                    },
                    None => vec![Type::Unknown; args.len()],
                };
                let mut subs: Vec<Vec<Pat>> = args.iter().zip(&arg_tys).map(|(a, at)| self.flat_pats(a, at)).collect();
                while subs.len() < arg_tys.len() {
                    subs.push(vec![Pat::Wild]);
                }
                product(subs).into_iter().map(|args| Pat::Ctor(name.clone(), args)).collect()
            }
        }
    }

    /// Maranget's usefulness: a value matched by `q` but by no row of
    /// `rows`, if one exists. Called with `q = [_]` it answers "is the match
    /// exhaustive?" and produces a witness when it is not.
    fn useful(&self, rows: &[Vec<Pat>], q: &[Pat], tys: &[Type]) -> Option<Vec<Pat>> {
        if q.is_empty() {
            return if rows.is_empty() { Some(Vec::new()) } else { None };
        }
        let t = &tys[0];
        let arity_of = |c: &str, cs: &Option<Vec<(String, Vec<Type>)>>| -> Vec<Type> {
            cs.as_ref()
                .and_then(|cs| cs.iter().find(|(n, _)| n == c).map(|(_, tys)| tys.clone()))
                .unwrap_or_else(|| {
                    // constructor of an unknown type: take the arity from a row
                    let n = rows
                        .iter()
                        .find_map(|r| match &r[0] {
                            Pat::Ctor(rc, args) if rc == c => Some(args.len()),
                            _ => None,
                        })
                        .unwrap_or(0);
                    vec![Type::Unknown; n]
                })
        };
        let cs = self.ctors(t);
        let specialize = |c: &str, arity: usize| -> Vec<Vec<Pat>> {
            rows.iter()
                .filter_map(|r| match &r[0] {
                    Pat::Ctor(rc, args) if rc == c => {
                        let mut nr = args.clone();
                        nr.extend_from_slice(&r[1..]);
                        Some(nr)
                    }
                    Pat::Wild => {
                        let mut nr = vec![Pat::Wild; arity];
                        nr.extend_from_slice(&r[1..]);
                        Some(nr)
                    }
                    _ => None,
                })
                .collect()
        };
        let default_rows = || -> Vec<Vec<Pat>> { rows.iter().filter(|r| r[0] == Pat::Wild).map(|r| r[1..].to_vec()).collect() };
        let rebuild = |c: &str, arity: usize, w: Vec<Pat>| -> Vec<Pat> {
            let mut out = vec![Pat::Ctor(c.to_string(), w[..arity].to_vec())];
            out.extend_from_slice(&w[arity..]);
            out
        };
        match &q[0] {
            Pat::Ctor(c, args) => {
                let arg_tys = arity_of(c, &cs);
                let mut nq = args.clone();
                nq.extend_from_slice(&q[1..]);
                let mut nt = arg_tys.clone();
                nt.extend_from_slice(&tys[1..]);
                self.useful(&specialize(c, arg_tys.len()), &nq, &nt).map(|w| rebuild(c, arg_tys.len(), w))
            }
            Pat::Lit => {
                // no row's constructor matches a literal in the query except a wildcard
                let mut out = vec![Pat::Lit];
                out.extend(self.useful(&default_rows(), &q[1..], &tys[1..])?);
                Some(out)
            }
            Pat::Wild => {
                let used: Vec<String> = rows
                    .iter()
                    .filter_map(|r| match &r[0] {
                        Pat::Ctor(c, _) => Some(c.clone()),
                        _ => None,
                    })
                    .collect();
                let complete = match &cs {
                    Some(cs) => !cs.is_empty() && cs.iter().all(|(c, _)| used.contains(c)),
                    None => false,
                };
                if complete {
                    for (c, arg_tys) in cs.as_ref().unwrap() {
                        let mut nq = vec![Pat::Wild; arg_tys.len()];
                        nq.extend_from_slice(&q[1..]);
                        let mut nt = arg_tys.clone();
                        nt.extend_from_slice(&tys[1..]);
                        if let Some(w) = self.useful(&specialize(c, arg_tys.len()), &nq, &nt) {
                            return Some(rebuild(c, arg_tys.len(), w));
                        }
                    }
                    None
                } else {
                    let rest = self.useful(&default_rows(), &q[1..], &tys[1..])?;
                    let head = match &cs {
                        Some(cs) => {
                            let (c, arg_tys) = cs.iter().find(|(c, _)| !used.contains(c)).unwrap();
                            Pat::Ctor(c.clone(), vec![Pat::Wild; arg_tys.len()])
                        }
                        None => Pat::Wild,
                    };
                    let mut out = vec![head];
                    out.extend(rest);
                    Some(out)
                }
            }
        }
    }

    // ----- calls ----------------------------------------------------------------

    /// Reorders positional + keyword arguments to match a parameter list.
    fn bind_args<'a>(&self, what: &str, params: &[(String, Type)], args: &'a [Arg], line: usize, col: usize) -> Result<Vec<&'a Expr>> {
        let mut slots: Vec<Option<&Expr>> = vec![None; params.len()];
        let mut pos = 0;
        for a in args {
            match &a.name {
                None => {
                    if pos >= params.len() {
                        return Err(LumeError::new(line, col, format!(
                            "{} takes {} {}, but {} {} given",
                            what,
                            params.len(),
                            plural(params.len(), "argument", "arguments"),
                            args.len(),
                            plural(args.len(), "was", "were")
                        )));
                    }
                    slots[pos] = Some(&a.value);
                    pos += 1;
                }
                Some(n) => match params.iter().position(|(p, _)| p == n) {
                    Some(i) => {
                        if slots[i].is_some() {
                            return Err(LumeError::new(a.value.line, a.value.col, format!("`{}` is given twice", n)));
                        }
                        slots[i] = Some(&a.value);
                    }
                    None => {
                        let e = LumeError::new(a.value.line, a.value.col, format!("{} has no parameter named `{}`", what, n));
                        let names: Vec<String> = params.iter().map(|(p, _)| p.clone()).collect();
                        return Err(match self.suggest_from(n, names.iter().cloned()) {
                            Some(s) => e.with_help(format!("did you mean `{}`?", s)),
                            None => e.with_help(format!("the parameters are: {}", names.join(", "))),
                        });
                    }
                },
            }
        }
        let missing: Vec<String> = params.iter().zip(&slots).filter(|(_, s)| s.is_none()).map(|((n, _), _)| n.clone()).collect();
        if !missing.is_empty() {
            return Err(LumeError::new(line, col, format!("{} is missing {}: {}", what, plural(missing.len(), "an argument", "arguments"), missing.join(", "))));
        }
        Ok(slots.into_iter().map(|s| s.unwrap()).collect())
    }

    /// `Enum.Variant(args)` or bare `Variant(args)`.
    fn variant_ctor(&mut self, en: &str, vname: &str, args: &[Arg], e: &Expr) -> Result<String> {
        let info = self.enums[en].clone();
        let (_, fields) = match info.variants.iter().find(|(n, _)| n == vname) {
            Some(v) => v.clone(),
            None => {
                let err = LumeError::new(e.line, e.col, format!("`{}` has no variant `{}`", en, vname));
                let names: Vec<String> = info.variants.iter().map(|(n, _)| n.clone()).collect();
                return Err(match self.suggest_from(vname, names.iter().cloned()) {
                    Some(s) => err.with_help(format!("did you mean `{}`?", s)),
                    None => err.with_help(format!("the variants are: {}", names.join(", "))),
                });
            }
        };
        let declared: Vec<Type> = fields.iter().map(|(_, t)| t.clone()).collect();
        let fields = if info.generics.is_empty() {
            fields
        } else {
            let whole = Type::App(en.to_string(), info.generics.iter().map(|p| Type::Var(p.name.clone())).collect());
            let what = format!("`{}.{}`", en, vname);
            let m = self.infer_call(&info.generics, &fields, args, &whole);
            let targs = self.all_bound(&info.generics, &m, &what, e.line, e.col)?;
            self.check_type_args(&info.generics, &targs, &what, e.line, e.col)?;
            fields.iter().map(|(n, t)| (n.clone(), Self::subst(t, &m))).collect()
        };
        let epath = self.path_of(en);
        if fields.is_empty() {
            if !args.is_empty() {
                return Err(LumeError::new(e.line, e.col, format!("`{}` carries no values; write `{}.{}` without parentheses", vname, en, vname)));
            }
            return Ok(format!("{}::{}", epath, vname));
        }
        let bound = self.bind_args(&format!("`{}.{}`", en, vname), &fields, args, e.line, e.col)?;
        let mut parts = Vec::new();
        for (i, (a, (fname, fty))) in bound.iter().zip(&fields).enumerate() {
            self.check_assign(a, fty, &format!("`{}.{}` takes `{}: {}`", en, vname, fname, type_name(fty)))?;
            let rest: Vec<&Expr> = bound[i + 1..].to_vec();
            let v = self.arg_with_rest(a, fty, &rest)?;
            // whether the field is boxed was settled by how it was declared,
            // not by what this instantiation filled in
            if self.boxed_field(en, declared.get(i).unwrap_or(fty)) {
                parts.push(format!("{}: ::std::boxed::Box::new({})", rust_name(fname), v));
            } else {
                parts.push(format!("{}: {}", rust_name(fname), v));
            }
        }
        Ok(format!("{}::{} {{ {} }}", epath, vname, parts.join(", ")))
    }

    fn expr(&mut self, e: &Expr) -> Result<String> {
        Ok(match &e.kind {
            ExprKind::Int(v) => format!("{}i64", v),
            ExprKind::Float(v) => {
                let s = format!("{}", v);
                if s.contains('.') || s.contains('e') { format!("{}f64", s) } else { format!("{}.0f64", s) }
            }
            ExprKind::Bool(b) => b.to_string(),
            ExprKind::Str(pieces) => {
                let only_lit = pieces.iter().all(|p| matches!(p, StrPiece::Lit(_)));
                if only_lit {
                    let s: String = pieces
                        .iter()
                        .map(|p| match p {
                            StrPiece::Lit(s) => s.clone(),
                            _ => unreachable!(),
                        })
                        .collect();
                    format!("String::from(\"{}\")", escape_rust_str(&s, false))
                } else {
                    let (fmt, args) = self.interp(pieces)?;
                    format!("format!(\"{}\", {})", fmt, args.join(", "))
                }
            }
            ExprKind::Ident(name) => {
                if let Some(ne) = self.split_ident_try(e) {
                    return self.expr(&ne);
                }
                if let Some(b) = self.lookup(name).cloned() {
                    if let Type::Shared(inner, true) = &b.ty {
                        if !self.want_handle {
                            // the value inside, copied out under a short lock
                            let g = self.fresh("g");
                            let take = if inner.is_copy() { format!("*{}", g) } else { format!("(*{}).clone()", g) };
                            return Ok(format!("({{ let {} = {}.lock().unwrap(); {} }})", g, rust_name(name), take));
                        }
                    }
                    rust_name(name)
                } else if self.consts.contains_key(&self.canon(name)) {
                    let key = self.canon(name);
                    let path = if self.paths.contains_key(&key) { self.path_of(&key) } else { rust_name(&key) };
                    format!("(*{})", path)
                } else if self.field_type(name).is_some() {
                    format!("self.{}", rust_name(name))
                } else if let Some(m) = self.bare_method(name) {
                    if m.self_kind == SelfKind::Mutate {
                        self.require_var_self(name, e.line, e.col)?;
                    }
                    format!("self.{}()", rust_name(name))
                } else if self.current_self_ty.as_ref().map(|st| builtin_method_type(st, name) != Type::Unknown || is_builtin_name(name)).unwrap_or(false) {
                    // inside `extend Str with ...`: a bare built-in method name is `self.name`
                    let call = Expr::new(ExprKind::Method { recv: Box::new(Expr::new(ExprKind::SelfRef, e.line, e.col)), name: name.clone(), args: vec![] }, e.line, e.col);
                    return self.expr(&call);
                } else if let Some(en) = self.resolve_variant(name, e.line, e.col)? {
                    self.variant_ctor(&en, name, &[], e)?
                } else if self.fns.contains_key(name) {
                    return Err(LumeError::new(e.line, e.col, format!("`{}` is a function; call it with `{}()`", name, name)));
                } else if self.structs.contains_key(name) {
                    return Err(LumeError::new(e.line, e.col, format!("`{}` is a type; construct one with `{}(...)`", name, name)));
                } else if self.enums.contains_key(name) {
                    return Err(LumeError::new(e.line, e.col, format!("`{}` is an enum; pick a variant like `{}.{}`", name, name, self.enums[name].variants[0].0)));
                } else {
                    return Err(self.unknown_name(name, e.line, e.col));
                }
            }
            ExprKind::SelfRef => {
                if self.current_type.is_none() {
                    return Err(LumeError::new(e.line, e.col, "`self` is only meaningful inside a method"));
                }
                // `extend Int with ...`: a method takes `self` by reference,
                // so a copied target is read through it
                if self.self_is_lent_copy() { "(*self)".into() } else { "self".into() }
            }
            ExprKind::List(items) => {
                let mut et = match self.ty_of(e).materialized() {
                    Type::List(t) => *t,
                    _ => Type::Unknown,
                };
                // `[Shape]` holds mixed types, so a literal whose items
                // disagree is read as the wanted interface rather than as
                // its first item
                if !self.is_interface(&et) && items.len() > 1 {
                    let mut ts = items.iter().map(|i| self.ty_of(i).materialized());
                    let first = ts.next().unwrap_or(Type::Unknown);
                    let mixed = ts.any(|t| t != first);
                    if mixed {
                        if let Some(Type::List(w)) = self.want.last().cloned() {
                            if self.is_interface(&w) {
                                et = *w;
                            }
                        }
                    }
                }
                let mut parts = Vec::new();
                for i in items {
                    if type_is_known(&et) && !self.is_interface(&et) {
                        self.check_assign(i, &et, &format!("the items of a list must all be the same type; the first is {}", a_type(&et)))?;
                    }
                    parts.push(self.expr_owned_as(i, &et)?);
                }
                format!("vec![{}]", parts.join(", "))
            }
            ExprKind::Tuple(items) => {
                let parts: Result<Vec<String>> = items.iter().map(|i| self.expr_owned(i)).collect();
                format!("({})", parts?.join(", "))
            }
            ExprKind::TupleIndex { recv, index } => {
                let rt = self.ty_of(recv);
                match &rt {
                    Type::Tuple(ts) => {
                        if *index >= ts.len() {
                            return Err(LumeError::new(e.line, e.col, format!("this tuple has {} parts, so `.{}` does not exist", ts.len(), index)));
                        }
                    }
                    Type::Unknown => {}
                    other => return Err(LumeError::new(e.line, e.col, format!("`.{}` needs a tuple, but this is a `{}`", index, type_name(other)))),
                }
                format!("{}.{}", self.expr(recv)?, index)
            }
            ExprKind::Some(x) => format!("Some({})", self.expr_owned(x)?),
            ExprKind::None => "None".into(),
            ExprKind::Try(x) => {
                let xt = self.ty_of(x);
                // A block declared to give back a `T or E` (or a `T?`) has
                // somewhere for `?` to go: out of the block, to its caller.
                let block_can_carry = matches!(&self.block_ret, Some(Type::Result(..)) | Some(Type::Option(_)));
                if self.in_block && !block_can_carry {
                    // A block the program declared can be given a different
                    // type; a block handed to a built-in like `.map` cannot,
                    // so only the first is told to change its declaration.
                    let help = if self.block_ret.is_some() {
                        "handle the missing or failed value with `match` or `.or(default)` inside the block, or declare the block `(T) -> U or Error` so `?` has somewhere to go"
                    } else {
                        "handle the missing or failed value with `match` or `.or(default)` inside the block"
                    };
                    return Err(LumeError::new(e.line, e.col, "`?` cannot be used inside a block").with_help(help));
                }
                match (&xt, &self.current_ret) {
                    (Type::Option(_), Type::Option(_)) | (Type::Result(..), Type::Result(..)) | (Type::Unknown, _) => {}
                    (Type::Option(_), Type::Result(..)) => {
                        return Err(LumeError::new(e.line, e.col, format!("`?` on an optional value returns `None`, but `{}` returns `{}`", self.current_fn, type_name(&self.current_ret)))
                            .with_help("turn the absence into an error first: `.or_error(\"what went wrong\")?`"));
                    }
                    (Type::Result(..), Type::Option(_)) => {
                        return Err(LumeError::new(e.line, e.col, format!("`?` on a `{}` returns the error, but `{}` returns `{}`", type_name(&xt), self.current_fn, type_name(&self.current_ret)))
                            .with_help("make the function return `T or Error`, or handle the error with `match` or `.or(default)`"));
                    }
                    (Type::Option(_), _) => {
                        return Err(LumeError::new(e.line, e.col, format!("`?` returns `None` early, but `{}` does not return an optional value", self.current_fn))
                            .with_help("make the function return `T?`, or handle the `None` case with `match` or `.or(default)`"));
                    }
                    (Type::Result(..), _) => {
                        return Err(LumeError::new(e.line, e.col, format!("`?` returns the error early, but `{}` does not return `T or E`", self.current_fn))
                            .with_help(format!("write `def {}(...) -> Type or Error`, or handle the error with `match` or `.or(default)`", self.current_fn)));
                    }
                    (other, _) => {
                        return Err(LumeError::new(e.line, e.col, format!("`?` needs an optional value or a `T or E`, but this is a `{}`", type_name(other))));
                    }
                }
                format!("({})?", self.expr(x)?)
            }
            ExprKind::Unwrap(x) => {
                let xt = self.ty_of(x);
                match xt {
                    Type::Option(_) | Type::Result(..) | Type::Unknown => {}
                    other => return Err(LumeError::new(e.line, e.col, format!("`!` unwraps an optional or a `T or E`, but this is a `{}`", type_name(&other)))),
                }
                if !self.in_test {
                    // `xs[i]!` after the bound was checked: `at` says that, and says it better
                    let indexed = match &x.kind {
                        ExprKind::Index { recv, index } => Some((recv.clone(), index.clone())),
                        ExprKind::Method { recv, name, .. } if name == "or_error" => match &recv.kind {
                            ExprKind::Index { recv, index } => Some((recv.clone(), index.clone())),
                            _ => None,
                        },
                        _ => None,
                    };
                    let list_index = matches!(&indexed, Some((r, i)) if matches!(self.ty_of(r).materialized(), Type::List(_)) && !matches!(i.kind, ExprKind::Range { .. }));
                    self.warnings.push(
                        LumeError::new(e.line, e.col, "`!` stops the program if the value is missing or an error").with_help(match &indexed {
                            Some((r, i)) if list_index => format!("when the position is already known to be good, write `{}.at({})`", snippet(r), snippet(i)),
                            _ => "fine in tests and quick scripts; elsewhere use `match`, `?` or `.or(default)`".to_string(),
                        }),
                    );
                }
                let inner = self.expr(x)?;
                match xt {
                    Type::Result(..) => format!("({}).unwrap_or_else(|e| panic!(\"{{}}\", e.message))", inner),
                    _ => format!("({}).expect(\"expected a value, found None\")", inner),
                }
            }
            ExprKind::Ok(x) => {
                // `Ok(5)` where nothing says what it fails with: Lume has one
                // error type, so that is what it fails with
                let known = self.want.last().map(|w| matches!(w, Type::Result(..))).unwrap_or(false) || matches!(self.current_ret, Type::Result(..));
                let v = self.expr_owned(x)?;
                if known {
                    format!("Ok({})", v)
                } else {
                    let xt = self.ty_of(x).materialized();
                    let t = self.rt(&xt);
                    format!("::std::result::Result::<{}, Error>::Ok({})", t, v)
                }
            }
            ExprKind::Index { recv, index } => {
                let rt = self.ty_of(recv).materialized();
                let r = self.expr_val(recv)?;
                match rt {
                    Type::List(_) => {
                        if let ExprKind::Range { lo, hi, inclusive } = &index.kind {
                            let (a, b) = self.slice_bounds(lo, hi, *inclusive)?;
                            return Ok(format!("lume_slice_list(&({}), {}, {})", r, a, b));
                        }
                        let i = self.expr(index)?;
                        let it = self.ty_of(index);
                        if it != Type::Int && it != Type::Unknown {
                            return Err(self.type_mismatch(index, &it, &Type::Int, "a list position is an `Int`"));
                        }
                        format!("({}).get(({}) as usize).cloned()", r, i)
                    }
                    Type::Map(ref k, _) => {
                        let kt = (**k).clone();
                        self.check_assign(index, &kt, &format!("this map's keys are `{}`", type_name(&kt)))?;
                        let key = self.map_key_text(index, &kt, &rt)?;
                        format!("({}).get({}).cloned()", r, key)
                    }
                    Type::Str => {
                        // by character, never by byte
                        if let ExprKind::Range { lo, hi, inclusive } = &index.kind {
                            let (a, b) = self.slice_bounds(lo, hi, *inclusive)?;
                            return Ok(format!("lume_slice_str(&({}), {}, {})", r, a, b));
                        }
                        let i = self.expr(index)?;
                        let it = self.ty_of(index);
                        if it != Type::Int && it != Type::Unknown {
                            return Err(LumeError::new(index.line, index.col, format!("a character position is an `Int`, but this is a `{}`", type_name(&it))));
                        }
                        format!("lume_char_at(&({}), {})", r, i)
                    }
                    Type::Set(_) => {
                        return Err(LumeError::new(e.line, e.col, "a set has no positions to index")
                            .with_help("use `.contains?(x)`, or `.to_list` for an ordered list"));
                    }
                    Type::Unknown => format!("({}).get(({}) as usize).cloned()", r, self.expr(index)?),
                    other => return Err(LumeError::new(e.line, e.col, format!("`{}` values cannot be indexed", type_name(&other)))),
                }
            }
            ExprKind::SetLit(items) => {
                let et = match self.ty_of(e) {
                    Type::Set(t) => *t,
                    _ => Type::Unknown,
                };
                self.check_key_type(&et, "set", e.line, e.col)?;
                let mut parts = Vec::new();
                for i in items {
                    self.check_assign(i, &et, &format!("the items of a set must all be the same type; the first is {}", a_type(&et)))?;
                    parts.push(self.expr_owned(i)?);
                }
                format!("LumeSet::from([{}])", parts.join(", "))
            }
            ExprKind::MapLit(pairs) => {
                if pairs.is_empty() {
                    // an empty map or an empty set: Rust takes the kind from where it goes
                    "Default::default()".to_string()
                } else {
                    let mut parts = Vec::new();
                    for (k, v) in pairs {
                        parts.push(format!("({}, {})", self.expr_owned(k)?, self.expr_owned(v)?));
                    }
                    format!("LumeMap::from([{}])", parts.join(", "))
                }
            }
            ExprKind::Await(x) => {
                if !self.in_async {
                    return Err(LumeError::new(e.line, e.col, "`await` only works inside an `async def` or a `spawn:` block")
                        .with_help("make this function `async def`, or move the waiting into `async def main`"));
                }
                self.uses_async = true;
                let xt = self.ty_of(x).materialized();
                // A task is the running work itself, not a handle to it, so
                // awaiting takes it. Without this the name would be cloned
                // for its later use, and a task is the one thing that cannot
                // be — which used to surface as a rustc error.
                let awaits_task = matches!(xt, Type::Task(_))
                    || matches!(&xt, Type::List(i) if matches!(**i, Type::Task(_)));
                if awaits_task {
                    if let ExprKind::Ident(n) = &x.kind {
                        if self.lookup(n).is_some() && self.used_after(n) {
                            let many = matches!(xt, Type::List(_));
                            let (what, keep) = if many {
                                ("these tasks", format!("results = await {}", n))
                            } else {
                                ("this task", format!("result = await {}", n))
                            };
                            return Err(LumeError::new(x.line, x.col, format!("`await` takes {}, and `{}` is used again below", what, n))
                                .with_help(format!("a task is the work itself, so waiting for it uses it up; keep what came back with `{}` and use that", keep)));
                        }
                    }
                }
                let v = self.expr(x)?;
                match xt {
                    Type::Future(_) => format!("({}).await", v),
                    Type::Task(_) => format!("({}).await.unwrap()", v),
                    Type::List(inner) if matches!(*inner, Type::Task(_)) => {
                        let v = self.expr_owned(x)?;
                        format!("{{ let mut lume_done = Vec::new(); for lume_t in {} {{ lume_done.push(lume_t.await.unwrap()); }} lume_done }}", v)
                    }
                    Type::Unknown => return Err(LumeError::new(x.line, x.col, "cannot tell what is being awaited here")),
                    other => {
                        return Err(LumeError::new(e.line, e.col, format!("`await` needs an async call or a task, but this is a `{}`", type_name(&other)))
                            .with_help("`await` goes in front of a call to an `async def`, a `Task`, or a list of tasks"))
                    }
                }
            }
            ExprKind::Spawn(body) => self.spawn_expr(body, e)?,
            ExprKind::Rust(code) => {
                self.has_rust_blocks = true;
                if code.contains('\n') {
                    let pad = "    ".repeat(self.indent + 1);
                    let body = code.lines().map(|l| format!("{}{}", pad, l)).collect::<Vec<_>>().join("\n");
                    format!("{{\n{}\n{}}}", body, "    ".repeat(self.indent))
                } else {
                    format!("{{ {} }}", code)
                }
            }
            ExprKind::Range { lo, hi, inclusive } => {
                let l = self.expr(lo)?;
                let h = self.expr(hi)?;
                if *inclusive { format!("({}..={})", l, h) } else { format!("({}..{})", l, h) }
            }
            ExprKind::Unary { op, expr } => {
                let x = self.expr(expr)?;
                match *op {
                    "not" => format!("(!{})", x),
                    "-" => format!("(-{})", x),
                    _ => unreachable!(),
                }
            }
            ExprKind::Binary { op, lhs, rhs } => {
                // user-defined operators: `def +(other)` on the left operand's type
                let lt0 = self.ty_of(lhs).materialized();
                if let Type::Named(tn) = &lt0 {
                    let lookup = |g: &Self, o: &str| g.methods_of(tn).and_then(|m| m.get(o).cloned());
                    // `!=` follows from `==`; `>`, `<=`, `>=` follow from `<`
                    let (found, method, negate, swap) = match lookup(self, op) {
                        Some(sg) => (Some(sg), *op, false, false),
                        None if *op == "!=" => (lookup(self, "=="), "==", true, false),
                        None if *op == ">" => (lookup(self, "<"), "<", false, true),
                        None if *op == "<=" => (lookup(self, "<"), "<", true, true),
                        None if *op == ">=" => (lookup(self, "<"), "<", true, false),
                        None => (None, *op, false, false),
                    };
                    if let Some(sg) = found {
                        let call = if swap {
                            let l = self.expr_arg(lhs, &sg.params[0].1)?;
                            let r = self.expr(rhs)?;
                            format!("({}).{}({})", r, rust_name(method), l)
                        } else {
                            let l = self.expr(lhs)?;
                            let r = self.expr_arg(rhs, &sg.params[0].1)?;
                            format!("({}).{}({})", l, rust_name(method), r)
                        };
                        return Ok(if negate { format!("(!{})", call) } else { call });
                    }
                    let user_type = self.structs.contains_key(&self.canon(tn)) || self.enums.contains_key(&self.canon(tn));
                    if user_type && matches!(*op, "+" | "-" | "*" | "/" | "%" | "<" | "<=" | ">" | ">=") {
                        let (want, extra) = match *op {
                            "<" | "<=" | ">" | ">=" => ("<", "; `<=`, `>` and `>=` then follow"),
                            _ => (*op, ""),
                        };
                        return Err(LumeError::new(e.line, e.col, format!("`{}` has no `{}` operator", tn, op))
                            .with_help(format!("add `def {}(other: {}) -> {}:` to `{}`{}", want, tn, if want == "<" { "Bool" } else { tn }, tn, extra)));
                    }
                    if user_type && *op == "**" {
                        return Err(LumeError::new(e.line, e.col, format!("`{}` has no `**` operator", tn)).with_help("`**` is not overloadable; write a method such as `def pow(n: Int)`"));
                    }
                }
                if matches!(*op, "+" | "-" | "*" | "/" | "%" | "**" | "<" | "<=" | ">" | ">=") {
                    for side in [lhs, rhs] {
                        let st = self.ty_of(side);
                        let what = match &st {
                            Type::Option(_) => Some("may be absent"),
                            Type::Result(..) => Some("may be an error"),
                            _ => None,
                        };
                        if let Some(w) = what {
                            return Err(LumeError::new(side.line, side.col, format!("this value is a `{}` — it {}, so it cannot be used with `{}` directly", type_name(&st), w, op))
                                .with_help(if matches!(st, Type::Option(_)) { "unwrap it first: `match` on `Some(x)`/`None`, `?`, or `.or(default)`" } else { "unwrap it first: `?` to pass the error up, `match` on `Ok(x)`/`Error(e)`, or `.or(default)`" }));
                        }
                    }
                }
                let lt = self.ty_of(lhs).materialized();
                if lt != Type::Unknown {
                    self.check_operands(op, &lt, rhs, e.line, e.col)?;
                }
                // two Int literals: the answer is known now, and so is a mistake
                if let (Some(a), Some(b)) = (int_literal(lhs), int_literal(rhs)) {
                    let folded = match *op {
                        "+" => a.checked_add(b),
                        "-" => a.checked_sub(b),
                        "*" => a.checked_mul(b),
                        "/" | "%" if b == 0 => {
                            return Err(LumeError::new(e.line, e.col, format!("{} by zero", if *op == "/" { "division" } else { "`%`" })));
                        }
                        "/" => a.checked_div(b),
                        "%" => a.checked_rem(b),
                        _ => Some(0),
                    };
                    if folded.is_none() {
                        return Err(LumeError::new(e.line, e.col, format!("`{} {} {}` overflows `Int`", a, op, b))
                            .with_help("Int is 64-bit; the largest value is 9223372036854775807"));
                    }
                }
                let rt_ = self.ty_of(rhs).materialized();
                // a character against a string: a one-character literal becomes a
                // `char`, anything else compares as text
                if (lt == Type::Char) != (rt_ == Type::Char) && (lt == Type::Str || rt_ == Type::Str) && matches!(*op, "==" | "!=" | "<" | "<=" | ">" | ">=" ) {
                    let (ce, se, char_left) = if lt == Type::Char { (lhs, rhs, true) } else { (rhs, lhs, false) };
                    let mut c = self.expr_val(ce)?;
                    if self.is_borrowed_ident(ce) {
                        c = format!("(*{})", c);
                    }
                    let text = match char_literal(se) {
                        Some(ch) => (format!("'{}'", rust_char(ch)), true),
                        None => (format!("({}).lume_as_str()", self.expr_val(se)?), false),
                    };
                    if !text.1 && matches!(*op, "==" | "!=") {
                        let neg = if *op == "!=" { "!" } else { "" };
                        return Ok(format!("({}lume_char_eq_str({}, {}))", neg, c, text.0));
                    }
                    let c = if text.1 { c } else { format!("({}).to_string().as_str()", c) };
                    let (l, r) = if char_left { (c, text.0) } else { (text.0, c) };
                    return Ok(format!("({} {} {})", l, op, r));
                }
                // `a + b + c` on text: one `format!` with every piece
                if *op == "+" && text_like(&lt) && text_like(&rt_) {
                    let mut leaves: Vec<&Expr> = Vec::new();
                    self.text_leaves(e, &mut leaves);
                    let mut parts = Vec::new();
                    for leaf in leaves {
                        parts.push(match str_literal(leaf) {
                            Some(t) => t,
                            None => self.expr_val(leaf)?,
                        });
                    }
                    return Ok(format!("format!(\"{}\", {})", "{}".repeat(parts.len()), parts.join(", ")));
                }
                let text_op = lt == Type::Str && matches!(*op, "+" | "==" | "!=" | "<" | "<=" | ">" | ">=");
                let mut l = match str_literal(lhs) {
                    Some(t) if text_op => t,
                    _ => self.expr_val(lhs)?,
                };
                let mut r = match str_literal(rhs) {
                    Some(t) if text_op => t,
                    _ => self.expr_val(rhs)?,
                };
                let lb = self.is_borrowed_ident(lhs);
                let rb = self.is_borrowed_ident(rhs);
                // strings are formatted, never dereferenced (`&str` has no `*`)
                if lt != Type::Str {
                    if lb && !rb {
                        l = format!("(*{})", l);
                    }
                    if rb && !lb {
                        r = format!("(*{})", r);
                    }
                }
                match *op {
                    "and" => format!("({} && {})", l, r),
                    "or" => format!("({} || {})", l, r),
                    "**" => format!("({}).lume_pow({})", l, r),
                    "+" if lt == Type::Str || lt == Type::Char => format!("format!(\"{{}}{{}}\", {}, {})", l, r),
                    "+" if matches!(lt, Type::List(_) | Type::Iter(..)) => format!("{{ let mut lume_v = ({}).clone(); lume_v.extend(({}).iter().cloned()); lume_v }}", l, r),
                    // strings compare as `&str` whatever they are held as
                    "==" | "!=" | "<" | "<=" | ">" | ">=" if lt == Type::Str => format!("(({}).lume_as_str() {} ({}).lume_as_str())", l, op, r),
                    _ => format!("({} {} {})", l, op, r),
                }
            }
            ExprKind::Call { name: raw_name, args } => {
                if args.is_empty() {
                    if let Some(ne) = self.split_ident_try(e) {
                        return self.expr(&ne);
                    }
                }
                let cname = self.canon(raw_name);
                let name = &cname;
                // `f(x)` where `f` is a block this function was handed: run it
                if let Some(Type::Fn(ins, _)) = self.lookup(raw_name).map(|b| b.ty.clone()) {
                    return self.call_block(raw_name, &ins, args, e);
                }
                // inside a type, a bare call is its own method before any
                // free function of the same name
                if let Some(tn) = self.current_type.clone() {
                    if self.lookup(name).is_none() && self.methods_of(&tn).map(|m| m.contains_key(name)).unwrap_or(false) {
                        let call = Expr::new(
                            ExprKind::Method { recv: Box::new(Expr::new(ExprKind::SelfRef, e.line, e.col)), name: name.clone(), args: args.clone() },
                            e.line,
                            e.col,
                        );
                        return self.expr(&call);
                    }
                }
                if let Some(decl) = self.fns.get(name).cloned() {
                    let sig = self.instantiate(&decl, args, &format!("`{}`", name), e.line, e.col)?;
                    let bound = self.bind_args(&format!("`{}`", name), &sig.params, args, e.line, e.col)?;
                    let mut parts = Vec::new();
                    for (i, (a, (pname, t))) in bound.iter().zip(&sig.params).enumerate() {
                        if sig.var_params[i] {
                            self.check_assign(a, t, &format!("`{}` takes `var {}: {}`", name, pname, type_name(t)))?;
                            parts.push(self.expr_var_arg(a, pname, name)?);
                        } else {
                            parts.push(self.expr_arg_named_as(a, t, &decl.params[i].1, name, pname)?);
                        }
                    }
                    let callee = if self.paths.contains_key(name) { self.path_of(name) } else { rust_name(name) };
                    format!("{}({})", callee, parts.join(", "))
                } else if let Some(info) = self.structs.get(name).cloned() {
                    let info = self.instantiate_struct(name, &info, args, e.line, e.col)?;
                    let bound = self.bind_args(&format!("`{}`", name), &info.fields, args, e.line, e.col)?;
                    let mut parts = Vec::new();
                    for (i, (a, (fname, fty))) in bound.iter().zip(&info.fields).enumerate() {
                        self.check_assign(a, fty, &format!("`{}` field `{}` is `{}`", name, fname, type_name(fty)))?;
                        let rest: Vec<&Expr> = bound[i + 1..].to_vec();
                        let v = self.arg_with_rest(a, fty, &rest)?;
                        parts.push(format!("{}: {}", rust_name(fname), v));
                    }
                    format!("{} {{ {} }}", self.path_of(name), parts.join(", "))
                } else if let Some(en) = self.resolve_variant(name, e.line, e.col)? {
                    self.variant_ctor(&en, name, args, e)?
                } else if self.lookup(name).is_some() {
                    return Err(LumeError::new(e.line, e.col, format!("`{}` is a value, not a function", name)));
                } else if let Some(tn) = self.current_type.clone() {
                    if self.methods_of(&tn).map(|m| m.contains_key(name)).unwrap_or(false) {
                        // a sibling method called bare: `deps_of(pkg)` is `self.deps_of(pkg)`
                        let call = Expr::new(
                            ExprKind::Method { recv: Box::new(Expr::new(ExprKind::SelfRef, e.line, e.col)), name: name.clone(), args: args.clone() },
                            e.line,
                            e.col,
                        );
                        return self.expr(&call);
                    }
                    if self.self_has_builtin(name) {
                        return self.expr(&self.self_call(name, args, e));
                    }
                    return Err(self.unknown_fn(name, e.line, e.col));
                } else {
                    if self.self_has_builtin(name) {
                        return self.expr(&self.self_call(name, args, e));
                    }
                    return Err(self.unknown_fn(name, e.line, e.col));
                }
            }
            ExprKind::Method { recv, name, args } => {
                if let Some(ne) = self.split_trailing_try(e) {
                    return self.expr(&ne);
                }
                if let Some(ne) = self.fn_ref_as_block(e) {
                    return self.expr(&ne);
                }
                if let Some(ne) = self.module_ref(e)? {
                    return self.expr(&ne);
                }
                if let Some(elem) = self.indexed_element(recv, name) {
                    return self.indexed_member(recv, elem, name, args, e);
                }
                if let Some(text) = self.indexed_collection_mutation(recv, name, args, e)? {
                    return Ok(text);
                }
                if is_mutating_builtin(name) && !self.is_place(recv) {
                    let rt = self.ty_of(recv).materialized();
                    if matches!(rt, Type::List(_) | Type::Set(_) | Type::Map(..) | Type::Option(_)) {
                        let err = LumeError::new(e.line, e.col, format!("`.{}` would change a temporary copy (`{}`), and the change would be lost", name, snippet(recv)));
                        return Err(match &recv.kind {
                            ExprKind::Method { recv: inner, name: m, .. } if m == "or" && matches!(inner.kind, ExprKind::Index { .. }) => {
                                err.with_help(format!("change the stored value directly: `{}.{}(...)` on a `var` map starts from an empty value when the key is missing", snippet(inner), name))
                            }
                            _ => err.with_help("bind it to a `var` first, change that, and store it back if it came from somewhere else"),
                        });
                    }
                }
                if let Some(fr) = self.foreign_ref(e) {
                    return match fr {
                        ForeignRef::Fn(f) => {
                            let f = f.clone();
                            let display = foreign_display(e);
                            self.foreign_call(&f, &display, args, e)
                        }
                        ForeignRef::Assoc(_, f) => {
                            let f = f.clone();
                            let display = foreign_display(e);
                            self.foreign_call(&f, &display, args, e)
                        }
                        ForeignRef::Type(key) => Err(LumeError::new(e.line, e.col, format!("`{}` is a type from the crate; call one of its functions, like `{}.new(...)`", key, key))),
                    };
                }
                if let Some(root) = self.foreign_root(e) {
                    if self.foreign_ref(recv).is_none() && matches!(recv.kind, ExprKind::Ident(_) | ExprKind::Method { .. }) && self.ty_of(recv) == Type::Unknown {
                        let err = LumeError::new(e.line, e.col, format!("crate `{}` has no `{}`", root, foreign_display(e).trim_start_matches(&format!("{}.", root))));
                        return Err(err.with_help(format!("`lume crate <file> {}` lists what the crate offers", root)));
                    }
                }
                if let Some(ft_key) = match self.ty_of(recv).materialized() { Type::Named(n) if self.foreign_types.contains_key(&n) => Some(n), _ => None } {
                    return self.foreign_method(recv, &ft_key, name, args, e);
                }
                if let Some((root, inner, mutable)) = self.shared_root_ex(recv) {
                    if mutable {
                        return self.shared_access(e, root, inner, Some(args));
                    }
                    // read-only `shared`: the handle reads like the value itself
                    let root_text = self.handle_expr(root)?;
                    let g = self.fresh("g");
                    self.push_scope();
                    self.declare(&g, false, true, inner, e.line);
                    let ge = Expr::new(ExprKind::Ident(g.clone()), root.line, root.col);
                    let rewritten = Self::replace_root(e, root, &ge);
                    let body = self.expr_owned(&rewritten);
                    self.pop_scope();
                    let body = match body {
                        Err(err) if err.msg.contains(&format!("`{}`", g)) => {
                            let rn = Self::root_name(root).unwrap_or_else(|| "it".into());
                            return Err(LumeError::new(e.line, e.col, format!("`{}` is `shared` and read-only, but `{}` changes it", rn, name))
                                .with_help(format!("declare it `shared var {}` if tasks change it", rn)));
                        }
                        other => other?,
                    };
                    return Ok(format!("({{ let {} = &*{}; {} }})", g, root_text, body));
                }
                let recv_ty = self.ty_of(recv);
                self.no_future(&recv_ty, recv)?;
                // Enum.Variant(...) constructor
                if let ExprKind::Ident(tn) = &recv.kind {
                    let ctn = self.canon(tn);
                    if self.lookup(tn).is_none() && self.enums.contains_key(&ctn) {
                        return self.variant_ctor(&ctn, name, args, e);
                    }
                    if self.lookup(tn).is_none() && !self.is_type(&self.canon(tn)) && builtin_namespace_type(tn, name).is_some() {
                        let mut parts = Vec::new();
                        for a in args {
                            parts.push(self.expr_val(&a.value)?);
                        }
                        let need = |n: usize| -> Result<()> {
                            if parts.len() != n {
                                Err(LumeError::new(e.line, e.col, format!("`{}.{}` takes {} {}", tn, name, n, plural(n, "argument", "arguments"))))
                            } else {
                                Ok(())
                            }
                        };
                        return Ok(match (tn.as_str(), name.as_str()) {
                            ("File", "read") => { need(1)?; format!("lume_read_file(&{})", parts[0]) }
                            ("File", "write") => { need(2)?; format!("lume_write_file(&{}, &{})", parts[0], parts[1]) }
                            ("File", "exists?") => { need(1)?; format!("std::path::Path::new(&*{}).exists()", parts[0]) }
                            ("File", "append") => { need(2)?; format!("lume_append_file(&{}, &{})", parts[0], parts[1]) }
                            ("File", "remove") => { need(1)?; format!("lume_remove_file(&{})", parts[0]) }
                            ("File", "size") => { need(1)?; format!("lume_file_size(&{})", parts[0]) }
                            ("File", "modified") => { need(1)?; format!("lume_file_modified(&{})", parts[0]) }
                            ("Dir", "exists?") => { need(1)?; format!("std::path::Path::new(&*{}).is_dir()", parts[0]) }
                            ("Dir", "make") => { need(1)?; format!("lume_dir_make(&{})", parts[0]) }
                            ("Dir", "list") => { need(1)?; format!("lume_dir_list(&{})", parts[0]) }
                            ("Dir", "walk") => { need(1)?; format!("lume_dir_walk(&{})", parts[0]) }
                            ("Dir", "remove") => { need(1)?; format!("lume_dir_remove(&{})", parts[0]) }
                            ("Path", "join") => { need(2)?; format!("lume_path_join(&{}, &{})", parts[0], parts[1]) }
                            ("Path", "dir") => { need(1)?; format!("lume_path_dir(&{})", parts[0]) }
                            ("Path", "base") => { need(1)?; format!("lume_path_base(&{})", parts[0]) }
                            ("Path", "ext") => { need(1)?; format!("lume_path_ext(&{})", parts[0]) }
                            ("Path", "stem") => { need(1)?; format!("lume_path_stem(&{})", parts[0]) }
                            ("Env", "exit") => { need(1)?; format!("std::process::exit(({}) as i32)", parts[0]) }
                            ("Env", "stdin") => { need(0)?; "lume_stdin()".to_string() }
                            ("Env", "args") => { need(0)?; "lume_args()".to_string() }
                            ("Env", "get") => { need(1)?; format!("std::env::var(&*{}).ok()", parts[0]) }
                            ("Time", "now") => { need(0)?; "lume_now()".to_string() }
                            ("Time", "now_ms") => { need(0)?; "lume_now_ms()".to_string() }
                            ("Time", "sleep") => {
                                need(1)?;
                                if self.in_async {
                                    self.uses_async = true;
                                    format!("tokio::time::sleep(std::time::Duration::from_millis(({}) as u64))", parts[0])
                                } else {
                                    format!("std::thread::sleep(std::time::Duration::from_millis(({}) as u64))", parts[0])
                                }
                            }
                            _ => unreachable!(),
                        });
                    }
                    if self.lookup(tn).is_none() && !self.is_type(&self.canon(tn)) && matches!(tn.as_str(), "File" | "Env" | "Time" | "Dir" | "Path") {
                        return Err(LumeError::new(e.line, e.col, format!("`{}` has no `{}`", tn, name))
                            .with_help(match tn.as_str() {
                                "File" => "File has read, write, append, exists?, remove, size and modified",
                                "Env" => "Env has args, get(name), stdin and exit(code)",
                                "Dir" => "Dir has exists?, make, list, walk and remove",
                                "Path" => "Path has join(a, b), dir, base, ext and stem",
                                _ => "Time has now (seconds), now_ms and sleep(ms)",
                            }));
                    }
                    if self.lookup(tn).is_none() && self.structs.contains_key(tn) {
                        return Err(LumeError::new(e.line, e.col, format!("`{}.{}` — static methods on a type are not supported yet", tn, name))
                            .with_help(format!("construct a value with `{}(...)` and call the method on it", tn)));
                    }
                }
                if let Some(last) = args.last() {
                    if matches!(last.value.kind, ExprKind::Lambda { .. }) {
                        let rt = self.ty_of(recv).materialized();
                        if !self.user_block_method(&rt, name) {
                            return self.block_method(recv, name, &args[..args.len() - 1], &last.value, e);
                        }
                    }
                }
                let rt = self.ty_of(recv);
                // Lazy chains: a few methods stay lazy, the rest collect first.
                if let Type::Iter(elem, by_ref) = &rt {
                    let r = self.expr(recv)?;
                    match name.as_str() {
                        "take" | "skip" if args.len() == 1 => {
                            let n = self.expr(&args[0].value)?;
                            return Ok(format!("{}.{}(({}) as usize)", r, name, n));
                        }
                        "enumerate" if args.is_empty() => {
                            return Ok(format!("{}.enumerate().map(|(i, x)| (i as i64, x))", r));
                        }
                        "len" if args.is_empty() => return Ok(format!("({}.count() as i64)", r)),
                        "to_list" if args.is_empty() => return Ok(self.collect_iter_t(&r, *by_ref, elem)),
                        "sum" if args.is_empty() => {
                            let t = if **elem == Type::Float { "f64" } else { "i64" };
                            let r = if *by_ref { format!("{}.cloned()", r) } else { r };
                            return Ok(format!("{}.sum::<{}>()", r, t));
                        }
                        "max" | "min" if args.is_empty() && elem.is_copy() => {
                            if **elem == Type::Float {
                                let r = if *by_ref { format!("{}.cloned()", r) } else { r };
                                return Ok(format!("{}.{}_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))", r, name));
                            }
                            return Ok(format!("{}.{}()", r, name));
                        }
                        "first" if args.is_empty() => {
                            // `&str` is not `Clone`, so a chain of text ends
                            // the way `collect_iter_t` ends one.
                            let r = if *by_ref && **elem == Type::Str {
                                format!("{}.map(|s| s.to_string())", r)
                            } else if *by_ref {
                                format!("{}.cloned()", r)
                            } else {
                                r
                            };
                            return Ok(format!("{}.next()", r));
                        }
                        "empty?" | "any?" if args.is_empty() => {
                            let neg = if name == "empty?" { "" } else { "!" };
                            return Ok(format!("({}{}.next().is_none())", neg, r));
                        }
                        _ => {
                            // A lazy chain is collected and then treated as a
                            // list, so a name that is no method of either has
                            // to be caught here — past this point it would
                            // reach rustc as a method on a `Vec`.
                            if !builtin_applies(&rt, name) && !builtins_for(&Type::List(elem.clone())).contains(&name.as_str()) {
                                return Err(LumeError::new(e.line, e.col, format!("`{}` values have no method `{}`", type_name(&rt), name))
                                    .with_help(format!("`{}` has: {}", type_name(&rt), builtins_for(&rt).join(", "))));
                            }
                            let collected = self.collect_iter_t(&r, *by_ref, elem);
                            let mut parts = Vec::new();
                            for a in args {
                                parts.push(self.expr_val(&a.value)?);
                            }
                            return self.method(&collected, name, &parts, &rt.materialized(), e);
                        }
                    }
                }
                let mut r = self.expr(recv)?;
                // `.or(default)` consumes the optional: take a copy when the place lives on
                if matches!(name.as_str(), "or" | "or_error") {
                    let inner_copy = match &rt {
                        Type::Option(i) => i.is_copy(),
                        // an `Error` is never copied, so neither is the
                        // `T or Error` around it, whatever `T` is
                        Type::Result(..) => false,
                        _ => true,
                    };
                    let lives_on = self.is_borrowed_place(recv) || matches!(&recv.kind, ExprKind::Ident(n) if self.lookup(n).is_some() && self.used_after(n));
                    if !inner_copy && lives_on {
                        r = format!("({}).clone()", r);
                    } else if inner_copy && self.is_borrowed_ident(recv) {
                        r = format!("(*{})", r);
                    }
                }
                if let Type::Named(_) | Type::App(..) | Type::Var(_) = &rt {
                    let tname = &self.type_key(&rt);
                    let sub = self.subst_for(&rt);
                    if let Some(info) = self.structs.get(tname).cloned() {
                        if info.fields.iter().any(|(n, _)| n == name) {
                            if !args.is_empty() {
                                return Err(LumeError::new(e.line, e.col, format!("`{}` is a field of `{}`, not a method; it takes no arguments", name, tname)));
                            }
                            return Ok(format!("{}.{}", r, rust_name(name)));
                        }
                    }
                    // universal methods on user types
                    if (name == "to_s" || name == "to_str") && args.is_empty() && self.methods_of(tname).map(|m| !m.contains_key(name)).unwrap_or(true) {
                        return Ok(format!("({}).lume_str()", r));
                    }
                    if let Some(decl) = self.methods_of(tname).and_then(|m| m.get(name).cloned()) {
                        // a method of a generic type: its `T` is whatever the receiver's is
                        let m = Sig {
                            params: decl.params.iter().map(|(n, t)| (n.clone(), Self::subst(t, &sub))).collect(),
                            ret: Self::subst(&decl.ret, &sub),
                            ..decl.clone()
                        };
                        let m = self.instantiate(&m, args, &format!("`{}.{}`", tname, name), e.line, e.col)?;
                        let bound = self.bind_args(&format!("`{}.{}`", tname, name), &m.params, args, e.line, e.col)?;
                        let mut parts = Vec::new();
                        let callee = format!("{}.{}", tname, name);
                        for (i, (a, (pname, t))) in bound.iter().zip(&m.params).enumerate() {
                            if m.var_params[i] {
                                self.check_assign(a, t, &format!("`{}` takes `var {}: {}`", callee, pname, type_name(t)))?;
                                parts.push(self.expr_var_arg(a, pname, name)?);
                            } else {
                                parts.push(self.expr_arg_named_as(a, t, &decl.params[i].1, &callee, pname)?);
                            }
                        }
                        if m.self_kind == SelfKind::Mutate {
                            self.check_receiver_mutable(recv, name, e.line, e.col)?;
                        }
                        return Ok(format!("{}.{}({})", r, rust_name(name), parts.join(", ")));
                    }
                    self.check_default_recursion(&rt, name, e.line, e.col)?;
                    if let Some((m, decl)) = self.iface_default_ex(&rt, name) {
                        let bound = self.bind_args(&format!("`{}.{}`", tname, name), &m.params, args, e.line, e.col)?;
                        let mut parts = Vec::new();
                        for (i, (a, (pname, t))) in bound.iter().zip(&m.params).enumerate() {
                            let d = decl.get(i).unwrap_or(t);
                            parts.push(self.expr_arg_named_as(a, t, d, &format!("{}.{}", tname, name), pname)?);
                        }
                        return Ok(format!("{}.{}({})", r, rust_name(name), parts.join(", ")));
                    }
                    return Err(self.no_such_member(tname, name, e.line, e.col));
                }
                // methods a built-in type gained through `extend`
                if !matches!(rt, Type::Named(_)) {
                    let key = self.type_key(&rt);
                    // through `methods_for`, so a generic `extend`'s names
                    // are read as what this receiver holds
                    let ext_here = self.ext_methods.get(&key).map(|m| m.contains_key(name)).unwrap_or(false);
                    let found = if ext_here { self.methods_for(&rt).and_then(|m| m.get(name).cloned()) } else { None };
                    if let Some(m) = found {
                        let bound = self.bind_args(&format!("`{}.{}`", type_name(&rt), name), &m.params, args, e.line, e.col)?;
                        let mut parts = Vec::new();
                        let callee = format!("{}.{}", type_name(&rt), name);
                        let decl = self.iface_decl_params(&key, name);
                        for (i, (a, (pname, t))) in bound.iter().zip(&m.params).enumerate() {
                            let d = decl.get(i).unwrap_or(t);
                            parts.push(self.expr_arg_named_as(a, t, d, &callee, pname)?);
                        }
                        return Ok(format!("({}).{}({})", r, rust_name(name), parts.join(", ")));
                    }
                    self.check_default_recursion(&rt, name, e.line, e.col)?;
                    if let Some((m, decl)) = self.iface_default_ex(&rt, name) {
                        let bound = self.bind_args(&format!("`{}.{}`", type_name(&rt), name), &m.params, args, e.line, e.col)?;
                        let mut parts = Vec::new();
                        for (i, (a, (pname, t))) in bound.iter().zip(&m.params).enumerate() {
                            let d = decl.get(i).unwrap_or(t);
                            parts.push(self.expr_arg_named_as(a, t, d, &format!("{}.{}", type_name(&rt), name), pname)?);
                        }
                        return Ok(format!("({}).{}({})", r, rust_name(name), parts.join(", ")));
                    }
                }
                if let Type::Option(inner) = &rt {
                    if !matches!(name.as_str(), "or" | "some?" | "none?" | "or_error" | "map" | "to_s" | "to_str") {
                        let known = match &**inner {
                            Type::Named(tn) => self.methods_of(tn).map(|m| m.contains_key(name)).unwrap_or(false)
                                || self.structs.get(tn).map(|s| s.fields.iter().any(|(n, _)| n == name)).unwrap_or(false),
                            other => builtin_method_type(other, name) != Type::Unknown,
                        };
                        let err = LumeError::new(e.line, e.col, format!("this value is a `{}` — it may be absent, so `.{}` cannot be called on it directly", type_name(&rt), name));
                        return Err(if known {
                            err.with_help(format!("unwrap it first: `match` on `Some(x)`/`None`, or `.or(default).{}`", name))
                        } else {
                            err.with_help("unwrap it first with `match` on `Some(x)`/`None`, or `.or(default)`")
                        });
                    }
                }
                if let Type::Result(..) = &rt {
                    if !matches!(name.as_str(), "or" | "ok?" | "error?" | "error" | "ok" | "to_s" | "to_str") {
                        return Err(LumeError::new(e.line, e.col, format!("this value is a `{}` — it may be an error, so `.{}` cannot be called on it directly", type_name(&rt), name))
                            .with_help("unwrap it first: `?` to pass the error up, `match` on `Ok(x)`/`Error(e)`, or `.or(default)`"));
                    }
                }
                if matches!(&rt, Type::List(el) if **el == Type::Unknown) && matches!(recv.kind, ExprKind::List(ref items) if items.is_empty()) && name != "len" && name != "empty?" {
                    return Err(LumeError::new(recv.line, recv.col, format!("cannot tell what this empty list holds, so `.{}` has no type", name))
                        .with_help("bind it with a type first: `xs: [Int] = []`"));
                }
                if rt != Type::Unknown && !builtin_applies(&rt, name) {
                    let err = LumeError::new(e.line, e.col, format!("`{}` values have no method `{}`", type_name(&rt), name));
                    let names = builtins_for(&rt);
                    if matches!(rt, Type::Set(_)) && name == "push" {
                        return Err(err.with_help("a set takes values with `.add(x)`, which says whether the value was new"));
                    }
                    if matches!(rt, Type::List(_) | Type::Iter(..)) && name == "add" {
                        return Err(err.with_help("a list takes values with `.push(x)`"));
                    }
                    return Err(match self.suggest_from(name, names.iter().map(|s| s.to_string())) {
                        Some(sug) => err.with_help(format!("did you mean `{}`?", sug)),
                        None if names.is_empty() => err,
                        None => err.with_help(format!("`{}` has: {}", type_name(&rt), names.join(", "))),
                    });
                }
                if args.iter().any(|a| a.name.is_some()) {
                    return Err(LumeError::new(e.line, e.col, format!("built-in method `{}` does not take keyword arguments", name)));
                }
                let mut want_types: Vec<Type> = Vec::new();
                if let Some(want) = builtin_params(&rt, name) {
                    if want.len() == args.len() {
                        for (a, w) in args.iter().zip(&want) {
                            if type_is_known(w) {
                                self.check_assign(&a.value, w, &format!("`.{}` on {} takes {}", name, a_type(&rt), a_type(w)))?;
                            }
                        }
                        want_types = want;
                    }
                }
                if matches!(name.as_str(), "push" | "pop" | "insert" | "remove_at" | "add") || (name == "remove" && matches!(rt, Type::Map(..) | Type::Set(_))) {
                    self.check_receiver_mutable(recv, name, e.line, e.col)?;
                }
                let mut parts = Vec::new();
                for (ai, a) in args.iter().enumerate() {
                    if want_types.get(ai) == Some(&Type::Char) {
                        if let Some(c) = char_literal(&a.value) {
                            parts.push(format!("'{}'", rust_char(c)));
                            continue;
                        }
                    }
                    if name == "or" && matches!(&rt, Type::Option(i) if **i == Type::Char) {
                        if let Some(c) = char_literal(&a.value) {
                            parts.push(format!("'{}'", rust_char(c)));
                            continue;
                        }
                        let at = self.ty_of(&a.value).materialized();
                        if at == Type::Str {
                            self.or_default_is_str = true;
                            parts.push(self.expr_owned(&a.value)?);
                            continue;
                        }
                        self.check_assign(&a.value, &Type::Char, "`.or` on a `Char?` takes a `Char` or a `Str`")?;
                    }
                    // a character where the method wants a string
                    if want_types.get(ai) == Some(&Type::Str) && self.ty_of(&a.value).materialized() == Type::Char {
                        let v = self.expr_val(&a.value)?;
                        parts.push(format!("({}).to_string()", v));
                        continue;
                    }
                    // a literal the method only reads: no String built for it
                    if want_types.get(ai) == Some(&Type::Str) && !matches!(name.as_str(), "push" | "add" | "insert" | "or" | "or_error") {
                        if let Some(t) = str_literal(&a.value) {
                            parts.push(t);
                            continue;
                        }
                    }
                    // `.or(default)` and `.push(x)` store their argument: owned position
                    parts.push(match (name.as_str(), &rt) {
                        ("push", Type::List(elem)) | ("add", Type::Set(elem)) | ("or", Type::Option(elem)) | ("or", Type::Result(elem, _)) => self.expr_owned_as(&a.value, &elem.clone())?,
                        ("insert", Type::List(elem)) if parts.len() == 1 => self.expr_owned_as(&a.value, &elem.clone())?,
                        ("or" | "push", _) => self.expr_owned(&a.value)?,
                        _ => self.expr_val(&a.value)?,
                    });
                }
                if let Type::List(elem) = &rt {
                    let base = if elem.is_copy() { format!("({}).iter().cloned()", r) } else { format!("({}).iter()", r) };
                    match name.as_str() {
                        "take" | "skip" if parts.len() == 1 => return Ok(format!("{}.{}(({}) as usize)", base, name, parts[0])),
                        "enumerate" if parts.is_empty() => return Ok(format!("{}.enumerate().map(|(i, x)| (i as i64, x))", base)),
                        "to_list" if parts.is_empty() => return Ok(format!("({}).clone()", r)),
                        _ => {}
                    }
                }
                self.method(&r, name, &parts, &rt, e)?
            }
            ExprKind::If { branches, else_block } => {
                if else_block.is_none() {
                    return Err(LumeError::new(e.line, e.col, "an `if` used as a value needs an `else`").with_help("every branch must produce the value"));
                }
                self.if_chain(branches, else_block.as_ref(), true)?
            }
            ExprKind::Match { scrutinee, arms } => self.match_expr(scrutinee, arms, true, e)?,
            ExprKind::Puts(arg) => self.print_call("println", arg)?,
            ExprKind::Warn(arg) => self.print_call("eprintln", arg)?,
            ExprKind::Placeholder => {
                return Err(LumeError::new(e.line, e.col, "`_` can only be used inside a method argument").with_help("write `xs.map(_.name)`; elsewhere give the value a name"));
            }
            ExprKind::Lambda { .. } => {
                return Err(LumeError::new(e.line, e.col, "a block must follow a method call").with_help("write `xs.each do |x|` or `xs.map { |x| ... }`"));
            }
        })
    }

    fn check_receiver_mutable(&self, recv: &Expr, method: &str, line: usize, col: usize) -> Result<()> {
        match &recv.kind {
            ExprKind::Ident(n) => match self.lookup(n) {
                Some(b) if b.mutable => Ok(()),
                Some(Binding { ty: Type::Shared(_, false), line: bl, .. }) => Err(LumeError::new(line, col, format!("`{}` is `shared` and read-only, but `{}` changes it", n, method))
                    .with_help(format!("declare it `shared var {}` on line {} if tasks change it", n, bl))),
                Some(b) => Err(LumeError::new(line, col, format!("`{}` is immutable, but `{}` changes it", n, method))
                    .with_help(format!("declare it with `var {} = ...` on line {}", n, b.line))),
                None if self.field_type(n).is_some() => self.require_var_self(n, line, col),
                None => Ok(()),
            },
            ExprKind::SelfRef => self.require_var_self(method, line, col),
            _ => Ok(()),
        }
    }

    /// Inside an `extend` on a built-in, `self` is a list, a set or a map,
    /// so a bare `contains?(x)` means `self.contains?(x)` the same way a
    /// bare sibling method does inside a struct.
    fn self_has_builtin(&self, name: &str) -> bool {
        match &self.current_self_ty {
            Some(st) => builtin_method_type(st, name) != Type::Unknown || is_builtin_name(name),
            None => false,
        }
    }

    fn self_call(&self, name: &str, args: &[Arg], e: &Expr) -> Expr {
        Expr::new(
            ExprKind::Method { recv: Box::new(Expr::new(ExprKind::SelfRef, e.line, e.col)), name: name.to_string(), args: args.to_vec() },
            e.line,
            e.col,
        )
    }

    fn unknown_fn(&self, name: &str, line: usize, col: usize) -> LumeError {
        let err = LumeError::new(line, col, format!("unknown function `{}`", name));
        match self.suggest(name) {
            Some(s) => err.with_help(format!("did you mean `{}`?", s)),
            None => err,
        }
    }

    /// Maps built-in method names onto Rust. `rt` is the receiver's type.
    fn method(&mut self, recv: &str, name: &str, args: &[String], rt: &Type, e: &Expr) -> Result<String> {
        let need = |n: usize| -> Result<()> {
            if args.len() != n {
                Err(LumeError::new(e.line, e.col, format!(
                    "`.{}` takes {} {}, but {} {} given",
                    name, n, plural(n, "argument", "arguments"), args.len(), plural(args.len(), "was", "were")
                )))
            } else {
                Ok(())
            }
        };
        Ok(match name {
            "pad" => { need(1)?; format!("lume_pad(&({}), {})", recv, args[0]) }
            "pad_right" => { need(1)?; format!("lume_pad_right(&({}), {})", recv, args[0]) }
            "capitalize" => { need(0)?; format!("lume_capitalize(&{})", recv) }
            "index_of" if *rt == Type::Str => { need(1)?; format!("{{ let lume_s = &({}); lume_s.find(&*({})).map(|i| lume_s[..i].chars().count() as i64) }}", recv, args[0]) }
            "index_of" => {
                need(1)?;
                match rt {
                    Type::List(el) if **el == Type::Str => format!("({}).iter().position(|lume_y| lume_y.lume_as_str() == ({}).lume_as_str()).map(|i| i as i64)", recv, args[0]),
                    _ => format!("({}).iter().position(|lume_y| *lume_y == ({})).map(|i| i as i64)", recv, args[0]),
                }
            }
            "digit?" if *rt == Type::Char => { need(0)?; format!("({}).is_ascii_digit()", recv) }
            "alpha?" if *rt == Type::Char => { need(0)?; format!("({}).is_alphabetic()", recv) }
            "space?" if *rt == Type::Char => { need(0)?; format!("({}).is_whitespace()", recv) }
            "alnum?" if *rt == Type::Char => { need(0)?; format!("({}).is_alphanumeric()", recv) }
            "upper?" if *rt == Type::Char => { need(0)?; format!("({}).is_uppercase()", recv) }
            "lower?" if *rt == Type::Char => { need(0)?; format!("({}).is_lowercase()", recv) }
            "upcase" if *rt == Type::Char => { need(0)?; format!("({}).to_uppercase().collect::<String>()", recv) }
            "downcase" if *rt == Type::Char => { need(0)?; format!("({}).to_lowercase().collect::<String>()", recv) }
            "code" => { need(0)?; format!("(({}).clone() as u32 as i64)", recv) }
            "to_char" => { need(0)?; format!("u32::try_from({}).ok().and_then(char::from_u32)", recv) }
            "digit?" => { need(0)?; format!("{{ let lume_s = &({}); !lume_s.is_empty() && lume_s.chars().all(|c| c.is_ascii_digit()) }}", recv) }
            "alpha?" => { need(0)?; format!("{{ let lume_s = &({}); !lume_s.is_empty() && lume_s.chars().all(|c| c.is_alphabetic()) }}", recv) }
            "space?" => { need(0)?; format!("{{ let lume_s = &({}); !lume_s.is_empty() && lume_s.chars().all(|c| c.is_whitespace()) }}", recv) }
            "max" | "min" if matches!(rt, Type::Int | Type::Float) => { need(1)?; format!("({}).{}({})", recv, name, args[0]) }
            "clamp" => { need(2)?; format!("lume_clamp({}, {}, {})", recv, args[0], args[1]) }
            "pow" => {
                need(1)?;
                if *rt == Type::Float { format!("({}).powf({})", recv, args[0]) } else { format!("({}).pow(({}) as u32)", recv, args[0]) }
            }
            "even?" => { need(0)?; format!("(({}) % 2 == 0)", recv) }
            "odd?" => { need(0)?; format!("(({}) % 2 != 0)", recv) }
            "avg" => { need(0)?; format!("{{ let lume_v = &({}); if lume_v.is_empty() {{ 0.0 }} else {{ lume_v.iter().map(|x| *x as f64).sum::<f64>() / lume_v.len() as f64 }} }}", recv) }
            "uniq" => { need(0)?; format!("{{ let mut lume_v = Vec::new(); for lume_x in ({}).iter() {{ if !lume_v.contains(lume_x) {{ lume_v.push(lume_x.clone()); }} }} lume_v }}", recv) }
            "flatten" => { need(0)?; format!("({}).iter().flatten().cloned().collect::<Vec<_>>()", recv) }
            "zip" => { need(1)?; format!("({}).iter().cloned().zip(({}).iter().cloned()).collect::<Vec<_>>()", recv, args[0]) }
            "insert" => { need(2)?; format!("({}).insert(({}) as usize, {})", recv, args[0], args[1]) }
            "remove_at" => { need(1)?; format!("({}).remove(({}) as usize)", recv, args[0]) }
            "to_set" => { need(0)?; format!("({}).iter().cloned().collect::<LumeSet<_>>()", recv) }
            "add" if matches!(rt, Type::Set(_)) => { need(1)?; format!("({}).insert({})", recv, args[0]) }
            "remove" if matches!(rt, Type::Set(_)) => { need(1)?; format!("({}).remove({})", recv, set_key(rt, &args[0])) }
            "contains?" if matches!(rt, Type::Set(_)) => { need(1)?; format!("({}).contains({})", recv, set_key(rt, &args[0])) }
            "to_list" if matches!(rt, Type::Set(_)) => { need(0)?; format!("({}).iter().cloned().collect::<Vec<_>>()", recv) }
            "union" if matches!(rt, Type::Set(_)) => { need(1)?; format!("({}).union(&({}))", recv, args[0]) }
            "intersect" if matches!(rt, Type::Set(_)) => { need(1)?; format!("({}).intersect(&({}))", recv, args[0]) }
            "diff" if matches!(rt, Type::Set(_)) => { need(1)?; format!("({}).diff(&({}))", recv, args[0]) }
            "subset?" if matches!(rt, Type::Set(_)) => { need(1)?; format!("({}).is_subset(&({}))", recv, args[0]) }
            "superset?" if matches!(rt, Type::Set(_)) => { need(1)?; format!("({}).is_superset(&({}))", recv, args[0]) }
            "sort" | "max" | "min" | "sum" | "join" | "first" if matches!(rt, Type::Set(_)) => {
                let list = format!("({}).iter().cloned().collect::<Vec<_>>()", recv);
                let lt = match rt { Type::Set(e) => Type::List(e.clone()), _ => unreachable!() };
                return self.method(&list, name, args, &lt, e);
            }
            "merge" if matches!(rt, Type::Map(..)) => { need(1)?; format!("{{ let mut lume_m = ({}).clone(); for (k, v) in ({}).iter() {{ lume_m.insert(k.clone(), v.clone()); }} lume_m }}", recv, args[0]) }
            "ok" => {
                need(0)?;
                if !matches!(rt, Type::Result(..) | Type::Unknown) {
                    return Err(LumeError::new(e.line, e.col, format!("`.ok` turns a `T or E` into an optional value, but this is a `{}`", type_name(rt))));
                }
                format!("({}).clone().ok()", recv)
            }
            "keys" if matches!(rt, Type::Map(..)) => { need(0)?; format!("({}).keys().cloned().collect::<Vec<_>>()", recv) }
            "values" if matches!(rt, Type::Map(..)) => { need(0)?; format!("({}).values().cloned().collect::<Vec<_>>()", recv) }
            "to_list" if matches!(rt, Type::Map(..)) => { need(0)?; format!("({}).iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>()", recv) }
            "contains?" if matches!(rt, Type::Map(..)) => { need(1)?; format!("({}).contains_key({})", recv, map_key(rt, &args[0])) }
            "remove" if matches!(rt, Type::Map(..)) => { need(1)?; format!("({}).remove({})", recv, map_key(rt, &args[0])) }
            "get" if matches!(rt, Type::Map(..)) => { need(1)?; format!("({}).get({}).cloned()", recv, map_key(rt, &args[0])) }
            "len" => { need(0)?; format!("({}).lume_len()", recv) }
            "empty?" => { need(0)?; format!("({}).lume_empty()", recv) }
            "any?" => { need(0)?; format!("(!({}).lume_empty())", recv) }
            "to_str" | "to_s" => { need(0)?; format!("({}).lume_str()", recv) }
            "to_float" => { need(0)?; if *rt == Type::Str { format!("lume_to_float(&{})", recv) } else { format!("(({}) as f64)", recv) } }
            "to_int" => { need(0)?; if *rt == Type::Str { format!("lume_to_int(&{})", recv) } else { format!("(({}) as i64)", recv) } }
            "upcase" => { need(0)?; format!("({}).to_uppercase()", recv) }
            "downcase" => { need(0)?; format!("({}).to_lowercase()", recv) }
            "trim" => { need(0)?; format!("({}).trim().to_string()", recv) }
            "trim_left" => { need(0)?; format!("({}).trim_start().to_string()", recv) }
            "trim_right" => { need(0)?; format!("({}).trim_end().to_string()", recv) }
            // `price.decimals(2)` — the number as text, to that many places
            "decimals" => { need(1)?; format!("lume_decimals(({}) as f64, {})", recv, args[0]) }
            // `xs.at(i)` — the item, stopping the program when there is none
            "at" => { need(1)?; format!("lume_at(&{}, {})", recv, args[0]) }
            "sqrt" | "abs" | "floor" | "ceil" | "round" => { need(0)?; format!("({}).{}()", recv, name) }
            "sum" => { need(0)?; format!("({}).lume_sum()", recv) }
            "push" => { need(1)?; format!("({}).push({})", recv, args[0]) }
            "pop" => { need(0)?; format!("({}).pop()", recv) }
            "first" => { need(0)?; format!("({}).first().cloned()", recv) }
            "last" => { need(0)?; format!("({}).last().cloned()", recv) }
            "contains?" => {
                need(1)?;
                match rt {
                    Type::Str => format!("({}).contains(&*({}))", recv, args[0]),
                    Type::List(e) if **e == Type::Str => format!("({}).iter().any(|x| x.lume_as_str() == ({}).lume_as_str())", recv, args[0]),
                    _ => format!("({}).contains(&({}))", recv, args[0]),
                }
            }
            "reverse" if *rt == Type::Str => { need(0)?; format!("({}).chars().rev().collect::<String>()", recv) }
            "reverse" => { need(0)?; format!("{{ let mut lume_v = ({}).clone(); lume_v.reverse(); lume_v }}", recv) }
            "slice" => { need(2)?; format!("({}).chars().skip(({}).max(0) as usize).take(({}).max(0) as usize).collect::<String>()", recv, args[0], args[1]) }
            "replace" => { need(2)?; format!("({}).replace(&*{}, &*{})", recv, args[0], args[1]) }
            "repeat" => { need(1)?; format!("({}).repeat(({}).max(0) as usize)", recv, args[0]) }
            "sort" | "max" | "min" => {
                need(0)?;
                let partial = match rt {
                    Type::List(el) => match &**el {
                        Type::Named(tn) => {
                            let key = self.canon(tn);
                            if (self.structs.contains_key(&key) || self.enums.contains_key(&key)) && self.methods_of(tn).map(|m| !m.contains_key("<")).unwrap_or(true) {
                                return Err(LumeError::new(e.line, e.col, format!("`.{}` needs to compare `{}` values, but `{}` has no `<` operator", name, tn, tn))
                                    .with_help(format!("add `def <(other: {}) -> Bool:` to `{}`, or use `.{}_by(_.field)`", tn, tn, if name == "sort" { "sort" } else { name })));
                            }
                            true
                        }
                        // `Ordered` promises `<`, which is a partial order:
                        // a type parameter cannot offer more than that, and
                        // one with no bound cannot be compared at all
                        Type::Var(vn) => {
                            if !self.meets_bound(el, &Type::Named("Ordered".into())) {
                                return Err(LumeError::new(e.line, e.col, format!("`{}` could be any type, so `.{}` cannot compare its values", vn, name))
                                    .with_help(format!("declare it `[{}: Ordered]`, which promises every type it is given has `<`", vn)));
                            }
                            true
                        }
                        Type::App(..) => true,
                        other => *other == Type::Float,
                    },
                    _ => false,
                };
                match (name, partial) {
                    ("sort", true) => format!("{{ let mut lume_v = ({}).clone(); lume_v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)); lume_v }}", recv),
                    ("sort", false) => format!("{{ let mut lume_v = ({}).clone(); lume_v.sort(); lume_v }}", recv),
                    (_, true) => format!("({}).iter().cloned().{}_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))", recv, name),
                    (_, false) => format!("({}).iter().cloned().{}()", recv, name),
                }
            }
            "lines" => { need(0)?; format!("({}).lines()", recv) }
            "split" if args.is_empty() => format!("({}).split_whitespace()", recv),
            "split" => {
                need(1)?;
                // A one-character separator is searched as a character, which is
                // a byte scan rather than a substring search.
                let sep = match &e.kind {
                    ExprKind::Method { args: a, .. } => a.first().and_then(|a| char_literal(&a.value)),
                    _ => None,
                };
                match sep {
                    Some(c) => format!("({}).split('{}')", recv, rust_char(c)),
                    None => format!("({}).split(&*({}))", recv, args[0]),
                }
            }
            "join" => {
                need(1)?;
                let strs = matches!(rt, Type::List(ref e) | Type::Iter(ref e, _) if **e == Type::Str);
                let chars = matches!(rt, Type::List(ref e) | Type::Iter(ref e, _) if **e == Type::Char);
                if strs {
                    format!("({}).join(&*{})", recv, args[0])
                } else if chars {
                    format!("lume_join_chars(&({}), &*{})", recv, args[0])
                } else {
                    format!("({}).iter().map(|x| x.lume_str()).collect::<Vec<_>>().join(&*{})", recv, args[0])
                }
            }
            "starts_with?" => { need(1)?; format!("({}).starts_with(&*{})", recv, args[0]) }
            "ends_with?" => { need(1)?; format!("({}).ends_with(&*{})", recv, args[0]) }
            "chars" => { need(0)?; format!("({}).chars().collect::<Vec<char>>()", recv) }
            // Option
            "or" => {
                need(1)?;
                if !matches!(rt, Type::Option(_) | Type::Result(..) | Type::Unknown) {
                    return Err(LumeError::new(e.line, e.col, format!("`.or` supplies a default for an optional value or a `T or E`, but this is a `{}`", type_name(rt))));
                }
                if matches!(rt, Type::Option(i) if **i == Type::Char) && self.or_default_is_str {
                    self.or_default_is_str = false;
                    format!("({}).map(|c| c.to_string()).unwrap_or({})", recv, args[0])
                } else {
                    format!("({}).unwrap_or({})", recv, args[0])
                }
            }
            "some?" => { need(0)?; format!("({}).is_some()", recv) }
            "none?" => { need(0)?; format!("({}).is_none()", recv) }
            "ok?" => { need(0)?; format!("({}).is_ok()", recv) }
            "error?" => { need(0)?; format!("({}).is_err()", recv) }
            "or_error" => {
                need(1)?;
                if !matches!(rt, Type::Option(_) | Type::Unknown) {
                    return Err(LumeError::new(e.line, e.col, format!("`.or_error` turns an optional value into a `T or Error`, but this is a `{}`", type_name(rt))));
                }
                format!("({}).ok_or_else(|| Error {{ message: ({}).lume_str() }})", recv, args[0])
            }
            "error" => {
                need(0)?;
                if !matches!(rt, Type::Result(..) | Type::Unknown) {
                    return Err(LumeError::new(e.line, e.col, format!("`.error` reads the error of a `T or E`, but this is a `{}`", type_name(rt))));
                }
                format!("({}).clone().err()", recv)
            }
            _ => format!("({}).{}({})", recv, rust_name(name), args.join(", ")),
        })
    }
}


impl Gen {
    fn collect_methods(&self, methods: &[FnDef], owner: &str, fields: &HashSet<String>) -> Result<HashMap<String, Sig>> {
        let mut out = HashMap::new();
        for m in methods {
            if fields.contains(&m.name) {
                return Err(LumeError::new(m.line, m.col, format!("`{}` is both a field and a method of `{}`", m.name, owner)));
            }
            if op_method_name(&m.name).is_some() {
                if m.params.len() != 1 {
                    return Err(LumeError::new(m.line, m.col, format!("`def {}` on `{}` must take exactly one argument, the right-hand side", m.name, owner))
                        .with_help(format!("write `def {}(other: {}) -> {}:`; the left-hand side is `self`", m.name, owner, if matches!(m.name.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=") { "Bool" } else { owner })));
                }
                if matches!(m.name.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=") && m.ret.as_ref().map(|t| *t != Type::Bool).unwrap_or(false) {
                    return Err(LumeError::new(m.line, m.col, format!("`def {}` must return `Bool`", m.name)));
                }
                if m.self_kind == SelfKind::Mutate {
                    return Err(LumeError::new(m.line, m.col, format!("`def {}` cannot take `var self`: operators produce a new value", m.name)));
                }
            }
            if out.insert(m.name.clone(), self.sig_of(m)).is_some() {
                return Err(LumeError::new(m.line, m.col, format!("method `{}` is defined twice in `{}`", m.name, owner)));
            }
        }
        Ok(out)
    }
}

/// Which built-in methods a receiver type has. The one source for
/// acceptance and for the "`Str` has: ..." help.
fn builtins_for(recv: &Type) -> Vec<&'static str> {
    let common = ["to_s", "to_str"];
    let mut v: Vec<&str> = match recv {
        Type::Str => vec![
            "len", "empty?", "to_int", "to_float", "upcase", "downcase", "trim", "lines", "split", "chars", "contains?", "starts_with?",
            "ends_with?", "pad", "pad_right", "reverse", "slice", "replace", "repeat", "capitalize", "index_of", "digit?", "alpha?", "space?",
            "trim_left", "trim_right",
        ],
        Type::Char => vec!["digit?", "alpha?", "space?", "alnum?", "upper?", "lower?", "upcase", "downcase", "code", "pad", "pad_right"],
        Type::List(_) => vec![
            "len", "empty?", "any?", "all?", "first", "last", "max", "min", "sum", "sort", "sort_by", "reverse", "push", "pop", "contains?",
            "join", "map", "filter", "reject", "each", "count", "find", "take", "skip", "take_while", "fold", "min_by", "max_by", "enumerate",
            "to_list", "zip", "flatten", "uniq", "index_of", "insert", "remove_at", "avg", "group_by", "partition", "flat_map", "to_set", "at",
        ],
        Type::Iter(..) => vec![
            "len", "empty?", "any?", "all?", "first", "last", "max", "min", "sum", "sort", "sort_by", "reverse", "contains?", "join", "map",
            "filter", "reject", "each", "count", "find", "take", "skip", "take_while", "fold", "min_by", "max_by", "enumerate", "to_list",
            "zip", "flatten", "uniq", "index_of", "avg", "group_by", "partition", "flat_map", "to_set",
        ],
        Type::Map(..) => vec!["len", "empty?", "any?", "contains?", "keys", "values", "to_list", "remove", "get", "merge", "filter", "reject", "map_values", "each", "count", "all?", "find"],
        Type::Set(_) => vec![
            "len", "empty?", "any?", "contains?", "add", "remove", "to_list", "union", "intersect", "diff", "subset?", "superset?", "sort", "max", "min", "sum", "first", "each",
            "map", "filter", "reject", "count", "all?", "find", "fold", "join", "group_by", "partition", "flat_map", "sort_by", "min_by", "max_by",
        ],
        Type::Option(_) => vec!["or", "some?", "none?", "or_error", "map"],
        Type::Result(..) => vec!["or", "ok?", "error?", "error", "ok", "map"],
        Type::Int => vec!["to_float", "to_int", "abs", "pad", "pad_right", "max", "min", "clamp", "pow", "even?", "odd?", "to_char", "decimals"],
        Type::Float => vec!["to_int", "to_float", "sqrt", "floor", "ceil", "round", "abs", "pad", "pad_right", "max", "min", "clamp", "pow", "decimals"],
        Type::Bool => vec!["pad", "pad_right"],
        _ => vec![],
    };
    v.extend(common);
    v
}

fn builtin_applies(recv: &Type, name: &str) -> bool {
    match recv {
        // user types, interfaces and crate types are checked elsewhere
        Type::Named(_) | Type::Unknown | Type::Shared(..) | Type::Task(_) | Type::Future(_) | Type::Tuple(_) | Type::Unit => true,
        _ => builtins_for(recv).contains(&name),
    }
}

fn is_builtin_name(name: &str) -> bool {
    matches!(
        name,
        "len" | "empty?" | "any?" | "to_str" | "to_s" | "to_float" | "to_int" | "upcase" | "downcase" | "trim"
            | "sqrt" | "abs" | "floor" | "ceil" | "round" | "sum" | "push" | "pop" | "first" | "last"
            | "contains?" | "reverse" | "sort" | "max" | "min" | "lines" | "split" | "join"
            | "starts_with?" | "ends_with?" | "chars" | "take" | "skip" | "to_list" | "enumerate"
            | "or" | "some?" | "none?" | "ok?" | "error?" | "or_error" | "error"
            | "pad" | "keys" | "values" | "remove" | "get" | "slice" | "replace" | "repeat"
            | "pad_right" | "capitalize" | "index_of" | "digit?" | "alpha?" | "space?" | "zip" | "flatten" | "uniq" | "insert"
            | "remove_at" | "avg" | "merge" | "clamp" | "pow" | "even?" | "odd?" | "ok"
            | "to_set" | "add" | "union" | "intersect" | "diff" | "subset?" | "superset?"
            | "alnum?" | "upper?" | "lower?" | "code" | "to_char"
            | "trim_left" | "trim_right" | "decimals" | "at"
    )
}

/// Types of the built-in `File` and `Env` namespaces.
fn builtin_namespace_type(ns: &str, name: &str) -> Option<Type> {
    let err = || Box::new(Type::Named("Error".into()));
    Some(match (ns, name) {
        ("File", "read") => Type::Result(Box::new(Type::Str), err()),
        ("File", "write") => Type::Result(Box::new(Type::Unit), err()),
        ("File", "exists?") => Type::Bool,
        ("File", "append") | ("File", "remove") => Type::Result(Box::new(Type::Unit), err()),
        ("File", "size") | ("File", "modified") => Type::Result(Box::new(Type::Int), err()),
        ("Dir", "exists?") => Type::Bool,
        ("Dir", "make") | ("Dir", "remove") => Type::Result(Box::new(Type::Unit), err()),
        ("Dir", "list") | ("Dir", "walk") => Type::Result(Box::new(Type::List(Box::new(Type::Str))), err()),
        ("Path", "join") | ("Path", "dir") | ("Path", "base") | ("Path", "ext") | ("Path", "stem") => Type::Str,
        ("Env", "exit") => Type::Unit,
        ("Env", "stdin") => Type::Result(Box::new(Type::Str), err()),
        ("Env", "args") => Type::List(Box::new(Type::Str)),
        ("Env", "get") => Type::Option(Box::new(Type::Str)),
        ("Time", "now") => Type::Int,
        ("Time", "now_ms") => Type::Int,
        ("Time", "sleep") => Type::Unit, // becomes `async ()` inside async code; see ty_of
        _ => return None,
    })
}

/// Result type of a built-in method on a value of type `recv`.
fn builtin_method_type(recv: &Type, name: &str) -> Type {
    let elem = match recv {
        Type::List(e) | Type::Iter(e, _) => Some((**e).clone()),
        _ => None,
    };
    if let Type::Set(t) = recv {
        return match name {
            "len" => Type::Int,
            "empty?" | "any?" | "contains?" | "subset?" | "superset?" | "add" | "remove" => Type::Bool,
            "to_list" | "sort" => Type::List(t.clone()),
            "union" | "intersect" | "diff" => recv.clone(),
            "max" | "min" | "first" => Type::Option(t.clone()),
            "sum" => (**t).clone(),
            "join" => Type::Str,
            "to_s" | "to_str" => Type::Str,
            _ => Type::Unknown,
        };
    }
    if let Type::Map(k, v) = recv {
        return match name {
            "len" => Type::Int,
            "empty?" | "any?" | "contains?" => Type::Bool,
            "keys" => Type::List(k.clone()),
            "values" => Type::List(v.clone()),
            "to_list" => Type::List(Box::new(Type::Tuple(vec![(**k).clone(), (**v).clone()]))),
            "remove" | "get" => Type::Option(v.clone()),
            "merge" => recv.clone(),
            "to_s" | "to_str" => Type::Str,
            _ => Type::Unknown,
        };
    }
    match name {
        "pad" | "pad_right" | "capitalize" => Type::Str,
        "digit?" | "alpha?" | "space?" | "even?" | "odd?" => Type::Bool,
        "index_of" => Type::Option(Box::new(Type::Int)),
        "max" | "min" if matches!(recv, Type::Int | Type::Float) => recv.clone(),
        "clamp" | "pow" => recv.clone(),
        "avg" => Type::Float,
        "uniq" => recv.materialized(),
        "to_set" => match &elem {
            Some(e) => Type::Set(Box::new(e.clone())),
            None => Type::Unknown,
        },
        "insert" => Type::Unit,
        "remove_at" => elem.clone().unwrap_or(Type::Unknown),
        "flatten" => match &elem {
            Some(Type::List(inner)) => Type::List(inner.clone()),
            _ => Type::Unknown,
        },
        "zip" => Type::Unknown, // needs the argument's type: see ty_of
        "ok" => match recv {
            Type::Result(t, _) => Type::Option(t.clone()),
            _ => Type::Unknown,
        },
        "take" | "skip" => match recv {
            Type::List(e) => Type::Iter(e.clone(), !e.is_copy()),
            Type::Iter(..) => recv.clone(),
            _ => Type::Unknown,
        },
        "enumerate" => match recv {
            Type::List(e) => Type::Iter(Box::new(Type::Tuple(vec![Type::Int, (**e).clone()])), !e.is_copy()),
            Type::Iter(e, by_ref) => Type::Iter(Box::new(Type::Tuple(vec![Type::Int, (**e).clone()])), *by_ref),
            _ => Type::Unknown,
        },
        "to_list" => recv.materialized(),
        "len" => Type::Int,
        "to_int" => if *recv == Type::Str { Type::Result(Box::new(Type::Int), Box::new(Type::Named("Error".into()))) } else { Type::Int },
        "empty?" | "any?" | "contains?" | "starts_with?" | "ends_with?" | "some?" | "none?" | "ok?" | "error?" => Type::Bool,
        "or_error" => match recv {
            Type::Option(i) => Type::Result(i.clone(), Box::new(Type::Named("Error".into()))),
            _ => Type::Unknown,
        },
        "error" => match recv {
            Type::Result(_, e) => Type::Option(e.clone()),
            _ => Type::Unknown,
        },
        "to_str" | "to_s" | "upcase" | "downcase" | "trim" | "join" | "slice" | "replace" | "repeat" => Type::Str,
        "trim_left" | "trim_right" | "decimals" => Type::Str,
        "to_float" => if *recv == Type::Str { Type::Result(Box::new(Type::Float), Box::new(Type::Named("Error".into()))) } else { Type::Float },
        "sqrt" | "floor" | "ceil" | "round" => Type::Float,
        "abs" => recv.clone(),
        "sum" => elem.unwrap_or(Type::Unknown),
        "first" | "last" | "max" | "min" | "pop" => elem.map(|e| Type::Option(Box::new(e))).unwrap_or(Type::Unknown),
        // `xs.at(i)` is the item itself: the bound was already checked
        "at" => elem.unwrap_or(Type::Unknown),
        "sort" | "reverse" => recv.materialized(),
        "push" => Type::Unit,
        "lines" | "split" => Type::Iter(Box::new(Type::Str), true),
        "chars" => Type::List(Box::new(Type::Char)),
        "code" => Type::Int,
        "to_char" => Type::Option(Box::new(Type::Char)),
        "alnum?" | "upper?" | "lower?" => Type::Bool,
        "or" => match recv {
            Type::Option(i) => (**i).clone(),
            Type::Result(t, _) => (**t).clone(),
            _ => Type::Unknown,
        },
        _ => Type::Unknown,
    }
}

/// The identifier an expression starts with, reading left to right:
/// `x.trim` -> x, `x + 1` -> x, `y + x` -> y, `3 + x` -> None.
fn leftmost_ident(e: &Expr) -> Option<&str> {
    match &e.kind {
        ExprKind::Ident(n) => Some(n.as_str()),
        ExprKind::Method { recv, .. } | ExprKind::TupleIndex { recv, .. } => leftmost_ident(recv),
        ExprKind::Binary { lhs, .. } => leftmost_ident(lhs),
        ExprKind::Range { lo, .. } => leftmost_ident(lo),
        ExprKind::Try(x) => leftmost_ident(x),
        _ => None,
    }
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// A type with no unknown parts.
fn type_is_known(t: &Type) -> bool {
    match t {
        Type::Unknown => false,
        Type::List(i) | Type::Option(i) | Type::Iter(i, _) | Type::Task(i) | Type::Future(i) | Type::Shared(i, _) | Type::Set(i) => type_is_known(i),
        Type::Tuple(ts) => ts.iter().all(type_is_known),
        Type::Result(a, b) | Type::Map(a, b) => type_is_known(a) && type_is_known(b),
        Type::Fn(ps, r) => ps.iter().all(type_is_known) && type_is_known(r),
        _ => true,
    }
}

fn suggest_type(t: &Type) -> String {
    match t {
        Type::List(i) if **i == Type::Unknown => "[Int]".into(),
        Type::Option(i) if **i == Type::Unknown => "Int?".into(),
        Type::Map(k, v) if **k == Type::Unknown || **v == Type::Unknown => "{Str: Int}".into(),
        Type::Set(i) if **i == Type::Unknown => "{Int}".into(),
        Type::Unknown => "Type".into(),
        other => type_name(other),
    }
}

fn describe_value(e: &Expr) -> String {
    match &e.kind {
        ExprKind::List(items) if items.is_empty() => "[]".into(),
        ExprKind::MapLit(pairs) if pairs.is_empty() => "{}".into(),
        ExprKind::None => "None".into(),
        _ => snippet(e),
    }
}

/// A short spelling of an expression for help text: `x`, `n + 1`, `f(...)`, `x.name`.
fn snippet(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Ident(n) => n.clone(),
        ExprKind::Int(v) => v.to_string(),
        ExprKind::Float(v) => format!("{:?}", v),
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Str(parts) => match parts.as_slice() {
            [StrPiece::Lit(t)] if t.len() <= 12 => format!("\"{}\"", t),
            _ => "\"...\"".into(),
        },
        ExprKind::Call { name, args } => if args.is_empty() { format!("{}()", name) } else { format!("{}(...)", name) },
        ExprKind::Method { recv, name, args } => {
            let r = snippet(recv);
            if args.is_empty() { format!("{}.{}", r, name) } else { format!("{}.{}(...)", r, name) }
        }
        ExprKind::Binary { op, lhs, rhs } => format!("{} {} {}", snippet(lhs), op, snippet(rhs)),
        ExprKind::SelfRef => "self".into(),
        ExprKind::Index { recv, index } => match &index.kind {
            ExprKind::Int(_) | ExprKind::Ident(_) | ExprKind::Str(_) => format!("{}[{}]", snippet(recv), snippet(index)),
            ExprKind::Range { lo, hi, inclusive } => format!("{}[{}{}{}]", snippet(recv), snippet(lo), if *inclusive { ".." } else { "..." }, snippet(hi)),
            _ => format!("{}[...]", snippet(recv)),
        },
        ExprKind::TupleIndex { recv, index } => format!("{}.{}", snippet(recv), index),
        _ => "x".into(),
    }
}

/// A borrowed map key: `&*k` works whether `k` is owned or already a
/// reference (for `String` it yields `&str`, which `Borrow` accepts).
fn map_key(map_ty: &Type, k: &str) -> String {
    match map_ty {
        // `&*` turns a `String` into `&str`; anything else is borrowed as is
        Type::Map(kt, _) if **kt == Type::Str => format!("&*({})", k),
        _ => format!("&({})", k),
    }
}

/// Structural equality for the simple expressions that appear as map
/// receivers and keys (names, fields, literals, tuple indexes).
fn same_expr(a: &Expr, b: &Expr) -> bool {
    match (&a.kind, &b.kind) {
        (ExprKind::Ident(x), ExprKind::Ident(y)) => x == y,
        (ExprKind::SelfRef, ExprKind::SelfRef) => true,
        (ExprKind::Int(x), ExprKind::Int(y)) => x == y,
        (ExprKind::Str(x), ExprKind::Str(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| matches!((p, q), (StrPiece::Lit(s), StrPiece::Lit(t)) if s == t))
        }
        (ExprKind::Method { recv: r1, name: n1, args: a1 }, ExprKind::Method { recv: r2, name: n2, args: a2 }) => {
            n1 == n2 && a1.is_empty() && a2.is_empty() && same_expr(r1, r2)
        }
        (ExprKind::TupleIndex { recv: r1, index: i1 }, ExprKind::TupleIndex { recv: r2, index: i2 }) => i1 == i2 && same_expr(r1, r2),
        _ => false,
    }
}

fn stmt_mentions(s: &Stmt, name: &str) -> bool {
    let blk = |b: &Block| b.stmts.iter().any(|st| stmt_mentions(st, name));
    match s {
        Stmt::Bind { value, .. } | Stmt::Var { value, .. } | Stmt::OpAssign { value, .. } | Stmt::Destructure { value, .. } => expr_mentions(value, name),
        Stmt::FieldAssign { recv, value, .. } => expr_mentions(recv, name) || expr_mentions(value, name),
        Stmt::IndexAssign { recv, index, value, .. } => expr_mentions(recv, name) || expr_mentions(index, name) || expr_mentions(value, name),
        Stmt::Expr(e) => expr_mentions(e, name),
        Stmt::Return { value, .. } => value.as_ref().map(|e| expr_mentions(e, name)).unwrap_or(false),
        Stmt::While { cond, body } => expr_mentions(cond, name) || blk(body),
        Stmt::For { iter, filter, body, .. } => expr_mentions(iter, name) || filter.as_ref().map(|f| expr_mentions(f, name)).unwrap_or(false) || blk(body),
        Stmt::Break { .. } | Stmt::Next { .. } => false,
        Stmt::Assert { cond, .. } => expr_mentions(cond, name),
        Stmt::Shared { value, .. } => expr_mentions(value, name),
    }
}

/// `stmt_mentions`, plus names that are assigned to (`x += 1`, `x = v`).
fn stmt_uses(s: &Stmt, name: &str) -> bool {
    let assigned = match s {
        Stmt::OpAssign { name: n, .. } | Stmt::Bind { name: n, .. } => n == name,
        Stmt::Destructure { names, .. } => names.iter().any(|n| n == name),
        Stmt::While { body, .. } | Stmt::For { body, .. } => body.stmts.iter().any(|st| stmt_uses(st, name)),
        Stmt::Expr(e) => expr_assigns(e, name),
        _ => false,
    };
    assigned || stmt_mentions(s, name)
}

fn expr_assigns(e: &Expr, name: &str) -> bool {
    let blk = |b: &Block| b.stmts.iter().any(|st| stmt_uses(st, name));
    match &e.kind {
        ExprKind::If { branches, else_block } => branches.iter().any(|(_, b)| blk(b)) || else_block.as_ref().map(blk).unwrap_or(false),
        ExprKind::Match { arms, .. } => arms.iter().any(|a| blk(&a.body)),
        ExprKind::Lambda { body, .. } | ExprKind::Spawn(body) => blk(body),
        _ => false,
    }
}

fn block_mentions_self(b: &Block) -> bool {
    b.stmts.iter().any(|st| stmt_mentions(st, "self"))
}

fn expr_mentions(e: &Expr, name: &str) -> bool {
    let blk = |b: &Block| b.stmts.iter().any(|st| stmt_mentions(st, name));
    match &e.kind {
        ExprKind::Ident(n) => n == name,
        ExprKind::Rust(code) => code.contains(name),
        ExprKind::SelfRef => name == "self",
        ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) | ExprKind::None | ExprKind::Placeholder => false,
        ExprKind::Str(pieces) => pieces.iter().any(|p| matches!(p, StrPiece::Expr(x) if expr_mentions(x, name))),
        ExprKind::List(items) | ExprKind::Tuple(items) => items.iter().any(|i| expr_mentions(i, name)),
        ExprKind::MapLit(pairs) => pairs.iter().any(|(k, v)| expr_mentions(k, name) || expr_mentions(v, name)),
        ExprKind::SetLit(items) => items.iter().any(|i| expr_mentions(i, name)),
        ExprKind::Await(x) => expr_mentions(x, name),
        ExprKind::Spawn(b) => blk(b),
        ExprKind::Range { lo, hi, .. } => expr_mentions(lo, name) || expr_mentions(hi, name),
        ExprKind::Unary { expr, .. } | ExprKind::Some(expr) | ExprKind::Ok(expr) | ExprKind::Try(expr) | ExprKind::Unwrap(expr) | ExprKind::Puts(expr) | ExprKind::Warn(expr) => expr_mentions(expr, name),
        ExprKind::TupleIndex { recv, .. } => expr_mentions(recv, name),
        ExprKind::Index { recv, index } => expr_mentions(recv, name) || expr_mentions(index, name),
        ExprKind::Binary { lhs, rhs, .. } => expr_mentions(lhs, name) || expr_mentions(rhs, name),
        // a call to a block is a use of the block itself
        ExprKind::Call { name: callee, args } => callee == name || args.iter().any(|a| expr_mentions(&a.value, name)),
        ExprKind::Method { recv, args, .. } => expr_mentions(recv, name) || args.iter().any(|a| expr_mentions(&a.value, name)),
        ExprKind::If { branches, else_block } => branches.iter().any(|(c, b)| expr_mentions(c, name) || blk(b)) || else_block.as_ref().map(blk).unwrap_or(false),
        ExprKind::Match { scrutinee, arms } => expr_mentions(scrutinee, name) || arms.iter().any(|a| a.guard.as_ref().map(|g| expr_mentions(g, name)).unwrap_or(false) || blk(&a.body)),
        ExprKind::Lambda { body, .. } => blk(body),
    }
}

/// Rust method name for an operator method (`def +(o)` -> `op_add`).
fn op_method_name(op: &str) -> Option<&'static str> {
    Some(match op {
        "+" => "op_add",
        "-" => "op_sub",
        "*" => "op_mul",
        "/" => "op_div",
        "%" => "op_rem",
        "==" => "op_eq",
        "!=" => "op_ne",
        "<" => "op_lt",
        "<=" => "op_le",
        ">" => "op_gt",
        ">=" => "op_ge",
        _ => return None,
    })
}

fn parser_pos(s: &Stmt) -> (usize, usize) {
    crate::parser::stmt_pos(s)
}

/// `regex.Regex.new` as written, for messages.
fn foreign_display(e: &Expr) -> String {
    let mut segs: Vec<String> = Vec::new();
    let mut cur = e;
    loop {
        match &cur.kind {
            ExprKind::Method { recv, name, .. } => {
                segs.push(name.clone());
                cur = recv;
            }
            ExprKind::Ident(n) => {
                segs.push(n.clone());
                break;
            }
            _ => break,
        }
    }
    segs.reverse();
    segs.join(".")
}

/// Can a Lume value of type `have` be passed where `want` is expected?
fn types_compatible(have: &Type, want: &Type) -> bool {
    match (have, want) {
        (_, Type::Unknown) | (Type::Unknown, _) => true,
        (Type::List(a), Type::List(b)) | (Type::Option(a), Type::Option(b)) => types_compatible(a, b),
        (Type::Iter(a, _), Type::List(b)) => types_compatible(a, b),
        (a, b) => a == b,
    }
}

/// The value of an `Int` literal, possibly negated.
fn int_literal(e: &Expr) -> Option<i64> {
    match &e.kind {
        ExprKind::Int(v) => Some(*v),
        ExprKind::Unary { op: "-", expr } => int_literal(expr).and_then(|v| v.checked_neg()),
        _ => None,
    }
}

/// A pattern in the exhaustiveness matrix.
#[derive(Clone, Debug, PartialEq)]
enum Pat {
    Wild,
    /// A literal (number, string, range): matches some values, never all of them.
    Lit,
    /// A variant, `true`/`false`, a tuple `()`, or a list cell `[]` / `::`.
    Ctor(String, Vec<Pat>),
}

fn product(subs: Vec<Vec<Pat>>) -> Vec<Vec<Pat>> {
    let mut acc: Vec<Vec<Pat>> = vec![Vec::new()];
    for s in subs {
        let mut next = Vec::new();
        for a in &acc {
            for p in &s {
                let mut row = a.clone();
                row.push(p.clone());
                next.push(row);
            }
        }
        acc = next;
    }
    acc
}

/// Prints a witness in Lume pattern syntax.
fn pat_text(p: &Pat) -> String {
    match p {
        Pat::Wild | Pat::Lit => "_".into(),
        Pat::Ctor(c, args) if c == "()" => format!("({})", args.iter().map(pat_text).collect::<Vec<_>>().join(", ")),
        Pat::Ctor(c, _) if c == "[]" => "[]".into(),
        Pat::Ctor(c, _) if c == "::" => {
            let mut items = Vec::new();
            let mut cur = p;
            loop {
                match cur {
                    Pat::Ctor(c, args) if c == "::" => {
                        items.push(pat_text(&args[0]));
                        cur = &args[1];
                    }
                    Pat::Ctor(c, _) if c == "[]" => break,
                    _ => {
                        items.push("..rest".into());
                        break;
                    }
                }
            }
            format!("[{}]", items.join(", "))
        }
        Pat::Ctor(c, args) if args.is_empty() => c.clone(),
        Pat::Ctor(c, args) => format!("{}({})", c, args.iter().map(pat_text).collect::<Vec<_>>().join(", ")),
    }
}

fn pat_has_list(p: &Pattern) -> bool {
    match &p.kind {
        PatKind::List { .. } => true,
        PatKind::Or(alts) => alts.iter().any(pat_has_list),
        _ => false,
    }
}

fn describe_names(names: &[&String]) -> String {
    if names.is_empty() {
        "nothing".into()
    } else {
        names.iter().map(|n| format!("`{}`", n)).collect::<Vec<_>>().join(", ")
    }
}

/// Names of built-in types and Rust types the generated code relies on.
fn reserved_type_name(name: &str, line: usize, col: usize) -> Result<()> {
    const RESERVED: &[&str] = &[
        "Int", "Float", "Str", "Char", "Bool", "List", "Map", "Set", "Option", "Vec", "String", "Task", "Time", "File", "Math", "Rc", "Arc", "Mutex", "Some",
        "None", "Ok", "Err", "Clone", "Copy", "Iterator", "Ordering", "Self",
    ];
    if RESERVED.contains(&name) {
        let why = match name {
            "Vec" | "String" | "Rc" | "Arc" | "Mutex" | "Clone" | "Copy" | "Iterator" | "Ordering" | "Self" => "the compiler keeps this name for itself",
            _ => "this is a built-in Lume name",
        };
        return Err(LumeError::new(line, col, format!("`{}` cannot be a type name: {}", name, why))
            .with_help(format!("pick another name, like `My{}`", name)));
    }
    Ok(())
}

/// Annotates a `|x| body` closure's parameter with `&T`, so Rust can call it
/// with a reference from a generic helper.
fn key_closure(f: &str, elem_rust: &str) -> String {
    annotate_closure(f, &format!("&{}", elem_rust))
}

/// Gives a `|pat| body` closure's parameter an explicit type.
fn annotate_closure(f: &str, ty: &str) -> String {
    match f.strip_prefix('|').and_then(|rest| rest.find('|').map(|i| (&rest[..i], &rest[i + 1..]))) {
        Some((pat, body)) => format!("|{}: {}|{}", pat, ty, body),
        None => f.to_string(),
    }
}

/// `(k, v)` for a map entry reached by reference, with Copy parts dereferenced.
fn map_pair(k: &Type, v: &Type) -> String {
    format!("({}k, {}v)", if k.is_copy() { "*" } else { "" }, if v.is_copy() { "*" } else { "" })
}

/// How a set item argument is borrowed for `contains`/`remove`.
fn set_key(set_ty: &Type, k: &str) -> String {
    match set_ty {
        Type::Set(t) if **t == Type::Str => format!("&*({})", k),
        _ => format!("&({})", k),
    }
}

/// "add the type: ..." for a binding whose value does not say its full type.
fn type_hint(lhs: &str, value: &Expr, inferred: &Type) -> String {
    if matches!(&value.kind, ExprKind::MapLit(p) if p.is_empty()) {
        return format!("`{{}}` is an empty map or an empty set; say which: `{}: {{Str: Int}} = {{}}` or `{}: {{Str}} = {{}}`", lhs, lhs);
    }
    if matches!(inferred, Type::Map(k, v) if **k == Type::Unknown && **v == Type::Unknown) {
        return format!("add the type: `{}: {{Str: Int}} = ...` for a map, `{}: {{Str}} = ...` for a set", lhs, lhs);
    }
    format!("add the type: `{}: {} = ...`", lhs, suggest_type(inferred))
}

/// `xs.fold({}) { |acc, x| acc.union(x) }`: an empty seed whose parts are not
/// known takes the item type when the items are the same kind of collection.
fn fold_acc_type(seed: Type, elem: &Type) -> Type {
    match (&seed, elem) {
        (Type::Map(k, v), Type::Set(_) | Type::Map(..)) if **k == Type::Unknown && **v == Type::Unknown => elem.clone(),
        (Type::List(i), Type::List(_)) if **i == Type::Unknown => elem.clone(),
        _ => seed,
    }
}

/// Built-in methods that change their receiver.
fn is_mutating_builtin(name: &str) -> bool {
    matches!(name, "push" | "pop" | "insert" | "remove_at" | "add" | "remove")
}

/// Parameter types of the built-in methods that take arguments, where they
/// can be stated. `None`: no check (a block method, or one checked elsewhere).
fn builtin_params(recv: &Type, name: &str) -> Option<Vec<Type>> {
    let int = || Type::Int;
    let st = || Type::Str;
    Some(match (recv, name) {
        (Type::List(e), "push" | "contains?" | "index_of") => vec![(**e).clone()],
        (Type::List(e), "insert") => vec![int(), (**e).clone()],
        (Type::List(_), "remove_at" | "take" | "skip") => vec![int()],
        (Type::List(e), "zip") => {
            let _ = e;
            return None;
        }
        (Type::List(_) | Type::Iter(..) | Type::Set(_), "join") => vec![st()],
        (Type::Set(e), "add" | "remove" | "contains?") => vec![(**e).clone()],
        (Type::Set(_), "union" | "intersect" | "diff" | "subset?" | "superset?") => vec![recv.clone()],
        (Type::Map(k, _), "contains?" | "remove" | "get") => vec![(**k).clone()],
        (Type::Map(..), "merge") => vec![recv.clone()],
        (Type::Str, "contains?" | "starts_with?" | "ends_with?" | "index_of" | "split") => vec![st()],
        (Type::Str, "replace") => vec![st(), st()],
        (Type::Str, "repeat" | "pad" | "pad_right") => vec![int()],
        (Type::Str, "slice") => vec![int(), int()],
        (Type::Int | Type::Float | Type::Bool, "pad" | "pad_right") => vec![int()],
        (Type::Int, "max" | "min" | "pow") => vec![int()],
        (Type::Int, "clamp") => vec![int(), int()],
        (Type::Float, "max" | "min" | "pow") => vec![Type::Float],
        (Type::Float, "clamp") => vec![Type::Float, Type::Float],
        (Type::Option(i), "or") if **i == Type::Char => return None,
        (Type::Option(i), "or") => vec![(**i).clone()],
        (Type::Result(t, _), "or") => vec![(**t).clone()],
        (Type::Option(_), "or_error") => vec![st()],
        _ => return None,
    })
}

/// "a `Str`" / "an `Int`", for messages that name a type mid-sentence.
fn a_type(t: &Type) -> String {
    let n = type_name(t);
    let article = if n.chars().next().map(|c| "AEIOU".contains(c.to_ascii_uppercase())).unwrap_or(false) { "an" } else { "a" };
    format!("{} `{}`", article, n)
}

/// `Str` and `Char` mix in comparisons and `+`.
fn text_like(t: &Type) -> bool {
    matches!(t, Type::Str | Type::Char)
}

/// A plain string literal (no interpolation) as a Rust `&str` literal, for
/// positions that only read it: operands of `+` and comparisons, method
/// arguments that borrow. Saves an allocation per use.
fn str_literal(e: &Expr) -> Option<String> {
    match &e.kind {
        ExprKind::Str(pieces) if pieces.iter().all(|p| matches!(p, StrPiece::Lit(_))) => {
            let t: String = pieces.iter().map(|p| match p { StrPiece::Lit(s) => s.as_str(), _ => "" }).collect();
            Some(format!("\"{}\"", escape_rust_str(&t, false)))
        }
        _ => None,
    }
}

/// A one-character string literal, as the character.
fn char_literal(e: &Expr) -> Option<char> {
    match &e.kind {
        ExprKind::Str(parts) => match parts.as_slice() {
            [StrPiece::Lit(t)] => {
                let mut cs = t.chars();
                match (cs.next(), cs.next()) {
                    (Some(c), None) => Some(c),
                    _ => None,
                }
            }
            _ => None,
        },
        _ => None,
    }
}

/// The body of a Rust `char` literal.
fn rust_char(c: char) -> String {
    match c {
        '\'' => "\\'".into(),
        '\\' => "\\\\".into(),
        '\n' => "\\n".into(),
        '\t' => "\\t".into(),
        '\r' => "\\r".into(),
        '\0' => "\\0".into(),
        c if (c as u32) < 0x20 => format!("\\u{{{:x}}}", c as u32),
        c => c.to_string(),
    }
}
