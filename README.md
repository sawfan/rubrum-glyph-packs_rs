# rubrum_glyph_packs

Backend-agnostic glyph-pack manifest parsing and glyph-set resolution for **Rubrum** SVG glyph assets.

This crate is intended to be reused by renderers that need to map canonical Rubrum keys (e.g. `"sun"`, `"aries"`, `"conjunction"`) to concrete `.svg` files on disk.

## Features

- **Parse glyph-pack manifests** (`manifest.toml`) into strongly-typed Rust structs.
- **Parse glyph-set configurations** (`glyph_sets.toml`) describing:
  - named *packs* (directory paths)
  - named *sets* (optionally extending other sets)
  - per-key overrides that can redirect to another key, file, pack, or absolute/relative path
- **Resolve SVG paths** for canonical Rubrum keys via [`GlyphResolver`]:
  - `Body` glyphs (with legacy `bodies/<key>.svg` fallback when no `manifest.toml` exists)
  - `Sign` glyphs
  - `Angle` glyphs
  - `ChartPoint` glyphs
- **Embedded defaults**: a default `glyph_sets.toml` is compiled in via `include_str!()` and used as a fallback if the configured file path is missing/unreadable.
- **Manifest caching**: pack `manifest.toml` files are cached by directory path to avoid repeated parsing.

## Data model

### `manifest.toml`

A glyph pack directory may contain a `manifest.toml` that maps canonical keys to SVG filenames:

```toml
[bodies]
sun = "Sun.svg"
moon = "Moon.svg"

[signs]
aries = "Aries.svg"

[angles]
conjunction = "Conjunction.svg"

[chart_points]
asc = "Asc.svg"
```

### `glyph_sets.toml`

Glyph sets provide named selection and override behavior:

```toml
[packs]
black = "assets/black"
colored = "assets/colored"

[sets.black]
pack = "black"

[sets.mixed]
extends = ["black"]

[sets.mixed.overrides.bodies]
moon = { pack = "colored", key = "moon" }
```

Override fields:

- `path`: direct filesystem path to an SVG
- `pack`: pack name (from `[packs]`)
- `file`: filename within the chosen pack directory
- `key`: canonical key to re-resolve within the chosen pack directory

## Usage

```rust
use rubrum_glyph_packs::GlyphResolver;
use std::path::PathBuf;

let mut resolver = GlyphResolver::new(
    PathBuf::from("assets/black"),              // default pack dir
    PathBuf::from("config/glyph_sets.toml"),   // sets config path (optional; has embedded fallback)
    Some("black".to_string()),                 // active set name
);

let sun = resolver.body_glyph_svg_path(rubrum::Body::Sun).unwrap();
assert!(sun.ends_with(".svg"));
```

Parsing helpers (useful for tooling/tests):

- `load_glyph_pack_manifest_from_str(...)`
- `load_glyph_sets_config_from_str(...)`

## Crate layout

- `src/types.rs` — serde models for manifests and glyph sets
- `src/parse.rs` — TOML parsing helpers
- `src/resolver.rs` — [`GlyphResolver`] implementation

## Sprite sheets

The pack assets in this crate are primarily intended to be consumed via `manifest.toml` + per-glyph
lookup (Cairo embedding, file-based overrides, etc.). For pure-SVG `<use href="sprite.svg#id">`
workflows, we also support building an **SVG `<symbol>` sprite sheet** from a pack directory.

This repo includes a simple bash script:

```sh
./scripts/build_sprite_sheet.sh assets/white assets/white/glyphs_white.svg
```

The generated sprite sheet contains IDs that match `rubrum_render::glyphs`:

- `rb-body-<key>`
- `rb-sign-<key>`
- `rb-angle-<key>`
- `rb-chart-point-<key>`

## Verification

```sh
RUSTFLAGS='-Dwarnings' cargo check -q
RUSTFLAGS='-Dwarnings' cargo test  -q
```

## License

See repository license information.

