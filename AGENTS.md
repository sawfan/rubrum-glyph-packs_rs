# AGENTS

This document is for humans and automated agents working on the **rubrum_glyph_packs** crate.

The goal is to describe the crate’s purpose, organization, and verification workflow.

---

## 1. Project summary

- **Name:** `rubrum_glyph_packs`
- **Type:** Rust library crate
- **Domain:** SVG glyph asset management
- **Purpose:** Parse glyph-pack manifests and resolve glyph lookups via named glyph sets.

This crate is backend-agnostic and is intended to be reused by both:

- `rubrum_cairo` (SVG injection into Cairo-rendered SVG)
- `rubrum_svg` (future glyph pack support)

---

## 2. Configuration model

- **Glyph pack manifest**: a TOML file (typically `manifest.toml`) that maps canonical keys (e.g. bodies/angles/points) to SVG asset filenames.
- **Glyph sets config**: a TOML file describing named sets (optionally inheriting from each other) and per-set overrides.

This crate embeds a default glyph-sets config at build time via `include_str!()`.

---

## 3. Key modules / organization

- `types` — serde models for manifests/sets:
  - `GlyphPackManifest`
  - `GlyphSetsConfig`, `GlyphSetDef`, `GlyphSetOverrides`, `GlyphOverride`
- `parse` — TOML parsing helpers:
  - `load_glyph_pack_manifest_from_str(...)`
  - `load_glyph_sets_config_from_str(...)`
- `resolver` — `GlyphResolver`:
  - resolves glyph pack dirs + manifests
  - resolves active glyph set names and inheritance chains
  - yields concrete SVG paths for a requested canonical key

---

## 4. Verification

From the repository root:

```sh
RUSTFLAGS='-Dwarnings' cargo check -q
RUSTFLAGS='-Dwarnings' cargo test  -q
```

---

## 5. Recent agent operations

- 2026-04-04: Added a repository root `README.md` documenting crate purpose, current features, basic usage, and verification commands.
- 2026-05-06: Added `scripts/build_sprite_sheet.sh` to generate an SVG `<symbol>` sprite sheet from a pack `manifest.toml`, and documented sprite-sheet generation in `README.md`.
- 2026-05-06: Standardized `assets/bold_outlined` filenames to canonical lowercase (`sun.svg`, `constellations/aries.svg`, etc.), updated `assets/bold_outlined/manifest.toml`, and verified sprite generation via `out/glyphs_bold_outlined.svg`.



