# FFmpeg in AI Clipper

AI Clipper uses unmodified [FFmpeg](https://ffmpeg.org/) n8.0.3, commit `8ae0b34901ba60a802f183ee75a250a9fc3e09a5`, as separate command-line executables (FFmpeg and FFprobe). It is **not linked into the application**. The build disables GPL, nonfree, version-3 components, external codec libraries and networking. The bundled build is LGPL 2.1 or later. The application's MIT license does not replace FFmpeg's license.

The complete corresponding upstream source archive, LGPL text, upstream licensing notes, build script and generated build configuration for both architectures are included in every `.app` under `Contents/Resources/third-party/`. In Finder, right-click AI Clipper → Show Package Contents to view these files. The SHA-256 of the source archive is `3822f5dea04f069a90f8b445d00116d6410d1baf8f15d5f750d0626e5d6994f3`.

Source: https://github.com/FFmpeg/FFmpeg/tree/8ae0b34901ba60a802f183ee75a250a9fc3e09a5

Rebuild in a macOS development/CI environment with Xcode using `scripts/build-ffmpeg-macos.sh`. No changes to upstream source are applied. FFmpeg's LGPL libraries are linked statically only into the standalone FFmpeg/FFprobe executables, which can be rebuilt or replaced independently. The app resolves the executables beside its own executable, without hard-coded hashes or a required Developer ID. Replacing binaries requires renewing the app's local ad-hoc signature; no such developer operation is needed for normal use.

H.264 encoding uses Apple's system VideoToolbox framework. Audio uses FFmpeg's native AAC encoder. AI Clipper includes no x264, GPL codec libraries, proprietary API services or purchased codecs. These license notices concern software copyright; availability of codec patent permissions depends on use and jurisdiction.


# whisper.cpp and Whisper model weights

AI Clipper includes whisper.cpp v1.8.7, commit `48f628a84833905ee4a0658ee6d4a5c915ce1997`, under the MIT license. Its unmodified source archive (SHA-256 `17c4189d2ba98fedac681dc76944476217008f2c7268784bd519bf069ca37e3a`), build script and license are bundled in `Contents/Resources/third-party/`. Source: https://github.com/ggml-org/whisper.cpp/tree/48f628a84833905ee4a0658ee6d4a5c915ce1997 . Metal kernels are embedded into the executable; no external backend libraries are required.

The `base.en` model is a GGML conversion of OpenAI's open-source Whisper weights (MIT), downloaded from the public repository linked by whisper.cpp's own model documentation: https://huggingface.co/ggerganov/whisper.cpp . It is not an OpenAI API integration. The model license is included as `Whisper-Models-MIT-LICENSE.txt` and in this repository under `licenses/`.

Expected base.en file: `ggml-base.en.bin`, 147,964,211 bytes, SHA-256 `a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002`. The pinned upstream whisper.cpp model README also lists SHA-1 `137c40403d78fd54d454da0f9bd998f78703390c`. The app installs only a verified file.

`tests/fixtures/jfk.wav` comes from the pinned whisper.cpp samples and contains President Kennedy's English public speech. It is used only for reproducible local/CI recognition tests, not uploaded to an inference service.
