#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 /path/to/tplai/kbsim" >&2
  exit 2
fi

source_root=$1
audio_root="$source_root/src/assets/audio"
license_source="$source_root/LICENSE.md"
expected_commit="ba103f3b0afa9dab80447aa2e7e2ed80b6bd80e4"

if [[ ! -d "$audio_root" || ! -f "$license_source" ]]; then
  echo "source must be a checkout of https://github.com/tplai/kbsim" >&2
  exit 2
fi

actual_commit=$(git -C "$source_root" rev-parse HEAD)
if [[ "$actual_commit" != "$expected_commit" ]]; then
  echo "expected kbsim commit $expected_commit, found $actual_commit" >&2
  exit 2
fi

if ! command -v ffmpeg >/dev/null 2>&1; then
  echo "ffmpeg is required to convert the licensed MP3 assets to PCM WAV" >&2
  exit 2
fi

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/.." && pwd)

pack_ids=(alpaca blackink bluealps boxnavy cream holypanda mxblack mxblue mxbrown redink topre turquoise)
pack_names=(
  "Alpaca Switch"
  "Gateron Black Ink"
  "Blue Alps"
  "Box Navy"
  "NK Cream"
  "Holy Panda"
  "Cherry MX Black"
  "Cherry MX Blue"
  "Cherry MX Brown"
  "Gateron Red Ink"
  "Topre"
  "Turquoise Tealios"
)
pack_types=(
  "Smooth linear"
  "Deep linear"
  "Vintage clicky"
  "Heavy clicky"
  "Buttery linear"
  "Premium tactile"
  "Classic linear"
  "Standard clicky"
  "Light tactile"
  "Light linear"
  "Premium rubber dome"
  "Premium linear"
)

convert_sample() {
  local input=$1
  local output=$2
  mkdir -p "$(dirname "$output")"
  ffmpeg -y -loglevel error -i "$input" -ac 1 -ar 48000 -c:a pcm_s16le "$output"
}

for index in "${!pack_ids[@]}"; do
  upstream_id=${pack_ids[$index]}
  pack_id="kbsim-${upstream_id}"
  pack_dir="$repo_root/packs/$pack_id"
  source_dir="$audio_root/$upstream_id"

  mkdir -p "$pack_dir/samples/press/default" "$pack_dir/samples/release/default"
  printf '1\n' > "$pack_dir/.keytone-bundled"
  cp "$license_source" "$pack_dir/LICENSE.txt"
  printf '%s\n' \
    "# Source" \
    "" \
    "Audio adapted from Thomas Lai's kbsim repository:" \
    "https://github.com/tplai/kbsim" \
    "" \
    "Pinned source commit: $expected_commit" \
    "" \
    "The original MP3 files were decoded to mono, 48 kHz, 16-bit PCM WAV for Keytone." \
    > "$pack_dir/SOURCE.md"

  for variation in 0 1 2 3 4; do
    output_index=$(printf '%02d' "$((variation + 1))")
    convert_sample \
      "$source_dir/press/GENERIC_R${variation}.mp3" \
      "$pack_dir/samples/press/default/${output_index}.wav"
  done

  if [[ "$upstream_id" != "mxblue" ]]; then
    for category in space enter backspace; do
      upstream_name=$(printf '%s' "$category" | tr '[:lower:]' '[:upper:]')
      convert_sample \
        "$source_dir/press/${upstream_name}.mp3" \
        "$pack_dir/samples/press/$category/01.wav"
      convert_sample \
        "$source_dir/release/${upstream_name}.mp3" \
        "$pack_dir/samples/release/$category/01.wav"
    done
  fi

  convert_sample \
    "$source_dir/release/GENERIC.mp3" \
    "$pack_dir/samples/release/default/01.wav"

  press_special='{}'
  release_special='{}'
  if [[ "$upstream_id" != "mxblue" ]]; then
    press_special='{
      "space": ["samples/press/space/01.wav"],
      "enter": ["samples/press/enter/01.wav"],
      "backspace": ["samples/press/backspace/01.wav"]
    }'
    release_special='{
      "space": ["samples/release/space/01.wav"],
      "enter": ["samples/release/enter/01.wav"],
      "backspace": ["samples/release/backspace/01.wav"]
    }'
  fi

  jq -n \
    --arg id "$pack_id" \
    --arg name "${pack_names[$index]}" \
    --arg type "${pack_types[$index]}" \
    --argjson pressSpecial "$press_special" \
    --argjson releaseSpecial "$release_special" \
    '{
      schemaVersion: 1,
      id: $id,
      name: $name,
      author: "Thomas Lai / kbsim contributors",
      version: "1.0.0",
      description: ($type + " recordings, adapted from the MIT-licensed kbsim project."),
      license: "MIT",
      tags: (($type | ascii_downcase | split(" ")) + ["recorded", "kbsim"]),
      preview: "samples/press/default/01.wav",
      samples: {
        press: ({
          default: [
            "samples/press/default/01.wav",
            "samples/press/default/02.wav",
            "samples/press/default/03.wav",
            "samples/press/default/04.wav",
            "samples/press/default/05.wav"
          ]
        } + $pressSpecial),
        release: ({
          default: ["samples/release/default/01.wav"]
        } + $releaseSpecial)
      }
    }' > "$pack_dir/manifest.json"
done

echo "Imported ${#pack_ids[@]} MIT-licensed kbsim packs into $repo_root/packs"
