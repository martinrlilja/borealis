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

use borealis::string_cache::QualName;
use borealis::html::{Attribute, Doctype, Document, Node, ElementNode, ElementType};

use std::io::Read;
use std::fs::File;
use std::path::Path;

use syntax::attr;
use syntax::ast::{Expr, Item, ItemKind, LitKind, MetaItem, MetaItemKind};
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

    let filename = filename.parent().unwrap().join(Path::new(&*argument));
    let mut file = File::open(&filename).unwrap();

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

    let document_expr = document_expression(cx, builder, &document);

    Some(quote_item!(cx,
                   impl $impl_generics ::borealis::DocumentTemplate for $ty $where_clause {
                       fn document_template(self) -> ::borealis::html::Document {
                           $document_expr
                       }
                   })
             .unwrap())
}

fn document_expression(cx: &ExtCtxt, builder: &aster::AstBuilder, document: &Document) -> P<Expr> {
    fn option_to_expr(cx: &ExtCtxt, builder: &aster::AstBuilder, expr: Option<P<Expr>>) -> P<Expr> {
        match expr {
            Some(expr) => quote_expr!(cx, { Some($expr) }),
            None => builder.expr().none(),
        }
    }

    let doctype_expr = option_to_expr(cx,
                                      builder,
                                      document.doctype()
                                              .map(|d| doctype_expression(cx, builder, d)));
    let child_expr = option_to_expr(cx,
                                    builder,
                                    document.child().map(|c| node_expression(cx, builder, c)));

    quote_expr!(cx, {
        ::borealis::html::Document::new($doctype_expr, $child_expr)
    })
}

fn doctype_expression(cx: &ExtCtxt, builder: &aster::AstBuilder, doctype: &Doctype) -> P<Expr> {
    let name = string_expr(builder, doctype.name());
    let public_id = string_expr(builder, doctype.public_id());
    let system_id = string_expr(builder, doctype.system_id());

    quote_expr!(cx, {
        ::borealis::html::Doctype::new($name, $public_id, $system_id)
    })
}

fn node_expression(cx: &ExtCtxt, builder: &aster::AstBuilder, node: &Node) -> P<Expr> {
    match *node {
        Node::Comment(ref comment) => {
            let s = string_expr(builder, comment.comment());
            quote_expr!(cx, {
                ::borealis::html::CommentNode::new($s).into()
            })
        }
        Node::Text(ref text) => {
            let s = string_expr(builder, text.text());
            quote_expr!(cx, {
                ::borealis::html::TextNode::new($s).into()
            })
        }
        Node::Element(ref element) => element_node_expression(cx, builder, element),
    }
}

fn element_node_expression(cx: &ExtCtxt,
                           builder: &aster::AstBuilder,
                           element: &ElementNode)
                           -> P<Expr> {
    let name_expr = qualname_str(cx, builder, element.name());

    let attribute_exprs = element.attributes().iter().map(|a| attribute_expression(cx, builder, a));
    let attribute_exprs = builder.expr().vec().with_exprs(attribute_exprs).build();

    match *element.element_type() {
        ElementType::Normal(ref children) => {
            let child_exprs = children.iter().map(|c| node_expression(cx, builder, c));
            let child_exprs = builder.expr().vec().with_exprs(child_exprs).build();

            quote_expr!(cx, {
                ::borealis::html::ElementNode::new_normal(
                    $name_expr,
                    $attribute_exprs,
                    $child_exprs
                ).into()
            })
        }
        ElementType::Template(ref document) => {
            let document_expr = document_expression(cx, builder, document);
            quote_expr!(cx, {
                ::borealis::html::ElementNode::new_template(
                    $name_expr,
                    $attribute_exprs,
                    $document_expr
                ).into()
            })
        }
    }
}

fn attribute_expression(cx: &ExtCtxt,
                        builder: &aster::AstBuilder,
                        attribute: &Attribute)
                        -> P<Expr> {
    let name = qualname_str(cx, builder, attribute.name());
    let value = string_expr(builder, attribute.value());

    quote_expr!(cx, {
        ::borealis::html::Attribute::new($name, $value)
    })
}

fn string_expr<'a, T: Into<String> + Clone>(builder: &aster::AstBuilder, s: &'a T) -> P<Expr> {
    let s: String = s.clone().into();
    builder.expr().str(&s[..])
}

fn str_expr(builder: &aster::AstBuilder, s: &str) -> P<Expr> {
    builder.expr().str(s)
}

fn qualname_str(cx: &ExtCtxt, builder: &aster::AstBuilder, q: &QualName) -> P<Expr> {
    let ns = str_expr(builder, &q.ns.0);
    let local = str_expr(builder, &q.local);

    quote_expr!(cx, {
        ::borealis::string_cache::QualName::new(
            ::borealis::string_cache::Namespace($ns.into()),
            $local.into()
        )
    })
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
