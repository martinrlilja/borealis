#![feature(plugin_registrar, rustc_private, plugin)]
#![plugin(quasi_macros)]

extern crate aster;
extern crate quasi;
extern crate rustc;
extern crate rustc_plugin;
extern crate string_cache;
#[macro_use]
extern crate syntax;

use syntax::ast::{Item, ItemKind, MetaItem};
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt, SyntaxExtension};
use syntax::parse::token;
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
                        "`#[derive(DocumentTemplate(..))]` may only be applied to structs");
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
    let generics = match item.node {
        ItemKind::Struct(_, ref generics) => generics,
        _ => {
            cx.span_err(item.span,
                        "`#[derive(DocumentTemplate(..))]` may only be applied to structs");
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

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        token::intern("derive_DocumentTemplate"),
        SyntaxExtension::MultiDecorator(
            Box::new(expand_derive_document_template)));
}
