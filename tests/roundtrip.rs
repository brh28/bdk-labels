use bdk_labels::convert::{annotations_from_bip329, annotations_to_bip329};
use bip329::Labels;

const WALLET1: &str = include_str!("data/wallet1.jsonl");
const WALLET2: &str = include_str!("data/wallet2.jsonl");

#[test]
fn wallet1_parse_and_print() {
    let labels = Labels::try_from_str(WALLET1).expect("parse failed");
    let annotations = annotations_from_bip329(labels);
    let roundtripped = annotations_to_bip329(annotations).expect("conversion failed");

    let mut buf = Vec::new();
    roundtripped.export_to_writer(&mut buf).expect("export failed");
    println!("{}", std::str::from_utf8(&buf).unwrap());
}

#[test]
fn wallet1_roundtrip() {
    let labels = Labels::try_from_str(WALLET1).expect("parse failed");
    let annotations = annotations_from_bip329(labels.clone());
    let roundtripped = annotations_to_bip329(annotations).expect("conversion failed");
    assert_eq!(labels, roundtripped);
}

#[test]
fn wallet2_parse_and_print() {
    let labels = Labels::try_from_str(WALLET2).expect("parse failed");
    let annotations = annotations_from_bip329(labels);
    let roundtripped = annotations_to_bip329(annotations).expect("conversion failed");

    let mut buf = Vec::new();
    roundtripped.export_to_writer(&mut buf).expect("export failed");
    println!("{}", std::str::from_utf8(&buf).unwrap());
}

#[test]
fn wallet2_roundtrip() {
    let labels = Labels::try_from_str(WALLET2).expect("parse failed");
    let annotations = annotations_from_bip329(labels.clone());
    let roundtripped = annotations_to_bip329(annotations).expect("conversion failed");
    assert_eq!(labels, roundtripped);
}
