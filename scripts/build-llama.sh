#!/bin/bash
# Cloud/CI build only; the installed app does not invoke development tools.
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
COMMIT=a691b2345844ce1b430201fc5fd404ce062e05f2
SHA256=3180262acb9c27dc8f61925d898855e408a2f2b040dd9975f21a9c47816536a1
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$ROOT/src-tauri/binaries" "$ROOT/src-tauri/third-party"
ARCHIVE="$ROOT/src-tauri/third-party/llama-source.tar.gz"
curl --fail --location --retry 3 "https://codeload.github.com/ggml-org/llama.cpp/tar.gz/$COMMIT" -o "$ARCHIVE"
if [[ $(uname) == Darwin ]]; then printf '%s  %s\n' "$SHA256" "$ARCHIVE" | shasum -a 256 -c -; else printf '%s  %s\n' "$SHA256" "$ARCHIVE" | sha256sum -c -; fi
tar -xzf "$ARCHIVE" -C "$WORK"
SOURCE="$WORK/llama.cpp-$COMMIT"
cp "$SOURCE/LICENSE" "$ROOT/src-tauri/third-party/Llama-MIT-LICENSE.txt"
cp "$ROOT/licenses/qwen-Apache-2.0.txt" "$ROOT/src-tauri/third-party/Qwen-Apache-2.0.txt"
cp "$ROOT/scripts/build-llama.sh" "$ROOT/src-tauri/third-party/build-llama.sh"
COMMON=(-DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=OFF -DGGML_BACKEND_DL=OFF -DGGML_NATIVE=OFF -DGGML_OPENMP=OFF -DGGML_BLAS=OFF -DLLAMA_OPENSSL=OFF -DLLAMA_BUILD_TESTS=OFF -DLLAMA_BUILD_SERVER=OFF -DLLAMA_BUILD_APP=OFF -DLLAMA_BUILD_UI=OFF -DLLAMA_BUILD_EXAMPLES=OFF)
if [[ $(uname) == Darwin ]]; then
  for ARCH in x86_64 arm64; do
    if [[ "$ARCH" == arm64 ]]; then TARGET=aarch64-apple-darwin; else TARGET=x86_64-apple-darwin; fi
    BUILD="$WORK/build-$ARCH"
    cmake -S "$SOURCE" -B "$BUILD" "${COMMON[@]}" -DCMAKE_OSX_ARCHITECTURES="$ARCH" -DCMAKE_OSX_DEPLOYMENT_TARGET=12.0 -DGGML_METAL=ON -DGGML_METAL_EMBED_LIBRARY=ON
    cmake --build "$BUILD" --target llama-completion -j3
    DEST="$ROOT/src-tauri/binaries/llama-completion-$TARGET"
    cp "$BUILD/bin/llama-completion" "$DEST"; chmod 755 "$DEST"; codesign --force --sign - "$DEST"
    test "$(lipo -archs "$DEST")" = "$ARCH"
    otool -L "$DEST" | tail -n +2 | awk '{print $1}' | while IFS= read -r LIB; do
      case "$LIB" in /usr/lib/*|/System/Library/*) ;; *) echo "Unexpected llama dependency: $LIB" >&2; exit 1;; esac
    done
  done
  "$ROOT/src-tauri/binaries/llama-completion-x86_64-apple-darwin" --version
else
  cmake -S "$SOURCE" -B "$WORK/build" "${COMMON[@]}" -DGGML_METAL=OFF
  cmake --build "$WORK/build" --target llama-completion -j3
  cp "$WORK/build/bin/llama-completion" "$ROOT/src-tauri/binaries/llama-completion-x86_64-unknown-linux-gnu"
fi
