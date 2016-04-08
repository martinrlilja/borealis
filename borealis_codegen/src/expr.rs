
use aster::AstBuilder;

use borealis::string_cache::QualName;

use syntax::ast::Expr;
use syntax::ext::base::ExtCtxt;
use syntax::ext::quote::rt::ExtParseUtils;
use syntax::ptr::P;

pub fn string_expr<'a, T: Into<String> + Clone>(builder: &AstBuilder, s: &'a T) -> P<Expr> {
    let s: String = s.clone().into();
    builder.expr().str(&s[..])
}

pub fn string_code_expr<'a, T: Into<String> + Clone>(cx: &ExtCtxt,
                                                     builder: &AstBuilder,
                                                     s: &'a T)
                                                     -> P<Expr> {
    let s: String = s.clone().into();

    if s.starts_with("{{") && s.ends_with("}}") {
        let s = s[2..s.len() - 2].to_owned();
        cx.parse_expr(s)
    } else if s.starts_with("{{") {
        panic!("unmatched {{");
    } else {
        builder.expr().str(&s[..])
    }
}

pub fn str_expr(builder: &AstBuilder, s: &str) -> P<Expr> {
    builder.expr().str(s)
}

pub fn qualname_expr(cx: &ExtCtxt, builder: &AstBuilder, q: &QualName) -> P<Expr> {
    let ns = str_expr(builder, &q.ns.0);
    let local = str_expr(builder, &q.local);

    quote_expr!(cx, {
        ::borealis::string_cache::QualName::new(
            ::borealis::string_cache::Namespace($ns.into()),
            $local.into()
        )
    })
}
