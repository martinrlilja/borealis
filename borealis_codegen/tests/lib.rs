#![feature(plugin)]
#![plugin(borealis_codegen)]

extern crate borealis;

use std::io::Read;
use std::fs::File;
use std::path::Path;

use borealis::Document;
use borealis::serializer::{SerializeDocument, serialize};

#[template_document(file="test_template.html")]
struct TestTemplate {
    value: String,
    fragment: TestFragment,
}

#[template_fragment(file="test_fragment.html", trim)]
struct TestFragment {
    value: i32,
}

#[test]
fn test_test_template() {
    let template = TestTemplate {
        value: "Test".to_owned(),
        fragment: TestFragment {
            value: 10,
        }
    };

    test_document(template, "test_template", false);
}

#[template_document(file="empty.html")]
struct EmptyTemplate;

#[test]
fn test_empty_template() {
    test_document(EmptyTemplate, "empty", true);
}

#[template_document(file="doctype.html")]
struct DoctypeTemplate;

#[test]
fn test_doctype_template() {
    test_document(DoctypeTemplate, "doctype", true);
}

#[template_document(file="element.html")]
struct ElementTemplate;

#[test]
fn test_element_template() {
    test_document(ElementTemplate, "element", true);
}

fn test_document<T: SerializeDocument>(document: T, file: &str, c: bool) {
    let document_a = serialize_doc(document);
    let document_b = read_file(format!("tests/{}_expected.html", file));
    let document_c = serialize_doc(read_document(format!("tests/{}.html", file)));

    assert_eq!(document_a, document_b.trim());

    if c {
        assert_eq!(document_c, document_b.trim());
    }
}

fn read_file<P: AsRef<Path>>(path: P) -> String {
    let mut file = File::open(path).unwrap();
    let mut file_str = String::new();
    file.read_to_string(&mut file_str).unwrap();

    file_str
}

fn read_document<P: AsRef<Path>>(path: P) -> Document {
    Document::parse_str(&read_file(path))
}

fn serialize_doc<T: SerializeDocument>(document: T) -> String {
    let mut w = Vec::new();
    serialize(&mut w, document).unwrap();
    String::from_utf8(w).unwrap()
}
