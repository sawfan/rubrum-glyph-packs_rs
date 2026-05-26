use super::*;

#[test]
fn parses_pack_manifest_bodies() {
    let s = r#"
[bodies]
sun = "Sun.svg"
moon = "Moon.svg"
"#;

    let m = load_glyph_pack_manifest_from_str(s).unwrap();
    assert_eq!(m.bodies.get("sun").unwrap(), "Sun.svg");
    assert_eq!(m.bodies.get("moon").unwrap(), "Moon.svg");
}

#[test]
fn parses_sets_config_and_overrides() {
    let s = r#"
[packs]
black = "assets/black"

[sets.base]
pack = "black"

[sets.mixed]
extends = ["base"]

[sets.mixed.overrides.bodies]
moon = { pack = "black", file = "FullMoon.svg" }
"#;

    let cfg = load_glyph_sets_config_from_str(s).unwrap();
    assert_eq!(cfg.packs.get("black").unwrap(), "assets/black");
    assert_eq!(cfg.sets.get("base").unwrap().pack.as_deref(), Some("black"));
    let ov = cfg
        .sets
        .get("mixed")
        .unwrap()
        .overrides
        .bodies
        .get("moon")
        .unwrap();
    assert_eq!(ov.pack.as_deref(), Some("black"));
    assert_eq!(ov.file.as_deref(), Some("FullMoon.svg"));
}

#[test]
fn body_key_matches_expected() {
    assert_eq!(rubrum::Body::canonical_key(rubrum::Body::Sun), "sun");
    assert_eq!(rubrum::Body::canonical_key(rubrum::Body::Chiron), "chiron");
    assert_eq!(rubrum::Body::canonical_key(rubrum::Body::Earth), "earth");
}
