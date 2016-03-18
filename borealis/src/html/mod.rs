
use html5ever;
use html5ever::tendril::StrTendril;
use html5ever::serialize::{Serializable, Serializer, TraversalScope};

use string_cache::QualName;

pub use self::doctype::Doctype;
pub use self::document::Document;
pub use self::node::{Attribute, ElementType, Node, TextNode, CommentNode, ElementNode};

mod doctype;
mod document;
mod node;
