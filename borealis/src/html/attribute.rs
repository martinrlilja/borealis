
use html5ever;
use html5ever::tendril::StrTendril;

use string_cache::QualName;

/// Represents an attribute of an element node.
#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    name: QualName,
    value: StrTendril,
}

impl Attribute {
    /// Creates a new attribute from the given name and value.
    pub fn new(name: QualName, value: StrTendril) -> Attribute {
        Attribute {
            name: name,
            value: value,
        }
    }

    /// Convenience function for `Attribute::new`, assumes no namespace for `name`.
    pub fn new_str(name: &str, value: &str) -> Attribute {
        Attribute::new_string(name.to_owned(), value.to_owned())
    }

    /// Convenience function for `Attribute::new`, assumes no namespace for `name`.
    pub fn new_string(name: String, value: String) -> Attribute {
        Attribute::new(QualName::new(ns!(), name.into()), value.into())
    }

    /// Returns the name of the attribute.
    pub fn name(&self) -> &QualName {
        &self.name
    }

    /// Returns the value of the attribute.
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

#[cfg(test)]
mod tests {
    use super::*;

    use html5ever::tendril::StrTendril;

    use string_cache::QualName;

    #[test]
    fn test_new() {
        let name = "name".to_owned();
        let qualname = QualName::new(ns!(), name.clone().into());

        let value = "Test".to_owned();
        let tendril = StrTendril::from(value.clone());

        let attribute = Attribute::new(qualname.clone(), tendril.clone());
        assert_eq!(*attribute.name(), qualname);
        assert_eq!(*attribute.value(), tendril);

        let attribute = Attribute::new_str(&name, &value);
        assert_eq!(*attribute.name(), qualname);
        assert_eq!(*attribute.value(), tendril);

        let attribute = Attribute::new_string(name.clone(), value.clone());
        assert_eq!(*attribute.name(), qualname);
        assert_eq!(*attribute.value(), tendril);
    }
}
