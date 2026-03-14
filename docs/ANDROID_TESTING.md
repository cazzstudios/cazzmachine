# Android Testing Guide

This document describes how to run tests for the Android version of Cazzmachine.

## Prerequisites

Before running tests, verify your environment:

```bash
./scripts/verify-android-test-env.sh
```

Required:
- Android SDK (ANDROID_HOME or ANDROID_SDK_ROOT)
- Rust toolchain with Android targets
- Java JDK 11+
- Node.js and npm

Optional (for E2E tests):
- Android emulator (API 24+)
- adb in PATH

## Running Tests

### 1. Rust Unit Tests

Run Rust tests for the Android modules:

```bash
cd src-tauri
cargo test
```

To run tests for specific Android modules:

```bash
cargo test android::
```

### 2. Android Unit Tests (JVM)

Run JVM-based unit tests:

```bash
cd src-tauri/gen/android
./gradlew test
```

Run specific test class:

```bash
./gradlew test --tests "com.cazzmachine.app.ExampleUnitTest"
```

### 3. Android Instrumentation Tests

Run tests on device/emulator:

```bash
cd src-tauri/gen/android
./gradlew connectedCheck
```

Run specific instrumentation test:

```bash
./gradlew connectedCheck --tests "com.cazzmachine.app.ExampleInstrumentedTest"
```

Note: Requires a running emulator or connected device.

### 4. E2E Tests (Browser-based)

Run end-to-end tests with Playwright (browser automation):

```bash
npm run test:e2e
```

### 5. Emulator E2E Tests

Run end-to-end tests on actual Android emulator:

```bash
npm run test:e2e:emulator
```

This will:
1. Start Android emulator automatically (if not running)
2. Run all emulator-specific tests
3. Stop emulator after tests complete

**Test files:**
- `e2e/emulator.spec.ts` - Core emulator tests
- `e2e/androidBackButtonFix.spec.ts` - Back button verification

**What the tests verify:**
- APK is installed on emulator
- App launches successfully
- Manifest contains `android:enableOnBackInvokedCallback="true"`
- Navigation works (idle <-> detail view)
- Back button does not kill the app (no WIN DEATH in logcat)

> **Note**: The app uses `OnBackPressedCallback` (registered in `MainActivity.kt`) for
> back button handling. On Android 13+, `onKeyDown(KEYCODE_BACK)` is not called for back
> actions — the system routes them through `OnBackPressedDispatcher` instead. Look for
> `[BackButton]` logs in logcat to verify the JS callback is invoked.

### 6. Release APK E2E Tests

**IMPORTANT:** Debug builds and release builds behave differently due to ProGuard/R8 code stripping. Always test with a signed release APK before publishing.

Run E2E tests against your actual signed release APK:

```bash
# Build and test (uses existing signed APK or builds debug APK)
npm run test:e2e:release

# Test with specific signed APK
TEST_APK_PATH=./cazzmachine-signed.apk npm run test:e2e:release
```

**Test files:**
- `e2e/release/healthCheck.spec.ts` - Quick diagnostics
- `e2e/release/backButton.spec.ts` - All release APK behavioral tests (unified)

**Test approach:**
- Uses ADB-based UI verification (no WebView debugging required)
- Visual regression testing with Python PIL for screenshot analysis
- Detects UI elements via uiautomator dumps
- Color-based detection for notifications and doomscrolling state

**What the tests verify:**
- APK installs and launches without crashes
- `tauri-plugin-app-events` is properly initialized
- Back button from idle view exits the app
- Back button from detail/summary navigates to idle
- Doomscrolling stops when app goes to background (visual diff)
- Notification appears on resume after >60 seconds elapsed
- No notification if elapsed <60 seconds
- App survives background/foreground transitions
- No ProGuard/R8 stripping issues

**Why this matters:**
- Debug builds (~660MB) skip ProGuard optimizations
- Release builds (~80MB) with `isMinifyEnabled=true` may strip plugin classes
- Plugins like `tauri-plugin-app-events` can fail silently in release

**Visual Regression Testing:**
The release tests use Python PIL for screenshot analysis:

```python
# Detects doomscrolling state via color analysis
# Checks for accent colors in status area

# Detects notification toast
# Looks for orange [SYSTEM_NOTIFICATION] label in top-right
```

This approach works because:
- WebView content isn't accessible via uiautomator in release builds
- Screenshot analysis bypasses the WebView accessibility limitation
- Color-based detection is robust against minor UI variations

**Expected Test Results (Release APK):**

| Test | Expected | Known Issues |
|------|----------|--------------|
| Back button: detail→idle | PASS | Verified via logcat `[BackButton] view: detail intercept: true` |
| Back button: summary→idle | PASS | Verified via logcat `[BackButton] view: summary intercept: true` |
| Back button: idle exits | PASS | Verified on Samsung Galaxy (Android 13+) |
| Doomscrolling stops on background | PASS | Visual diff detects UI change |
| Notification after >60s | FAIL | Notification not appearing (bug) |
| No notification <60s | PASS | |
| App survives transitions | PASS | |

**ProGuard Configuration:**
The file `src-tauri/gen/android/app/proguard-rules.pro` contains rules to prevent stripping of critical plugin classes. If release tests fail, verify these rules are up to date:

```proguard
# Keep Tauri Plugin App Events
-keep class wang.tato.tauri_plugin_app_events.** { *; }
```

**Manual emulator control:**

```bash
# List available emulators
$ANDROID_HOME/emulator/emulator -list-avds

# Start emulator manually
$ANDROID_HOME/emulator/emulator -avd Pixel_3a_API_34_extension_level_7_x86_64 -no-audio -no-window -no-boot-anim &

# Check emulator status
adb devices
```

**Why emulator testing is required:**

Some features can only be tested on a real Android device/emulator:

1. **Back button handling** - Uses `OnBackPressedCallback`; must verify JS callback is invoked via logcat
2. **Hardware buttons** - Browser automation cannot simulate physical back button
3. **Manifest changes** - `android:enableOnBackInvokedCallback` must be verified on device
4. **Native integrations** - Background services, notifications, permissions

**ADB commands for manual testing:**

```bash
# Install APK
adb install -r src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk

# Launch app
adb shell am start -n com.cazzmachine.app/.MainActivity

# Simulate taps (coordinates)
adb shell input tap 540 1900

# Press back button
adb shell input keyevent 4

# View logs
adb logcat -d | grep -E "\[BackButton\]|cazzmachine"
```

## Test Structure

```
src-tauri/
├── src/android/
│   ├── background_service.rs    # Rust background service
│   ├── mod.rs                  # Android module commands
│   └── test_utils.rs           # Test utilities and mocks
├── tests/
│   ├── db_tests.rs             # Database integration tests
│   └── crawler_tests.rs        # Crawler tests with mocks
└── Cargo.toml

src-tauri/gen/android/app/src/
├── main/java/com/cazzmachine/app/
│   ├── MainActivity.kt         # Main activity
│   └── CrawlService.kt         # Background service
├── test/                       # Unit tests (JVM)
│   └── java/com/cazzmachine/app/
│       └── ExampleUnitTest.kt
└── androidTest/                # Instrumentation tests (device)
    └── java/com/cazzmachine/app/
        ├── ExampleInstrumentedTest.kt
        └── TestHelpers.kt
```

## Test Utilities

### Release Test Helpers (e2e/release/backButton.spec.ts)

The release tests use these helper functions:
- `pressBack()` - Send back button keyevent via ADB
- `pressHome()` - Send home button keyevent via ADB
- `launchApp()` - Launch via `adb shell am start`
- `clearLogcat()` - Clear logcat buffer before test actions
- `getLogcatLines(filter)` - Get logcat lines matching a pattern
- `waitForLogcatPattern(pattern, timeout)` - Poll logcat until pattern appears
- `dismissSplashIfPresent()` - Tap BEGIN_DOOMSCROLLING to dismiss first-run splash
- `findElementByText()` - Find UI elements via uiautomator dump
- `takeScreenshot()` - Capture screen via ADB
- `visualDiff()` - Compare screenshots using Python PIL
- `hasDoomscrollingInScreenshot()` - Color-based doomscrolling detection
- `checkResumeNotificationContent()` - Detect notification toast in screenshot

### Python PIL Requirements

Visual regression tests require Python with PIL:

```bash
pip install Pillow numpy
```

The tests use:
- `PIL.Image` for screenshot loading
- `numpy` for pixel array manipulation
- Color thresholding to detect UI states

## Troubleshooting

### Tests fail to compile

Ensure all dependencies are installed:
```bash
cd src-tauri && cargo fetch
cd src-tauri/gen/android && ./gradlew dependencies
```

### Instrumentation tests won't start

Check that emulator is running:
```bash
adb devices
```

If no emulator:
```bash
# List available emulators
emulator -list-avds

# Start emulator
emulator -avd Pixel_3a_API_24
```

### Rust tests hang

Check for deadlocks in database operations. Use:
```bash
cargo test -- --nocapture
```

### Logcat viewing

View app logs:
```bash
adb logcat | grep -i cazzmachine
```

View Rust logs:
```bash
adb logcat -s RustStdoutStderr:D
```

## CI/CD (Future)

Currently, tests are designed to run locally. CI/CD integration can be added later with:
- GitHub Actions for automated testing
- Emulator farm for device testing
- Test report aggregation

---

## Script Reference

### Environment Verification

Check if your environment is ready for testing:

```bash
./scripts/verify-android-test-env.sh
```

Checks:
- Android SDK installation
- Emulator availability
- Rust Android targets
- Required dependencies

### Build Helper

Build APKs with proper configuration:

```bash
./scripts/fix-android-tests.sh
```

This script:
1. Builds debug APK with `npm run tauri android build -- --debug`
2. Builds test APK without triggering Rust rebuild
3. Works around Tauri CLI WebSocket issues

### Run Android Tests

Execute instrumentation tests directly:

```bash
./scripts/run-android-tests.sh
```

This script:
1. Starts emulator if not running
2. Installs main APK and test APK
3. Runs tests via `adb shell am instrument`
4. Displays results

**Note**: Bypasses Gradle/Tauri CLI to avoid WebSocket connection errors.

### E2E Tests

Full end-to-end testing:

```bash
./scripts/test-android-e2e.sh
```

Performs:
1. APK installation
2. App launch via MainActivity
3. Log assertions (Rust runtime, CrawlService)
4. Screenshot capture
5. Report generation

---

## Test Architecture

### Test Pyramid

```
        ┌─────────────┐
        │   E2E Tests │  (scripts/test-android-e2e.sh)
        │  Full Stack │  - APK install
        └──────┬──────┘  - App launch
               │         - Log verification
       ┌───────┴───────┐
       │  Integration  │  (androidTest/)
       │    Tests      │  - Service lifecycle
       └───────┬───────┘  - Notification tests
               │
    ┌──────────┴──────────┐
    │    Rust Unit Tests  │  (tests/)
    │   + JVM Unit Tests  │  - Database tests
    └─────────────────────┘  - Crawler tests
```

### Key Components

| Component | Type | Purpose |
|-----------|------|---------|
| `CrawlService` | Android Service | Foreground service wrapper |
| `AndroidBackgroundService` | Rust | Background crawl logic |
| `MockServer` | Rust test util | HTTP mocking for providers |
| `CrawlServiceTestHelper` | Kotlin test util | Service lifecycle testing |

---

## Known Issues & Limitations

### Tauri CLI WebSocket Error

**Error**: `failed to build WebSocket client, Connection refused`

**Cause**: Tauri CLI tries to connect to a WebSocket server for live-reload that doesn't exist in non-interactive environments.

**Solution**: Use scripts that bypass Gradle's Rust build tasks:
```bash
./scripts/run-android-tests.sh
# or
./gradlew connectedCheck -x app:rustBuildArm64Debug
```

### ServiceTestRule Limitation

**Issue**: `ServiceTestRule` doesn't work with started services (only bound services).

**Workaround**: Services tested with manual lifecycle management and `Thread.sleep()` calls.

### Notification Timing

**Issue**: Foreground service notifications may not be immediately detectable in tests.

**Solution**: Tests use `Thread.sleep(500)` to allow service state changes. In production, this isn't an issue.

### ProGuard/R8 Plugin Stripping

**Issue**: Back button and lifecycle events work in debug builds but fail in release builds.

**Cause**: ProGuard/R8 removes plugin classes that appear unused during static analysis.

**Symptoms**:
- Back button doesn't navigate in release APK
- App doesn't handle suspend/resume correctly
- Tests fail with "tauri-plugin-app-events not initialized"

**Solution**: Ensure `src-tauri/gen/android/app/proguard-rules.pro` contains:
```proguard
# Keep Tauri Plugin App Events (uses wang.tato package, not app.tauri)
-keep class wang.tato.tauri_plugin_app_events.** { *; }
-keep class app.tauri.** { *; }
-keep class app.tauri.plugin.** { *; }
```

**Verification**: Run `npm run test:e2e:release` with your signed APK before publishing.

### Known Test Failures (Release APK)

The following test is expected to fail in the current release build:
| Test | Issue |
|------|-------|
| Notification after >60s | Resume notification doesn't appear |
**Fixed (2026-02-24):** Back button from idle now correctly exits the app. All three back button scenarios (detail→idle, summary→idle, idle→exit) are verified working via `OnBackPressedCallback`.
### Emulator Requirements

- **Architecture**: x86_64 emulator recommended for speed
- **API Level**: Minimum API 24 (Android 7.0)
- **RAM**: At least 2GB allocated
- **Play Store**: Not required

---

## Writing New Tests

### Rust Tests

Add tests to `src-tauri/tests/android_tests.rs`:

```rust
#[test]
fn test_your_feature() {
    let (db, _temp_dir) = create_test_db();
    // Your test code
    assert!(condition);
}
```

### Android Instrumentation Tests

Add tests to `src-tauri/gen/android/app/src/androidTest/...`:

```kotlin
@Test
fun testYourFeature() {
    val serviceHelper = CrawlServiceTestHelper(context)
    serviceHelper.startService()
    Thread.sleep(500)
    
    // Your assertions
    assertTrue("Condition", someCondition)
    
    serviceHelper.cleanup()
}
```

### Test Data

- Use `tempfile::TempDir` for Rust database tests
- Tests run in isolated contexts
- Mock HTTP servers run on random ports

---

## Debug Tips

### View All Logs

```bash
adb logcat | grep -E "(CrawlService|RustStdoutStderr|cazzmachine)"
```

### Check Service Status

```bash
adb shell dumpsys activity services | grep CrawlService
```

### Check Notifications

```bash
adb shell dumpsys notification | grep cazzmachine
```

### Screenshot Capture

```bash
adb shell screencap -p /sdcard/screen.png
adb pull /sdcard/screen.png .
```

---

## Quick Reference Card

| Task | Command |
|------|---------|
| Check environment | `./scripts/verify-android-test-env.sh` |
| Build APKs | `./scripts/fix-android-tests.sh` |
| Run Rust tests | `cd src-tauri && cargo test` |
| Run Android tests | `./scripts/run-android-tests.sh` |
| Run E2E tests | `./scripts/test-android-e2e.sh` |
| **Test release APK** | `npm run test:e2e:release` |
| View logs | `adb logcat -s RustStdoutStderr:D` |
| Start emulator | `emulator -avd Pixel_3a_API_34` |
| Install APK | `adb install -r app.apk` |
