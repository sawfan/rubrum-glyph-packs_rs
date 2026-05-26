#!/usr/bin/env bash
set -euo pipefail

# Build an SVG <symbol> sprite sheet from a rubrum_glyph_packs pack directory.
#
# This is intentionally implemented as a POSIX-ish bash script (no Node/Python)
# to keep it easy to run in CI and for end users.
#
# The script reads a pack's manifest.toml and emits a single SVG file that
# contains <symbol> definitions whose IDs match rubrum_render::glyphs::*_svg_symbol_id(...):
# - bodies:       rb-body-<canonical_key>
# - signs:        rb-sign-<canonical_key>
# - angles:       rb-angle-<canonical_key>
# - chart_points: rb-chart-point-<canonical_key>
# - lots:         rb-lot-<canonical_key>
#
# The output is appropriate for use as an external sprite sheet, e.g.
#   <use href="glyphs_white.svg#rb-sign-aries" ... />
#
# Limitations / assumptions:
# - Each input file is an SVG containing at least one graphical element.
# - We try to preserve viewBox if present, otherwise we omit it.
# - We inline the SVG's "inner" content (everything inside the outer <svg ...>).
# - We drop XML prolog / DOCTYPE and any outer <svg> wrapper.
#
# Usage:
#   ./scripts/build_sprite_sheet.sh assets/white assets/white/sprite.svg
#

usage() {
  cat <<'EOF'
Usage:
  build_sprite_sheet.sh <pack_dir> <output_svg>

Examples:
  ./scripts/build_sprite_sheet.sh assets/white assets/white/glyphs_white.svg
EOF
}

PACK_DIR=${1:-}
OUT_PATH=${2:-}

if [[ -z "${PACK_DIR}" || -z "${OUT_PATH}" ]]; then
  usage
  exit 2
fi

MANIFEST_PATH="${PACK_DIR%/}/manifest.toml"
if [[ ! -f "${MANIFEST_PATH}" ]]; then
  echo "error: manifest not found: ${MANIFEST_PATH}" >&2
  exit 1
fi

if ! command -v rg >/dev/null 2>&1; then
  echo "error: ripgrep (rg) is required" >&2
  exit 1
fi

# Known keys for placeholder emission.
#
# The manifest remains the source of truth for which glyphs *exist*, but these lists let us
# additionally emit placeholder <symbol> entries for commonly-used canonical keys that are
# missing from the manifest.
KNOWN_SIGNS=(
  aries taurus gemini cancer leo virgo libra scorpio sagittarius capricorn aquarius pisces
)

KNOWN_ANGLES=(
  ascendant midheaven descendant imum_coeli vertex antivertex
)

KNOWN_CHART_POINTS=(
  ecl_nut mean_node true_node mean_apog oscu_apog intp_apog intp_perg
)

KNOWN_LOTS=(
  fortune spirit
)

# Keep this list intentionally small; `rubrum::Body` includes many bodies and we don't want to
# spam a sprite sheet with hundreds of placeholders.
KNOWN_BODIES=(
  sun moon mercury venus mars jupiter saturn uranus neptune pluto earth chiron
  ceres pallas juno vesta
)

# Extract the viewBox from an SVG file if present.
svg_viewbox() {
  local svg_file="$1"
  rg --no-messages -o 'viewBox="[^"]+"' "${svg_file}" | head -n 1 || true
}

# Extract the inner content of an SVG file:
# remove XML prolog/doctype and drop the outer <svg ...> and </svg>.
#
# NOTE: Many authoring tools (notably Inkscape) format the opening <svg> tag across multiple
# lines. This implementation handles both single-line and multi-line opening tags.
svg_inner() {
  local svg_file="$1"

  # We intentionally do not attempt full XML parsing here.
  # The goal is a best-effort robust extraction for pack-authored SVGs.
  awk '
    BEGIN {
      seen_svg = 0
      in_body = 0
    }

    # Drop XML prolog / doctype regardless of where they appear.
    /^<\?xml/ { next }
    /^<!DOCTYPE/ { next }

    {
      gsub(/\r/, "")
    }

    # Until we have seen the <svg ...> opening tag, skip content.
    seen_svg == 0 {
      pos = index($0, "<svg")
      if (pos == 0) {
        next
      }

      seen_svg = 1

      # If the opening tag ends on this line, start body after the first ">".
      end = index($0, ">")
      if (end > 0) {
        in_body = 1
        rest = substr($0, end + 1)

        # Handle the common case where the entire SVG is on one line:
        # <svg ...>...content...</svg>
        close_pos = index(rest, "</svg>")
        if (close_pos > 0) {
          pre = substr(rest, 1, close_pos - 1)
          if (length(pre) > 0) {
            print pre
          }
          exit
        }

        if (length(rest) > 0) {
          print rest
        }
      }
      next
    }

    # We have seen <svg, but may still be inside its multi-line attribute list.
    in_body == 0 {
      end = index($0, ">")
      if (end > 0) {
        in_body = 1
        rest = substr($0, end + 1)
        if (length(rest) > 0) {
          print rest
        }
      }
      next
    }

    # In body: skip common metadata/authoring blocks that introduce namespaced
    # elements/attributes (rdf/dc/cc/inkscape/sodipodi) which wont have xmlns bindings
    # once we inline into a <symbol>.
    in_metadata == 1 {
      if (index($0, "</metadata>") > 0) {
        in_metadata = 0
      }
      next
    }
    in_namedview == 1 {
      if (index($0, "</sodipodi:namedview>") > 0) {
        in_namedview = 0
      }
      next
    }
    {
      if (index($0, "<metadata") > 0) {
        in_metadata = 1
        next
      }
      if (index($0, "<sodipodi:namedview") > 0) {
        in_namedview = 1
        next
      }
    }

    # In body: stop at closing </svg>.
    {
      close_pos = index($0, "</svg>")
      if (close_pos > 0) {
        pre = substr($0, 1, close_pos - 1)
        if (length(pre) > 0) {
          print pre
        }
        exit
      }
      print
    }
  ' "${svg_file}"
}

# Parse a [section] of manifest.toml and print "key|file" lines.
# This is a minimal parser for the simple mapping format we use:
#
#   [bodies]
#   sun = "sun.svg"
#
manifest_section_pairs() {
  local section="$1"

  # Print lines after [section] until the next [something] header.
  awk -v sec="[${section}]" '
    BEGIN { in_sec = 0 }
    $0 ~ /^[[:space:]]*\[/ {
      in_sec = ($0 == sec)
      next
    }
    in_sec { print }
  ' "${MANIFEST_PATH}" \
    | rg --no-messages '^[[:space:]]*[a-zA-Z0-9_]+[[:space:]]*=' \
    | sed -E 's/^[[:space:]]*([a-zA-Z0-9_]+)[[:space:]]*=[[:space:]]*"([^"]+)"[[:space:]]*$/\1|\2/'
}

emit_placeholder_symbol() {
  local id="$1"

  # Keep the placeholder visually obvious but simple.
  # Use a predictable viewBox so consumers can size consistently.
  printf '    <symbol id="%s" viewBox="0 0 100 100" overflow="visible">\n' "${id}"
  cat <<'EOF'
      <rect x="5" y="5" width="90" height="90" fill="none" stroke="currentColor" stroke-width="8" />
      <line x1="15" y1="15" x2="85" y2="85" stroke="currentColor" stroke-width="8" />
      <line x1="85" y1="15" x2="15" y2="85" stroke="currentColor" stroke-width="8" />
EOF
  printf '    </symbol>\n\n'
}

emit_symbol() {
  local id="$1"
  local file_rel="$2"

  local svg_file="${PACK_DIR%/}/${file_rel}"
  if [[ ! -f "${svg_file}" ]]; then
    echo "warning: missing svg for ${id}: ${svg_file}; using placeholder" >&2
    emit_placeholder_symbol "${id}"
    return 0
  fi

  local vb
  vb=$(svg_viewbox "${svg_file}")

  if [[ -n "${vb}" ]]; then
    printf '    <symbol id="%s" %s overflow="visible">\n' "${id}" "${vb}"
  else
    printf '    <symbol id="%s" overflow="visible">\n' "${id}"
  fi

  svg_inner "${svg_file}" \
    | sed -e 's/^/      /'

  printf '\n    </symbol>\n\n'
}

tmp_out="${OUT_PATH}.tmp"
mkdir -p "$(dirname "${OUT_PATH}")"

{
  echo '<?xml version="1.0" encoding="UTF-8"?>'
  echo '<svg xmlns="http://www.w3.org/2000/svg" style="display:none">'
  echo '  <defs>'

  # Emit symbols from the manifest. If a listed file is missing, we fall back to a placeholder.
  #
  # Additionally, emit placeholder symbols for commonly-used canonical keys that are *not*
  # listed in the manifest. This makes sprite sheets more robust for consumers that expect a
  # fixed set of ids (e.g. rb-sign-aries) even when a given pack is incomplete.

  declare -A seen_bodies=()
  declare -A seen_signs=()
  declare -A seen_angles=()
  declare -A seen_chart_points=()
  declare -A seen_lots=()

  while IFS='|' read -r key file; do
    [[ -z "${key}" ]] && continue
    seen_bodies["${key}"]=1
    emit_symbol "rb-body-${key}" "${file}"
  done < <(manifest_section_pairs bodies)

  for key in "${KNOWN_BODIES[@]}"; do
    if [[ -z "${seen_bodies[${key}]+x}" ]]; then
      emit_placeholder_symbol "rb-body-${key}"
    fi
  done

  while IFS='|' read -r key file; do
    [[ -z "${key}" ]] && continue
    seen_signs["${key}"]=1
    emit_symbol "rb-sign-${key}" "${file}"
  done < <(manifest_section_pairs signs)

  for key in "${KNOWN_SIGNS[@]}"; do
    if [[ -z "${seen_signs[${key}]+x}" ]]; then
      emit_placeholder_symbol "rb-sign-${key}"
    fi
  done

  while IFS='|' read -r key file; do
    [[ -z "${key}" ]] && continue
    seen_angles["${key}"]=1
    emit_symbol "rb-angle-${key}" "${file}"
  done < <(manifest_section_pairs angles)

  for key in "${KNOWN_ANGLES[@]}"; do
    if [[ -z "${seen_angles[${key}]+x}" ]]; then
      emit_placeholder_symbol "rb-angle-${key}"
    fi
  done

  while IFS='|' read -r key file; do
    [[ -z "${key}" ]] && continue
    seen_chart_points["${key}"]=1
    emit_symbol "rb-chart-point-${key}" "${file}"
  done < <(manifest_section_pairs chart_points)

  for key in "${KNOWN_CHART_POINTS[@]}"; do
    if [[ -z "${seen_chart_points[${key}]+x}" ]]; then
      emit_placeholder_symbol "rb-chart-point-${key}"
    fi
  done

  while IFS='|' read -r key file; do
    [[ -z "${key}" ]] && continue
    seen_lots["${key}"]=1
    emit_symbol "rb-lot-${key}" "${file}"
  done < <(manifest_section_pairs lots)

  for key in "${KNOWN_LOTS[@]}"; do
    if [[ -z "${seen_lots[${key}]+x}" ]]; then
      emit_placeholder_symbol "rb-lot-${key}"
    fi
  done

  echo '  </defs>'
  echo '</svg>'
} >"${tmp_out}"

mv "${tmp_out}" "${OUT_PATH}"

echo "wrote ${OUT_PATH}"

