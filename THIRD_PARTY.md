# FFmpeg in AI Clipper

AI Clipper uses unmodified [FFmpeg](https://ffmpeg.org/) n8.0.3, commit `8ae0b34901ba60a802f183ee75a250a9fc3e09a5`, as separate command-line executables (FFmpeg and FFprobe). It is **not linked into the application**. The build disables GPL, nonfree, version-3 components, external codec libraries and networking. The bundled build is LGPL 2.1 or later. The application's MIT license does not replace FFmpeg's license.

The complete corresponding upstream source archive, LGPL text, upstream licensing notes, build script and generated build configuration for both architectures are included in every `.app` under `Contents/Resources/third-party/`. In Finder, right-click AI Clipper → Show Package Contents to view these files. The SHA-256 of the source archive is `3822f5dea04f069a90f8b445d00116d6410d1baf8f15d5f750d0626e5d6994f3`.

Source: https://github.com/FFmpeg/FFmpeg/tree/8ae0b34901ba60a802f183ee75a250a9fc3e09a5

Rebuild in a macOS development/CI environment with Xcode using `scripts/build-ffmpeg-macos.sh`. No changes to upstream source are applied. FFmpeg's LGPL libraries are linked statically only into the standalone FFmpeg/FFprobe executables, which can be rebuilt or replaced independently. The app resolves the executables beside its own executable, without hard-coded hashes or a required Developer ID. Replacing binaries requires renewing the app's local ad-hoc signature; no such developer operation is needed for normal use.

H.264 encoding uses Apple's system VideoToolbox framework. Audio uses FFmpeg's native AAC encoder. AI Clipper includes no x264, GPL codec libraries, proprietary API services or purchased codecs. These license notices concern software copyright; availability of codec patent permissions depends on use and jurisdiction.
