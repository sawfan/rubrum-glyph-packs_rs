use rubrum::{Angle, Body, ChartPoint, Sign};

use crate::types::{GlyphOverride, GlyphPackManifest, GlyphSetsConfig};

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

const EMBEDDED_GLYPH_SETS_CONFIG: &str = include_str!("../config/glyph_sets.toml");

pub struct GlyphResolver {
    default_pack_dir: PathBuf,
    glyph_sets_config_path: PathBuf,
    glyph_set_name: Option<String>,

    sets_config: Option<GlyphSetsConfig>,
    manifest_cache: HashMap<PathBuf, GlyphPackManifest>,
}

impl GlyphResolver {
    pub fn new(
        default_pack_dir: PathBuf,
        glyph_sets_config_path: PathBuf,
        glyph_set_name: Option<String>,
    ) -> Self {
        Self {
            default_pack_dir,
            glyph_sets_config_path,
            glyph_set_name,
            sets_config: None,
            manifest_cache: HashMap::new(),
        }
    }

    pub fn body_glyph_available(&mut self, body: Body) -> bool {
        self.body_glyph_svg_path(body)
            .map(|p| p.is_file())
            .unwrap_or(false)
    }

    pub fn body_glyph_svg_path(&mut self, body: Body) -> Option<PathBuf> {
        let key = Body::canonical_key(body);

        if let Some(set_name) = self.glyph_set_name.clone() {
            if let Some(p) = self.resolve_body_from_set(&set_name, key) {
                return Some(p);
            }
        }

        let default_pack_dir = self.default_pack_dir.clone();

        // Prefer a manifest in the pack dir if present; otherwise, fall back to legacy mapping.
        if let Some(p) = self.resolve_body_from_pack_manifest(&default_pack_dir, key) {
            return Some(p);
        }

        self.resolve_body_from_legacy_pack_dir(&default_pack_dir, body)
    }

    pub fn sign_glyph_available(&mut self, sign: Sign) -> bool {
        self.sign_glyph_svg_path(sign)
            .map(|p| p.is_file())
            .unwrap_or(false)
    }

    pub fn sign_glyph_svg_path(&mut self, sign: Sign) -> Option<PathBuf> {
        let key = Sign::canonical_key(sign);

        if let Some(set_name) = self.glyph_set_name.clone() {
            if let Some(p) = self.resolve_sign_from_set(&set_name, key) {
                return Some(p);
            }
        }

        let default_pack_dir = self.default_pack_dir.clone();

        self.resolve_sign_from_pack_manifest(&default_pack_dir, key)
    }

    pub fn angle_glyph_available(&mut self, angle: Angle) -> bool {
        self.angle_glyph_svg_path(angle)
            .map(|p| p.is_file())
            .unwrap_or(false)
    }

    pub fn angle_glyph_svg_path(&mut self, angle: Angle) -> Option<PathBuf> {
        let key = Angle::canonical_key(angle);

        if let Some(set_name) = self.glyph_set_name.clone() {
            if let Some(p) = self.resolve_angle_from_set(&set_name, key) {
                return Some(p);
            }
        }

        let default_pack_dir = self.default_pack_dir.clone();

        self.resolve_angle_from_pack_manifest(&default_pack_dir, key)
    }

    pub fn chart_point_glyph_available(&mut self, point: ChartPoint) -> bool {
        self.chart_point_glyph_svg_path(point)
            .map(|p| p.is_file())
            .unwrap_or(false)
    }

    pub fn chart_point_glyph_svg_path(&mut self, point: ChartPoint) -> Option<PathBuf> {
        let key = ChartPoint::canonical_key(point);

        if let Some(set_name) = self.glyph_set_name.clone() {
            if let Some(p) = self.resolve_chart_point_from_set(&set_name, key) {
                return Some(p);
            }
        }

        let default_pack_dir = self.default_pack_dir.clone();

        self.resolve_chart_point_from_pack_manifest(&default_pack_dir, key)
    }

    fn resolve_body_from_legacy_pack_dir(
        &self,
        default_pack_dir: &Path,
        body: Body,
    ) -> Option<PathBuf> {
        let key = Body::canonical_key(body);
        Some(default_pack_dir.join("bodies").join(format!("{key}.svg")))
    }

    fn resolve_body_from_pack_manifest(&mut self, pack_dir: &Path, key: &str) -> Option<PathBuf> {
        let manifest = self.load_manifest_cached(pack_dir)?;
        let file = manifest.bodies.get(key)?;
        Some(pack_dir.join(file))
    }

    fn resolve_sign_from_pack_manifest(&mut self, pack_dir: &Path, key: &str) -> Option<PathBuf> {
        let manifest = self.load_manifest_cached(pack_dir)?;
        let file = manifest.signs.get(key)?;
        Some(pack_dir.join(file))
    }

    fn resolve_angle_from_pack_manifest(&mut self, pack_dir: &Path, key: &str) -> Option<PathBuf> {
        let manifest = self.load_manifest_cached(pack_dir)?;
        let file = manifest.angles.get(key)?;
        Some(pack_dir.join(file))
    }

    fn resolve_chart_point_from_pack_manifest(
        &mut self,
        pack_dir: &Path,
        key: &str,
    ) -> Option<PathBuf> {
        let manifest = self.load_manifest_cached(pack_dir)?;
        let file = manifest.chart_points.get(key)?;
        Some(pack_dir.join(file))
    }

    fn load_manifest_cached(&mut self, pack_dir: &Path) -> Option<&GlyphPackManifest> {
        let pack_dir = pack_dir.to_path_buf();
        if self.manifest_cache.contains_key(&pack_dir) {
            return self.manifest_cache.get(&pack_dir);
        }

        let manifest_path = pack_dir.join("manifest.toml");
        let Ok(s) = std::fs::read_to_string(&manifest_path) else {
            return None;
        };
        let Ok(manifest) = toml::from_str::<GlyphPackManifest>(&s) else {
            return None;
        };

        self.manifest_cache.insert(pack_dir.clone(), manifest);
        self.manifest_cache.get(&pack_dir)
    }

    fn ensure_sets_config_loaded(&mut self) {
        if self.sets_config.is_some() {
            return;
        }

        let s = std::fs::read_to_string(&self.glyph_sets_config_path)
            .unwrap_or_else(|_| EMBEDDED_GLYPH_SETS_CONFIG.to_owned());

        self.sets_config = toml::from_str::<GlyphSetsConfig>(&s).ok();
    }

    fn pack_dir_for_name(&self, sets_config: &GlyphSetsConfig, pack_name: &str) -> Option<PathBuf> {
        sets_config.packs.get(pack_name).map(PathBuf::from)
    }

    fn pack_dir_for_set(&self, sets_config: &GlyphSetsConfig, set_name: &str) -> Option<PathBuf> {
        let set = sets_config.sets.get(set_name)?;
        let pack_dir = set
            .pack
            .as_deref()
            .and_then(|pack| self.pack_dir_for_name(sets_config, pack))
            .unwrap_or_else(|| self.default_pack_dir.clone());
        Some(pack_dir)
    }

    fn find_body_override_in_set_chain(
        &self,
        sets_config: &GlyphSetsConfig,
        set_name: &str,
        key: &str,
        visited: &mut HashSet<String>,
    ) -> Option<GlyphOverride> {
        if !visited.insert(set_name.to_owned()) {
            return None;
        }

        let set = sets_config.sets.get(set_name)?;

        if let Some(ov) = set.overrides.bodies.get(key) {
            return Some(ov.clone());
        }

        for parent in &set.extends {
            if let Some(ov) =
                self.find_body_override_in_set_chain(sets_config, parent, key, visited)
            {
                return Some(ov);
            }
        }

        None
    }

    fn find_sign_override_in_set_chain(
        &self,
        sets_config: &GlyphSetsConfig,
        set_name: &str,
        key: &str,
        visited: &mut HashSet<String>,
    ) -> Option<GlyphOverride> {
        if !visited.insert(set_name.to_owned()) {
            return None;
        }

        let set = sets_config.sets.get(set_name)?;

        if let Some(ov) = set.overrides.signs.get(key) {
            return Some(ov.clone());
        }

        for parent in &set.extends {
            if let Some(ov) =
                self.find_sign_override_in_set_chain(sets_config, parent, key, visited)
            {
                return Some(ov);
            }
        }

        None
    }

    fn find_angle_override_in_set_chain(
        &self,
        sets_config: &GlyphSetsConfig,
        set_name: &str,
        key: &str,
        visited: &mut HashSet<String>,
    ) -> Option<GlyphOverride> {
        if !visited.insert(set_name.to_owned()) {
            return None;
        }

        let set = sets_config.sets.get(set_name)?;

        if let Some(ov) = set.overrides.angles.get(key) {
            return Some(ov.clone());
        }

        for parent in &set.extends {
            if let Some(ov) =
                self.find_angle_override_in_set_chain(sets_config, parent, key, visited)
            {
                return Some(ov);
            }
        }

        None
    }

    fn find_chart_point_override_in_set_chain(
        &self,
        sets_config: &GlyphSetsConfig,
        set_name: &str,
        key: &str,
        visited: &mut HashSet<String>,
    ) -> Option<GlyphOverride> {
        if !visited.insert(set_name.to_owned()) {
            return None;
        }

        let set = sets_config.sets.get(set_name)?;

        if let Some(ov) = set.overrides.chart_points.get(key) {
            return Some(ov.clone());
        }

        for parent in &set.extends {
            if let Some(ov) =
                self.find_chart_point_override_in_set_chain(sets_config, parent, key, visited)
            {
                return Some(ov);
            }
        }

        None
    }

    fn resolve_body_override_to_path(
        &mut self,
        sets_config: &GlyphSetsConfig,
        base_pack_dir: &Path,
        ov: &GlyphOverride,
    ) -> Option<PathBuf> {
        if let Some(path) = ov.path.as_deref() {
            return Some(PathBuf::from(path));
        }

        let pack_dir = if let Some(pack_name) = ov.pack.as_deref() {
            self.pack_dir_for_name(sets_config, pack_name)
                .unwrap_or_else(|| base_pack_dir.to_path_buf())
        } else {
            base_pack_dir.to_path_buf()
        };

        if let Some(file) = ov.file.as_deref() {
            return Some(pack_dir.join(file));
        }

        if let Some(key) = ov.key.as_deref() {
            return self.resolve_body_from_pack_manifest(&pack_dir, key);
        }

        None
    }

    fn resolve_sign_override_to_path(
        &mut self,
        sets_config: &GlyphSetsConfig,
        base_pack_dir: &Path,
        ov: &GlyphOverride,
    ) -> Option<PathBuf> {
        if let Some(path) = ov.path.as_deref() {
            return Some(PathBuf::from(path));
        }

        let pack_dir = if let Some(pack_name) = ov.pack.as_deref() {
            self.pack_dir_for_name(sets_config, pack_name)
                .unwrap_or_else(|| base_pack_dir.to_path_buf())
        } else {
            base_pack_dir.to_path_buf()
        };

        if let Some(file) = ov.file.as_deref() {
            return Some(pack_dir.join(file));
        }

        if let Some(key) = ov.key.as_deref() {
            return self.resolve_sign_from_pack_manifest(&pack_dir, key);
        }

        None
    }

    fn resolve_angle_override_to_path(
        &mut self,
        sets_config: &GlyphSetsConfig,
        base_pack_dir: &Path,
        ov: &GlyphOverride,
    ) -> Option<PathBuf> {
        if let Some(path) = ov.path.as_deref() {
            return Some(PathBuf::from(path));
        }

        let pack_dir = if let Some(pack_name) = ov.pack.as_deref() {
            self.pack_dir_for_name(sets_config, pack_name)
                .unwrap_or_else(|| base_pack_dir.to_path_buf())
        } else {
            base_pack_dir.to_path_buf()
        };

        if let Some(file) = ov.file.as_deref() {
            return Some(pack_dir.join(file));
        }

        if let Some(key) = ov.key.as_deref() {
            return self.resolve_angle_from_pack_manifest(&pack_dir, key);
        }

        None
    }

    fn resolve_chart_point_override_to_path(
        &mut self,
        sets_config: &GlyphSetsConfig,
        base_pack_dir: &Path,
        ov: &GlyphOverride,
    ) -> Option<PathBuf> {
        if let Some(path) = ov.path.as_deref() {
            return Some(PathBuf::from(path));
        }

        let pack_dir = if let Some(pack_name) = ov.pack.as_deref() {
            self.pack_dir_for_name(sets_config, pack_name)
                .unwrap_or_else(|| base_pack_dir.to_path_buf())
        } else {
            base_pack_dir.to_path_buf()
        };

        if let Some(file) = ov.file.as_deref() {
            return Some(pack_dir.join(file));
        }

        if let Some(key) = ov.key.as_deref() {
            return self.resolve_chart_point_from_pack_manifest(&pack_dir, key);
        }

        None
    }

    fn resolve_from_set(
        &mut self,
        set_name: &str,
        key: &str,
        find_override: fn(
            &Self,
            &GlyphSetsConfig,
            &str,
            &str,
            &mut HashSet<String>,
        ) -> Option<GlyphOverride>,
        resolve_override: fn(&mut Self, &GlyphSetsConfig, &Path, &GlyphOverride) -> Option<PathBuf>,
    ) -> Option<PathBuf> {
        self.ensure_sets_config_loaded();

        // Clone out of `self` so we can call helper methods that need `&mut self` without
        // fighting borrow-checker aliasing between `self.sets_config` and mutable methods.
        let sets_config = self.sets_config.as_ref()?.clone();

        let base_pack_dir = self.pack_dir_for_set(&sets_config, set_name)?;

        let mut visited = HashSet::new();
        if let Some(ov) = find_override(self, &sets_config, set_name, key, &mut visited) {
            if let Some(path) = resolve_override(self, &sets_config, &base_pack_dir, &ov) {
                return Some(path);
            }
        }

        // Fall back to the pack manifest mapping for this set.
        //
        // NOTE: We don't know which section is desired here, so callers should provide a fallback.
        None
    }

    fn resolve_body_from_set(&mut self, set_name: &str, key: &str) -> Option<PathBuf> {
        self.resolve_from_set(
            set_name,
            key,
            Self::find_body_override_in_set_chain,
            Self::resolve_body_override_to_path,
        )
        .or_else(|| {
            // Fallback to pack manifest for set's pack.
            self.ensure_sets_config_loaded();
            let sets_config = self.sets_config.as_ref()?;
            let pack_dir = self.pack_dir_for_set(sets_config, set_name)?;
            self.resolve_body_from_pack_manifest(&pack_dir, key)
        })
    }

    fn resolve_sign_from_set(&mut self, set_name: &str, key: &str) -> Option<PathBuf> {
        self.resolve_from_set(
            set_name,
            key,
            Self::find_sign_override_in_set_chain,
            Self::resolve_sign_override_to_path,
        )
        .or_else(|| {
            self.ensure_sets_config_loaded();
            let sets_config = self.sets_config.as_ref()?;
            let pack_dir = self.pack_dir_for_set(sets_config, set_name)?;
            self.resolve_sign_from_pack_manifest(&pack_dir, key)
        })
    }

    fn resolve_angle_from_set(&mut self, set_name: &str, key: &str) -> Option<PathBuf> {
        self.resolve_from_set(
            set_name,
            key,
            Self::find_angle_override_in_set_chain,
            Self::resolve_angle_override_to_path,
        )
        .or_else(|| {
            self.ensure_sets_config_loaded();
            let sets_config = self.sets_config.as_ref()?;
            let pack_dir = self.pack_dir_for_set(sets_config, set_name)?;
            self.resolve_angle_from_pack_manifest(&pack_dir, key)
        })
    }

    fn resolve_chart_point_from_set(&mut self, set_name: &str, key: &str) -> Option<PathBuf> {
        self.resolve_from_set(
            set_name,
            key,
            Self::find_chart_point_override_in_set_chain,
            Self::resolve_chart_point_override_to_path,
        )
        .or_else(|| {
            self.ensure_sets_config_loaded();
            let sets_config = self.sets_config.as_ref()?;
            let pack_dir = self.pack_dir_for_set(sets_config, set_name)?;
            self.resolve_chart_point_from_pack_manifest(&pack_dir, key)
        })
    }
}
