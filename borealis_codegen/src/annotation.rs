
use syntax::attr;
use syntax::ast::{Item, Lit, MetaItemKind};
use syntax::parse::token::InternedString;

use std::collections::{HashMap, HashSet};

pub struct Annotation {
    values: HashMap<InternedString, Lit>,
    flags: HashSet<InternedString>,
}

impl Annotation {
    pub fn new(item: &Item, attribute: &str) -> Annotation {
        let items = item.attrs().iter().filter_map(|a| {
            match a.node.value.node {
                MetaItemKind::List(ref name, ref items) if name == &attribute => {
                    attr::mark_used(&a);
                    Some(items)
                }
                _ => None,
            }
        });

        let mut values = HashMap::new();
        let mut flags = HashSet::new();

        for attr_items in items {
            for attr_item in attr_items {
                match attr_item.node {
                    MetaItemKind::NameValue(ref name, ref value) => {
                        values.insert(name.clone(), value.clone());
                    }
                    MetaItemKind::Word(ref name) => {
                        flags.insert(name.clone());
                    }
                    _ => continue,
                }
            }
        }

        Annotation {
            values: values,
            flags: flags,
        }
    }

    pub fn find_value(&self, name: &'static str) -> Option<&Lit> {
        self.values.get(&InternedString::new(name))
    }

    pub fn has_flag(&self, name: &'static str) -> bool {
        self.flags.contains(&InternedString::new(name))
    }
}
