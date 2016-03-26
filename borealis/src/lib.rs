#![feature(custom_attribute, test)]

extern crate test;
extern crate html5ever;
#[macro_use(qualname, ns, atom)]
extern crate string_cache;

pub use tohtml::{DocumentTemplate, FragmentTemplate};

mod dom;
pub mod html;
mod tohtml;
