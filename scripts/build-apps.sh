#!/usr/bin/env bash
# Build the installable apps the /download page offers, into `downloads/`.
#
# Four artifacts, from three toolchains:
#   Windows installer + portable  — Tauri, cross-compiled with cargo-xwin
#   Linux .deb                    — Tauri, native
#   Android APK                   — Capacitor, via the Android SDK
#
# Cross-compiling to Windows from Linux needs a linker and a resource
# compiler that the Rust target alone does not bring:
#   apt-get install clang lld nsis
#   ln -s /usr/lib/llvm-19/bin/clang-cl /usr/local/bin/clang-cl
#   ln -s /usr/lib/llvm-19/bin/llvm-lib /usr/local/bin/llvm-lib   # = llvm-ar
#   ln -s /usr/lib/llvm-19/bin/llvm-rc  /usr/local/bin/llvm-rc
# Debian's NSIS ships no Swahili translation, and tauri.conf.json asks for one,
# so Swahili.nlf/.nsh must be copied into /usr/share/nsis/Contrib/Language files/.
# Without it makensis aborts with "Can't open language file".
#
#   bash scripts/build-apps.sh            # everything
#   bash scripts/build-apps.sh windows    # one target
set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TOOLCHAIN="${MTS_TOOLCHAIN:-/workspace/.toolchain}"
export CARGO_HOME="${CARGO_HOME:-$TOOLCHAIN/cargo}"
export RUSTUP_HOME="${RUSTUP_HOME:-$TOOLCHAIN/rustup}"
export PATH="$CARGO_HOME/bin:$PATH"

ANDROID_HOME="${ANDROID_HOME:-/workspace/.android-sdk}"
export ANDROID_HOME ANDROID_SDK_ROOT="$ANDROID_HOME"
export JAVA_HOME="${JAVA_HOME:-/usr/lib/jvm/java-21-openjdk-amd64}"

OUT="$ROOT/downloads"
mkdir -p "$OUT"
WANT="${1:-all}"

log() { echo "[build-apps] $*"; }

build_windows() {
  log "Windows — Tauri installer + portable (cargo-xwin)"
  ( cd "$ROOT/desktop" && \
    cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis )
  local rel="$ROOT/desktop/src-tauri/target/x86_64-pc-windows-msvc/release"
  cp "$rel/bundle/nsis/Mbilinyi Tech_1.0.0_x64-setup.exe" "$OUT/MbilinyiTech-Setup-Windows.exe"
  cp "$rel/mbilinyi-tech-desktop.exe" "$OUT/MbilinyiTech-Windows-Portable.exe"
}

build_linux() {
  log "Linux — Tauri .deb (native)"
  ( cd "$ROOT/desktop" && cargo tauri build --bundles deb )
  cp "$ROOT/desktop/src-tauri/target/release/bundle/deb/Mbilinyi Tech_1.0.0_amd64.deb" \
     "$OUT/MbilinyiTech-Linux-amd64.deb"
}

build_android() {
  log "Android — Capacitor APK"
  ( cd "$ROOT/mobile" && npm install --no-audit --no-fund && npx cap sync android )
  ( cd "$ROOT/mobile/android" && ./gradlew assembleDebug --no-daemon )
  cp "$ROOT/mobile/android/app/build/outputs/apk/debug/app-debug.apk" \
     "$OUT/MbilinyiTech-Android.apk"
}

case "$WANT" in
  windows) build_windows ;;
  linux)   build_linux ;;
  android) build_android ;;
  all)     build_windows; build_linux; build_android ;;
  *) echo "usage: $0 [all|windows|linux|android]" >&2; exit 2 ;;
esac

log "done — artifacts in $OUT"
ls -la "$OUT"
