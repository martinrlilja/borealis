
use std::io::{self, Write};

use html5ever::tendril::StrTendril;
use html5ever::serialize::{Serializable, Serializer, TraversalScope};

#[derive(Clone, Debug, PartialEq)]
pub struct DoctypeValue(StrTendril);

impl From<StrTendril> for DoctypeValue {
    fn from(value: StrTendril) -> DoctypeValue {
        DoctypeValue(value)
    }
}

impl From<String> for DoctypeValue {
    fn from(value: String) -> DoctypeValue {
        DoctypeValue(value.into())
    }
}

impl<'a> From<&'a str> for DoctypeValue {
    fn from(value: &'a str) -> DoctypeValue {
        DoctypeValue(value.to_owned().into())
    }
}

/// Represents the doctype of a document.
#[derive(Clone, Debug, PartialEq)]
pub struct Doctype {
    name: DoctypeValue,
    public_id: DoctypeValue,
    system_id: DoctypeValue,
}

impl Doctype {
    pub fn new<N: Into<DoctypeValue>, P: Into<DoctypeValue>, S: Into<DoctypeValue>>(name: N,
                                                                                    public_id: P,
                                                                                    system_id: S)
                                                                                    -> Doctype {
        Doctype {
            name: name.into(),
            public_id: public_id.into(),
            system_id: system_id.into(),
        }
    }

    pub fn new_html5() -> Doctype {
        Doctype::new("html", "", "")
    }

    #[inline]
    pub fn name(&self) -> &StrTendril {
        &self.name.0
    }

    #[inline]
    pub fn public_id(&self) -> &StrTendril {
        &self.public_id.0
    }

    #[inline]
    pub fn system_id(&self) -> &StrTendril {
        &self.system_id.0
    }
}

impl Serializable for Doctype {
    fn serialize<'wr, Wr: Write>(&self,
                                 serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope)
                                 -> io::Result<()> {
        match traversal_scope {
            TraversalScope::IncludeNode => serializer.write_doctype(&self.name()),
            TraversalScope::ChildrenOnly => Ok(()),
        }
    }
}
