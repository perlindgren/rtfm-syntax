//! Parser of the `app!` macro used by the Real Time For the Masses (RTFM)
//! framework
// #![deny(missing_debug_implementations)]
// #![deny(missing_docs)]
// #![deny(warnings)]
#![feature(match_default_bindings)]
#![feature(proc_macro)]

#[macro_use]
extern crate error_chain;
// #[macro_use]
extern crate proc_macro;
extern crate quote;
#[macro_use]
extern crate syn;
extern crate either;
extern crate proc_macro2;

// use either::Either;
// use proc_macro2::TokenStream;

pub mod check;
pub mod error;

mod util;

mod parse;

use std::collections::{HashMap, HashSet};

// use quote::Tokens;
use syn::{Expr, Ident, Path, Type};

use error::*;

/// `[$($ident),*]`
pub type Resources = HashSet<Ident>;

/// `$(static $Ident: $Ty = $expr;)*`
pub type Statics = HashMap<Ident, Static>;

/// `$($Ident: { .. },)*`
pub type Tasks = HashMap<Ident, Task>;

/// `app! { .. }`
#[derive(Debug)]
pub struct App {
    /// `device: $path`
    pub device: Path,
    /// `idle: { $Idle }`
    pub idle: Option<Idle>,
    /// `init: { $Init }`
    pub init: Option<Init>,
    /// `resources: $Statics`
    pub resources: Option<Statics>,
    // /// `tasks: { $Tasks }`
    pub tasks: Option<Tasks>,
    _extensible: (),
}

/// `idle: { .. }`
#[derive(Debug, PartialEq)]
pub struct Idle {
    /// `path: $Path`
    pub path: Option<Path>,
    /// `resources: $Resources`
    pub resources: Option<Resources>,
    _extensible: (),
}

/// `init: { .. }`
#[derive(Debug, PartialEq)]
pub struct Init {
    /// `path: $Path`
    pub path: Option<Path>,
    /// `resources: $Resources`
    pub resources: Option<Resources>,
    _extensible: (),
}

/// `$Ident: { .. }`
#[derive(Debug)]
pub struct Task {
    /// `enabled: $bool`
    pub enabled: Option<bool>,
    /// `path: $Path`
    pub path: Option<Path>,
    /// `priority: $u8`
    pub priority: Option<u8>,
    /// `interarrival: $u32`
    pub interarrival: Option<u32>,
    /// `resources: $Resources`
    pub resources: Option<Resources>,
    _extensible: (),
}

/// `static $Ident: $Ty = $Expr;`
#[derive(Debug)]
pub struct Static {
    /// `$Expr`
    pub expr: Option<Expr>,
    /// `$Ty`
    pub ty: Type,
    _extensible: (),
}

impl App {
    /// Parses the contents of the `app! { .. }` macro
    pub fn parse(input: proc_macro::TokenStream) -> Result<Self> {
        parse::parse_app(input)
    }
}
