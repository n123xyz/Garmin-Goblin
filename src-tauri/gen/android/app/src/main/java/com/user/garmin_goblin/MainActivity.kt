package com.user.garmin_goblin

import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  private external fun initBtleplug()

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)

    android.webkit.WebView.setWebContentsDebuggingEnabled(true)

    requestBluetoothPermissions()

    try {
      initBtleplug()
    } catch (e: Throwable) {
      android.util.Log.w("MainActivity", "btleplug init: ${e.message}")
    }

    val insetsController = WindowCompat.getInsetsController(window, window.decorView)
    insetsController.isAppearanceLightStatusBars = true

    ViewCompat.setOnApplyWindowInsetsListener(window.decorView) { v, insets ->
        val systemBars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
        v.setPadding(systemBars.left, systemBars.top, systemBars.right, systemBars.bottom)
        insets
    }
  }

  private fun requestBluetoothPermissions() {
    val permissions = mutableListOf<String>()
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
      if (ContextCompat.checkSelfPermission(this, Manifest.permission.BLUETOOTH_SCAN) != PackageManager.PERMISSION_GRANTED) {
        permissions.add(Manifest.permission.BLUETOOTH_SCAN)
      }
      if (ContextCompat.checkSelfPermission(this, Manifest.permission.BLUETOOTH_CONNECT) != PackageManager.PERMISSION_GRANTED) {
        permissions.add(Manifest.permission.BLUETOOTH_CONNECT)
      }
    }
    if (ContextCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION) != PackageManager.PERMISSION_GRANTED) {
      permissions.add(Manifest.permission.ACCESS_FINE_LOCATION)
    }
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      if (ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
        permissions.add(Manifest.permission.POST_NOTIFICATIONS)
      }
    }

    if (permissions.isNotEmpty()) {
      ActivityCompat.requestPermissions(this, permissions.toTypedArray(), 1001)
    }
  }
}
