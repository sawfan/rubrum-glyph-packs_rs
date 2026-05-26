use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GlyphPackManifest {
    #[serde(default)]
    pub bodies: HashMap<String, String>,

    #[serde(default)]
    pub signs: HashMap<String, String>,

    #[serde(default)]
    pub angles: HashMap<String, String>,

    #[serde(default)]
    pub chart_points: HashMap<String, String>,

    // Not yet used by the Cairo SVG injector, but allowed in manifests.
    #[serde(default)]
    pub lots: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GlyphSetsConfig {
    #[serde(default)]
    pub packs: HashMap<String, String>,

    #[serde(default)]
    pub sets: HashMap<String, GlyphSetDef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GlyphSetDef {
    pub pack: Option<String>,

    #[serde(default)]
    pub extends: Vec<String>,

    #[serde(default)]
    pub overrides: GlyphSetOverrides,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct GlyphSetOverrides {
    #[serde(default)]
    pub bodies: HashMap<String, GlyphOverride>,

    #[serde(default)]
    pub signs: HashMap<String, GlyphOverride>,

    #[serde(default)]
    pub angles: HashMap<String, GlyphOverride>,

    #[serde(default)]
    pub chart_points: HashMap<String, GlyphOverride>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlyphOverride {
    pub pack: Option<String>,
    pub key: Option<String>,
    pub file: Option<String>,
    pub path: Option<String>,
}
