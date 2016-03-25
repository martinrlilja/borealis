
use std::io::{self, Write};

use html5ever::tendril::StrTendril;
use html5ever::serialize::{Serializable, Serializer, TraversalScope};

#[derive(Clone, Debug, PartialEq)]
pub struct Doctype {
    name: StrTendril,
    public_id: StrTendril,
    system_id: StrTendril,
}

impl Doctype {
    pub fn new(name: StrTendril, public_id: StrTendril, system_id: StrTendril) -> Doctype {
        Doctype {
            name: name,
            public_id: public_id,
            system_id: system_id,
        }
    }

    pub fn new_string(name: String, public_id: String, system_id: String) -> Doctype {
        Doctype::new(name.into(), public_id.into(), system_id.into())
    }

    pub fn new_str(name: &str, public_id: &str, system_id: &str) -> Doctype {
        Doctype::new_string(name.to_owned(), public_id.to_owned(), system_id.to_owned())
    }

    pub fn new_html5() -> Doctype {
        Doctype::new_str("html", "", "")
    }

    pub fn name(&self) -> &StrTendril {
        &self.name
    }

    pub fn public_id(&self) -> &StrTendril {
        &self.public_id
    }

    pub fn system_id(&self) -> &StrTendril {
        &self.system_id
    }
}

impl Serializable for Doctype {
    fn serialize<'wr, Wr: Write>(&self,
                                 serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope)
                                 -> io::Result<()> {
        match traversal_scope {
            TraversalScope::IncludeNode => serializer.write_doctype(&self.name),
            TraversalScope::ChildrenOnly => Ok(()),
        }
    }
}
