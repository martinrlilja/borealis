
use html5ever;
use html5ever::tendril::StrTendril;
use string_cache::QualName;

#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    name: QualName,
    value: StrTendril,
}

impl Attribute {
    pub fn new(name: QualName, value: StrTendril) -> Attribute {
        Attribute {
            name: name,
            value: value,
        }
    }

    pub fn new_string(name: String, value: String) -> Attribute {
        Attribute::new(QualName::new(ns!(), name.into()), value.into())
    }

    pub fn new_str(name: &str, value: &str) -> Attribute {
        Attribute::new_string(name.to_owned(), value.to_owned())
    }

    pub fn name(&self) -> &QualName {
        &self.name
    }

    pub fn value(&self) -> &StrTendril {
        &self.value
    }
}

impl From<html5ever::Attribute> for Attribute {
    fn from(attr: html5ever::Attribute) -> Attribute {
        Attribute {
            name: attr.name,
            value: attr.value,
        }
    }
}

impl Into<html5ever::Attribute> for Attribute {
    fn into(self) -> html5ever::Attribute {
        html5ever::Attribute {
            name: self.name,
            value: self.value,
        }
    }
}
