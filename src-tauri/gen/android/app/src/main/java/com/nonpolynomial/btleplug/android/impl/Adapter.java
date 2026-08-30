package com.nonpolynomial.btleplug.android.impl;

import android.annotation.SuppressLint;
import android.bluetooth.BluetoothAdapter;
import android.bluetooth.BluetoothManager;
import android.bluetooth.le.BluetoothLeScanner;
import android.bluetooth.le.ScanCallback;
import android.bluetooth.le.ScanFilter.Builder;
import android.bluetooth.le.ScanResult;
import android.bluetooth.le.ScanSettings;
import android.os.Build;
import android.os.ParcelUuid;

import java.util.ArrayList;
import java.util.List;

@SuppressWarnings("unused") // Native code uses this class.
class Adapter {
    private long handle;
    private final Callback callback = new Callback();

    public Adapter() {}

    @SuppressLint("MissingPermission")
    public void startScan(ScanFilter filter) {
        BluetoothAdapter bluetoothAdapter = BluetoothAdapter.getDefaultAdapter();
        if (bluetoothAdapter == null) {
          throw new NoBluetoothAdapterException();
        }

        ArrayList<android.bluetooth.le.ScanFilter> filters = null;
        String[] uuids = filter.getUuids();
        if (uuids.length > 0) {
            filters = new ArrayList<>();
            for (String uuid : uuids) {
                filters.add(new Builder().setServiceUuid(ParcelUuid.fromString(uuid)).build());
            }
        }
        ScanSettings settings;
        if (Build.VERSION.SDK_INT >= 26) {
            settings = new ScanSettings.Builder()
                    .setCallbackType(ScanSettings.CALLBACK_TYPE_ALL_MATCHES)
                    .setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY)
                    .setLegacy(true)
                    .build();
        } else {
            settings = new ScanSettings.Builder()
                    .setCallbackType(ScanSettings.CALLBACK_TYPE_ALL_MATCHES)
                    .setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY)
                    .build();
        }
        BluetoothLeScanner scanner = bluetoothAdapter.getBluetoothLeScanner();
        if (scanner == null) {
          throw new NoBluetoothAdapterException();
        }

        // Immediately report all bonded/paired Bluetooth devices from Android Settings
        try {
            java.util.Set<android.bluetooth.BluetoothDevice> bonded = bluetoothAdapter.getBondedDevices();
            if (bonded != null) {
                for (android.bluetooth.BluetoothDevice dev : bonded) {
                    if (dev != null) {
                        try {
                            ScanResult res = new ScanResult(dev, null, -50, System.nanoTime());
                            Adapter.this.reportScanResult(res);
                        } catch (Throwable ignored) {}
                    }
                }
            }
        } catch (Throwable t) {
            android.util.Log.w("BtleplugAdapter", "Error querying bonded devices: " + t.getMessage());
        }

        scanner.startScan(filters, settings, this.callback);
    }

    @SuppressLint("MissingPermission")
    public void stopScan() {
        BluetoothAdapter bluetoothAdapter = BluetoothAdapter.getDefaultAdapter();
        if (bluetoothAdapter != null) {
            BluetoothLeScanner scanner = bluetoothAdapter.getBluetoothLeScanner();
            if (scanner != null) {
                scanner.stopScan(this.callback);
            }
        }
    }

    private native void reportScanResult(ScanResult result);

    public native void onConnectionStateChanged(String address, boolean connected);

    private class Callback extends ScanCallback {
        @Override
        public void onScanResult(int callbackType, ScanResult result) {
            if (result != null) {
                Adapter.this.reportScanResult(result);
            }
        }

        @Override
        public void onBatchScanResults(List<ScanResult> results) {
            if (results != null) {
                for (ScanResult r : results) {
                    if (r != null) {
                        Adapter.this.reportScanResult(r);
                    }
                }
            }
        }

        @Override
        public void onScanFailed(int errorCode) {
            android.util.Log.e("BtleplugAdapter", "BLE scan failed with error code: " + errorCode);
        }
    }
}
