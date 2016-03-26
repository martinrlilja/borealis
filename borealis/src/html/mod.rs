
pub use self::doctype::Doctype;
pub use self::document::Document;
pub use self::node::{Attribute, ElementType, Node, TextNode, CommentNode, ElementNode};

mod doctype;
mod document;
mod node;
