package com.cazzmachine.app

import android.content.Context
import android.content.SharedPreferences
import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import android.widget.Toast
import android.view.KeyEvent
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback
import androidx.activity.result.contract.ActivityResultContracts

class MainActivity : TauriActivity() {
  private lateinit var wv: WebView

  // SharedPreferences key for background timestamp
  companion object {
    const val PREFS_NAME = "cazzmachine_prefs"
    const val KEY_BACKGROUND_TIMESTAMP = "background_timestamp_ms"
  }

  // Disable WryActivity's default webview-history-based back handler.
  // We install our own OnBackPressedCallback that routes through JS.
  override val handleBackNavigation: Boolean = false

  private val requestPermissionLauncher = registerForActivityResult(
      ActivityResultContracts.RequestPermission()
  ) { isGranted: Boolean ->
      if (!isGranted) {
          Toast.makeText(this, "Notification permission required for background crawling", Toast.LENGTH_LONG).show()
      }
  }

  override fun onWebViewCreate(webView: WebView) {
    wv = webView

    // Add JavaScript interface to read background timestamp from SharedPreferences
    // This is a fallback in case localStorage is cleared when WebView restarts
    wv.addJavascriptInterface(object {
      @android.webkit.JavascriptInterface
      fun getBackgroundTimestamp(): Long {
        val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        return prefs.getLong(KEY_BACKGROUND_TIMESTAMP, -1)
      }
    }, "AndroidPrefs")

    // Register an OnBackPressedCallback so that Android 13+ back gestures
    // and ADB "input keyevent 4" both route through our JS handler.
    // onKeyDown(KEYCODE_BACK) is NOT called on Android 13+ for back actions;
    // the OnBackPressedDispatcher is used instead.
    val callback = object : OnBackPressedCallback(true) {
      override fun handleOnBackPressed() {
        wv.evaluateJavascript(
          """
            try {
              window.__tauri_android_on_back_key_down__()
            } catch (_) {
              true
            }
          """.trimIndent()
        ) { result ->
          if (result != "false") {
            // JS did not intercept - allow default (exit app)
            this.isEnabled = false
            onBackPressedDispatcher.onBackPressed()
            this.isEnabled = true
          }
        }
      }
    }
    onBackPressedDispatcher.addCallback(this, callback)
  }

  // Map for non-back key events (menu, search, volume).
  // Back is handled by OnBackPressedCallback above.
  private val keyEventMap = mapOf(
    KeyEvent.KEYCODE_MENU to "menu",
    KeyEvent.KEYCODE_SEARCH to "search",
    KeyEvent.KEYCODE_VOLUME_DOWN to "volume_down",
    KeyEvent.KEYCODE_VOLUME_UP to "volume_up"
  )

  override fun onKeyDown(keyCode: Int, event: KeyEvent?): Boolean {
    val jsCallbackName = keyEventMap[keyCode] ?: return super.onKeyDown(keyCode, event)
    wv.evaluateJavascript(
      """
        try {
          window.__tauri_android_on_${jsCallbackName}_key_down__()
        } catch (_) {
          true
        }
      """.trimIndent()
    ) { result ->
      if (result != "false") {
        super.onKeyDown(keyCode, event)
      }
    }
    return true
  }

  override fun onPause() {
    super.onPause()
    
    // Write timestamp synchronously to both SharedPreferences and localStorage
    // This ensures the timestamp is available when the app resumes.
    // SharedPreferences persists across WebView restarts.
    // localStorage is read by JS onResume for immediate access.
    val timestamp = System.currentTimeMillis()
    val prefs = getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    prefs.edit().putLong(KEY_BACKGROUND_TIMESTAMP, timestamp).apply()
    
    // Also write to localStorage via JS bridge for immediate access on resume
    wv.evaluateJavascript(
      "localStorage.setItem('backgroundedAtMs', '$timestamp');"
    ) { }
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        if (checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
            requestPermissionLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
        }
    }

  }
}
