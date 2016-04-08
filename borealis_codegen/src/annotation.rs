
use syntax::attr;
use syntax::ast::{Item, Lit, MetaItemKind};
use syntax::parse::token::InternedString;

use std::collections::HashMap;

pub struct Annotation {
    attributes: HashMap<InternedString, Lit>,
}

impl Annotation {
    pub fn new(item: &Item, attribute: &str) -> Annotation {
        let map = Annotation::items_to_map(item, attribute);

        Annotation { attributes: map }
    }

    fn items_to_map(item: &Item, attribute: &str) -> HashMap<InternedString, Lit> {
        let items = item.attrs().iter().filter_map(|a| {
            match a.node.value.node {
                MetaItemKind::List(ref name, ref items) if name == &attribute => {
                    attr::mark_used(&a);
                    Some(items)
                }
                _ => None,
            }
        });

        let mut map = HashMap::new();

        for attr_items in items {
            for attr_item in attr_items {
                match attr_item.node {
                    MetaItemKind::NameValue(ref name, ref value) => {
                        map.insert(name.clone(), value.clone());
                    }
                    _ => continue,
                }
            }
        }

        map
    }

    pub fn find(&self, name: &'static str) -> Option<&Lit> {
        self.attributes.get(&InternedString::new(name))
    }
}
