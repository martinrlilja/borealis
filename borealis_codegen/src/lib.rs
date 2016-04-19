#![feature(plugin_registrar, rustc_private, plugin)]
#![plugin(quasi_macros, regex_macros)]

extern crate aster;
extern crate borealis;
extern crate quasi;
extern crate regex;
extern crate rustc;
extern crate rustc_plugin;
extern crate string_cache;
#[macro_use]
extern crate syntax;

use borealis::{Document, Fragment};

use std::rc::Rc;
use std::path::Path;

use syntax::ast::{Item, ItemKind, LitKind, MetaItem};
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt, SyntaxExtension};
use syntax::parse::token::{self, InternedString};
use syntax::ptr::P;

use rustc_plugin::Registry;

use annotation::Annotation;
use html_expr::{document_expression, node_expression};

mod annotation;
mod expr;
mod html_expr;

fn get_file_argument<'a>(annotation: &'a Annotation) -> Option<&'a InternedString> {
    match annotation.find_value("file") {
        Some(lit) => {
            match lit.node {
                LitKind::Str(ref s, _) => Some(s),
                _ => None,
            }
        }
        None => None,
    }
}

fn get_file(cx: &ExtCtxt, item: &Item, annotation: &Annotation) -> Result<Rc<String>, ()> {
    let filename = cx.filename.clone().unwrap();
    let filename = Path::new(&filename);

    let file = match get_file_argument(&annotation) {
        Some(file) => file,
        None => {
            cx.span_err(item.span,
                        "`#[template_document(..)]` requires file argument of the type string");
            return Err(());
        }
    };

    let filename = filename.parent().unwrap().join(Path::new(&**file));

    match cx.codemap().load_file(&filename) {
        Ok(ref file) => {
            let s = file.src.as_ref().unwrap().clone();

            if annotation.has_flag("trim") {
                Ok(Rc::new(s.trim().into()))
            } else {
                Ok(s)
            }
        }
        Err(err) => {
            cx.span_err(item.span,
                        &format!("`#[template_document(..)]` gave an error when opening {:?}: \
                                  {:?}",
                                 filename,
                                 err));
            return Err(());
        }
    }
}

fn expand_derive_document_template(cx: &mut ExtCtxt,
                                   span: Span,
                                   meta_item: &MetaItem,
                                   annotatable: &Annotatable,
                                   push: &mut FnMut(Annotatable)) {
    let item = match *annotatable {
        Annotatable::Item(ref item) => item,
        _ => {
            cx.span_err(meta_item.span,
                        "`#[template_document(..)]` may only be applied to structs");
            return;
        }
    };

    let builder = aster::AstBuilder::new().span(span);

    if let Ok(item) = build_document_template_item(&cx, &builder, &item) {
        push(Annotatable::Item(item));
    }
}

fn build_document_template_item(cx: &ExtCtxt,
                                builder: &aster::AstBuilder,
                                item: &Item)
                                -> Result<P<Item>, ()> {
    let annotation = Annotation::new(item, "template_document");

    let file = try!(get_file(cx, item, &annotation));

    let document = Document::parse_str(&file).handle();
    let generics = match item.node {
        ItemKind::Struct(_, ref generics) => generics,
        _ => {
            cx.span_err(item.span,
                        "`#[template_document(..)]` may only be applied to structs");
            return Err(());
        }
    };

    let impl_generics = builder.from_generics(generics.clone())
                               .add_ty_param_bound(builder.path()
                                                          .global()
                                                          .ids(&["borealis", "serialize", "SerializeDocument"])
                                                          .build())
                               .build();
    let ty = builder.ty()
                    .path()
                    .segment(item.ident)
                    .with_generics(impl_generics.clone())
                    .build()
                    .build();

    let where_clause = &impl_generics.where_clause;

    let document_expr = document_expression(cx, builder, &document);

    Ok(quote_item!(cx,
            impl $impl_generics ::borealis::serialize::SerializeDocument for $ty
                $where_clause
            {
                fn serialize_document<W>(self, s: ::borealis::serialize::DocumentSerializer<W>)
                    where W: ::std::io::Write
                {
                    $document_expr
                }
            })
           .unwrap())
}

fn expand_derive_fragment_template(cx: &mut ExtCtxt,
                                   span: Span,
                                   meta_item: &MetaItem,
                                   annotatable: &Annotatable,
                                   push: &mut FnMut(Annotatable)) {
    let item = match *annotatable {
        Annotatable::Item(ref item) => item,
        _ => {
            cx.span_err(meta_item.span,
                        "`#[template_fragment(..)]` may only be applied to structs");
            return;
        }
    };

    let builder = aster::AstBuilder::new().span(span);

    if let Ok(item) = build_fragment_template_item(&cx, &builder, &item) {
        push(Annotatable::Item(item));
    }
}

fn build_fragment_template_item(cx: &ExtCtxt,
                                builder: &aster::AstBuilder,
                                item: &Item)
                                -> Result<P<Item>, ()> {
    let annotation = Annotation::new(item, "template_fragment");

    let file = try!(get_file(cx, item, &annotation));

    let handles = Fragment::parse_str(&file).handles();
    let generics = match item.node {
        ItemKind::Struct(_, ref generics) => generics,
        _ => {
            cx.span_err(item.span,
                        "`#[template_fragment(..)]` may only be applied to structs");
            return Err(());
        }
    };

    let impl_generics = builder.from_generics(generics.clone())
                               .add_ty_param_bound(builder.path()
                                                          .global()
                                                          .ids(&["borealis", "serialize", "SerializeNode"])
                                                          .build())
                               .build();
    let ty = builder.ty()
                    .path()
                    .segment(item.ident)
                    .with_generics(impl_generics.clone())
                    .build()
                    .build();

    let where_clause = &impl_generics.where_clause;
    let exprs: Vec<_> = handles.iter().map(|e| node_expression(cx, builder, e)).collect();

    Ok(quote_item!(cx,
            impl $impl_generics ::borealis::serialize::SerializeNode for $ty
                $where_clause
            {
                fn serialize_node<W>(self, s: &mut ::borealis::serialize::NodeSerializer<W>)
                    where W: ::std::io::Write
                {
                    $exprs
                }
            }
        ).unwrap())
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        token::intern("template_document"),
        SyntaxExtension::MultiDecorator(
            Box::new(expand_derive_document_template)));

    reg.register_syntax_extension(
        token::intern("template_fragment"),
        SyntaxExtension::MultiDecorator(
            Box::new(expand_derive_fragment_template)));
}
