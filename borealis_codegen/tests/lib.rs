#![feature(plugin)]
#![plugin(borealis_codegen)]

extern crate borealis;

use std::io::Read;
use std::fs::File;
use std::path::Path;

use borealis::Document;
use borealis::serialize::{SerializeDocument, serialize};

#[template_document(file="test_template.html")]
struct TestTemplate {
    value: String,
    fragment: TestFragment,
}

#[test]
fn test_test_template() {
    let template = TestTemplate {
        value: "Test".to_owned(),
        fragment: TestFragment {
            value: 10,
        }
    };
    let document_a = serialize_doc(template);
    let document_b = serialize_doc(read_document("tests/test_template_expected.html"));

    assert_eq!(document_a, document_b);
}

#[template_fragment(file="test_fragment.html", trim)]
struct TestFragment {
    value: i32,
}

fn read_document<P: AsRef<Path>>(path: P) -> Document {
    let mut file = File::open(path).unwrap();

    let file_str = {
        let mut file_str = String::new();
        file.read_to_string(&mut file_str).unwrap();

        file_str
    };

    Document::parse_str(&file_str)
}

fn serialize_doc<T: SerializeDocument>(document: T) -> String {
    let mut w = Vec::new();
    serialize(&mut w, document).unwrap();
    String::from_utf8(w).unwrap()
}
