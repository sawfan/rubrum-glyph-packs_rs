use crate::types::{GlyphPackManifest, GlyphSetsConfig};

pub fn load_glyph_pack_manifest_from_str(s: &str) -> Option<GlyphPackManifest> {
    toml::from_str::<GlyphPackManifest>(s).ok()
}

pub fn load_glyph_sets_config_from_str(s: &str) -> Option<GlyphSetsConfig> {
    toml::from_str::<GlyphSetsConfig>(s).ok()
}
