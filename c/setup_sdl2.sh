#!/usr/bin/env bash
# Prepare a native (host-arch) SDL2 dylib for the C build.
#
# On Apple Silicon with an x86_64-only Homebrew, pkg-config points at an
# x86_64 SDL2 that cannot link into an arm64 binary. This script finds an
# arm64 SDL2 dylib already present on the machine, copies it into ./build,
# rewrites its install name to @rpath, and ad-hoc signs it.
#
# Override the source with SDL2_DYLIB=/path/to/libSDL2-2.0.0.dylib

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
build_dir="$here/build"
target="$build_dir/libSDL2-2.0.0.dylib"

arch="$(uname -m)"

candidates=(
    "${SDL2_DYLIB:-}"
    "/opt/homebrew/lib/libSDL2-2.0.0.dylib"
    "/usr/local/lib/libSDL2-2.0.0.dylib"
    "/Applications/digiKam.org/digikam.app/Contents/lib/libSDL2-2.0.0.dylib"
)

# OpenCV wheel bundles a native SDL2; find it in any python site-packages.
while IFS= read -r found; do
    candidates+=("$found")
done < <(find "$HOME/.local/share/uv/python" -name 'libSDL2-2.0.0.dylib' 2>/dev/null | head -n 20)

source_dylib=""
for candidate in "${candidates[@]}"; do
    [ -n "$candidate" ] || continue
    [ -f "$candidate" ] || continue
    if file "$candidate" | grep -q "$arch"; then
        source_dylib="$candidate"
        break
    fi
done

if [ -z "$source_dylib" ]; then
    echo "No $arch SDL2 dylib found. Set SDL2_DYLIB=/path/to/libSDL2-2.0.0.dylib" >&2
    exit 1
fi

mkdir -p "$build_dir"
cp -f "$source_dylib" "$target"
chmod u+w "$target"

install_name_tool -id "@rpath/libSDL2-2.0.0.dylib" "$target"
ln -sf "libSDL2-2.0.0.dylib" "$build_dir/libSDL2.dylib"
codesign --force --sign - "$target" 2>/dev/null || true

echo "Prepared SDL2 from: $source_dylib"
