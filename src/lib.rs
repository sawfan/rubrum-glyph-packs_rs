//! Glyph-pack manifest parsing and glyph-set resolution.
//!
//! This crate is backend-agnostic and is intended to be usable by both the Cairo and pure-SVG
//! renderers.

pub mod parse;
pub mod resolver;
pub mod types;

#[cfg(test)]
mod tests;

pub use parse::{load_glyph_pack_manifest_from_str, load_glyph_sets_config_from_str};
pub use resolver::GlyphResolver;
pub use types::{
    GlyphOverride, GlyphPackManifest, GlyphSetDef, GlyphSetOverrides, GlyphSetsConfig,
};
