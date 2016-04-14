# Borealis

[![Build Status](https://travis-ci.org/martinrlilja/borealis.svg?branch=master)](https://travis-ci.org/martinrlilja/borealis)

Borealis is a templating engine for HTML5 written in Rust.

* [Documents](#documents)
* [Fragments](#fragments)

## Documents

```rust
// Enable compiler plugins. Note that this only works in nightly.
#![feature(plugin)]
#![plugin(borealis_codegen)]

// Let the plugin derive `IntoDocument` for us.
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

You can now call `borealis::IntoDocument::into_document(self)` on a `Template` to get the document tree. When you do, you probably want to serialize it.

```rust
// Create a buffer which we can write too.
// Normally this would be a TCP stream or similar.
let mut writer: Vec<u8> = Vec::new();

// Create a new template.
let template = Template {
    value: "test".into(),
};

// Turn it into a document and serialize it into our buffer.
let document = template.into_document();
document.serialize(&mut writer);
```

## Fragments

```rust
#![feature(plugin)]
#![plugin(borealis_codegen)]

#[template_fragment(file="template.html", trim)]
struct Template {
    value: u32,
}
```

The trim flag means that the plugin will trim any whitespace before parsing the fragment file. (This is done using `String::trim`) This can be useful if your editor inserts an extra newline at the end of files.

As with the document template, the file needs to be in the same directory as the code.

```html
<ul>
    {{ (0..self.value).map(|v| format!("{}", v)) }}
</ul>
```

The fragment can now be turned into a vector of nodes with `borealis::IntoNodes::into_nodes(self)`, or you can insert it into another fragment or document.

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
