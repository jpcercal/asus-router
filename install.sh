#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

# =================================
# Logging Functions
# =================================
log() {
  local level="$1" msg="$2"
  printf "[%s] [%-7s] %s\n" "$(date '+%Y-%m-%d %H:%M:%S')" "$level" "$msg"
}
info()    { log "INFO"    "$*"; }
warn()    { log "WARN"    "$*"; }
success() { log "SUCCESS" "$*"; }
error()   { log "ERROR"   "$*"; }

# Trap unexpected errors
trap 'error "Unexpected error on line $LINENO. Exiting."; exit 1' ERR

# =================================
# Configuration
# =================================
REPO="jpcercal/asus-router"
API_URL="https://api.github.com/repos/${REPO}/releases/latest"
TMP_JSON="/tmp/${REPO//\//_}_releases_assets.json"
ARCHIVE_NAME="asus-router.aarch64.tar.gz"
BINARY_NAME="asus-router"
TMP_BIN_DIR="/tmp/opt/bin"

# Ensure temp and bin directories exist
info "Creating necessary directories"
mkdir -p "$(dirname "${TMP_JSON}")" "${TMP_BIN_DIR}"
success "Directories ready"

# =================================
# Fetch Release Metadata
# =================================
info "Fetching release metadata from ${API_URL}"
wget -q -O "${TMP_JSON}" "${API_URL}"
success "Metadata saved to ${TMP_JSON}"

# Helper: extract download URL for asset by name
get_url() {
  local name="$1"
  jq -r --arg name "$name" '.assets[] | select(.name == $name).browser_download_url' "${TMP_JSON}"
}

# =================================
# Download Archive
# =================================
info "Downloading ${ARCHIVE_NAME}"
ARCHIVE_URL="$(get_url "${ARCHIVE_NAME}")"
wget -q "${ARCHIVE_URL}" -O "${ARCHIVE_NAME}"
success "Downloaded ${ARCHIVE_NAME} from ${ARCHIVE_URL}"

# =================================
# Checksum Verification Function
# =================================
verify_checksum() {
  local file="$1" url="$2" name="$3"
  info "Verifying checksum for ${name}"
  local expected actual
  expected="$(wget -q -O - "${url}" | awk '{print $1}')"
  actual="$(md5sum "${file}" | awk '{print $1}')"
  if [[ "${actual}" == "${expected}" ]]; then
    success "Checksum OK for ${name}"
  else
    error "Checksum mismatch for ${name}! Expected ${expected}, got ${actual}"
    exit 1
  fi
}

# Verify archive checksum
verify_checksum "${ARCHIVE_NAME}" "$(get_url "${ARCHIVE_NAME}.md5sum.txt")" "${ARCHIVE_NAME}"

# =================================
# Extract Archive
# =================================
info "Extracting ${ARCHIVE_NAME}"
tar -xzf "${ARCHIVE_NAME}"
success "Extracted to current directory"

# =================================
# Verify Binary Checksum
# =================================
verify_checksum "${BINARY_NAME}" "$(get_url "${BINARY_NAME}.md5sum.txt")" "${BINARY_NAME}"

# =================================
# Install Binary
# =================================
info "Moving ${BINARY_NAME} to ${TMP_BIN_DIR}"
mv "${BINARY_NAME}" "${TMP_BIN_DIR}/"
success "Moved ${BINARY_NAME} to ${TMP_BIN_DIR}"

# =================================
# Cleanup
# =================================
info "Cleaning up temporary files"
rm -f "${ARCHIVE_NAME}" "${TMP_JSON}"
success "Temporary files removed"

# =================================
# Completion
# =================================
success "All steps completed successfully!"
