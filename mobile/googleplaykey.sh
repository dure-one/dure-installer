#!/usr/bin/env bash
# googleplaykey.sh - Generate Google Play signing keys for Dure
# Usage: ./googleplaykey.sh /path/to/encryption_public_key.pem [password]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$SCRIPT_DIR/app"
ENCRYPTION_KEY="${1:-}"
PASSWORD="${2:-}"

# App configuration - extracted from build.gradle
APP_ID="app.dure.installer"
APP_NAME_SAFE="${APP_ID//./-}"  # app-dure-installer

# Key files
UPLOAD_KEYSTORE="$APP_DIR/release.keystore"
APP_SIGNING_KEYSTORE="$APP_DIR/app-dure-installer-signing.keystore"
UPLOAD_CERT="$APP_DIR/upload_certificate.pem"
PEPK_JAR="$APP_DIR/pepk.jar"
APP_SIGNING_ZIP="$APP_DIR/app-dure-installer-signing-play-generated.zip"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
error() {
    echo -e "${RED}❌ Error: $1${NC}" >&2
    exit 1
}

warn() {
    echo -e "${YELLOW}⚠️  Warning: $1${NC}"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

info() {
    echo -e "ℹ️  $1"
}

# Validation
if [[ -z "$ENCRYPTION_KEY" ]]; then
    error "Usage: $0 /path/to/encryption_public_key.pem [password]"
fi

if [[ ! -f "$ENCRYPTION_KEY" ]]; then
    error "encryption_public_key.pem not found: $ENCRYPTION_KEY"
fi

# Check if keys already exist
if [[ -f "$UPLOAD_KEYSTORE" ]]; then
    error "Upload keystore already exists: $UPLOAD_KEYSTORE (aborting to prevent overwrite)"
fi

if [[ -f "$UPLOAD_CERT" ]]; then
    error "Upload certificate already exists: $UPLOAD_CERT (aborting to prevent overwrite)"
fi

# Prompt for password if not provided
if [[ -z "$PASSWORD" ]]; then
    info "Generating keys for: $APP_ID"
    read -sp "Enter keystore password (used for both upload and app signing): " PASSWORD
    echo
    if [[ -z "$PASSWORD" ]]; then
        error "Password cannot be empty"
    fi
fi

# Find keytool
if command -v keytool &> /dev/null; then
    KEYTOOL=$(command -v keytool)
elif [[ -x ~/.local/jdk-24.0.1/bin/keytool ]]; then
    KEYTOOL=~/.local/jdk-24.0.1/bin/keytool
else
    error "keytool not found. Install JDK or set JAVA_HOME"
fi

# Ensure app directory exists
mkdir -p "$APP_DIR"
cd "$APP_DIR"

# Step 1: Generate upload keystore
info "Step 1/5: Generating upload keystore..."
"$KEYTOOL" -genkey -v \
    -keystore release.keystore \
    -keyalg RSA -keysize 2048 -validity 10000 \
    -alias upload \
    -storepass "$PASSWORD" \
    -keypass "$PASSWORD" \
    -dname "CN=Dure Installer Upload, OU=Dure, O=Dure, L=Seoul, ST=Seoul, C=KR" \
    2>&1 | grep -E "Generating|Storing"
success "Upload keystore created: release.keystore"

# Step 2: Generate app signing keystore
info "Step 2/5: Generating app signing keystore..."
"$KEYTOOL" -genkey -v \
    -keystore app-dure-installer-signing.keystore \
    -keyalg RSA -keysize 2048 -validity 10000 \
    -alias app-signing \
    -storepass "$PASSWORD" \
    -keypass "$PASSWORD" \
    -dname "CN=Dure Installer App Signing, OU=Dure, O=Dure, L=Seoul, ST=Seoul, C=KR" \
    2>&1 | grep -E "Generating|Storing"
success "App signing keystore created: app-dure-installer-signing.keystore"

# Step 3: Export upload certificate
info "Step 3/5: Exporting upload certificate..."
"$KEYTOOL" -export -rfc \
    -keystore release.keystore \
    -alias upload \
    -file upload_certificate.pem \
    -storepass "$PASSWORD"
success "Upload certificate exported: upload_certificate.pem"

# Step 4: Download pepk.jar
info "Step 4/5: Downloading pepk.jar..."
if [[ ! -f pepk.jar ]]; then
    if wget -q https://www.gstatic.com/play-apps-publisher-rapid/signing-tool/prod/pepk.jar; then
        success "pepk.jar downloaded"
    else
        error "Failed to download pepk.jar"
    fi
else
    info "pepk.jar already exists"
fi

# Step 5: Encrypt app signing keystore with pepk
info "Step 5/5: Encrypting app signing keystore (requires password input)..."
info "When prompted, enter keystore password: $PASSWORD"

# Try to run pepk.jar interactively with script/expect
if command -v script &> /dev/null; then
    # Use script to allocate pseudo-terminal
    script -qec "java -jar pepk.jar \
        --keystore=app-dure-installer-signing.keystore \
        --alias=app-signing \
        --output=app-dure-installer-signing-play-generated.zip \
        --include-cert \
        --rsa-aes-encryption \
        --encryption-key-path=encryption_public_key.pem" /dev/null << EOF
$PASSWORD
$PASSWORD
EOF
    if [[ $? -eq 0 ]] && [[ -f "$APP_SIGNING_ZIP" ]]; then
        success "App signing key encrypted: app-dure-installer-signing-play-generated.zip"
    else
        warn "pepk.jar may require manual input. Run:"
        echo "  java -jar pepk.jar \\"
        echo "    --keystore=app-dure-installer-signing.keystore \\"
        echo "    --alias=app-signing \\"
        echo "    --output=app-dure-installer-signing-play-generated.zip \\"
        echo "    --include-cert --rsa-aes-encryption \\"
        echo "    --encryption-key-path=encryption_public_key.pem"
    fi
else
    warn "pepk.jar requires manual interactive input. Run:"
    echo "  java -jar pepk.jar \\"
    echo "    --keystore=app-dure-installer-signing.keystore \\"
    echo "    --alias=app-signing \\"
    echo "    --output=app-dure-installer-signing-play-generated.zip \\"
    echo "    --include-cert --rsa-aes-encryption \\"
    echo "    --encryption-key-path=encryption_public_key.pem"
    echo "Password: $PASSWORD"
fi

# Summary
echo ""
echo "=========================================="
success "Key generation complete!"
echo "=========================================="
echo ""
info "Files created in: $APP_DIR"
echo "  ✓ release.keystore (upload key - shared)"
echo "  ✓ app-dure-installer-signing.keystore (app signing key)"
echo "  ✓ upload_certificate.pem (for resetting upload key)"
echo "  ✓ app-dure-installer-signing-play-generated.zip (for app signing)"
echo ""
info "Next steps:"
echo "  1. Upload upload_certificate.pem to Play Console > App Integrity > Change Signing Key"
echo "  2. Upload app-dure-installer-signing-play-generated.zip to Play Console > App Integrity > App Signing"
echo ""
info "Store keystore password safely: $PASSWORD"
echo ""
