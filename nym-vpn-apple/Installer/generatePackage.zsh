#!/bin/zsh

set -x

cd "${BUILT_PRODUCTS_DIR}"

PATH="$PATH:/opt/homebrew/bin"

STAGING_DIRECTORY="${TMPDIR}/staging"
APP_NAME=${PROJECT_NAME}.app
IDENTIFIER="${PRODUCT_BUNDLE_IDENTIFIER}"
VERSION="${MARKETING_VERSION}"

DIST_XML_FILE="Distribution"
NESTED_PKG_PATH="$IDENTIFIER.pkg"
SCRIPTS_DIR="${SCRIPT_INPUT_FILE_1}"
PKG_OUT_DIR="${SCRIPT_OUTPUT_FILE_0}"

# Set up a staging directory with the contents to install.
mkdir -p "${STAGING_DIRECTORY}"
cp -r "${APP_NAME}" "${STAGING_DIRECTORY}"

# Generate the component property list.
pkgbuild --analyze --root "${STAGING_DIRECTORY}" component.plist

# Force the installation package (.pkg) to not be relocatable.
# This ensures the package components install in /Applications
plutil -replace "0.BundleIsRelocatable" -bool NO component.plist

# Allow downgrades
plutil -replace "0.BundleIsVersionChecked" -bool NO component.plist

# Build a temporary package using the component property list.
pkgbuild --root "${STAGING_DIRECTORY}" \
    --identifier "${IDENTIFIER}" \
    --version "${VERSION}" \
    --install-location "/Applications" \
    --scripts "${SCRIPTS_DIR}" \
    --component-plist component.plist \
    "$NESTED_PKG_PATH"

# Synthesize the distribution for the temporary package.
productbuild --synthesize --package "$NESTED_PKG_PATH" "$DIST_XML_FILE"

# Customize installer:
# - enable_anywhere=false - restrict installation to system volume only, i.e no installs on external drives
# - enable_currentUserHome=false - prevent installation into home dir
# - enable_localSystem=true - enable installation into root
xmlstarlet edit --inplace \
    --subnode '//installer-gui-script' --type elem -n 'domains' \
    --append '//installer-gui-script/domains' --type attr -n 'enable_anywhere' --value 'false' \
    --append '//installer-gui-script/domains' --type attr -n 'enable_currentUserHome' --value 'false' \
    --append '//installer-gui-script/domains' --type attr -n 'enable_localSystem' --value 'true' \
    "$DIST_XML_FILE"

# Synthesize the final package from the distribution.
# todo: add --sign --timestamp for signing
BUILD_ARGS=""
if [ "$CONFIGURATION" == "Release" ]; then
  BUILD_ARGS="--sign --timestamp"
fi
productbuild --distribution Distribution --package-path "${BUILT_PRODUCTS_DIR}" "${PKG_OUT_DIR}" $BUILD_ARGS

# Remove original pkg produced by pkgbuild
rm "$NESTED_PKG_PATH" || true
