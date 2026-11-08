#!/bin/bash
# Development/CI only. The installed app never invokes a compiler.
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
COMMIT=48f628a84833905ee4a0658ee6d4a5c915ce1997 # whisper.cpp v1.8.7
SHA256=17c4189d2ba98fedac681dc76944476217008f2c7268784bd519bf069ca37e3a
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$ROOT/src-tauri/binaries" "$ROOT/src-tauri/third-party"
ARCHIVE="$ROOT/src-tauri/third-party/whisper-source.tar.gz"
curl --fail --location --retry 3 "https://codeload.github.com/ggml-org/whisper.cpp/tar.gz/$COMMIT" -o "$ARCHIVE"
if [[ $(uname) == Darwin ]]; then printf '%s  %s\n' "$SHA256" "$ARCHIVE" | shasum -a 256 -c -; else printf '%s  %s\n' "$SHA256" "$ARCHIVE" | sha256sum -c -; fi
tar -xzf "$ARCHIVE" -C "$WORK"
SOURCE="$WORK/whisper.cpp-$COMMIT"
cp "$ROOT/licenses/whisper-models-MIT.txt" "$ROOT/src-tauri/third-party/Whisper-Models-MIT-LICENSE.txt"
cp "$SOURCE/LICENSE" "$ROOT/src-tauri/third-party/Whisper-MIT-LICENSE.txt"
cp "$ROOT/scripts/build-whisper.sh" "$ROOT/src-tauri/third-party/build-whisper.sh"
cp "$SOURCE/samples/jfk.wav" "$ROOT/src-tauri/third-party/whisper-test-jfk.wav"
if [[ $(uname) == Darwin ]]; then
  for ARCH in x86_64 arm64; do
    if [[ "$ARCH" == arm64 ]]; then TARGET=aarch64-apple-darwin; else TARGET=x86_64-apple-darwin; fi
    BUILD="$WORK/build-$ARCH"
    cmake -S "$SOURCE" -B "$BUILD" -DCMAKE_BUILD_TYPE=Release -DCMAKE_OSX_ARCHITECTURES="$ARCH" -DCMAKE_OSX_DEPLOYMENT_TARGET=12.0 \
      -DBUILD_SHARED_LIBS=OFF -DGGML_BACKEND_DL=OFF -DGGML_NATIVE=OFF -DGGML_OPENMP=OFF -DWHISPER_BUILD_TESTS=OFF -DWHISPER_BUILD_SERVER=OFF \
      -DGGML_METAL=ON -DGGML_METAL_EMBED_LIBRARY=ON -DGGML_BLAS=OFF
    cmake --build "$BUILD" --config Release --target whisper-cli -j3
    DEST="$ROOT/src-tauri/binaries/whisper-cli-$TARGET"
    cp "$BUILD/bin/whisper-cli" "$DEST"; chmod 755 "$DEST"; codesign --force --sign - "$DEST"
    test "$(lipo -archs "$DEST")" = "$ARCH"
    otool -L "$DEST" | tail -n +2 | awk '{print $1}' | while IFS= read -r LIB; do
      case "$LIB" in /usr/lib/*|/System/Library/*) ;; *) echo "Unexpected Whisper dependency: $LIB" >&2; exit 1;; esac
    done
  done
  "$ROOT/src-tauri/binaries/whisper-cli-x86_64-apple-darwin" -h
else
  cmake -S "$SOURCE" -B "$WORK/build" -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=OFF -DGGML_BACKEND_DL=OFF -DGGML_NATIVE=OFF -DGGML_OPENMP=OFF -DGGML_METAL=OFF -DGGML_BLAS=OFF -DWHISPER_BUILD_TESTS=OFF -DWHISPER_BUILD_SERVER=OFF
  cmake --build "$WORK/build" --target whisper-cli -j3
  cp "$WORK/build/bin/whisper-cli" "$ROOT/src-tauri/binaries/whisper-cli-x86_64-unknown-linux-gnu"
fi
