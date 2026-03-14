#!/bin/bash
#
# Android Test Environment Verification Script
# 
# This script verifies that all required tools and configurations
# are in place for running Android tests for the Cazzmachine app.
#
# Usage: ./scripts/verify-android-test-env.sh
#
# Exit codes:
#   0 - All checks passed
#   1 - One or more checks failed
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
PASS=0
FAIL=0
WARN=0

# Function to print status
print_status() {
    local status=$1
    local message=$2
    if [ "$status" = "PASS" ]; then
        echo -e "${GREEN}[PASS]${NC} $message"
        ((PASS++))
    elif [ "$status" = "FAIL" ]; then
        echo -e "${RED}[FAIL]${NC} $message"
        ((FAIL++))
    elif [ "$status" = "WARN" ]; then
        echo -e "${YELLOW}[WARN]${NC} $message"
        ((WARN++))
    fi
}

echo "=============================================="
echo "Android Test Environment Verification"
echo "=============================================="
echo ""

# ============================================
# Check 1: Android SDK
# ============================================
echo "--- Checking Android SDK ---"

if [ -n "$ANDROID_HOME" ] && [ -d "$ANDROID_HOME" ]; then
    print_status "PASS" "ANDROID_HOME is set: $ANDROID_HOME"
elif [ -n "$ANDROID_SDK_ROOT" ] && [ -d "$ANDROID_SDK_ROOT" ]; then
    print_status "PASS" "ANDROID_SDK_ROOT is set: $ANDROID_SDK_ROOT"
    export ANDROID_HOME="$ANDROID_SDK_ROOT"
else
    print_status "FAIL" "Android SDK not found. Set ANDROID_HOME or ANDROID_SDK_ROOT"
    echo "       Install Android SDK from: https://developer.android.com/studio#command-line-tools"
fi

# Check adb
if command -v adb &> /dev/null; then
    ADB_VERSION=$(adb version 2>&1 | head -n1)
    print_status "PASS" "adb found: $ADB_VERSION"
else
    print_status "FAIL" "adb not found in PATH"
    echo "       Install Android SDK Platform Tools"
fi

# Check emulator binary
if command -v emulator &> /dev/null; then
    EMULATOR_VERSION=$(emulator -version 2>&1 | head -n1)
    print_status "PASS" "emulator found: $EMULATOR_VERSION"
else
    print_status "WARN" "emulator not found in PATH"
    echo "       Install Android SDK Emulator (optional for E2E tests)"
fi

echo ""

# ============================================
# Check 2: Emulator AVD
# ============================================
echo "--- Checking Emulator AVD ---"

if command -v emulator &> /dev/null; then
    # Check for Pixel_3a_API_24
    if emulator -list-avds 2>/dev/null | grep -q "Pixel_3a_API_24"; then
        print_status "PASS" "Emulator 'Pixel_3a_API_24' exists"
    else
        # List available emulators
        AVAILABLE_AVDS=$(emulator -list-avds 2>/dev/null || echo "")
        if [ -n "$AVAILABLE_AVDS" ]; then
            print_status "WARN" "Pixel_3a_API_24 not found. Available emulators:"
            echo "$AVAILABLE_AVDS" | while read avd; do
                echo "       - $avd"
            done
        else
            print_status "FAIL" "No emulators found. Create one with:"
            echo "       android create avd -n Pixel_3a_API_24 -t android-24"
        fi
    fi
else
    print_status "WARN" "Cannot check emulators (emulator not in PATH)"
fi

echo ""

# ============================================
# Check 3: Rust and Android Targets
# ============================================
echo "--- Checking Rust Android Targets ---"

if command -v rustup &> /dev/null; then
    print_status "PASS" "rustup found"
    
    # Check for Android targets
    ANDROID_TARGETS=$(rustup target list --installed 2>/dev/null | grep android || true)
    if echo "$ANDROID_TARGETS" | grep -q "aarch64"; then
        print_status "PASS" "Android aarch64 target installed"
    else
        print_status "WARN" "Android aarch64 target not installed"
        echo "       Install with: rustup target add aarch64-linux-android"
    fi
    
    if echo "$ANDROID_TARGETS" | grep -q "armv7"; then
        print_status "PASS" "Android armv7 target installed"
    else
        print_status "WARN" "Android armv7 target not installed"
    fi
    
    if echo "$ANDROID_TARGETS" | grep -q "x86_64"; then
        print_status "PASS" "Android x86_64 target installed"
    else
        print_status "WARN" "Android x86_64 target not installed"
    fi
else
    print_status "FAIL" "rustup not found"
    echo "       Install Rust from: https://rustup.rs"
fi

echo ""

# ============================================
# Check 4: Cargo Dependencies
# ============================================
echo "--- Checking Cargo Dependencies ---"

cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"/src-tauri

# Check if Cargo.toml has mockito
if grep -q 'mockito' Cargo.toml; then
    print_status "PASS" "mockito found in Cargo.toml"
else
    print_status "FAIL" "mockito not found in Cargo.toml"
    echo "       Add to [dev-dependencies]: mockito = \"1\""
fi

# Check if cargo test compiles
echo "Checking if cargo test compiles..."
if cargo check --tests 2>/dev/null; then
    print_status "PASS" "cargo test compiles successfully"
else
    print_status "FAIL" "cargo test compilation failed"
    echo "       Run 'cargo check --tests' for details"
fi

cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo ""

# ============================================
# Check 5: Gradle (via npm)
# ============================================
echo "--- Checking Build Tools ---"

# Check for npm/node
if command -v npm &> /dev/null; then
    NPM_VERSION=$(npm --version)
    print_status "PASS" "npm found: v$NPM_VERSION"
else
    print_status "FAIL" "npm not found"
    echo "       Install Node.js from: https://nodejs.org"
fi

# Check for Java
if command -v java &> /dev/null; then
    JAVA_VERSION=$(java -version 2>&1 | head -n1)
    print_status "PASS" "Java found: $JAVA_VERSION"
else
    print_status "FAIL" "Java not found"
    echo "       Install Java JDK 11+ from: https://adoptium.net/"
fi

echo ""

# ============================================
# Summary
# ============================================
echo "=============================================="
echo "Summary"
echo "=============================================="
echo -e "Passed:   ${GREEN}$PASS${NC}"
echo -e "Failed:   ${RED}$FAIL${NC}"
echo -e "Warnings: ${YELLOW}$WARN${NC}"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}Environment verification PASSED${NC}"
    echo "You can proceed with running Android tests."
    exit 0
else
    echo -e "${RED}Environment verification FAILED${NC}"
    echo "Please fix the failed checks before running tests."
    exit 1
fi
