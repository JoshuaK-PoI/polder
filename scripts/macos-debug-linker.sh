#!/bin/sh
set -u
: "${MACOS_DEBUG_SIGN:=1}"
: "${MACOS_DEBUG_SIGN_CC:=cc}"
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" 2>/dev/null && pwd) || script_dir=""
: "${MACOS_DEBUG_ENTITLEMENTS:=$script_dir/debug-entitlements.plist}"
# Recover the link output (-o) so we know what to sign.
out=""; prev=""
for arg in "$@"; do
    case "$prev" in -o) out=$arg ;; esac
    case "$arg" in -o?*) out=${arg#-o} ;; esac
    prev=$arg
done
"$MACOS_DEBUG_SIGN_CC" "$@"
status=$?
[ "$status" -eq 0 ] || exit "$status"
[ "$MACOS_DEBUG_SIGN" = "0" ] && exit 0
[ -n "$out" ] && [ -f "$out" ] || exit 0
# Only debug artifacts. Release passes through untouched.
case "/$out" in */debug/*) ;; *) exit 0 ;; esac
# A signing failure must not break the build: warn, don't die.
if ! err=$(codesign --force --sign - --entitlements "$MACOS_DEBUG_ENTITLEMENTS" "$out" 2>&1); then
    printf 'macos-debug-linker: warning: codesign failed for %s: %s\n' "$out" "$err" >&2
fi
exit 0