
use aster::AstBuilder;

use borealis::dom::{Handle, Node};

use syntax::ast::Expr;
use syntax::ext::base::ExtCtxt;
use syntax::ext::quote::rt::ExtParseUtils;
use syntax::ptr::P;

use expr::{string_expr, string_code_expr, str_expr, qualname_expr};

pub fn document_expression(cx: &ExtCtxt, builder: &AstBuilder, document: &Handle) -> P<Expr> {
    match *document.borrow() {
        (Node::Document(ref doctype, ref child), _) => {
            let child_expr = match *child {
                Some(ref child) => node_expression(cx, builder, child),
                None => quote_expr!(cx, {}),
            };

            let doctype_expr = match *doctype {
                Some(ref doctype) => {
                    let doctype = string_expr(builder, doctype);
                    quote_expr!(cx, s.doctype($doctype).node())
                }
                None => quote_expr!(cx, s.node()),
            };

            match *child {
                Some(_) => {
                    quote_expr!(cx, {
                        let mut s = $doctype_expr;
                        $child_expr
                    })
                }
                None => {
                    quote_expr!(cx, {
                        $doctype_expr;
                    })
                }
            }
        }
        _ => panic!("expected document, got: {:?}", document),
    }
}

pub fn node_expression(cx: &ExtCtxt, builder: &AstBuilder, node: &Handle) -> P<Expr> {
    match *node.borrow() {
        (Node::Comment(ref comment), _) => {
            let comment = string_expr(builder, comment);
            quote_expr!(cx, {
                s.comment($comment);
            })
        }
        (Node::Text(ref text), _) => text_node_expression(cx, builder, &text[..]),
        (Node::Element(ref name, ref attrs, ref children), _) => {
            let name = qualname_expr(cx, builder, name);

            let attrs_expr = attrs.iter().map(|a| {
                let key = qualname_expr(cx, builder, &a.0);
                let value = string_code_expr(cx, builder, &a.1);

                quote_expr!(cx, {
                    (&$key, $value)
                })
            });
            let attrs_expr = builder.expr().slice().with_exprs(attrs_expr).build();

            let child_exprs: Vec<_> = children.iter()
                                              .map(|c| node_expression(cx, builder, c))
                                              .collect();
            let expr = quote_expr!(cx, {
                s.element($name, $attrs_expr.iter())
            });

            if child_exprs.len() == 0 {
                quote_expr!(cx, {
                    $expr;
                })
            } else {
                quote_expr!(cx, {
                    let mut s = $expr;
                    $child_exprs
                })
            }
        }
        _ => panic!("expected comment, text or element, got {:?}", node),
    }
}

pub fn text_node_expression(cx: &ExtCtxt, builder: &AstBuilder, string: &str) -> P<Expr> {
    let regex = regex!(r#"\{{2}([^"]|"(\\"|[^"])*")*?(\}{2}|"([^"]|\\")*$|$)"#);
    let mut last_end = 0;
    let mut exprs = Vec::new();

    fn add_text_node_str(cx: &ExtCtxt, builder: &AstBuilder, exprs: &mut Vec<P<Expr>>, s: &str) {
        let text = str_expr(builder, s);
        exprs.push(quote_expr!(cx, {
            s.text($text);
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
                #[allow(unused_imports)]
                use ::borealis::serializer::{SerializeNode, SerializeNodes};
                $expr.serialize_node(&mut s);
            }));
        }

        last_end = end;
    }

    if last_end != string.len() {
        add_text_node_str(cx, builder, &mut exprs, &string[last_end..]);
    }

    quote_expr!(cx, {
        $exprs
    })
}
