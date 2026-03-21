#!/usr/bin/env sh
set -eu

PROJECT_NAME="therepo"
BINARY_NAME="repo"
REPO_SLUG="${THEREPO_REPO:-__GITHUB_REPOSITORY__}"
VERSION="${THEREPO_VERSION:-latest}"
INSTALL_DIR="${THEREPO_INSTALL_DIR:-${CARGO_HOME:-$HOME/.cargo}/bin}"
ARCHIVE_BASENAME="${THEREPO_ARCHIVE_BASENAME:-therepo}"

if [ -z "$REPO_SLUG" ] || [ "$REPO_SLUG" = "__GITHUB_REPOSITORY__" ]; then
    echo "error: set THEREPO_REPO to your GitHub repo, for example: owner/therepo" >&2
    exit 1
fi

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "error: missing required command: $1" >&2
        exit 1
    }
}

detect_target() {
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux) os_part="unknown-linux-gnu" ;;
        Darwin) os_part="apple-darwin" ;;
        *)
            echo "error: unsupported operating system: $os" >&2
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64) arch_part="x86_64" ;;
        aarch64|arm64) arch_part="aarch64" ;;
        *)
            echo "error: unsupported architecture: $arch" >&2
            exit 1
            ;;
    esac

    printf "%s-%s" "$arch_part" "$os_part"
}

build_url() {
    target="$1"
    archive="${ARCHIVE_BASENAME}-${target}.tar.gz"

    if [ "$VERSION" = "latest" ]; then
        printf "https://github.com/%s/releases/latest/download/%s" "$REPO_SLUG" "$archive"
    else
        printf "https://github.com/%s/releases/download/%s/%s" "$REPO_SLUG" "$VERSION" "$archive"
    fi
}

main() {
    need_cmd curl
    need_cmd tar
    need_cmd mktemp

    target="$(detect_target)"
    url="$(build_url "$target")"
    tmpdir="$(mktemp -d)"
    archive_path="${tmpdir}/${PROJECT_NAME}.tar.gz"

    cleanup() {
        rm -rf "$tmpdir"
    }
    trap cleanup EXIT INT TERM

    mkdir -p "$INSTALL_DIR"

    echo "Downloading ${PROJECT_NAME} for ${target}..."
    curl --fail --location --silent --show-error "$url" --output "$archive_path"

    tar -xzf "$archive_path" -C "$tmpdir"

    if [ ! -f "${tmpdir}/${BINARY_NAME}" ]; then
        found="$(find "$tmpdir" -type f -name "$BINARY_NAME" | head -n 1 || true)"
        if [ -z "$found" ]; then
            echo "error: could not find '${BINARY_NAME}' in downloaded archive" >&2
            exit 1
        fi
    else
        found="${tmpdir}/${BINARY_NAME}"
    fi

    install -m 0755 "$found" "${INSTALL_DIR}/${BINARY_NAME}"

    echo "Installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
    echo "Run '${BINARY_NAME} --help' to verify the install."
}

main "$@"
