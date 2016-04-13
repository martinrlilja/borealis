#![cfg_attr(feature = "nightly", feature(custom_attribute, test))]

#[cfg(feature = "nightly")]
extern crate test;
extern crate html5ever;
#[macro_use(qualname, ns, atom)]
extern crate string_cache as sc;

pub use html5ever::tendril;
pub use template::{IntoDocument, IntoNode, IntoNodes, IntoNodesIter};

mod dom;
pub mod html;
mod template;

pub mod string_cache {
    pub use sc::*;
}
