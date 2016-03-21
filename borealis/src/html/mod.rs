
use std::cell::RefCell;
use std::io::{self, Write};
use std::ops::Deref;
use std::rc::{Rc, Weak};

use html5ever;
use html5ever::tendril::StrTendril;
use html5ever::serialize::{Serializable, Serializer, TraversalScope};

use string_cache::QualName;

pub use self::doctype::Doctype;
pub use self::document::Document;
pub use self::handle::{Handle, ParentHandle, TreeHandle};
pub use self::node::{Attribute, ElementType, Node, TextNode, CommentNode, ElementNode};

mod doctype;
mod document;
mod handle;
mod node;
