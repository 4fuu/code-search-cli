#!/usr/bin/env bash
set -euo pipefail

REPO_SLUG="${CODES_REPO_SLUG:-4fuu/code-search-cli}"
INSTALL_DIR="${CODES_INSTALL_DIR:-$HOME/.local/bin}"
VERSION_INPUT="${1:-latest}"

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64) echo "x86_64-unknown-linux-gnu" ;;
        *) echo "unsupported Linux architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64) echo "x86_64-apple-darwin" ;;
        arm64|aarch64) echo "aarch64-apple-darwin" ;;
        *) echo "unsupported macOS architecture: $arch" >&2; exit 1 ;;
      esac
      ;;
    *)
      echo "unsupported operating system: $os" >&2
      exit 1
      ;;
  esac
}

resolve_tag() {
  if [[ "$VERSION_INPUT" == "latest" ]]; then
    curl -fsSL "https://api.github.com/repos/$REPO_SLUG/releases/latest" |
      sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' |
      head -n 1
  else
    local version="${VERSION_INPUT#v}"
    printf 'v%s\n' "$version"
  fi
}

verify_checksum() {
  local file="$1"
  local archive_name="$2"
  local sums_file="$3"
  local expected
  expected="$(grep "  $archive_name\$" "$sums_file" | awk '{print $1}')"
  if [[ -z "$expected" ]]; then
    echo "checksum entry not found for $archive_name" >&2
    exit 1
  fi

  if command -v sha256sum >/dev/null 2>&1; then
    echo "$expected  $file" | sha256sum --check --status
  elif command -v shasum >/dev/null 2>&1; then
    local actual
    actual="$(shasum -a 256 "$file" | awk '{print $1}')"
    [[ "$actual" == "$expected" ]]
  else
    echo "warning: no sha256 tool found, skipping checksum verification" >&2
  fi
}

tmpdir=""

main() {
  require_cmd curl
  require_cmd tar
  require_cmd install

  local target tag archive_name archive_path sums_path
  target="$(detect_target)"
  tag="$(resolve_tag)"
  if [[ -z "$tag" ]]; then
    echo "failed to resolve release tag" >&2
    exit 1
  fi

  archive_name="codes-$tag-$target.tar.gz"
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  archive_path="$tmpdir/$archive_name"
  sums_path="$tmpdir/SHA256SUMS.txt"

  curl -fsSL "https://github.com/$REPO_SLUG/releases/download/$tag/$archive_name" -o "$archive_path"
  curl -fsSL "https://github.com/$REPO_SLUG/releases/download/$tag/SHA256SUMS.txt" -o "$sums_path"
  verify_checksum "$archive_path" "$archive_name" "$sums_path"

  mkdir -p "$INSTALL_DIR"
  tar -xzf "$archive_path" -C "$tmpdir"
  install -m 0755 "$tmpdir/codes" "$INSTALL_DIR/codes"

  echo "installed codes to $INSTALL_DIR/codes"
  echo "ensure $INSTALL_DIR is in PATH"
}

main
