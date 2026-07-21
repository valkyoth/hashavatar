#![cfg(not(any(feature = "blake3", feature = "xxh3")))]

use std::fmt::Write as _;

use hashavatar::prelude::*;
use sha2::{Digest, Sha512};

const FIXTURE: &str = include_str!("compatibility_corpus_v1.tsv");

fn sha512_hex(bytes: &[u8]) -> String {
    let digest = Sha512::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing into String cannot fail");
    }
    encoded
}

fn generated_corpus() -> String {
    let namespace = AvatarNamespace::new("compatibility-corpus", "v1")
        .expect("fixture namespace must be valid");
    let spec = AvatarSpec::new(96, 80, 7).expect("fixture spec must be valid");
    let mut corpus = String::from(
        "# hashavatar 1.x compatibility corpus v1\n\
         # hash=sha512 tenant=compatibility-corpus style_version=v1\n\
         kind\twidth\theight\tseed\tbackground\taccessory\tcolor\texpression\tshape\tasset_key\trgba_sha512\tsvg_sha512\n",
    );

    for kind in AvatarKind::ALL {
        let style = AvatarStyleOptions::new(
            *kind,
            AvatarBackground::Ocean,
            AvatarAccessory::None,
            AvatarColor::Gold,
            AvatarExpression::Default,
            AvatarShape::Square,
        );
        let identity = AvatarIdentity::new_with_namespace(namespace, kind.as_str())
            .expect("fixture identity must be valid");
        let prepared = AvatarRequest::new(identity, spec, style)
            .prepare()
            .expect("fixture request must prepare");
        let image = prepared.render().expect("fixture request must render");
        let svg = prepared.render_svg();

        writeln!(
            corpus,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            kind.as_str(),
            spec.width(),
            spec.height(),
            spec.seed(),
            style.background.as_str(),
            style.accessory.as_str(),
            style.color.as_str(),
            style.expression.as_str(),
            style.shape.as_str(),
            prepared.avatar_asset_key().to_hex(),
            sha512_hex(image.as_raw()),
            sha512_hex(svg.as_bytes()),
        )
        .expect("writing into String cannot fail");
    }
    corpus
}

#[test]
fn all_legacy_families_match_the_frozen_request_and_output_corpus() {
    let generated = generated_corpus();
    assert_eq!(generated, FIXTURE, "1.x compatibility corpus changed");
    assert_eq!(
        FIXTURE
            .lines()
            .filter(|line| !line.starts_with('#'))
            .count(),
        AvatarKind::ALL.len() + 1,
        "fixture must contain one header and every legacy family"
    );
}

#[test]
#[ignore = "prints fixture updates for an intentional, reviewed contract change"]
fn print_compatibility_corpus() {
    print!("{}", generated_corpus());
}
