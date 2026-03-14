#!/bin/bash
# Patch AndroidManifest.xml to add android:enableOnBackInvokedCallback="true"
# This is needed for Android 13+ back button support

set -e

MANIFEST_PATH="src-tauri/gen/android/app/src/main/AndroidManifest.xml"

# Check if manifest exists
if [ ! -f "$MANIFEST_PATH" ]; then
    echo "ERROR: AndroidManifest.xml not found at $MANIFEST_PATH"
    echo "Please run 'npm run tauri android build' first to generate the manifest."
    exit 1
fi

# Check if the fix is already applied
if grep -q 'android:enableOnBackInvokedCallback="true"' "$MANIFEST_PATH"; then
    echo "SUCCESS: android:enableOnBackInvokedCallback is already present in AndroidManifest.xml"
    exit 0
fi

# Add the attribute to the <application> tag (handles multi-line tags)
# Find the line with <application and add the attribute before the closing >
sed -i '/<application/,/>/ {
    s/android:usesCleartextTraffic="\${usesCleartextTraffic}">/android:usesCleartextTraffic="\${usesCleartextTraffic}" android:enableOnBackInvokedCallback="true">/
}' "$MANIFEST_PATH"

# Alternative: if the above doesn't work, try matching the end of the application tag
if ! grep -q 'android:enableOnBackInvokedCallback="true"' "$MANIFEST_PATH"; then
    sed -i 's/<application$/<application android:enableOnBackInvokedCallback="true"/' "$MANIFEST_PATH"
fi

# Verify the change was applied
if grep -q 'android:enableOnBackInvokedCallback="true"' "$MANIFEST_PATH"; then
    echo "SUCCESS: Added android:enableOnBackInvokedCallback=\"true\" to AndroidManifest.xml"
    exit 0
else
    echo "ERROR: Failed to add android:enableOnBackInvokedCallback attribute"
    exit 1
fi
