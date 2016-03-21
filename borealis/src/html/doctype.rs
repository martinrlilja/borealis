
use std::io::{self, Write};

use html5ever::tendril::StrTendril;
use html5ever::serialize::{Serializable, Serializer, TraversalScope};

#[derive(Clone, Debug, PartialEq)]
pub struct Doctype {
    name:      StrTendril,
    public_id: StrTendril,
    system_id: StrTendril,
}

impl Doctype {
    pub fn new(name: StrTendril, public_id: StrTendril, system_id: StrTendril) -> Doctype {
        Doctype {
            name:      name,
            public_id: public_id,
            system_id: system_id,
        }
    }

    pub fn new_string(name: String, public_id: String, system_id: String) -> Doctype {
        Doctype::new(StrTendril::from(name), StrTendril::from(public_id), StrTendril::from(system_id))
    }

    pub fn new_str(name: &str, public_id: &str, system_id: &str) -> Doctype {
        Doctype::new_string(name.to_owned(), public_id.to_owned(), system_id.to_owned())
    }

    pub fn new_html5() -> Doctype {
        Doctype::new_str("html", "", "")
    }
}

impl Serializable for Doctype {
    fn serialize<'wr, Wr: Write>(&self, serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope) -> io::Result<()>
    {
        match traversal_scope {
            TraversalScope::IncludeNode  => serializer.write_doctype(&self.name),
            TraversalScope::ChildrenOnly => Ok(()),
        }
    }
}
