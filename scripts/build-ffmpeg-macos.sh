#!/bin/bash
# Run in GitHub's free Intel macOS runner, never on the end user's Mac.
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
COMMIT=8ae0b34901ba60a802f183ee75a250a9fc3e09a5 # FFmpeg n8.0.3
SHA256=3822f5dea04f069a90f8b445d00116d6410d1baf8f15d5f750d0626e5d6994f3
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$ROOT/src-tauri/binaries" "$ROOT/src-tauri/third-party"
ARCHIVE="$ROOT/src-tauri/third-party/ffmpeg-source.tar.gz"
curl --fail --location --retry 3 "https://codeload.github.com/FFmpeg/FFmpeg/tar.gz/$COMMIT" -o "$ARCHIVE"
printf '%s  %s\n' "$SHA256" "$ARCHIVE" | shasum -a 256 -c -
tar -xzf "$ARCHIVE" -C "$WORK"
SOURCE="$WORK/FFmpeg-$COMMIT"
cp "$SOURCE/COPYING.LGPLv2.1" "$ROOT/src-tauri/third-party/FFmpeg-LGPL-2.1.txt"
cp "$SOURCE/LICENSE.md" "$ROOT/src-tauri/third-party/FFmpeg-LICENSE.md"
cp "$ROOT/scripts/build-ffmpeg-macos.sh" "$ROOT/src-tauri/third-party/build-ffmpeg-macos.sh"
cp "$ROOT/THIRD_PARTY.md" "$ROOT/src-tauri/third-party/NOTICE.md"
SDK=$(xcrun --sdk macosx --show-sdk-path)
for TARGET in x86_64-apple-darwin aarch64-apple-darwin; do
  if [[ "$TARGET" == aarch64* ]]; then ARCH=aarch64; MACH=arm64; else ARCH=x86_64; MACH=x86_64; fi
  BUILD="$WORK/build-$TARGET"
  mkdir -p "$BUILD"
  cd "$BUILD"
  "$SOURCE/configure" \
    --prefix="$BUILD/install" --target-os=darwin --arch="$ARCH" --cc=clang \
    --enable-cross-compile --sysroot="$SDK" \
    --extra-cflags="-arch $MACH -mmacosx-version-min=12.0" \
    --extra-ldflags="-arch $MACH -mmacosx-version-min=12.0" \
    --disable-autodetect --disable-everything --disable-network \
    --disable-gpl --disable-nonfree --disable-version3 \
    --disable-shared --enable-static --disable-doc --disable-debug --disable-x86asm \
    --enable-ffmpeg --enable-ffprobe --disable-ffplay --enable-videotoolbox \
    --enable-protocol=file,pipe \
    --enable-demuxer=mov,matroska,avi,wav,mp3,flac,ogg \
    --enable-muxer=mp4,null,wav \
    --enable-decoder=h264,hevc,vp8,vp9,av1,mpeg4,aac,mp3,opus,vorbis,flac,alac,pcm_s16le,pcm_s24le,pcm_s32le,pcm_f32le,pcm_f64le \
    --enable-parser=h264,hevc,vp8,vp9,av1,mpeg4video,aac,mpegaudio,opus,vorbis,flac \
    --enable-encoder=h264_videotoolbox,aac,wrapped_avframe,pcm_s16le \
    --enable-filter=scale,crop,setsar,fps,format,aformat,aresample,anull,null,apad,trim,atrim,transpose,hflip,vflip \
    --enable-bsf=h264_mp4toannexb,hevc_mp4toannexb,aac_adtstoasc,extract_extradata,null
  make -j3
  for TOOL in ffmpeg ffprobe; do
    DEST="$ROOT/src-tauri/binaries/$TOOL-$TARGET"
    cp "$BUILD/$TOOL" "$DEST"
    chmod 755 "$DEST"
    strip -x "$DEST"
    codesign --force --sign - "$DEST"
    test "$(lipo -archs "$DEST")" = "$MACH"
    # No Homebrew or other third-party dylibs may be required on the user's Mac.
    otool -L "$DEST" | tail -n +2 | awk '{print $1}' | while IFS= read -r LIB; do
      case "$LIB" in /usr/lib/*|/System/Library/*) ;; *) echo "Unexpected dependency: $LIB" >&2; exit 1;; esac
    done
  done
  cp "$BUILD/config.h" "$ROOT/src-tauri/third-party/ffmpeg-config-$TARGET.h"
  cp "$BUILD/ffbuild/config.mak" "$ROOT/src-tauri/third-party/ffmpeg-config-$TARGET.mak"
done
"$ROOT/src-tauri/binaries/ffmpeg-x86_64-apple-darwin" -version
"$ROOT/src-tauri/binaries/ffprobe-x86_64-apple-darwin" -version
