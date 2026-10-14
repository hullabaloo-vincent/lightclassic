#!/bin/bash
#
# Double-clickable macOS build script.
#
# Builds LightCraft.app from this checkout and offers to put it in Applications.
# Because the app is compiled on this Mac rather than downloaded, macOS never
# attaches the quarantine flag, so it opens with no Gatekeeper warning.
#
# Checks what the build needs (Xcode Command Line Tools, Rust) and offers to
# install or update it. Nothing is installed without asking first.
#
# The `.command` extension is what makes Finder run this in Terminal on a
# double-click; a plain `.sh` would open in a text editor instead.
#
# From a terminal:
#   "scripts/Build LightCraft (macOS).command" [--no-install]
#
#   --no-install            build dist/LightCraft.app and stop
#   LIGHTCRAFT_INSTALL_DIR  where to install (default: /Applications)
#   CRAFT_FONTS_DIR         a storytold/craft-fonts checkout, for Chinese and Japanese text
#                           (found by itself when it sits in or next to this repository)
#
# Run without a terminal attached (a pipe, CI) it asks nothing: it builds the
# app and neither installs nor opens it.
#
# The bundle is put together the way packaging/macos/package.sh does it for a
# release (same Info.plist template, icon, entitlements and signing order), but
# for this Mac's processor only, signed ad hoc, with no DMG and no notarization.
# If the bundle's layout changes there, change it here too.

set -uo pipefail

# --------------------------------------------------------------- appearance

if [ -t 1 ]; then
  BOLD=$'\033[1m'; DIM=$'\033[2m'; RED=$'\033[31m'; GREEN=$'\033[32m'
  YELLOW=$'\033[33m'; BLUE=$'\033[34m'; RESET=$'\033[0m'
else
  BOLD=""; DIM=""; RED=""; GREEN=""; YELLOW=""; BLUE=""; RESET=""
fi

step()  { printf '\n%s==>%s %s%s%s\n' "$BLUE" "$RESET" "$BOLD" "$1" "$RESET"; }
ok()    { printf '  %s✓%s %s\n' "$GREEN" "$RESET" "$1"; }
err()   { printf '  %s✗%s %s\n' "$RED" "$RESET" "$1"; }
info()  { printf '    %s%s%s\n' "$DIM" "$1" "$RESET"; }
caution() { printf '  %s!%s %s\n' "$YELLOW" "$RESET" "$1"; }

# Keep the Terminal window readable after a double-click, win or lose.
pause_and_exit() {
  local code="${1:-0}"
  if [ -t 0 ]; then
    printf '\n%sPress Return to close this window.%s ' "$DIM" "$RESET"
    read -r _ || true
  fi
  exit "$code"
}

fail() {
  err "$1"
  printf '\n%sBuild failed.%s If you are stuck, open an issue with the output above:\n' "$RED" "$RESET"
  printf '  https://github.com/hullabaloo-vincent/lightclassic/issues\n'
  pause_and_exit 1
}

# Ask a yes/no question; Return means yes. With no terminal to ask on, the
# answer is no: nothing is installed, replaced or opened unattended.
confirm() {
  local prompt="$1" reply
  if [ ! -t 0 ]; then
    info "not asked (no terminal), so no: $prompt"
    return 1
  fi
  printf '  %s%s%s [Y/n] ' "$BOLD" "$prompt" "$RESET"
  read -r reply || return 1
  case "$reply" in
    [nN]*) return 1 ;;
    *)     return 0 ;;
  esac
}

# ------------------------------------------------------------------ prelude

INSTALL=1
for arg in "$@"; do
  case "$arg" in
    --no-install) INSTALL=0 ;;
    -h | --help) sed -n '3,30p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) err "unknown argument: $arg (see --help)"; exit 2 ;;
  esac
done

# A double-click starts in the home directory, so anchor to the repo root.
cd "$(dirname "${BASH_SOURCE[0]}")/.." || fail "cannot locate the repository"
REPO_ROOT="$(pwd)"

printf '%s\n' "$BOLD"
printf '  LightCraft — build from source\n'
printf '%s' "$RESET"
printf '  %sCompiles the app on this Mac. Because it is built here rather than%s\n' "$DIM" "$RESET"
printf '  %sdownloaded, macOS will not quarantine it — it just opens.%s\n' "$DIM" "$RESET"
printf '  %sThe first build takes a while (Rust compiles a lot); later ones are fast.%s\n' "$DIM" "$RESET"

[ "$(uname -s)" = "Darwin" ] || fail "this script is for macOS. Elsewhere: cargo build --release -p lightcraft"
for needed in Cargo.toml apps/lightcraft/Cargo.toml packaging/env.sh packaging/macos/Info.plist.in \
  packaging/macos/entitlements.plist assets/app-icon/lightcraft.icns; do
  [ -f "$REPO_ROOT/$needed" ] || fail "run this from inside the LightCraft repository ($needed is missing)"
done

# Rust installed by rustup is on PATH in a login shell, but not always in the
# shell Finder starts for a double-click.
# shellcheck disable=SC1091
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

step "Checking what this build needs"
info "$(sw_vers -productName) $(sw_vers -productVersion) on $(uname -m)"

# ------------------------------------------------------- Xcode Command Line Tools

if xcode-select -p >/dev/null 2>&1; then
  ok "Xcode Command Line Tools"
else
  caution "Xcode Command Line Tools are missing (needed to link and sign the app)"
  if confirm "Install them now? A macOS installer window will open."; then
    xcode-select --install >/dev/null 2>&1 || true
    info "Finish the install in the window that just opened, then press Return here."
    read -r _ || true
    xcode-select -p >/dev/null 2>&1 || fail "still not found — re-run this script once the install finishes"
    ok "Xcode Command Line Tools installed"
  else
    fail "cannot build without the Command Line Tools (install them with: xcode-select --install)"
  fi
fi

# -------------------------------------------------------------------- Rust

if command -v cargo >/dev/null 2>&1; then
  ok "Rust $(rustc --version 2>/dev/null | cut -d' ' -f2)"
else
  caution "Rust is not installed (LightCraft is written in Rust)"
  if confirm "Install Rust via rustup? This is the official installer."; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path \
      || fail "rustup install failed"
    # shellcheck disable=SC1091
    [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
    export PATH="$HOME/.cargo/bin:$PATH"
    command -v cargo >/dev/null 2>&1 || fail "cargo still not found — open a new Terminal and re-run this script"
    ok "Rust $(rustc --version | cut -d' ' -f2) installed"
  else
    fail "Rust is required to build the app (install it from https://rustup.rs)"
  fi
fi

# ------------------------------------------------------------------- fonts

# Chinese and Japanese text needs the fonts of storytold/craft-fonts, an optional build input.
if [ -z "${CRAFT_FONTS_DIR:-}" ]; then
  for fonts in "$REPO_ROOT/craft-fonts" "$REPO_ROOT/../craft-fonts"; do
    if [ -f "$fonts/manifest.json" ] || [ -d "$fonts/fonts" ]; then
      CRAFT_FONTS_DIR="$(cd "$fonts" && pwd)"
      export CRAFT_FONTS_DIR
      break
    fi
  done
fi
if [ -n "${CRAFT_FONTS_DIR:-}" ]; then
  ok "craft-fonts at $CRAFT_FONTS_DIR"
else
  info "no craft-fonts checkout: Chinese and Japanese text will show as empty boxes (optional)"
fi

# ------------------------------------------------------------------- build

# The version, the commit recorded in Info.plist and copy_docs (the licences that go into the bundle).
# The app is kept in dist/, which git ignores.
export DIST="$REPO_ROOT/dist"
# shellcheck source=../packaging/env.sh
. "$REPO_ROOT/packaging/env.sh"
PACKAGING="$REPO_ROOT/packaging/macos"
APP="$DIST/LightCraft.app"
LOG="$CARGO_TARGET_DIR/build-app.log"

# Keep in sync with LSMinimumSystemVersion in packaging/macos/Info.plist.in.
export MACOSX_DEPLOYMENT_TARGET=11.0

COLOR=()
[ -t 1 ] && COLOR=(--color always)

# heif: open HEIC / HEIF photos, as the released app does.
build() {
  mkdir -p "$CARGO_TARGET_DIR"
  (cd "$REPO_ROOT" && cargo build --release --locked -p lightcraft --features lightcraft/heif ${COLOR[@]+"${COLOR[@]}"}) 2>&1 | tee "$LOG"
  return "${PIPESTATUS[0]}"
}

step "Building LightCraft $VERSION (this is the slow part — grab a coffee)"
if ! build; then
  # An older Rust than a dependency needs is the usual reason a fresh checkout fails to build.
  if grep -q "is not supported by the following package\|requires rustc" "$LOG" && command -v rustup >/dev/null 2>&1; then
    caution "Rust $(rustc --version | cut -d' ' -f2) is older than this version of LightCraft needs"
    if confirm "Update Rust now (rustup update stable)?"; then
      rustup update stable --no-self-update || fail "rustup could not update Rust"
      ok "Rust $(rustc --version | cut -d' ' -f2)"
      build || fail "the build failed — the compiler output above says why"
    else
      fail "update Rust with: rustup update stable"
    fi
  elif grep -q "lock file .* needs to be updated\|--locked was passed" "$LOG"; then
    fail "Cargo.lock does not match Cargo.toml — run 'cargo build --release -p lightcraft' once, commit Cargo.lock, and re-run this script"
  else
    fail "the build failed — the compiler output above says why"
  fi
fi
BIN="$CARGO_TARGET_DIR/release/lightcraft"
[ -x "$BIN" ] || fail "build reported success but $BIN is missing"
ok "compiled"

# ------------------------------------------------------------------ bundle

step "Putting LightCraft.app together"
SHORT_VERSION="${VERSION%%-*}"
rm -rf "$APP" || fail "could not remove the previous $APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources/Licenses" || fail "could not create $APP"
# Executable and icon carry the display name (CFBundleExecutable / CFBundleIconFile).
cp "$BIN" "$APP/Contents/MacOS/LightCraft" || fail "could not copy the executable"
cp "$REPO_ROOT/assets/app-icon/lightcraft.icns" "$APP/Contents/Resources/LightCraft.icns" || fail "could not copy the icon"
sed -e "s/@VERSION@/$VERSION/g" -e "s/@SHORT_VERSION@/$SHORT_VERSION/g" \
  -e "s/@BUILD_SHA@/${LIGHTCRAFT_BUILD_SHA:-unknown}/g" \
  "$PACKAGING/Info.plist.in" >"$APP/Contents/Info.plist" || fail "could not write Info.plist"
plutil -lint "$APP/Contents/Info.plist" >/dev/null || fail "Info.plist is not valid"
printf 'APPL????' >"$APP/Contents/PkgInfo"
copy_docs "$APP/Contents/Resources/Licenses"
ok "LightCraft $VERSION ($(echo "${LIGHTCRAFT_BUILD_SHA:-unknown}" | cut -c1-8)) for $(uname -m)"

# Sign ad hoc, inside out: the executable, then the bundle. Extended attributes
# picked up during the build are removed first; they would invalidate the seal.
xattr -cr "$APP"
sign() {
  local out
  # (codesign's "replacing existing signature" is expected: the linker signs what it links)
  out="$(codesign --force --sign - --timestamp=none --options runtime --entitlements "$PACKAGING/entitlements.plist" "$@" 2>&1)" && return 0
  printf '%s\n' "$out" >&2
  return 1
}
sign "$APP/Contents/MacOS/LightCraft" || fail "codesign could not sign the executable"
sign "$APP" || fail "codesign could not sign the app"
# An invalid signature is the thing that makes macOS call an app "damaged".
if codesign --verify --strict --deep "$APP" >/dev/null 2>&1; then
  ok "code signature valid (ad hoc)"
else
  fail "the code signature could not be verified"
fi
ok "built $APP"

# ---------------------------------------------------------------- install

if [ "$INSTALL" = 0 ]; then
  info "--no-install: left in place"
  pause_and_exit 0
fi

step "Installing"
INSTALL_DIR="${LIGHTCRAFT_INSTALL_DIR:-/Applications}"
DEST="$INSTALL_DIR/LightCraft.app"
FINAL="$APP"
if confirm "Put LightCraft.app in $INSTALL_DIR?"; then
  # A copy that is running keeps running from files that are about to go.
  while pgrep -f "$DEST/Contents/MacOS/LightCraft" >/dev/null 2>&1; do
    caution "LightCraft is open. Quit it (⌘Q) so it can be replaced."
    confirm "Quit it and try again?" || break
  done
  if pgrep -f "$DEST/Contents/MacOS/LightCraft" >/dev/null 2>&1; then
    info "still open: the installed copy is kept; the fresh build stays in dist/"
  elif [ -e "$DEST" ] && ! confirm "LightCraft is already in $INSTALL_DIR. Replace it?"; then
    info "keeping the installed copy; the fresh build stays in dist/"
  else
    mkdir -p "$INSTALL_DIR" || fail "could not create $INSTALL_DIR"
    if [ -e "$DEST" ]; then
      rm -rf "$DEST" || fail "could not remove the old copy"
    fi
    # ditto keeps the bundle exactly as signed.
    ditto "$APP" "$DEST" || fail "could not copy to $INSTALL_DIR"
    codesign --verify --strict --deep "$DEST" >/dev/null 2>&1 || fail "the installed copy's signature is not valid"
    ok "installed $DEST"
    FINAL="$DEST"
  fi
else
  info "left in place: $APP"
fi

printf '\n%s%sLightCraft is ready.%s\n' "$BOLD" "$GREEN" "$RESET"
info "Built on this Mac, so it is not quarantined — it opens like any other app."
info "A rebuilt app counts as a new one to macOS: it may ask again before reading Desktop, Documents or Pictures."

if confirm "Open LightCraft now?"; then
  open "$FINAL" || caution "could not launch it; open it from Finder"
elif [ -t 0 ]; then
  open -R "$FINAL"
fi

pause_and_exit 0
