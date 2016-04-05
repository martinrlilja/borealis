# Borealis

[![Build Status](https://travis-ci.org/martinrlilja/borealis.svg?branch=master)](https://travis-ci.org/martinrlilja/borealis)

*Borealis is a templating engine for HTML5 written in Rust.*

It is currently in early development, so there is not much to see here at the moment.

## How to use

```rust
#![feature(plugin)]
#![plugin(borealis_codegen)]

#[template_document(file="template.html")]
struct Template {
    value: String,
}
```
The template file needs to be in the same directory as the code file.

```html
<!DOCTYPE html>
<html>
    <body>
        {{ self.value }}
    </body>
</html>
```

You can now call `borealis::DocumentTemplate::document_template(template)`
on an instance of `Template` to get the document tree.

Serialization is coming soon.

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
