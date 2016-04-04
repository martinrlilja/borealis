#![feature(custom_attribute, test)]

extern crate test;
extern crate html5ever;
#[macro_use(qualname, ns, atom)]
extern crate string_cache as sc;

pub use template::{DocumentTemplate, FragmentTemplate};
pub use html5ever::tendril;

mod dom;
pub mod html;
mod template;

pub mod string_cache {
    pub use sc::*;
}
