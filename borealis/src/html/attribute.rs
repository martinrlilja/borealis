
use html5ever;
use html5ever::tendril::StrTendril;

use string_cache::QualName;

#[derive(Clone, Debug, PartialEq)]
pub struct AttributeName(QualName);

impl From<QualName> for AttributeName {
    fn from(name: QualName) -> AttributeName {
        AttributeName(name)
    }
}

impl From<String> for AttributeName {
    fn from(name: String) -> AttributeName {
        AttributeName(QualName::new(ns!(), name.into()))
    }
}

impl<'a> From<&'a str> for AttributeName {
    fn from(name: &'a str) -> AttributeName {
        AttributeName(QualName::new(ns!(), name.clone().into()))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AttributeValue(StrTendril);

impl From<StrTendril> for AttributeValue {
    fn from(value: StrTendril) -> AttributeValue {
        AttributeValue(value)
    }
}

impl From<String> for AttributeValue {
    fn from(value: String) -> AttributeValue {
        AttributeValue(value.into())
    }
}

impl<'a> From<&'a str> for AttributeValue {
    fn from(value: &'a str) -> AttributeValue {
        AttributeValue(value.clone().into())
    }
}

/// Represents an attribute of an element node.
#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    name: AttributeName,
    value: AttributeValue,
}

impl Attribute {
    /// Creates a new attribute from the given name and value.
    pub fn new<N: Into<AttributeName>, V: Into<AttributeValue>>(name: N, value: V) -> Attribute {
        Attribute {
            name: name.into(),
            value: value.into(),
        }
    }

    /// The name of the attribute.
    #[inline]
    pub fn name(&self) -> &QualName {
        &self.name.0
    }

    /// The value of the attribute.
    #[inline]
    pub fn value(&self) -> &StrTendril {
        &self.value.0
    }
}

impl From<html5ever::Attribute> for Attribute {
    fn from(attr: html5ever::Attribute) -> Attribute {
        Attribute::new(attr.name, attr.value)
    }
}

impl Into<html5ever::Attribute> for Attribute {
    fn into(self) -> html5ever::Attribute {
        html5ever::Attribute {
            name: self.name.0,
            value: self.value.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use html5ever::tendril::StrTendril;

    use string_cache::QualName;

    #[test]
    fn test_new() {
        let name = QualName::new(ns!(), "name".to_owned().clone().into());
        let value = StrTendril::from("Test".to_owned().clone());

        let attribute = Attribute::new(name.clone(), value.clone());
        assert_eq!(*attribute.name(), name);
        assert_eq!(*attribute.value(), value);
    }
}
