
use aster::AstBuilder;

use borealis::html::{Attribute, Doctype, Document, Node, ElementNode, ElementType, TextNode};

use syntax::ast::Expr;
use syntax::ext::base::ExtCtxt;
use syntax::ext::quote::rt::ExtParseUtils;
use syntax::ptr::P;

use expr::{string_expr, string_code_expr, str_expr, qualname_expr};

pub fn document_expression(cx: &ExtCtxt, builder: &AstBuilder, document: &Document) -> P<Expr> {
    fn option_to_expr(cx: &ExtCtxt, builder: &AstBuilder, expr: Option<P<Expr>>) -> P<Expr> {
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
                                    document.child().map(|c| {
                                        let expr = node_expression(cx, builder, c);
                                        quote_expr!(cx, {
                                            $expr.remove(0)
                                        })
                                    }));

    quote_expr!(cx, {
        ::borealis::html::Document::new($doctype_expr, $child_expr)
    })
}

pub fn doctype_expression(cx: &ExtCtxt, builder: &AstBuilder, doctype: &Doctype) -> P<Expr> {
    let name = string_expr(builder, doctype.name());
    let public_id = string_expr(builder, doctype.public_id());
    let system_id = string_expr(builder, doctype.system_id());

    quote_expr!(cx, {
        ::borealis::html::Doctype::new($name, $public_id, $system_id)
    })
}

pub fn node_expression(cx: &ExtCtxt, builder: &AstBuilder, node: &Node) -> P<Expr> {
    match *node {
        Node::Comment(ref comment) => {
            let s = string_expr(builder, comment.comment());
            quote_expr!(cx, {
                vec![::borealis::html::Node::from(::borealis::html::CommentNode::new($s))]
            })
        }
        Node::Text(ref text) => text_node_expression(cx, builder, text),
        Node::Element(ref element) => {
            let expr = element_node_expression(cx, builder, element);
            quote_expr!(cx, {
                vec![$expr]
            })
        }
    }
}

pub fn text_node_expression(cx: &ExtCtxt, builder: &AstBuilder, text: &TextNode) -> P<Expr> {
    let string: String = text.text().into();
    let regex = regex!(r#"\{{2}([^"]|"(\\"|[^"])*")*?(\}{2}|"([^"]|\\")*$|$)"#);
    let mut last_end = 0;
    let mut exprs = Vec::new();

    fn add_text_node_str(cx: &ExtCtxt,
                         builder: &AstBuilder,
                         exprs: &mut Vec<P<Expr>>,
                         s: &str) {
        let s = str_expr(builder, s);
        exprs.push(quote_expr!(cx, {
            vec![::borealis::html::TextNode::new($s).into()] as Vec<::borealis::html::Node>
        }));
    };

    for (start, end) in regex.find_iter(&string[..]) {
        if last_end != start {
            add_text_node_str(cx, builder, &mut exprs, &string[last_end..start]);
        }

        if !string[start + 2..end].ends_with("}}") {
            cx.span_err(cx.original_span(),
                        &format!("unmatched {} around: {}", "{{", &string[start..end]));
        } else {
            let expr = cx.parse_expr(string[start + 2..end - 2].to_owned());
            exprs.push(quote_expr!(cx, {
                use ::borealis::{IntoNode, IntoNodes};
                $expr.into_nodes()
            }));
        }

        last_end = end;
    }

    if last_end != string.len() {
        add_text_node_str(cx, builder, &mut exprs, &string[last_end..]);
    }

    let exprs = builder.expr().vec().with_exprs(exprs).build();

    quote_expr!(cx, {
        let exprs: Vec<Vec<::borealis::html::Node>> = $exprs;
        let mut out = Vec::new();

        for expr in exprs {
            out.extend(expr);
        }

        out
    })
}

pub fn element_node_expression(cx: &ExtCtxt,
                           builder: &AstBuilder,
                           element: &ElementNode)
                           -> P<Expr> {
    let name_expr = qualname_expr(cx, builder, element.name());

    let attribute_exprs = element.attributes().iter().map(|a| attribute_expression(cx, builder, a));
    let attribute_exprs = builder.expr().vec().with_exprs(attribute_exprs).build();

    match *element.element_type() {
        ElementType::Normal(ref children) => {
            let child_exprs = children.iter().map(|c| node_expression(cx, builder, c));
            let child_exprs = builder.expr().vec().with_exprs(child_exprs).build();

            quote_expr!(cx, {
                let exprs: Vec<Vec<::borealis::html::Node>> = $child_exprs;
                let children = {
                    let mut out = Vec::new();

                    for expr in exprs {
                        out.extend(expr);
                    }

                    out
                };

                ::borealis::html::ElementNode::new_normal(
                    $name_expr,
                    $attribute_exprs,
                    children
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

pub fn attribute_expression(cx: &ExtCtxt,
                        builder: &AstBuilder,
                        attribute: &Attribute)
                        -> P<Expr> {
    let name = qualname_expr(cx, builder, attribute.name());
    let value = string_code_expr(cx, builder, attribute.value());

    quote_expr!(cx, {
        ::borealis::html::Attribute::new($name, $value)
    })
}
