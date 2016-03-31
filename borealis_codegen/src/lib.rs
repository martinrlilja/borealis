#![feature(plugin_registrar, rustc_private, plugin)]
#![plugin(quasi_macros)]

extern crate aster;
extern crate borealis;
extern crate quasi;
extern crate rustc;
extern crate rustc_plugin;
extern crate string_cache;
#[macro_use]
extern crate syntax;

use borealis::html::Document;

use std::io::Read;
use std::fs::File;
use std::path::Path;

use syntax::attr;
use syntax::ast::{Item, ItemKind, LitKind, MetaItem, MetaItemKind};
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt, SyntaxExtension};
use syntax::parse::token::{self, InternedString};
use syntax::ptr::P;

use rustc_plugin::Registry;

// https://github.com/serde-rs/serde/blob/master/serde_codegen/src/ser.rs
fn expand_derive_document_template(cx: &mut ExtCtxt,
                                   span: Span,
                                   meta_item: &MetaItem,
                                   annotatable: &Annotatable,
                                   push: &mut FnMut(Annotatable)) {
    let item = match *annotatable {
        Annotatable::Item(ref item) => item,
        _ => {
            cx.span_err(meta_item.span,
                        "`#[DocumentTemplate(..)]` may only be applied to structs");
            return;
        }
    };

    let builder = aster::AstBuilder::new().span(span);

    if let Some(item) = build_document_template_item(&cx, &builder, &item) {
        push(Annotatable::Item(item));
    }
}

fn build_document_template_item(cx: &ExtCtxt,
                                builder: &aster::AstBuilder,
                                item: &Item)
                                -> Option<P<Item>> {
    let filename = cx.filename.clone().unwrap();
    let filename = Path::new(&filename);
    let argument = attribute_argument(cx, item, "DocumentTemplate").unwrap();

    filename.join(Path::new(&*argument));
    let mut file = match File::open(&filename) {
        Ok(f) => f,
        Err(e) => panic!("{:?}", e),
    };

    let mut file_str = String::new();
    file.read_to_string(&mut file_str).unwrap();

    let document = Document::parse_str(&file_str);

    let generics = match item.node {
        ItemKind::Struct(_, ref generics) => generics,
        _ => {
            cx.span_err(item.span,
                        "`#[DocumentTemplate(..)]` may only be applied to structs");
            return None;
        }
    };

    let impl_generics = builder.from_generics(generics.clone())
                               .add_ty_param_bound(builder.path()
                                                          .global()
                                                          .ids(&["borealis", "DocumentTemplate"])
                                                          .build())
                               .build();
    let ty = builder.ty()
                    .path()
                    .segment(item.ident)
                    .with_generics(impl_generics.clone())
                    .build()
                    .build();

    let where_clause = &impl_generics.where_clause;

    Some(quote_item!(cx,
                   impl $impl_generics ::borealis::DocumentTemplate for $ty $where_clause {
                       fn document_template(self) -> ::borealis::html::Document {
                           ::borealis::html::Document::new(None, None)
                       }
                   })
           .unwrap())
}

fn attribute_argument(cx: &ExtCtxt, item: &Item, attribute: &str) -> Option<InternedString> {
    let attribute_items = item.attrs().iter().filter_map(|a| {
        match a.node.value.node {
            MetaItemKind::List(ref name, ref items) if name == &attribute => {
                attr::mark_used(&a);
                Some(items)
            }
            _ => None,
        }
    });

    for attr_items in attribute_items {
        for attr_item in attr_items {
            match attr_item.node {
                MetaItemKind::NameValue(ref name, ref value) if name == &"file" => {
                    if let LitKind::Str(ref s, _) = value.node {
                        return Some(s.clone());
                    } else {
                        cx.span_err(value.span, "this must be a string");
                    }
                }
                _ => continue,
            }
        }
    }

    return None;
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        token::intern("DocumentTemplate"),
        SyntaxExtension::MultiDecorator(
            Box::new(expand_derive_document_template)));
}
