package com.plugin.litert

import android.app.Activity
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.ActivityResult
import com.google.ai.edge.litertlm.*
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File
import java.io.FileOutputStream
import java.net.URL
import android.content.Intent
import android.content.IntentFilter
import android.content.BroadcastReceiver
import android.net.Uri
import android.content.Context
import android.content.pm.PackageManager
import android.bluetooth.BluetoothManager
import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothGatt
import android.bluetooth.BluetoothGattCallback
import android.bluetooth.BluetoothGattCharacteristic
import android.bluetooth.BluetoothProfile
import android.bluetooth.le.BluetoothLeScanner
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanResult
import android.bluetooth.le.ScanSettings
import android.bluetooth.BluetoothGattDescriptor
import android.util.Base64
import android.os.Build
import android.provider.CalendarContract
import android.content.ContentUris
import android.speech.SpeechRecognizer
import android.speech.RecognizerIntent
import android.speech.RecognitionListener
import android.os.Bundle
import app.tauri.plugin.JSArray
import java.io.ByteArrayOutputStream
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.Collections
import java.util.LinkedHashMap
import java.util.UUID
import android.hardware.usb.UsbManager
import android.hardware.usb.UsbDevice
import android.hardware.usb.UsbDeviceConnection
import android.hardware.usb.UsbConstants
import android.mtp.MtpDevice
import android.mtp.MtpObjectInfo
import android.mtp.MtpConstants
import android.app.PendingIntent
import java.security.MessageDigest

@InvokeArg
class SyncUsbMtpArgs {
  var forcePullAll: Boolean? = false
  var existingHashes: List<String>? = null
}

@InvokeArg
class GetCalendarEventsArgs {
  var startTimeEpochMs: Long = 0L
  var endTimeEpochMs: Long = 0L
}

@InvokeArg
class SyncBleDeviceArgs {
  var macAddress: String = ""
  var forcePullAll: Boolean? = false
}

@InvokeArg
class InitModelArgs {
  var modelPath: String = ""
  var accelerator: String = "Auto"
  var maxTokens: Int = 5000
}

@InvokeArg
class CheckModelArgs {
  var modelPath: String = ""
}

@InvokeArg
class DownloadModelArgs {
  var modelPath: String = ""
  var token: String? = null
  var downloadUrl: String? = null
}

@InvokeArg
class PurgeModelArgs {
  var modelPath: String = ""
}

@InvokeArg
class GenerateChatArgs {
  var prompt: String = ""
  var reset: Boolean? = false
  var audioBase64: String? = null
  var imageUri: String? = null
  var systemInstruction: String? = null
}

@TauriPlugin
class LitertPlugin(private val activity: Activity): Plugin(activity) {
    companion object {
        init {
            try {
                System.loadLibrary("LiteRt")
                Log.i("LitertPlugin", "Loaded libLiteRt.so into global namespace to fix OpenCL sampler linker issue")
            } catch (e: UnsatisfiedLinkError) {
                Log.w("LitertPlugin", "Could not pre-load libLiteRt.so, namespace isolation fix may not apply", e)
            }
        }
    }

    private var engine: Engine? = null
    private var conversation: Conversation? = null
    private var conversationConfig: ConversationConfig? = null
    private var currentLoadedModelPath: String? = null
    private var currentLoadedAccelerator: String? = null
    private val scope = CoroutineScope(Dispatchers.IO)

    @Synchronized
    private fun safeCloseEngine() {
        try {
            conversation?.close()
        } catch (e: Throwable) {
            Log.w("LitertPlugin", "Ignored conversation close exception: ${e.message}")
        } finally {
            conversation = null
        }

        try {
            engine?.close()
        } catch (e: Throwable) {
            Log.w("LitertPlugin", "Ignored engine close exception: ${e.message}")
        } finally {
            engine = null
        }
        currentLoadedModelPath = null
        currentLoadedAccelerator = null
        System.gc()
    }

    // Manual ActivityResultRegistry registration to bypass Tauri's broken lateinit launcher bug
    // AND bypass the LifecycleOwner STARTED requirement
    private var pendingImageInvoke: Invoke? = null
    private val pickerLauncher: ActivityResultLauncher<Intent> = 
        (activity as ComponentActivity).activityResultRegistry.register(
            "litert_gallery_picker",
            ActivityResultContracts.StartActivityForResult()
        ) { result ->
            if (result.resultCode == Activity.RESULT_OK && result.data != null && result.data!!.data != null) {
                val uri: Uri = result.data!!.data!!
                
                try {
                    val fileName = "vision_target_${System.currentTimeMillis()}.jpg"
                    val cacheFile = File(activity.cacheDir, fileName)
                    
                    activity.contentResolver.openInputStream(uri)?.use { input ->
                        FileOutputStream(cacheFile).use { output ->
                            input.copyTo(output)
                        }
                    }
                    
                    val ret = JSObject()
                    ret.put("path", cacheFile.absolutePath)
                    ret.put("uri", uri.toString())
                    pendingImageInvoke?.resolve(ret)
                    
                } catch (e: Exception) {
                    pendingImageInvoke?.reject("Failed to process image: ${e.message}")
                }
            } else {
                pendingImageInvoke?.reject("Image selection cancelled by user")
            }
            pendingImageInvoke = null
        }

    private var pendingCameraInvoke: Invoke? = null
    private var currentPhotoPath: String? = null
    private var currentPhotoUri: Uri? = null

    private val cameraPermissionLauncher: ActivityResultLauncher<String> =
        (activity as ComponentActivity).activityResultRegistry.register(
            "litert_camera_permission",
            ActivityResultContracts.RequestPermission()
        ) { isGranted ->
            if (isGranted) {
                launchCameraIntent()
            } else {
                pendingCameraInvoke?.reject("Camera permission denied by user")
                pendingCameraInvoke = null
            }
        }

    private var pendingCalendarPermissionInvoke: Invoke? = null
    private val calendarPermissionLauncher: ActivityResultLauncher<String> =
        (activity as ComponentActivity).activityResultRegistry.register(
            "litert_calendar_permission",
            ActivityResultContracts.RequestPermission()
        ) { isGranted ->
            val ret = JSObject()
            ret.put("granted", isGranted)
            pendingCalendarPermissionInvoke?.resolve(ret)
            pendingCalendarPermissionInvoke = null
        }

    private var pendingAudioPermissionInvoke: Invoke? = null
    private val audioPermissionLauncher: ActivityResultLauncher<String> =
        (activity as ComponentActivity).activityResultRegistry.register(
            "litert_audio_permission",
            ActivityResultContracts.RequestPermission()
        ) { isGranted ->
            if (isGranted) {
                startNativeSpeechInternal()
            } else {
                pendingSpeechInvoke?.reject("Audio recording permission denied")
                pendingSpeechInvoke = null
            }
        }

    private var speechRecognizer: SpeechRecognizer? = null
    private var pendingSpeechInvoke: Invoke? = null
    private var latestSpeechTranscript: String = ""

    private val cameraLauncher: ActivityResultLauncher<Intent> =
        (activity as ComponentActivity).activityResultRegistry.register(
            "litert_camera_capture",
            ActivityResultContracts.StartActivityForResult()
        ) { result ->
            if (result.resultCode == Activity.RESULT_OK && currentPhotoPath != null) {
                try {
                    val ret = JSObject()
                    ret.put("path", currentPhotoPath)
                    if (currentPhotoUri != null) {
                        ret.put("uri", currentPhotoUri.toString())
                    }
                    pendingCameraInvoke?.resolve(ret)
                } catch (e: Exception) {
                    pendingCameraInvoke?.reject("Failed to process captured photo: ${e.message}")
                }
            } else {
                pendingCameraInvoke?.reject("Camera capture cancelled by user")
            }
            pendingCameraInvoke = null
        }

    private fun downloadModelWithRedirects(urlStr: String, destFile: File) {
        var currentUrl = URL(urlStr)
        var connection = currentUrl.openConnection() as java.net.HttpURLConnection
        var redirectCount = 0
        val maxRedirects = 5

        while (redirectCount < maxRedirects) {
            connection.instanceFollowRedirects = false
            connection.connect()
            val responseCode = connection.responseCode

            if (responseCode in 300..399) {
                val location = connection.getHeaderField("Location") ?: throw Exception("Redirect without location header")
                currentUrl = URL(location)
                connection = currentUrl.openConnection() as java.net.HttpURLConnection
                redirectCount++
            } else if (responseCode in 200..299) {
                connection.inputStream.use { input ->
                    FileOutputStream(destFile).use { output ->
                        input.copyTo(output)
                    }
                }
                break
            } else {
                throw Exception("Failed to download file: HTTP $responseCode")
            }
        }
    }

    @Command
    fun checkModelExists(invoke: Invoke) {
        val args = invoke.parseArgs(CheckModelArgs::class.java)
        scope.launch {
            try {
                val internalFilesDir = activity.filesDir.absolutePath
                val absoluteModelPath = "$internalFilesDir/${args.modelPath}"
                val modelFile = File(absoluteModelPath)
                
                val exists = modelFile.exists() && modelFile.length() > 100000000L

                val workManager = androidx.work.WorkManager.getInstance(activity.applicationContext)
                val workInfos = workManager.getWorkInfosForUniqueWork("litert_download").get()
                var isDownloading = false
                if (workInfos.isNotEmpty()) {
                    val state = workInfos.first().state
                    if (state == androidx.work.WorkInfo.State.RUNNING || state == androidx.work.WorkInfo.State.ENQUEUED) {
                        isDownloading = true
                        pollDownloadProgress()
                    }
                }
                
                val ret = JSObject()
                ret.put("exists", exists)
                ret.put("isDownloading", isDownloading)
                invoke.resolve(ret)
            } catch (e: Exception) {
                invoke.reject(e.message)
            }
        }
    }

    @Command
    fun downloadModel(invoke: Invoke) {
        val args = invoke.parseArgs(DownloadModelArgs::class.java)
        
        // Request POST_NOTIFICATIONS permission for Android 13+
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU) {
            if (androidx.core.content.ContextCompat.checkSelfPermission(activity, android.Manifest.permission.POST_NOTIFICATIONS) != android.content.pm.PackageManager.PERMISSION_GRANTED) {
                androidx.core.app.ActivityCompat.requestPermissions(activity, arrayOf(android.Manifest.permission.POST_NOTIFICATIONS), 1002)
            }
        }

        scope.launch {
            try {
                val downloadUrl = if (!args.downloadUrl.isNullOrBlank()) {
                    args.downloadUrl!!
                } else if (args.modelPath.contains("medgemma", ignoreCase = true)) {
                    "https://huggingface.co/litert-community/MedGemma-1.5-4B-IT/resolve/main/medgemma-1.5-4b-it_q4_block32_ekv2048.litertlm"
                } else {
                    "https://huggingface.co/litert-community/gemma-4-E2B-it-litert-lm/resolve/main/gemma-4-E2B-it.litertlm"
                }
                
                val workData = androidx.work.workDataOf(
                    "modelPath" to args.modelPath,
                    "downloadUrl" to downloadUrl,
                    "token" to args.token
                )

                val workRequest = androidx.work.OneTimeWorkRequestBuilder<ModelDownloadWorker>()
                    .setInputData(workData)
                    .build()

                androidx.work.WorkManager.getInstance(activity.applicationContext)
                    .enqueueUniqueWork(
                        "litert_download",
                        androidx.work.ExistingWorkPolicy.KEEP,
                        workRequest
                    )
                
                // Launch progress polling
                pollDownloadProgress()

                val ret = JSObject()
                ret.put("success", true)
                invoke.resolve(ret)
            } catch (e: Exception) {
                invoke.reject(e.message)
            }
        }
    }

    @Command
    fun purgeModel(invoke: Invoke) {
        val args = invoke.parseArgs(PurgeModelArgs::class.java)
        try {
            val internalFilesDir = activity.filesDir.absolutePath
            val absoluteModelPath = "$internalFilesDir/${args.modelPath}"
            val tmpModelPath = "$absoluteModelPath.tmp"
            
            val modelFile = File(absoluteModelPath)
            if (modelFile.exists()) {
                modelFile.delete()
            }
            
            val tmpFile = File(tmpModelPath)
            if (tmpFile.exists()) {
                tmpFile.delete()
            }
            
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message)
        }
    }

    private fun pollDownloadProgress() {
        scope.launch {
            val workManager = androidx.work.WorkManager.getInstance(activity.applicationContext)
            var isCompleted = false
            
            while (!isCompleted) {
                val workInfos = workManager.getWorkInfosForUniqueWork("litert_download").get()
                if (workInfos.isNotEmpty()) {
                    val info = workInfos.first()
                    
                    val progress = info.progress
                    val downloaded = progress.getLong("downloaded", -1L)
                    val total = progress.getLong("total", -1L)
                    
                    if (downloaded >= 0) {
                        val eventData = JSObject()
                        eventData.put("downloaded", downloaded)
                        eventData.put("total", total)
                        trigger("download_progress", eventData)
                    }
                    
                    if (info.state == androidx.work.WorkInfo.State.SUCCEEDED || 
                        info.state == androidx.work.WorkInfo.State.FAILED ||
                        info.state == androidx.work.WorkInfo.State.CANCELLED) {
                        isCompleted = true
                        
                        // Emit final event
                        val finalEvent = JSObject()
                        finalEvent.put("downloaded", total)
                        finalEvent.put("total", total)
                        finalEvent.put("state", info.state.name)
                        trigger("download_progress", finalEvent)
                    }
                }
                if (!isCompleted) {
                    kotlinx.coroutines.delay(500)
                }
            }
        }
    }

    @Command
    fun initModel(invoke: Invoke) {
        val args = invoke.parseArgs(InitModelArgs::class.java)
        
        scope.launch {
            try {
                // Determine absolute path
                val internalFilesDir = activity.filesDir.absolutePath
                val absoluteModelPath = "$internalFilesDir/${args.modelPath}"
                
                val modelFile = File(absoluteModelPath)
                if (!modelFile.exists() || modelFile.length() < 100000000L) {
                    invoke.reject("Model file not found or corrupted. Please download it first.")
                    return@launch
                }

                // If already loaded the exact same model & accelerator, reuse without re-allocating memory
                if (engine != null && currentLoadedModelPath == args.modelPath && currentLoadedAccelerator == args.accelerator) {
                    Log.i("LitertPlugin", "Model already loaded in memory, reusing: ${args.modelPath}")
                    val ret = JSObject()
                    ret.put("success", true)
                    invoke.resolve(ret)
                    return@launch
                }

                // CRITICAL FOR MOBILE RAM: Phone cannot hold two models in memory simultaneously.
                // Safely and completely tear down any previous engine before loading another.
                safeCloseEngine()

                val requestedAccelerator = args.accelerator

                if (requestedAccelerator != "Auto") {
                    val configBackend = when (requestedAccelerator) {
                        "CPU" -> Backend.CPU()
                        "GPU" -> Backend.GPU()
                        "NPU" -> Backend.NPU(nativeLibraryDir = activity.applicationInfo.nativeLibraryDir)
                        else -> Backend.CPU()
                    }
                    val engineConfig = EngineConfig(
                        modelPath = absoluteModelPath,
                        maxNumTokens = args.maxTokens,
                        backend = configBackend,
                        visionBackend = Backend.CPU(),
                        audioBackend = Backend.CPU(),
                        maxNumImages = 1,
                        cacheDir = activity.cacheDir.path
                    )
                    engine = Engine(engineConfig)
                    engine!!.initialize()
                    Log.i("LitertPlugin", "Model initialized with explicit $requestedAccelerator backend")
                } else {
                    // Auto mode: Try GPU -> NPU -> CPU fallback
                    var initialized = false
                    var lastErr: Exception? = null

                    // 1. Try GPU first
                    try {
                        val gpuConfig = EngineConfig(
                            modelPath = absoluteModelPath,
                            maxNumTokens = args.maxTokens,
                            backend = Backend.GPU(),
                            visionBackend = Backend.CPU(),
                            audioBackend = Backend.CPU(),
                            maxNumImages = 1,
                            cacheDir = activity.cacheDir.path
                        )
                        engine = Engine(gpuConfig)
                        engine!!.initialize()
                        initialized = true
                        Log.i("LitertPlugin", "Model initialized with GPU backend")
                    } catch (gpuErr: Exception) {
                        Log.w("LitertPlugin", "GPU backend failed, trying NPU: ${gpuErr.message}")
                        lastErr = gpuErr
                        safeCloseEngine()
                    }

                    // 2. If GPU failed, try NPU
                    if (!initialized) {
                        try {
                            val npuConfig = EngineConfig(
                                modelPath = absoluteModelPath,
                                maxNumTokens = args.maxTokens,
                                backend = Backend.NPU(nativeLibraryDir = activity.applicationInfo.nativeLibraryDir),
                                visionBackend = Backend.CPU(),
                                audioBackend = Backend.CPU(),
                                maxNumImages = 1,
                                cacheDir = activity.cacheDir.path
                            )
                            engine = Engine(npuConfig)
                            engine!!.initialize()
                            initialized = true
                            Log.i("LitertPlugin", "Model initialized with NPU backend")
                        } catch (npuErr: Exception) {
                            Log.w("LitertPlugin", "NPU backend failed, falling back to CPU: ${npuErr.message}")
                            lastErr = npuErr
                            safeCloseEngine()
                        }
                    }

                    // 3. Fallback to CPU (universal compatibility)
                    if (!initialized) {
                        try {
                            val cpuConfig = EngineConfig(
                                modelPath = absoluteModelPath,
                                maxNumTokens = args.maxTokens,
                                backend = Backend.CPU(),
                                visionBackend = Backend.CPU(),
                                audioBackend = Backend.CPU(),
                                maxNumImages = 1,
                                cacheDir = activity.cacheDir.path
                            )
                            engine = Engine(cpuConfig)
                            engine!!.initialize()
                            initialized = true
                            Log.i("LitertPlugin", "Model initialized with CPU backend")
                        } catch (cpuErr: Exception) {
                            safeCloseEngine()
                            throw lastErr ?: cpuErr
                        }
                    }
                }

                currentLoadedModelPath = args.modelPath
                currentLoadedAccelerator = args.accelerator

                // Store the conversation config for later reuse
                conversationConfig = ConversationConfig(
                    samplerConfig = SamplerConfig(
                        topK = 64,
                        topP = 0.95,
                        temperature = 1.0
                    )
                )
                conversation = engine!!.createConversation(conversationConfig!!)

                val ret = JSObject()
                ret.put("success", true)
                invoke.resolve(ret)
            } catch (e: LiteRtLmJniException) {
                Log.e("LitertPlugin", "JNI Exception initializing model", e)
                safeCloseEngine()
                invoke.reject("Native failure: ${e.message}")
            } catch (e: Exception) {
                Log.e("LitertPlugin", "Failed to init model", e)
                safeCloseEngine()
                invoke.reject("Failed to load model: ${e.message}")
            }
        }
    }

    @Command
    fun generateChat(invoke: Invoke) {
        val args = invoke.parseArgs(GenerateChatArgs::class.java)
        
        if (engine == null) {
            invoke.reject("Model is not initialized.")
            return
        }

        scope.launch {
            try {
                if (args.reset == true || conversation == null) {
                    conversation?.close()
                    
                    val config = ConversationConfig(
                        systemInstruction = if (!args.systemInstruction.isNullOrEmpty()) Contents.of(args.systemInstruction!!) else null,
                        samplerConfig = conversationConfig?.samplerConfig ?: SamplerConfig(
                            topK = 64,
                            topP = 0.95,
                            temperature = 1.0
                        )
                    )
                    conversationConfig = config
                    conversation = engine!!.createConversation(config)
                }
                
                var responseText = ""
                
                val contents = mutableListOf<Content>()
                
                if (!args.imageUri.isNullOrEmpty()) {
                    val file = File(args.imageUri!!)
                    if (file.exists()) {
                        val rawBitmap = android.graphics.BitmapFactory.decodeFile(file.absolutePath)
                        if (rawBitmap != null) {
                            // LiteRT Gemma 4 vision encoder uses vision_280 with exact 280x280 input
                            val scaledBitmap = android.graphics.Bitmap.createScaledBitmap(rawBitmap, 280, 280, true)
                            
                            val tempFile = java.io.File(activity.cacheDir, "litert_scaled_temp.jpg")
                            val outStream = java.io.FileOutputStream(tempFile)
                            scaledBitmap.compress(android.graphics.Bitmap.CompressFormat.JPEG, 90, outStream)
                            outStream.flush()
                            outStream.close()
                            
                            contents.add(Content.ImageFile(tempFile.absolutePath))
                            
                            if (scaledBitmap != rawBitmap) {
                                rawBitmap.recycle()
                            }
                        } else {
                            Log.w("LitertPlugin", "Failed to decode bitmap from path: ${args.imageUri}")
                        }
                    }
                }
                
                if (!args.audioBase64.isNullOrEmpty()) {
                    val audioBytes = android.util.Base64.decode(args.audioBase64, android.util.Base64.DEFAULT)
                    contents.add(Content.AudioBytes(audioBytes))
                }
                
                if (args.prompt.trim().isNotEmpty()) {
                    contents.add(Content.Text(args.prompt))
                }
                
                if (contents.isNotEmpty()) {
                    conversation!!.sendMessageAsync(Contents.of(contents)).collect { token ->
                        responseText += token
                    }
                } else {
                    invoke.reject("No content provided to generate chat.")
                    return@launch
                }
                
                val ret = JSObject()
                ret.put("response", responseText)
                invoke.resolve(ret)
            } catch (e: LiteRtLmJniException) {
                Log.e("LitertPlugin", "JNI Exception during chat", e)
                invoke.reject("Native generation failed: ${e.message}")
            } catch (e: Exception) {
                Log.e("LitertPlugin", "Failed to generate chat", e)
                invoke.reject("Generation failed: ${e.message}")
            }
        }
    }

    @Command
    fun closeModel(invoke: Invoke) {
        scope.launch {
            try {
                safeCloseEngine()
                val ret = JSObject()
                ret.put("success", true)
                invoke.resolve(ret)
            } catch (e: Exception) {
                invoke.reject("Failed to close model: ${e.message}")
            }
        }
    }

    @Command
    fun pickGalleryImage(invoke: Invoke) {
        pendingImageInvoke = invoke
        val intent = Intent(Intent.ACTION_GET_CONTENT).apply {
            type = "image/*"
            addCategory(Intent.CATEGORY_OPENABLE)
        }
        
        pickerLauncher.launch(intent)
    }

    @Command
    fun takeCameraPhoto(invoke: Invoke) {
        pendingCameraInvoke = invoke
        val hasPermission = androidx.core.content.ContextCompat.checkSelfPermission(
            activity,
            android.Manifest.permission.CAMERA
        ) == android.content.pm.PackageManager.PERMISSION_GRANTED

        if (hasPermission) {
            launchCameraIntent()
        } else {
            cameraPermissionLauncher.launch(android.Manifest.permission.CAMERA)
        }
    }

    private fun launchCameraIntent() {
        try {
            val fileName = "camera_capture_${System.currentTimeMillis()}.jpg"
            val photoFile = File(activity.cacheDir, fileName)
            currentPhotoPath = photoFile.absolutePath

            val photoUri: Uri = androidx.core.content.FileProvider.getUriForFile(
                activity,
                "${activity.packageName}.fileprovider",
                photoFile
            )
            currentPhotoUri = photoUri

            val intent = Intent(android.provider.MediaStore.ACTION_IMAGE_CAPTURE).apply {
                putExtra(android.provider.MediaStore.EXTRA_OUTPUT, photoUri)
                addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION or Intent.FLAG_GRANT_READ_URI_PERMISSION)
            }

            cameraLauncher.launch(intent)
        } catch (e: Exception) {
            Log.e("LitertPlugin", "Failed to launch camera intent", e)
            pendingCameraInvoke?.reject("Failed to launch camera: ${e.message}")
            pendingCameraInvoke = null
        }
    }

    @Command
    fun scanBleDevices(invoke: Invoke) {
        val bluetoothManager = activity.getSystemService(Context.BLUETOOTH_SERVICE) as? BluetoothManager
        val adapter = bluetoothManager?.adapter
        if (adapter == null || !adapter.isEnabled) {
            val ret = JSObject()
            ret.put("devices", JSArray())
            ret.put("status", "Bluetooth is disabled")
            invoke.resolve(ret)
            return
        }

        val scanner = adapter.bluetoothLeScanner
        if (scanner == null) {
            val ret = JSObject()
            ret.put("devices", JSArray())
            ret.put("status", "Bluetooth LE Scanner unavailable")
            invoke.resolve(ret)
            return
        }

        val foundDevices = Collections.synchronizedMap(LinkedHashMap<String, JSObject>())

        // 1. Immediately populate from bonded / paired devices in Android Settings
        try {
            val bonded = adapter.bondedDevices
            if (bonded != null) {
                for (dev in bonded) {
                    val address = dev.address ?: continue
                    var name = ""
                    try {
                        name = dev.name ?: ""
                    } catch (e: SecurityException) {
                        name = ""
                    }
                    val lowerName = name.lowercase()
                    val isGarmin = lowerName.contains("garmin") || lowerName.contains("forerunner") ||
                            lowerName.contains("fenix") || lowerName.contains("venu") ||
                            lowerName.contains("instinct") || lowerName.contains("approach") ||
                            lowerName.contains("edge") || lowerName.contains("vivo") ||
                            lowerName.contains("epix") || lowerName.contains("descent") ||
                            lowerName.contains("marq") || lowerName.contains("enduro") ||
                            lowerName.contains("tactix")

                    val displayName = if (name.isNotEmpty()) name else if (isGarmin) "Garmin Watch" else "Bluetooth Device (${address.take(8)})"
                    val devObj = JSObject().apply {
                        put("deviceId", "ble-" + address.replace(":", "").lowercase())
                        put("deviceName", displayName)
                        put("macAddress", address)
                        put("rssi", -50)
                        put("isPaired", dev.bondState == BluetoothDevice.BOND_BONDED)
                        put("isConnected", false)
                    }
                    foundDevices[address] = devObj
                }
            }
        } catch (e: SecurityException) {
            Log.w("LitertPlugin", "Security exception getting bonded devices: ${e.message}")
        }

        // 2. Broadcast receiver for Android OS level Bluetooth device discovery
        val receiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context?, intent: Intent?) {
                if (intent?.action == BluetoothDevice.ACTION_FOUND) {
                    val dev = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                        intent.getParcelableExtra(BluetoothDevice.EXTRA_DEVICE, BluetoothDevice::class.java)
                    } else {
                        @Suppress("DEPRECATION")
                        intent.getParcelableExtra(BluetoothDevice.EXTRA_DEVICE)
                    } ?: return

                    val address = dev.address ?: return
                    var name = ""
                    try {
                        name = dev.name ?: ""
                    } catch (e: SecurityException) {
                        name = ""
                    }
                    val rssi = intent.getShortExtra(BluetoothDevice.EXTRA_RSSI, (-50).toShort()).toInt()
                    val lowerName = name.lowercase()
                    val isGarmin = lowerName.contains("garmin") || lowerName.contains("forerunner") ||
                            lowerName.contains("fenix") || lowerName.contains("venu") ||
                            lowerName.contains("instinct") || lowerName.contains("approach") ||
                            lowerName.contains("edge") || lowerName.contains("vivo") ||
                            lowerName.contains("epix") || lowerName.contains("descent") ||
                            lowerName.contains("marq") || lowerName.contains("enduro") ||
                            lowerName.contains("tactix")

                    val displayName = if (name.isNotEmpty()) name else if (isGarmin) "Garmin Watch" else "Bluetooth Device (${address.take(8)})"

                    val devObj = JSObject().apply {
                        put("deviceId", "ble-" + address.replace(":", "").lowercase())
                        put("deviceName", displayName)
                        put("macAddress", address)
                        put("rssi", rssi)
                        put("isPaired", dev.bondState == BluetoothDevice.BOND_BONDED)
                        put("isConnected", false)
                    }
                    foundDevices[address] = devObj
                }
            }
        }

        val scanCallback = object : ScanCallback() {
            override fun onScanResult(callbackType: Int, result: ScanResult?) {
                result ?: return
                val dev = result.device ?: return
                var name = ""
                try {
                    name = dev.name ?: result.scanRecord?.deviceName ?: ""
                } catch (e: SecurityException) {
                    name = result.scanRecord?.deviceName ?: ""
                }
                val address = dev.address ?: return
                val rssi = result.rssi

                val lowerName = name.lowercase()
                val isGarmin = lowerName.contains("garmin") || lowerName.contains("forerunner") ||
                        lowerName.contains("fenix") || lowerName.contains("venu") ||
                        lowerName.contains("instinct") || lowerName.contains("approach") ||
                        lowerName.contains("edge") || lowerName.contains("vivo") ||
                        lowerName.contains("epix") || lowerName.contains("descent") ||
                        lowerName.contains("marq") || lowerName.contains("enduro") ||
                        lowerName.contains("tactix")

                val displayName = if (name.isNotEmpty()) name else if (isGarmin) "Garmin Watch" else "Bluetooth Device (${address.take(8)})"

                val devObj = JSObject().apply {
                    put("deviceId", "ble-" + address.replace(":", "").lowercase())
                    put("deviceName", displayName)
                    put("macAddress", address)
                    put("rssi", rssi)
                    put("isPaired", dev.bondState == BluetoothDevice.BOND_BONDED)
                    put("isConnected", false)
                }
                foundDevices[address] = devObj
            }

            override fun onBatchScanResults(results: MutableList<ScanResult>?) {
                results?.forEach { onScanResult(ScanSettings.CALLBACK_TYPE_ALL_MATCHES, it) }
            }

            override fun onScanFailed(errorCode: Int) {
                Log.e("LitertPlugin", "BLE scan failed with code: $errorCode")
            }
        }

        try {
            val filter = IntentFilter(BluetoothDevice.ACTION_FOUND)
            activity.registerReceiver(receiver, filter)
            adapter.startDiscovery()
        } catch (e: Exception) {
            Log.w("LitertPlugin", "Error registering discovery receiver: ${e.message}")
        }

        try {
            val settings = ScanSettings.Builder()
                .setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY)
                .build()
            scanner.startScan(null, settings, scanCallback)

            scope.launch {
                kotlinx.coroutines.delay(4000)
                try {
                    scanner.stopScan(scanCallback)
                } catch (e: Exception) {
                    Log.w("LitertPlugin", "Error stopping BLE scan", e)
                }
                try {
                    adapter.cancelDiscovery()
                    activity.unregisterReceiver(receiver)
                } catch (e: Exception) {}

                val array = JSArray()
                for (obj in foundDevices.values) {
                    array.put(obj)
                }
                val ret = JSObject().apply {
                    put("devices", array)
                    put("status", "Scan complete")
                }
                invoke.resolve(ret)
            }
        } catch (e: SecurityException) {
            Log.e("LitertPlugin", "Security exception during BLE scan", e)
            try {
                activity.unregisterReceiver(receiver)
            } catch (ignored: Exception) {}
            val array = JSArray()
            for (obj in foundDevices.values) {
                array.put(obj)
            }
            val ret = JSObject().apply {
                put("devices", array)
                put("status", "Permission missing: ${e.message}")
            }
            invoke.resolve(ret)
        } catch (e: Exception) {
            Log.e("LitertPlugin", "Error during BLE scan", e)
            try {
                activity.unregisterReceiver(receiver)
            } catch (ignored: Exception) {}
            val array = JSArray()
            for (obj in foundDevices.values) {
                array.put(obj)
            }
            val ret = JSObject().apply {
                put("devices", array)
                put("status", "Scan error: ${e.message}")
            }
            invoke.resolve(ret)
        }
    }

    private fun computeGarminCrc(data: ByteArray, offset: Int, length: Int): Int {
        val constants = intArrayOf(
            0x0000, 0xCC01, 0xD801, 0x1400, 0xF001, 0x3C00, 0x2800, 0xE401,
            0xA001, 0x6C00, 0x7800, 0xB401, 0x5000, 0x9C01, 0x8801, 0x4400
        )
        var crc = 0
        for (i in offset until (offset + length).coerceAtMost(data.size)) {
            val b = data[i].toInt() and 0xFF
            crc = (((crc shr 4) and 4095) xor constants[crc and 15]) xor constants[b and 15]
            crc = (((crc shr 4) and 4095) xor constants[crc and 15]) xor constants[(b shr 4) and 15]
        }
        return crc
    }

    private fun cobsEncode(data: ByteArray): ByteArray {
        val encoded = ByteArrayOutputStream()
        encoded.write(0x00) // Garmin leading zero
        var i = 0
        val n = data.size

        while (i < n) {
            val start = i
            while (i < n && data[i] != 0.toByte()) {
                i++
            }
            var payloadSize = i - start
            val lastWasZero = (i < n && data[i] == 0.toByte())
            var startPos = start

            while (payloadSize >= 0xFE) {
                encoded.write(0xFF)
                encoded.write(data, startPos, 0xFE)
                payloadSize -= 0xFE
                startPos += 0xFE
            }

            encoded.write(payloadSize + 1)
            encoded.write(data, startPos, payloadSize)

            if (lastWasZero) {
                i++
            }
        }

        if (data.isNotEmpty() && data[data.size - 1] == 0.toByte()) {
            encoded.write(0x01)
        }

        encoded.write(0x00) // Garmin trailing delimiter
        return encoded.toByteArray()
    }

    private fun cobsDecode(buffer: ByteArray): ByteArray? {
        if (buffer.size < 4) return null
        if (buffer[0] != 0.toByte() || buffer[buffer.size - 1] != 0.toByte()) return null

        val frame = buffer.copyOfRange(1, buffer.size - 1)
        val decoded = ByteArrayOutputStream()
        var i = 0
        while (i < frame.size) {
            val code = frame[i].toInt() and 0xFF
            i++
            if (code == 0) break
            val payloadSize = code - 1
            if (i + payloadSize > frame.size) break
            decoded.write(frame, i, payloadSize)
            i += payloadSize
            if (code != 0xFF && i < frame.size) {
                decoded.write(0)
            }
        }
        return decoded.toByteArray()
    }

    private fun bytesToHex(bytes: ByteArray, maxLen: Int = 32): String {
        val len = bytes.size.coerceAtMost(maxLen)
        val sb = StringBuilder()
        for (i in 0 until len) {
            sb.append(String.format("%02X ", bytes[i]))
        }
        if (bytes.size > maxLen) sb.append("... (+${bytes.size - maxLen}b)")
        return sb.toString().trim()
    }

    private fun buildGfdiMessage(messageId: Int, payload: ByteArray): ByteArray {
        val totalLength = 6 + payload.size
        val buf = ByteBuffer.allocate(totalLength).order(ByteOrder.LITTLE_ENDIAN)
        buf.putShort(totalLength.toShort())
        buf.putShort(messageId.toShort())
        buf.put(payload)
        val dataForCrc = buf.array()
        val crc = computeGarminCrc(dataForCrc, 0, totalLength - 2)
        buf.putShort(crc.toShort())
        return buf.array()
    }

    private fun writeGattData(gatt: BluetoothGatt, char: BluetoothGattCharacteristic, data: ByteArray) {
        Log.d("LitertPlugin", "TX WRITE [${data.size}b]: ${bytesToHex(data)}")
        char.value = data
        char.writeType = BluetoothGattCharacteristic.WRITE_TYPE_NO_RESPONSE
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            gatt.writeCharacteristic(char, data, BluetoothGattCharacteristic.WRITE_TYPE_NO_RESPONSE)
        } else {
            @Suppress("DEPRECATION")
            gatt.writeCharacteristic(char)
        }
    }

    private inner class GarminCobsDecoder {
        private val rxBuffer = ByteArrayOutputStream()

        @Synchronized
        fun receiveBytes(bytes: ByteArray): List<ByteArray> {
            val result = mutableListOf<ByteArray>()
            rxBuffer.write(bytes)
            while (true) {
                val all = rxBuffer.toByteArray()
                if (all.size < 4) break
                
                // Find leading 0x00 delimiter
                val startIdx = all.indexOfFirst { it == 0.toByte() }
                if (startIdx < 0) {
                    rxBuffer.reset()
                    break
                }
                if (startIdx > 0) {
                    val trimmed = all.copyOfRange(startIdx, all.size)
                    rxBuffer.reset()
                    rxBuffer.write(trimmed)
                    continue
                }
                
                // startIdx == 0. Find trailing 0x00 delimiter
                var endIdx = -1
                for (i in 1 until all.size) {
                    if (all[i] == 0.toByte()) {
                        endIdx = i
                        break
                    }
                }
                if (endIdx < 0) break // Incomplete frame, wait for next chunk
                
                val frameWithDelims = all.copyOfRange(0, endIdx + 1)
                val remainder = all.copyOfRange(endIdx + 1, all.size)
                rxBuffer.reset()
                rxBuffer.write(remainder)
                
                val decoded: ByteArray? = cobsDecode(frameWithDelims)
                if (decoded != null && decoded.isNotEmpty()) {
                    result.add(decoded)
                }
            }
            return result
        }
    }

    private fun supportsProperty(characteristic: BluetoothGattCharacteristic, property: Int): Boolean {
        return (characteristic.properties and property) != 0
    }

    private interface WriteQueue {
        fun enqueue(data: ByteArray)
        fun clear()
        fun onWriteComplete(status: Int)
    }

    private inner class ResponseWriteQueue(
        private val gatt: BluetoothGatt,
        private val char: BluetoothGattCharacteristic,
        private val onError: ((String) -> Unit)? = null
    ) : WriteQueue {
        private val queue = java.util.ArrayDeque<ByteArray>()
        private var busy = false

        @Synchronized
        override fun enqueue(data: ByteArray) {
            queue.addLast(data)
            Log.d("LitertPlugin", "RESPONSE_QUEUE: enqueued=${data.size}b, pending=${queue.size}, busy=$busy")
            pump()
        }

        @Synchronized
        override fun clear() {
            queue.clear()
            busy = false
        }

        @Synchronized
        override fun onWriteComplete(status: Int) {
            Log.d("LitertPlugin", "RESPONSE_QUEUE onWriteComplete: status=$status, pending=${queue.size}")
            busy = false
            if (status != BluetoothGatt.GATT_SUCCESS) {
                Log.e("LitertPlugin", "GATT write failed with status $status. Aborting queue.")
                clear()
                onError?.invoke("Garmin GATT write failed: $status")
                return
            }
            pump()
        }

        @Synchronized
        private fun pump() {
            if (busy || queue.isEmpty()) return
            val next = queue.removeFirst()
            busy = true
            Log.d("LitertPlugin", "TX WRITE_DEFAULT [${next.size}b]: ${bytesToHex(next)}")

            val writeType = BluetoothGattCharacteristic.WRITE_TYPE_DEFAULT

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                val status = gatt.writeCharacteristic(char, next, writeType)
                if (status != 0) { // 0 = BluetoothStatusCodes.SUCCESS
                    Log.e("LitertPlugin", "gatt.writeCharacteristic failed: status=$status")
                    busy = false
                    clear()
                    onError?.invoke("gatt.writeCharacteristic failed: status=$status")
                }
            } else {
                @Suppress("DEPRECATION")
                run {
                    char.writeType = writeType
                    char.value = next
                    if (!gatt.writeCharacteristic(char)) {
                        Log.e("LitertPlugin", "gatt.writeCharacteristic returned false")
                        busy = false
                        clear()
                        onError?.invoke("gatt.writeCharacteristic returned false")
                    }
                }
            }
        }
    }

    private inner class NoResponseWriteQueue(
        private val gatt: BluetoothGatt,
        private val char: BluetoothGattCharacteristic,
        private val scope: CoroutineScope,
        private val onError: ((String) -> Unit)? = null
    ) : WriteQueue {
        private val queue = java.util.ArrayDeque<ByteArray>()
        private var busy = false

        @Synchronized
        override fun enqueue(data: ByteArray) {
            queue.addLast(data)
            Log.d("LitertPlugin", "NO_RESPONSE_QUEUE: enqueued=${data.size}b, pending=${queue.size}, busy=$busy")
            pump()
        }

        @Synchronized
        override fun clear() {
            queue.clear()
            busy = false
        }

        override fun onWriteComplete(status: Int) {
            // No-op for no-response writes
        }

        @Synchronized
        private fun pump() {
            if (busy || queue.isEmpty()) return
            val next = queue.removeFirst()
            busy = true
            Log.d("LitertPlugin", "TX WRITE_NO_RESPONSE [${next.size}b]: ${bytesToHex(next)}")

            val accepted = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                gatt.writeCharacteristic(
                    char,
                    next,
                    BluetoothGattCharacteristic.WRITE_TYPE_NO_RESPONSE
                ) == 0 // 0 = BluetoothStatusCodes.SUCCESS
            } else {
                @Suppress("DEPRECATION")
                run {
                    char.writeType = BluetoothGattCharacteristic.WRITE_TYPE_NO_RESPONSE
                    char.value = next
                    gatt.writeCharacteristic(char)
                }
            }

            if (!accepted) {
                Log.e("LitertPlugin", "No-response GATT write was rejected by Android")
                busy = false
                clear()
                onError?.invoke("No-response GATT write was rejected by Android")
                return
            }

            scope.launch {
                kotlinx.coroutines.delay(15)
                synchronized(this@NoResponseWriteQueue) {
                    busy = false
                    pump()
                }
            }
        }
    }

    private fun isFitFile(bytes: ByteArray?): Boolean {
        if (bytes == null || bytes.size < 14) return false
        return bytes[8] == '.'.code.toByte() &&
               bytes[9] == 'F'.code.toByte() &&
               bytes[10] == 'I'.code.toByte() &&
               bytes[11] == 'T'.code.toByte()
    }

    private fun tryExtractCompleteFit(bytes: ByteArray): ByteArray? {
        if (bytes.size < 14) return null

        fun findValidFit(data: ByteArray): ByteArray? {
            if (data.size < 14) return null
            for (offset in 0..(data.size - 14)) {
                val headerSize = data[offset].toInt() and 0xFF
                if (headerSize !in setOf(12, 14)) continue
                if (offset + headerSize + 2 > data.size) continue
                if (data[offset + 8] != '.'.code.toByte() ||
                    data[offset + 9] != 'F'.code.toByte() ||
                    data[offset + 10] != 'I'.code.toByte() ||
                    data[offset + 11] != 'T'.code.toByte()) {
                    continue
                }
                val dataSize = ByteBuffer.wrap(data, offset + 4, 4)
                    .order(ByteOrder.LITTLE_ENDIAN)
                    .int
                if (dataSize < 0) continue
                val totalSize = headerSize + dataSize + 2
                val end = offset + totalSize
                if (end > data.size) {
                    Log.d("LitertPlugin", "FIT incomplete: have=${data.size - offset}, need=$totalSize")
                    continue
                }
                val fit = data.copyOfRange(offset, end)
                val expectedCrc = ByteBuffer.wrap(fit, fit.size - 2, 2)
                    .order(ByteOrder.LITTLE_ENDIAN)
                    .short.toInt() and 0xFFFF
                val actualCrc = computeGarminCrc(fit, 0, fit.size - 2) and 0xFFFF
                if (actualCrc != expectedCrc) {
                    Log.w("LitertPlugin", "FIT CRC mismatch: expected=$expectedCrc actual=$actualCrc")
                    continue
                }
                return fit
            }
            return null
        }

        // 1. Direct raw check
        findValidFit(bytes)?.let { return it }

        // 2. Try inflate (zlib standard)
        try {
            val inflater = java.util.zip.Inflater(false)
            inflater.setInput(bytes)
            val out = ByteArrayOutputStream()
            val buf = ByteArray(2048)
            while (!inflater.needsInput()) {
                val count = inflater.inflate(buf)
                if (count > 0) out.write(buf, 0, count)
                if (inflater.finished()) break
            }
            inflater.end()
            findValidFit(out.toByteArray())?.let { return it }
        } catch (e: Exception) {}

        // 3. Try raw deflate (no zlib wrapper)
        try {
            val inflater = java.util.zip.Inflater(true)
            inflater.setInput(bytes)
            val out = ByteArrayOutputStream()
            val buf = ByteArray(2048)
            while (!inflater.needsInput()) {
                val count = inflater.inflate(buf)
                if (count > 0) out.write(buf, 0, count)
                if (inflater.finished()) break
            }
            inflater.end()
            findValidFit(out.toByteArray())?.let { return it }
        } catch (e: Exception) {}

        // 4. Try GZIP decompression
        try {
            val gzip = java.util.zip.GZIPInputStream(java.io.ByteArrayInputStream(bytes))
            val out = ByteArrayOutputStream()
            val buf = ByteArray(2048)
            var count: Int
            while (gzip.read(buf).also { count = it } > 0) {
                out.write(buf, 0, count)
            }
            findValidFit(out.toByteArray())?.let { return it }
        } catch (e: Exception) {}

        return null
    }

    @Command
    fun connectAndSyncGarminDevice(invoke: Invoke) {
        val args = invoke.parseArgs(SyncBleDeviceArgs::class.java)
        val macAddress = args.macAddress
        Log.i("LitertPlugin", "Starting Garmin BLE Sync on MAC: $macAddress")
        if (macAddress.isEmpty()) {
            invoke.reject("MAC address required")
            return
        }

        val bluetoothManager = activity.getSystemService(Context.BLUETOOTH_SERVICE) as? android.bluetooth.BluetoothManager
        val adapter = bluetoothManager?.adapter
        if (adapter == null || !adapter.isEnabled) {
            invoke.reject("Bluetooth is disabled on Android device")
            return
        }

        try {
            val device = adapter.getRemoteDevice(macAddress)
            if (device == null) {
                invoke.reject("Bluetooth device not found: $macAddress")
                return
            }

            var gattRef: BluetoothGatt? = null
            var batteryLevel: Int? = null
            var isConnected = false
            var isServicesDiscovered = false
            var gfdiHandle = 1
            var fileTransferHandle: Int? = null
            val fitFilesList = Collections.synchronizedList(ArrayList<String>())
            val garminCobsDecoder = GarminCobsDecoder()
            val fitRecordStream = ByteArrayOutputStream()
            val fileBuffers = mutableMapOf<Int, ByteArrayOutputStream>()
            data class ProtobufTransfer(
                val totalLength: Int,
                var nextOffset: Int = 0,
                val bytes: ByteArrayOutputStream = ByteArrayOutputStream()
            )
            val protobufReassemblyMap = mutableMapOf<Int, ProtobufTransfer>()
            val fileTypeCodeMap = mutableMapOf<Int, String>()
            var txChar: BluetoothGattCharacteristic? = null
            var rxChar: BluetoothGattCharacteristic? = null
            var batteryChar: BluetoothGattCharacteristic? = null
            var lastActivityTime = System.currentTimeMillis()
            var hasFinished = false
            var writeQueue: WriteQueue? = null

            data class OffloadTransfer(
                val protobufRequestId: Int,
                val fileDescriptor: ByteArray,
                var garminFileHandle: Int? = null,
                var mlrTransportHandle: Int? = null,
                var expectedOffset: Int = 0,
                val bytes: ByteArrayOutputStream = ByteArrayOutputStream()
            )

            val transferByRequestId = mutableMapOf<Int, OffloadTransfer>()
            val transferByMlrHandle = mutableMapOf<Int, OffloadTransfer>()
            var activeTransfer: OffloadTransfer? = null

            fun wrapInFitFile(records: ByteArray): ByteArray {
                val dataSize = records.size
                val header = ByteBuffer.allocate(14).order(ByteOrder.LITTLE_ENDIAN)
                header.put(14.toByte()) // header size
                header.put(0x20.toByte()) // protocol version (2.0)
                header.putShort(2140.toShort()) // profile version (21.40)
                header.putInt(dataSize) // data size
                header.put(".FIT".toByteArray()) // signature
                val headerBytes = header.array()
                val headerCrc = computeGarminCrc(headerBytes, 0, 12)
                header.position(12)
                header.putShort(headerCrc.toShort())

                val fullFile = ByteArrayOutputStream()
                fullFile.write(headerBytes)
                fullFile.write(records)
                val fileData = fullFile.toByteArray()
                val fileCrc = computeGarminCrc(fileData, 0, fileData.size)
                val finalBuf = ByteBuffer.allocate(fileData.size + 2).order(ByteOrder.LITTLE_ENDIAN)
                finalBuf.put(fileData)
                finalBuf.putShort(fileCrc.toShort())
                return finalBuf.array()
            }

            fun sendGfdiPacket(handle: Int, cobsData: ByteArray, label: String = "GFDI") {
                val packet = byteArrayOf(handle.toByte()) + cobsData
                Log.d("LitertPlugin", "TX $label (handle $handle, ${packet.size}b): ${bytesToHex(packet)}")
                writeQueue?.enqueue(packet)
            }

            val sharedPrefs = activity.getSharedPreferences("garmin_synced_files", Context.MODE_PRIVATE)
            val macKey = macAddress.replace(":", "").uppercase()
            val prefKey = "synced_keys_$macKey"

            if (args.forcePullAll == true) {
                sharedPrefs.edit().remove(prefKey).apply()
                Log.i("LitertPlugin", "forcePullAll enabled: cleared synced keys for $macAddress")
            }

            val persistedSyncedKeys = Collections.synchronizedSet(
                java.util.HashSet(sharedPrefs.getStringSet(prefKey, emptySet()) ?: emptySet())
            )
            Log.i("LitertPlugin", "Loaded ${persistedSyncedKeys.size} previously synced file descriptors for watch $macAddress")

            val pendingFilesQueue = java.util.ArrayDeque<ByteArray>()
            val queuedFileKeys = mutableSetOf<String>()
            val requestedCursorIds = mutableSetOf<Int>()
            val requestedPageIds = mutableSetOf<Int>()
            var fileListFinished = false
            var lastProtocolProgressAt = System.currentTimeMillis()
            var lastProgressDescription = "Initial connect"

            var statPagesEnumerated = 0
            var statDescriptorsDiscovered = 0
            var statDescriptorsFiltered = 0
            var statDescriptorsQueued = 0
            var statFilesRequested = 0
            var statFileResponsesOk = 0
            var statFileResponsesRejected = 0
            var statOffloadsRegistered = 0
            var statOffloadsClosed = 0
            var statFitsExtracted = 0

            var nextReqId = 10

            fun markFileDescriptorSynced(descriptor: ByteArray) {
                val key = Base64.encodeToString(descriptor, Base64.NO_WRAP)
                if (persistedSyncedKeys.add(key)) {
                    sharedPrefs.edit().putStringSet(prefKey, java.util.HashSet(persistedSyncedKeys)).apply()
                    Log.i("LitertPlugin", "Marked file descriptor as synced (total synced: ${persistedSyncedKeys.size})")
                }
            }

            fun writeVarint(out: ByteArrayOutputStream, value: Int) {
                var v = value
                while (v >= 0x80) {
                    out.write((v and 0x7F) or 0x80)
                    v = v ushr 7
                }
                out.write(v and 0x7F)
            }

            fun startNextFileDownload() {
                if (activeTransfer != null || pendingFilesQueue.isEmpty()) return
                val nextFile = pendingFilesQueue.removeFirst()
                val reqId = nextReqId++
                val transfer = OffloadTransfer(protobufRequestId = reqId, fileDescriptor = nextFile)
                activeTransfer = transfer
                transferByRequestId[reqId] = transfer
                statFilesRequested++
                lastProtocolProgressAt = System.currentTimeMillis()
                lastProgressDescription = "Requested file #$statFilesRequested (req $reqId)"
                Log.i("LitertPlugin", "Starting download for file #$statFilesRequested of $statDescriptorsQueued (${nextFile.size}b, req $reqId, ${pendingFilesQueue.size} remaining)...")

                // Construct FileRequest: Smart { file_sync_service (43: 0xDA 0x02) { file_request (1: 0x0A) { file: nextFile, unk2: 24, unk3: 0, unk4: 0, unk5: 15 } } }
                val fReq = ByteArrayOutputStream()
                fReq.write(0x0A) // tag 1 (file)
                writeVarint(fReq, nextFile.size)
                fReq.write(nextFile)
                fReq.write(0x10) // tag 2 (unk2 = 24)
                writeVarint(fReq, 24)
                fReq.write(0x18) // tag 3 (unk3 = 0)
                writeVarint(fReq, 0)
                fReq.write(0x20) // tag 4 (unk4 = 0)
                writeVarint(fReq, 0)
                fReq.write(0x28) // tag 5 (unk5 = 15)
                writeVarint(fReq, 15)
                val fReqBytes = fReq.toByteArray()

                val fSync = ByteArrayOutputStream()
                fSync.write(0x0A) // tag 1 (file_request)
                writeVarint(fSync, fReqBytes.size)
                fSync.write(fReqBytes)
                val fSyncBytes = fSync.toByteArray()

                val smart = ByteArrayOutputStream()
                smart.write(0xDA); smart.write(0x02) // tag 43 (file_sync_service)
                writeVarint(smart, fSyncBytes.size)
                smart.write(fSyncBytes)
                val smartBytes = smart.toByteArray()

                val fReqPayload = ByteBuffer.allocate(14 + smartBytes.size).order(ByteOrder.LITTLE_ENDIAN)
                fReqPayload.putShort(reqId.toShort())
                fReqPayload.putInt(0)
                fReqPayload.putInt(smartBytes.size)
                fReqPayload.putInt(smartBytes.size)
                fReqPayload.put(smartBytes)
                val fReqMsg = buildGfdiMessage(5043, fReqPayload.array())
                sendGfdiPacket(gfdiHandle, cobsEncode(fReqMsg), "Protobuf FileRequest (req $reqId)")
            }

            fun enqueueFileIfNew(file: ByteArray) {
                val key = Base64.encodeToString(file, Base64.NO_WRAP)
                if (persistedSyncedKeys.contains(key)) {
                    statDescriptorsFiltered++
                    Log.d("LitertPlugin", "Skipping previously synced file descriptor (${file.size}b, key=${key.take(12)}...)")
                    return
                }
                if (!queuedFileKeys.add(key)) {
                    Log.d("LitertPlugin", "Skipping duplicate file descriptor in current queue (${file.size}b)")
                    return
                }
                pendingFilesQueue.addLast(file)
                statDescriptorsQueued++
                lastProtocolProgressAt = System.currentTimeMillis()
                Log.i("LitertPlugin", "Queued NEW file descriptor: totalQueued=${pendingFilesQueue.size}")
                if (fileListFinished) {
                    startNextFileDownload()
                }
            }

            fun sendFileListRequest(cursorId: Int? = null, startPageId: Int? = null) {
                val fListReq = ByteArrayOutputStream()

                if (cursorId != null) {
                    fListReq.write(0x08) // tag 1: cursor_id
                    writeVarint(fListReq, cursorId)
                } else if (startPageId != null) {
                    fListReq.write(0x10) // tag 2: start_page_id
                    writeVarint(fListReq, startPageId)
                }

                val fListBytes = fListReq.toByteArray()
                val fSync = ByteArrayOutputStream()
                fSync.write(0x4A) // tag 9 (file_list_request)
                writeVarint(fSync, fListBytes.size)
                fSync.write(fListBytes)
                val fSyncBytes = fSync.toByteArray()

                val smart = ByteArrayOutputStream()
                smart.write(0xDA); smart.write(0x02) // tag 43 (file_sync_service)
                writeVarint(smart, fSyncBytes.size)
                smart.write(fSyncBytes)
                val smartBytes = smart.toByteArray()

                val reqId = nextReqId++
                val protoReqPayload = ByteBuffer.allocate(14 + smartBytes.size).order(ByteOrder.LITTLE_ENDIAN)
                protoReqPayload.putShort(reqId.toShort())
                protoReqPayload.putInt(0)             // dataOffset = 0
                protoReqPayload.putInt(smartBytes.size)
                protoReqPayload.putInt(smartBytes.size)
                protoReqPayload.put(smartBytes)
                val protoReqMsg = buildGfdiMessage(5043, protoReqPayload.array())
                sendGfdiPacket(gfdiHandle, cobsEncode(protoReqMsg), "Protobuf FileListRequest (cursor=$cursorId, page=$startPageId, req $reqId)")
            }

            var hasTriggeredSync = false
            fun triggerSyncSequence() {
                if (hasTriggeredSync) return
                hasTriggeredSync = true
                Log.i("LitertPlugin", "Triggering Garmin full sync sequence (HOST_DID_ENTER_FOREGROUND, SYNC_READY, FileListRequest)...")

                // 1. Send HOST_DID_ENTER_FOREGROUND (event 6)
                val fgPayload = byteArrayOf(6.toByte(), 0.toByte())
                val fgMsg = buildGfdiMessage(5030, fgPayload)
                sendGfdiPacket(gfdiHandle, cobsEncode(fgMsg), "HOST_DID_ENTER_FOREGROUND")

                // 2. Send SYNC_READY (event 8)
                val syncReadyPayload = byteArrayOf(8.toByte(), 0.toByte())
                val syncReadyMsg = buildGfdiMessage(5030, syncReadyPayload)
                sendGfdiPacket(gfdiHandle, cobsEncode(syncReadyMsg), "SYNC_READY")

                // 3. Send Initial Protobuf FileListRequest
                requestedCursorIds.clear()
                requestedPageIds.clear()
                fileListFinished = false
                sendFileListRequest()
            }

            fun finish(gatt: BluetoothGatt?, battery: Int?, errMsg: String? = null) {
                if (hasFinished) return
                hasFinished = true

                Log.i("LitertPlugin", "=== Garmin Sync Audit ===")
                Log.i("LitertPlugin", "Pages enumerated: $statPagesEnumerated")
                Log.i("LitertPlugin", "Descriptors discovered: $statDescriptorsDiscovered")
                Log.i("LitertPlugin", "Descriptors filtered (already synced): $statDescriptorsFiltered")
                Log.i("LitertPlugin", "Descriptors queued (new): $statDescriptorsQueued")
                Log.i("LitertPlugin", "Files requested: $statFilesRequested")
                Log.i("LitertPlugin", "FileResponses accepted: $statFileResponsesOk")
                Log.i("LitertPlugin", "FileResponses rejected: $statFileResponsesRejected")
                Log.i("LitertPlugin", "Offloads registered: $statOffloadsRegistered")
                Log.i("LitertPlugin", "Offloads closed: $statOffloadsClosed")
                Log.i("LitertPlugin", "Valid FIT files captured: ${fitFilesList.size}")
                Log.i("LitertPlugin", "Total persisted synced file keys: ${persistedSyncedKeys.size}")
                Log.i("LitertPlugin", "=========================")

                try {
                    gatt?.disconnect()
                    gatt?.close()
                } catch (e: Exception) {}

                if (errMsg != null && fitFilesList.isEmpty()) {
                    invoke.reject(errMsg)
                    return
                }

                val res = JSObject()
                res.put("success", true)
                res.put("status_code", if (fitFilesList.isNotEmpty()) 200 else 204)
                val msg = if (fitFilesList.isNotEmpty()) "Successfully synced ${fitFilesList.size} Garmin FIT files!" else "Garmin sync complete. No new activity records found."
                res.put("status_message", msg)
                res.put("statusMessage", msg)
                if (battery != null) {
                    res.put("battery_level", battery)
                    res.put("batteryLevel", battery)
                }
                res.put("syncedActivitiesCount", fitFilesList.size)
                val fitArr = JSArray()
                for (f in fitFilesList) {
                    fitArr.put(f)
                }
                res.put("fit_files", fitArr)
                res.put("fitFiles", fitArr)
                invoke.resolve(res)
            }

            fun processDecodedGfdiMessage(decoded: ByteArray, gatt: BluetoothGatt) {
                if (decoded.size < 6) {
                    Log.w("LitertPlugin", "Dropping undersized GFDI frame (${decoded.size}b)")
                    return
                }
                val totalLength = ByteBuffer.wrap(decoded, 0, 2).order(ByteOrder.LITTLE_ENDIAN).short.toInt() and 0xFFFF
                if (totalLength != decoded.size) {
                    Log.w("LitertPlugin", "Dropping malformed GFDI frame: declared=$totalLength, actual=${decoded.size}")
                    return
                }
                val expectedCrc = ByteBuffer.wrap(decoded, decoded.size - 2, 2).order(ByteOrder.LITTLE_ENDIAN).short.toInt() and 0xFFFF
                val actualCrc = computeGarminCrc(decoded, 0, decoded.size - 2) and 0xFFFF
                if (actualCrc != expectedCrc) {
                    Log.w("LitertPlugin", "Dropping GFDI CRC failure: expected=$expectedCrc actual=$actualCrc")
                    return
                }

                val rawMsgType = ByteBuffer.wrap(decoded, 2, 2).order(ByteOrder.LITTLE_ENDIAN).short.toInt() and 0xFFFF
                val messageId = if ((rawMsgType and 0x8000) != 0) {
                    (rawMsgType and 0xFF) + 5000
                } else {
                    rawMsgType
                }
                val seqNumber = if ((rawMsgType and 0x8000) != 0) (rawMsgType shr 8) and 0x7F else 0
                Log.i("LitertPlugin", "Received GFDI Message: ID $messageId (raw $rawMsgType, seq $seqNumber, length $totalLength, bytes ${decoded.size}): ${bytesToHex(decoded)}")

                when (messageId) {
                    5024 -> { // DEVICE_INFORMATION (5024)
                        val btNameBytes = "GarminGoblin\u0000".toByteArray()
                        val mfgBytes = "Android\u0000".toByteArray()
                        val devBytes = "Phone\u0000".toByteArray()
                        val payloadSize = 2 + 1 + 2 + 2 + 4 + 2 + 2 + btNameBytes.size + mfgBytes.size + devBytes.size + 1
                        val devInfoPayload = ByteBuffer.allocate(payloadSize).order(ByteOrder.LITTLE_ENDIAN)
                        devInfoPayload.putShort(5024.toShort()) // target message ID 5024
                        devInfoPayload.put(0.toByte())          // ACK
                        devInfoPayload.putShort(150.toShort())  // ourProtocolVersion
                        devInfoPayload.putShort((-1).toShort()) // ourProductNumber
                        devInfoPayload.putInt(-1)               // ourUnitNumber
                        devInfoPayload.putShort(7791.toShort()) // ourSoftwareVersion
                        devInfoPayload.putShort((-1).toShort()) // ourMaxPacketSize
                        devInfoPayload.put(btNameBytes)
                        devInfoPayload.put(mfgBytes)
                        devInfoPayload.put(devBytes)
                        devInfoPayload.put(1.toByte())          // protocolFlags
                        val devInfoMsg = buildGfdiMessage(5000, devInfoPayload.array())
                        sendGfdiPacket(gfdiHandle, cobsEncode(devInfoMsg), "DeviceInformationResponse")
                        Log.i("LitertPlugin", "Replied to DEVICE_INFORMATION (5024) with DeviceInformationResponse.")

                        triggerSyncSequence()
                    }
                    5050 -> { // CONFIGURATION (5050)
                        // 1. Send status ACK targeting 5050
                        val configAckPayload = ByteBuffer.allocate(3).order(ByteOrder.LITTLE_ENDIAN)
                        configAckPayload.putShort(5050.toShort()) // target 5050
                        configAckPayload.put(0.toByte()) // ACK
                        val configAckMsg = buildGfdiMessage(5000, configAckPayload.array())
                        sendGfdiPacket(gfdiHandle, cobsEncode(configAckMsg), "CONFIGURATION ACK (5050)")
                        Log.i("LitertPlugin", "Replied ACK to CONFIGURATION (5050)")

                        // 2. Echo capabilities back
                        if (decoded.size >= 5) {
                            val numCaps = decoded[4].toInt() and 0xFF
                            val capBytes = decoded.copyOfRange(4, (4 + 1 + numCaps).coerceAtMost(decoded.size - 2))
                            val outConfigMsg = buildGfdiMessage(5050, capBytes)
                            sendGfdiPacket(gfdiHandle, cobsEncode(outConfigMsg), "CONFIGURATION Capabilities")
                        }

                        triggerSyncSequence()
                    }
                    5011 -> { // FIT_DEFINITION (5011)
                        if (decoded.size > 6) {
                            val defPayload = decoded.copyOfRange(4, decoded.size - 2)
                            fitRecordStream.write(defPayload)
                        }
                        val fitDefAck = ByteBuffer.allocate(4).order(ByteOrder.LITTLE_ENDIAN)
                        fitDefAck.putShort(5011.toShort())
                        fitDefAck.put(0.toByte()) // ACK
                        fitDefAck.put(0.toByte()) // APPLIED
                        val fitDefMsg = buildGfdiMessage(5000, fitDefAck.array())
                        sendGfdiPacket(gfdiHandle, cobsEncode(fitDefMsg), "FIT_DEFINITION Status (APPLIED)")
                        Log.i("LitertPlugin", "Replied APPLIED to FIT_DEFINITION (5011)")
                    }
                    5012 -> { // FIT_DATA (5012)
                        if (decoded.size > 6) {
                            val dataPayload = decoded.copyOfRange(4, decoded.size - 2)
                            fitRecordStream.write(dataPayload)
                        }
                        val fitDataAck = ByteBuffer.allocate(4).order(ByteOrder.LITTLE_ENDIAN)
                        fitDataAck.putShort(5012.toShort())
                        fitDataAck.put(0.toByte()) // ACK
                        fitDataAck.put(0.toByte()) // APPLIED
                        val fitDataMsg = buildGfdiMessage(5000, fitDataAck.array())
                        sendGfdiPacket(gfdiHandle, cobsEncode(fitDataMsg), "FIT_DATA Status (APPLIED)")
                        Log.i("LitertPlugin", "Replied APPLIED to FIT_DATA (5012)")
                    }
                    5052 -> { // CURRENT_TIME_REQUEST (5052)
                        if (decoded.size >= 8) {
                            val refId = ByteBuffer.wrap(decoded, 4, 4).order(ByteOrder.LITTLE_ENDIAN).int
                            val nowGarmin = (System.currentTimeMillis() / 1000) - 631065600
                            val tzOffset = java.util.TimeZone.getDefault().getOffset(System.currentTimeMillis()) / 1000
                            val respPayload = ByteBuffer.allocate(21).order(ByteOrder.LITTLE_ENDIAN)
                            respPayload.putShort(5052.toShort())       // target 5052
                            respPayload.put(0.toByte())                // ACK
                            respPayload.putInt(refId)
                            respPayload.putInt(nowGarmin.toInt())
                            respPayload.putInt(tzOffset)
                            respPayload.putInt(0)
                            respPayload.putInt(0)
                            val respMsg = buildGfdiMessage(5000, respPayload.array())
                            sendGfdiPacket(gfdiHandle, cobsEncode(respMsg), "CURRENT_TIME Response")
                            Log.i("LitertPlugin", "Replied to CURRENT_TIME_REQUEST with timestamp $nowGarmin")
                        }
                    }
                    5036 -> { // NOTIFICATION_SUBSCRIPTION (5036)
                        val notifAckPayload = ByteBuffer.allocate(6).order(ByteOrder.LITTLE_ENDIAN)
                        notifAckPayload.putShort(5036.toShort()) // target message ID
                        notifAckPayload.put(0.toByte())          // Status.ACK (0)
                        notifAckPayload.put(0.toByte())          // NotificationStatus.ENABLED (0)
                        notifAckPayload.put(1.toByte())          // enable (1)
                        notifAckPayload.put(0.toByte())          // unk (0)
                        val notifAckMsg = buildGfdiMessage(5000, notifAckPayload.array())
                        sendGfdiPacket(gfdiHandle, cobsEncode(notifAckMsg), "NotificationSubscription Status (5036)")
                        Log.i("LitertPlugin", "Replied Status ACK to NOTIFICATION_SUBSCRIPTION (5036)")

                        triggerSyncSequence()
                    }
                    5037 -> { // SYNCHRONIZATION (5037)
                        val syncAckPayload = ByteBuffer.allocate(3).order(ByteOrder.LITTLE_ENDIAN)
                        syncAckPayload.putShort(5037.toShort())
                        syncAckPayload.put(0.toByte()) // ACK
                        val syncAckMsg = buildGfdiMessage(5000, syncAckPayload.array())
                        sendGfdiPacket(gfdiHandle, cobsEncode(syncAckMsg), "SYNCHRONIZATION ACK (5037)")

                        val filterMsg = buildGfdiMessage(5007, byteArrayOf(3.toByte()))
                        sendGfdiPacket(gfdiHandle, cobsEncode(filterMsg), "FilterMessage")
                        Log.i("LitertPlugin", "Replied to SYNCHRONIZATION with FilterMessage")
                    }
                    5043, 5044 -> { // PROTOBUF REQUEST / RESPONSE (5043, 5044)
                        if (decoded.size >= 18) {
                            val protoRequestId = ByteBuffer.wrap(decoded, 4, 2).order(ByteOrder.LITTLE_ENDIAN).short.toInt() and 0xFFFF
                            val protoOffset = ByteBuffer.wrap(decoded, 6, 4).order(ByteOrder.LITTLE_ENDIAN).int
                            val protoTotalLen = ByteBuffer.wrap(decoded, 10, 4).order(ByteOrder.LITTLE_ENDIAN).int
                            val protoDataLen = ByteBuffer.wrap(decoded, 14, 4).order(ByteOrder.LITTLE_ENDIAN).int
                            val chunkBytes = decoded.copyOfRange(18, (18 + protoDataLen).coerceAtMost(decoded.size - 2))

                            // Send official ProtobufStatusMessage ACK (11 bytes inside GFDI 5000 payload)
                            val protoStatusPayload = ByteBuffer.allocate(11).order(ByteOrder.LITTLE_ENDIAN)
                            protoStatusPayload.putShort(messageId.toShort())
                            protoStatusPayload.put(0.toByte()) // ACK
                            protoStatusPayload.putShort(protoRequestId.toShort())
                            protoStatusPayload.putInt(protoOffset)
                            protoStatusPayload.put(0.toByte()) // KEPT
                            protoStatusPayload.put(0.toByte()) // NO_ERROR
                            val protoStatusMsg = buildGfdiMessage(5000, protoStatusPayload.array())
                            sendGfdiPacket(gfdiHandle, cobsEncode(protoStatusMsg), "Protobuf Status ACK ($messageId req $protoRequestId @$protoOffset)")
                            Log.i("LitertPlugin", "Replied Protobuf Status ACK to msg $messageId req $protoRequestId offset $protoOffset")

                            // Reassemble chunked payload with strict offset verification
                            val state = protobufReassemblyMap.getOrPut(protoRequestId) {
                                ProtobufTransfer(protoTotalLen)
                            }
                            if (protoOffset != state.nextOffset) {
                                Log.w("LitertPlugin", "Unexpected protobuf offset for req $protoRequestId: expected=${state.nextOffset}, got=$protoOffset")
                                return
                            }
                            state.bytes.write(chunkBytes)
                            state.nextOffset += chunkBytes.size

                            Log.i("LitertPlugin", "Protobuf Chunk received (req $protoRequestId, offset $protoOffset, chunk ${chunkBytes.size}b, accumulated ${state.bytes.size()}b / $protoTotalLen total)")

                            // If payload is completely received, process full message
                            if (state.nextOffset == state.totalLength) {
                                val protoBytes = state.bytes.toByteArray()
                                protobufReassemblyMap.remove(protoRequestId)
                                Log.i("LitertPlugin", "Processing COMPLETE Protobuf Payload (req $protoRequestId, ${protoBytes.size}b): ${bytesToHex(protoBytes.take(64).toByteArray())}...")

                                val fit = tryExtractCompleteFit(protoBytes)
                                if (fit != null) {
                                    val b64 = Base64.encodeToString(fit, Base64.NO_WRAP)
                                    if (!fitFilesList.contains(b64)) {
                                        fitFilesList.add(b64)
                                        Log.i("LitertPlugin", "Captured authentic .FIT activity from Protobuf payload (${fit.size} bytes)!")
                                    }
                                }

                                // If watch sent an HTTP request, reply with HTTP 200 OK
                                if (protoBytes.isNotEmpty() && String(protoBytes).contains("http")) {
                                    val httpAck = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n".toByteArray()
                                    val httpAckPayload = ByteBuffer.allocate(14 + httpAck.size).order(ByteOrder.LITTLE_ENDIAN)
                                    httpAckPayload.putShort(protoRequestId.toShort())
                                    httpAckPayload.putInt(0)
                                    httpAckPayload.putInt(httpAck.size)
                                    httpAckPayload.putInt(httpAck.size)
                                    httpAckPayload.put(httpAck)
                                    val httpAckMsg = buildGfdiMessage(5044, httpAckPayload.array())
                                    sendGfdiPacket(gfdiHandle, cobsEncode(httpAckMsg), "Protobuf HTTP 200 OK")
                                    Log.i("LitertPlugin", "Replied HTTP 200 OK to watch request $protoRequestId")
                                }

                                fun readVarint(bytes: ByteArray, offset: Int): Pair<Long, Int> {
                                    var result = 0L
                                    var shift = 0
                                    var pos = offset
                                    while (pos < bytes.size) {
                                        val b = bytes[pos].toInt() and 0xFF
                                        result = result or ((b and 0x7F).toLong() shl shift)
                                        pos++
                                        if ((b and 0x80) == 0) break
                                        shift += 7
                                    }
                                    return Pair(result, pos - offset)
                                }

                                fun getFileTypeCodeAndName(candidate: ByteArray): Pair<Int?, String?> {
                                    var code: Int? = null
                                    var name: String? = null
                                    var cp = 0
                                    while (cp < candidate.size - 2) {
                                        val (tRaw, tBytes) = readVarint(candidate, cp)
                                        cp += tBytes
                                        val wType = (tRaw and 0x07).toInt()
                                        val fNum = (tRaw shr 3).toInt()
                                        if (wType == 2) {
                                            val (lVal, lBytes) = readVarint(candidate, cp)
                                            cp += lBytes
                                            val sLen = lVal.toInt()
                                            val sEnd = (cp + sLen).coerceAtMost(candidate.size)
                                            val sBytes = candidate.copyOfRange(cp, sEnd)
                                            cp = sEnd
                                            if (fNum == 2) { // FileType submessage
                                                var tp = 0
                                                while (tp < sBytes.size - 1) {
                                                    val (subTRaw, subTBytes) = readVarint(sBytes, tp)
                                                    tp += subTBytes
                                                    val subWType = (subTRaw and 0x07).toInt()
                                                    val subFNum = (subTRaw shr 3).toInt()
                                                    if (subWType == 2 && subFNum == 2) { // string name
                                                        val (strLenVal, strLenBytes) = readVarint(sBytes, tp)
                                                        tp += strLenBytes
                                                        val strLen = strLenVal.toInt()
                                                        val strEnd = (tp + strLen).coerceAtMost(sBytes.size)
                                                        name = String(sBytes.copyOfRange(tp, strEnd), java.nio.charset.StandardCharsets.UTF_8)
                                                        tp = strEnd
                                                    } else if (subWType == 0 && subFNum == 3) { // varint code
                                                        val (codeVal, cBytes) = readVarint(sBytes, tp)
                                                        code = codeVal.toInt()
                                                        tp += cBytes
                                                    } else if (subWType == 0) {
                                                        val (_, vb) = readVarint(sBytes, tp)
                                                        tp += vb
                                                    } else {
                                                        tp++
                                                    }
                                                }
                                            }
                                        } else if (wType == 0) {
                                            val (_, vB) = readVarint(candidate, cp)
                                            cp += vB
                                        } else if (wType == 1) {
                                            cp += 8
                                        } else if (wType == 5) {
                                            cp += 4
                                        } else {
                                            cp++
                                        }
                                    }
                                    return Pair(code, name)
                                }

                                 fun isPullableFile(candidate: ByteArray): Boolean {
                                     val (code, name) = getFileTypeCodeAndName(candidate)
                                     val resolvedName = name ?: (if (code != null) fileTypeCodeMap[code] else null)
                                     val fitSubtype = resolvedName
                                         ?.takeIf { it.startsWith("FIT_TYPE_") }
                                         ?.removePrefix("FIT_TYPE_")
                                         ?.toIntOrNull() ?: (if (resolvedName != null) null else code)

                                     Log.i("LitertPlugin", "File candidate [${candidate.size}b]: subtype=$fitSubtype, name=$resolvedName (code=$code) -> ACCEPTED (pullable=true)")
                                     return true
                                 }

                                 // Global FileType dictionary scanner across any protobuf message
                                 var dtP = 0
                                 while (dtP < protoBytes.size - 4) {
                                     if (protoBytes[dtP] == 0x08.toByte()) {
                                         val (tIdVal, tIdLen) = readVarint(protoBytes, dtP + 1)
                                         val namePos = dtP + 1 + tIdLen
                                         if (namePos < protoBytes.size - 2 && protoBytes[namePos] == 0x12.toByte()) {
                                             val (nameLenVal, nameLenBytes) = readVarint(protoBytes, namePos + 1)
                                             val nLen = nameLenVal.toInt()
                                             val nStart = namePos + 1 + nameLenBytes
                                             if (nLen in 3..40 && nStart + nLen <= protoBytes.size) {
                                                 val strBytes = protoBytes.copyOfRange(nStart, nStart + nLen)
                                                 val str = String(strBytes, java.nio.charset.StandardCharsets.UTF_8)
                                                 if (str.all { it.isLetterOrDigit() || it == '_' }) {
                                                     fileTypeCodeMap[tIdVal.toInt()] = str
                                                     Log.i("LitertPlugin", "Learned FileType mapping: ${tIdVal.toInt()} -> $str")
                                                 }
                                             }
                                         }
                                     }
                                     dtP++
                                 }

                                 // 1. Scan for FileSyncService (tag 43: 0xDA 0x02)
                                 var sPos = 0
                                 while (sPos < protoBytes.size - 2) {
                                     if (protoBytes[sPos] == 0xDA.toByte() && protoBytes[sPos + 1] == 0x02.toByte()) {
                                         val (fssLenVal, fssLenBytes) = readVarint(protoBytes, sPos + 2)
                                         val fssLen = fssLenVal.toInt()
                                         val fssStart = sPos + 2 + fssLenBytes
                                         val fssEnd = (fssStart + fssLen).coerceAtMost(protoBytes.size)
                                         if (fssStart < fssEnd) {
                                             val fssBytes = protoBytes.copyOfRange(fssStart, fssEnd)
                                             var p = 0
                                             while (p < fssBytes.size - 1) {
                                                 val (tagRaw, tagLen) = readVarint(fssBytes, p)
                                                 p += tagLen
                                                 val tagNum = (tagRaw shr 3).toInt()
                                                 val wireType = (tagRaw and 0x07).toInt()
                                                 if (wireType == 2) {
                                                     val (fieldLenVal, fieldLenBytes) = readVarint(fssBytes, p)
                                                     p += fieldLenBytes
                                                     val fieldLen = fieldLenVal.toInt()
                                                     val fieldEnd = (p + fieldLen).coerceAtMost(fssBytes.size)
                                                     val fieldBytes = fssBytes.copyOfRange(p, fieldEnd)
                                                     p = fieldEnd

                                                     when (tagNum) {
                                                         2 -> { // FileResponse (tag 2)
                                                             var fStatus: Int? = null
                                                             var fHandle: Int? = null
                                                             var fp = 0
                                                             while (fp < fieldBytes.size - 1) {
                                                                 val (fTag, fTagLen) = readVarint(fieldBytes, fp)
                                                                 fp += fTagLen
                                                                 val fTagNum = (fTag shr 3).toInt()
                                                                 val fWire = (fTag and 0x07).toInt()
                                                                 if (fWire == 0) {
                                                                     val (v, vb) = readVarint(fieldBytes, fp)
                                                                     fp += vb
                                                                     if (fTagNum == 1) fStatus = v.toInt()
                                                                     if (fTagNum == 3) fHandle = v.toInt()
                                                                 } else {
                                                                     fp++
                                                                 }
                                                             }
                                                             if (fStatus == 0 && (fHandle ?: 0) > 0) {
                                                                 statFileResponsesOk++
                                                                 lastProtocolProgressAt = System.currentTimeMillis()
                                                                 lastProgressDescription = "FileResponse accepted with handle $fHandle"
                                                                 val transfer = transferByRequestId[protoRequestId] ?: activeTransfer
                                                                 if (transfer != null) {
                                                                     transfer.garminFileHandle = fHandle
                                                                 }
                                                                 Log.i("LitertPlugin", "FileResponse SUCCESS with garminFileHandle: $fHandle (protoReq $protoRequestId). Registering FILE_TRANSFER_2 (0x2018)...")
                                                                 val regTransfer = ByteBuffer.allocate(13).order(ByteOrder.LITTLE_ENDIAN)
                                                                 regTransfer.put(0.toByte()) // handle 0
                                                                 regTransfer.put(0.toByte()) // REGISTER_ML_REQ
                                                                 regTransfer.putLong(2L)     // Client ID 2
                                                                 regTransfer.putShort(0x2018.toShort()) // Service 0x2018
                                                                 regTransfer.put(0.toByte()) // reliable = false (0)
                                                                 writeQueue?.enqueue(regTransfer.array())
                                                             } else {
                                                                  statFileResponsesRejected++
                                                                  lastProtocolProgressAt = System.currentTimeMillis()
                                                                  Log.i("LitertPlugin", "Protobuf FileResponse status $fStatus for req $protoRequestId. Skipping and advancing queue...")
                                                                  val rejected = transferByRequestId.remove(protoRequestId) ?: activeTransfer
                                                                  if (rejected != null) {
                                                                      markFileDescriptorSynced(rejected.fileDescriptor)
                                                                  }
                                                                  activeTransfer = null
                                                                  startNextFileDownload()
                                                              }
                                                         }
                                                         10 -> { // FileListResponse (tag 10)
                                                             statPagesEnumerated++
                                                             var flrCursorId: Int? = null
                                                             var flrNextPageId: Int? = null
                                                             var fp = 0
                                                             while (fp < fieldBytes.size - 1) {
                                                                 val (fTag, fTagLen) = readVarint(fieldBytes, fp)
                                                                 fp += fTagLen
                                                                 val fTagNum = (fTag shr 3).toInt()
                                                                 val fWire = (fTag and 0x07).toInt()
                                                                 if (fWire == 0) {
                                                                     val (v, vb) = readVarint(fieldBytes, fp)
                                                                     fp += vb
                                                                     if (fTagNum == 2) flrCursorId = v.toInt()
                                                                     if (fTagNum == 3) flrNextPageId = v.toInt()
                                                                 } else if (fWire == 2 && fTagNum == 4) { // file
                                                                     statDescriptorsDiscovered++
                                                                     val (subLenVal, subLenBytes) = readVarint(fieldBytes, fp)
                                                                     fp += subLenBytes
                                                                     val subLen = subLenVal.toInt()
                                                                     val sEnd = (fp + subLen).coerceAtMost(fieldBytes.size)
                                                                     val fileSub = fieldBytes.copyOfRange(fp, sEnd)
                                                                     fp = sEnd
                                                                     if (isPullableFile(fileSub)) {
                                                                         val (c, n) = getFileTypeCodeAndName(fileSub)
                                                                         Log.i("LitertPlugin", "Discovered File entry in FileListResponse (${fileSub.size}b, code $c, name $n)")
                                                                         enqueueFileIfNew(fileSub)
                                                                     } else {
                                                                         statDescriptorsFiltered++
                                                                     }
                                                                 } else if (fWire == 2 && (fTagNum == 1 || fTagNum == 2 || fTagNum == 3)) { // FileType definitions in FileListResponse
                                                                     val (subLenVal, subLenBytes) = readVarint(fieldBytes, fp)
                                                                     fp += subLenBytes
                                                                     val subLen = subLenVal.toInt()
                                                                     val sEnd = (fp + subLen).coerceAtMost(fieldBytes.size)
                                                                     val typeSub = fieldBytes.copyOfRange(fp, sEnd)
                                                                     fp = sEnd
                                                                     
                                                                     var tp = 0
                                                                     var tId: Int? = null
                                                                     var tName: String? = null
                                                                     while (tp < typeSub.size - 1) {
                                                                         val (tTag, tTagLen) = readVarint(typeSub, tp)
                                                                         tp += tTagLen
                                                                         val tTagNum = (tTag shr 3).toInt()
                                                                         val tWire = (tTag and 0x07).toInt()
                                                                         if (tWire == 0) {
                                                                             val (v, vb) = readVarint(typeSub, tp)
                                                                             tp += vb
                                                                             if (tTagNum == 1 || tTagNum == 3) tId = v.toInt()
                                                                         } else if (tWire == 2) {
                                                                             val (lVal, lBytes) = readVarint(typeSub, tp)
                                                                             tp += lBytes
                                                                             val strLen = lVal.toInt()
                                                                             val strEnd = (tp + strLen).coerceAtMost(typeSub.size)
                                                                             if (tTagNum == 2) {
                                                                                 tName = String(typeSub.copyOfRange(tp, strEnd), java.nio.charset.StandardCharsets.UTF_8)
                                                                             }
                                                                             tp = strEnd
                                                                         } else {
                                                                             tp++
                                                                         }
                                                                     }
                                                                     if (tId != null && tName != null) {
                                                                         fileTypeCodeMap[tId] = tName
                                                                         Log.i("LitertPlugin", "Discovered FileType in FileListResponse: $tId -> $tName")
                                                                     }
                                                                 } else if (fWire == 2) {
                                                                     val (subLenVal, subLenBytes) = readVarint(fieldBytes, fp)
                                                                     fp += subLenBytes + subLenVal.toInt()
                                                                 } else if (fWire == 1) {
                                                                     fp += 8
                                                                 } else if (fWire == 5) {
                                                                     fp += 4
                                                                 } else {
                                                                     fp++
                                                                 }
                                                             }
                                                              when {
                                                                flrCursorId != null && requestedCursorIds.add(flrCursorId) -> {
                                                                    lastProtocolProgressAt = System.currentTimeMillis()
                                                                    lastProgressDescription = "Requesting next file-list cursor $flrCursorId"
                                                                    Log.i("LitertPlugin", "Requesting next file-list cursor: $flrCursorId")
                                                                    sendFileListRequest(cursorId = flrCursorId)
                                                                }
                                                                flrNextPageId != null && requestedPageIds.add(flrNextPageId) -> {
                                                                    lastProtocolProgressAt = System.currentTimeMillis()
                                                                    lastProgressDescription = "Requesting next file-list page $flrNextPageId"
                                                                    Log.i("LitertPlugin", "Requesting next file-list page: $flrNextPageId")
                                                                    sendFileListRequest(startPageId = flrNextPageId)
                                                                }
                                                                else -> {
                                                                    fileListFinished = true
                                                                    lastProtocolProgressAt = System.currentTimeMillis()
                                                                    lastProgressDescription = "File list complete with ${pendingFilesQueue.size} files queued"
                                                                    Log.i("LitertPlugin", "FileList complete: ${pendingFilesQueue.size} files discovered across $statPagesEnumerated pages. Starting sequential download...")
                                                                    startNextFileDownload()
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else if (wireType == 0) {
                                                    val (_, vb) = readVarint(fssBytes, p)
                                                    p += vb
                                                } else if (wireType == 1) {
                                                    p += 8
                                                } else if (wireType == 5) {
                                                    p += 4
                                                } else {
                                                    p++
                                                }
                                            }
                                        }
                                        sPos = fssEnd
                                    } else {
                                        sPos++
                                    }
                                }

                                // 3. Recursively scan other protobuf fields for File entries
                                var p = 0
                                while (p < protoBytes.size - 1) {
                                    val (tagRaw, tagBytes) = readVarint(protoBytes, p)
                                    p += tagBytes
                                    val wireType = (tagRaw and 0x07).toInt()

                                    if (wireType == 2) {
                                        if (p >= protoBytes.size) break
                                        val (lenVal, lenBytes) = readVarint(protoBytes, p)
                                        p += lenBytes
                                        val subLen = lenVal.toInt()
                                        val subEnd = (p + subLen).coerceAtMost(protoBytes.size)
                                        val subBytes = protoBytes.copyOfRange(p, subEnd)
                                        p = subEnd

                                        if (subBytes.size >= 18 && subBytes[0] == 0x0A.toByte() && subBytes[1] == 0x12.toByte()) {
                                            if (isPullableFile(subBytes)) {
                                                val (c, n) = getFileTypeCodeAndName(subBytes)
                                                Log.i("LitertPlugin", "Discovered authentic File entry (${subBytes.size}b, code $c, name $n)")
                                                enqueueFileIfNew(subBytes)
                                            }
                                        }

                                        // Scan subBytes recursively for nested File entries
                                        var subP = 0
                                        while (subP < subBytes.size - 4) {
                                            if (subBytes[subP] == 0x22.toByte() || subBytes[subP] == 0x0A.toByte()) {
                                                val (candLenVal, candLenBytes) = readVarint(subBytes, subP + 1)
                                                val candLen = candLenVal.toInt()
                                                val candStart = subP + 1 + candLenBytes
                                                if (candLen in 10..250 && candStart + candLen <= subBytes.size) {
                                                    val candidate = subBytes.copyOfRange(candStart, candStart + candLen)
                                                    if (candidate.size >= 8 && candidate[0] == 0x0A.toByte()) {
                                                        if (isPullableFile(candidate)) {
                                                            val (c, n) = getFileTypeCodeAndName(candidate)
                                                            Log.i("LitertPlugin", "Found nested Activity File entry (${candidate.size}b, code $c, name $n)")
                                                            enqueueFileIfNew(candidate)
                                                        }
                                                    }
                                                }
                                            }
                                            subP++
                                        }
                                    } else if (wireType == 0) {
                                        val (_, vBytes) = readVarint(protoBytes, p)
                                        p += vBytes
                                    } else if (wireType == 1) {
                                        p += 8
                                    } else if (wireType == 5) {
                                        p += 4
                                    } else {
                                        p++
                                    }
                                }
                            }
                        }
                    }
                    5004 -> { // FILE_TRANSFER_DATA (5004)
                        val flags = decoded[4].toInt() and 0xFF
                        val dataOffset = ByteBuffer.wrap(decoded, 7, 4).order(ByteOrder.LITTLE_ENDIAN).int
                        val chunkPayload = decoded.copyOfRange(11, (decoded.size - 2).coerceAtLeast(11))
                        val nextOffset = dataOffset + chunkPayload.size

                        // Send FileTransferDataStatus ACK targeting 5004
                        val ackPayload = ByteBuffer.allocate(8).order(ByteOrder.LITTLE_ENDIAN)
                        ackPayload.putShort(5004.toShort()) // target 5004
                        ackPayload.put(0.toByte()) // ACK
                        ackPayload.put(0.toByte()) // OK
                        ackPayload.putInt(nextOffset)
                        val ackMsg = buildGfdiMessage(5000, ackPayload.array())
                        sendGfdiPacket(gfdiHandle, cobsEncode(ackMsg), "FileTransferDataStatus ACK ($nextOffset for 5004)")
                        Log.i("LitertPlugin", "ACKed FILE_TRANSFER_DATA (5004): offset $dataOffset -> $nextOffset (chunk ${chunkPayload.size}b, flags $flags)")

                        val fileBuffer = fileBuffers.getOrPut(0) { ByteArrayOutputStream() }
                        if (dataOffset == 0) fileBuffer.reset()
                        fileBuffer.write(chunkPayload)
                        val fullBuffer = fileBuffer.toByteArray()
                        val fit = tryExtractCompleteFit(fullBuffer) ?: tryExtractCompleteFit(chunkPayload)
                        if (fit != null) {
                            val b64 = Base64.encodeToString(fit, Base64.NO_WRAP)
                            if (!fitFilesList.contains(b64)) {
                                fitFilesList.add(b64)
                                statFitsExtracted++
                                lastProtocolProgressAt = System.currentTimeMillis()
                                Log.i("LitertPlugin", "SUCCESS: Captured authentic .FIT activity file (${fit.size} bytes) from GFDI stream!")
                            }
                        }
                    }
                    5030 -> { // SYSTEM_EVENT
                        val eventType = decoded[4].toInt() and 0xFF
                        Log.i("LitertPlugin", "Garmin SystemEvent: $eventType (0=SYNC_COMPLETE)")
                        if (eventType == 0) { // SYNC_COMPLETE
                            Log.i("LitertPlugin", "Received SYNC_COMPLETE from Garmin watch. Queue status: ${pendingFilesQueue.size} pending, activeTransfer=${activeTransfer != null}")
                            lastProtocolProgressAt = System.currentTimeMillis()
                            lastProgressDescription = "Received watch SYNC_COMPLETE"
                        }
                    }
                    else -> {
                        val genAck = ByteBuffer.allocate(3).order(ByteOrder.LITTLE_ENDIAN)
                        genAck.putShort(messageId.toShort())
                        genAck.put(0.toByte()) // ACK
                        val genMsg = buildGfdiMessage(5000, genAck.array())
                        sendGfdiPacket(gfdiHandle, cobsEncode(genMsg), "Generic ACK ($messageId)")
                        Log.i("LitertPlugin", "Replied Generic ACK to watch message ID $messageId")
                    }
                }
            }

            val gattCallback = object : BluetoothGattCallback() {
                override fun onConnectionStateChange(gatt: BluetoothGatt?, status: Int, newState: Int) {
                    Log.d("LitertPlugin", "onConnectionStateChange: status=$status, newState=$newState")
                    if (status == BluetoothGatt.GATT_SUCCESS && newState == BluetoothProfile.STATE_CONNECTED) {
                        isConnected = true
                        Log.i("LitertPlugin", "Successfully connected to Garmin GATT: $macAddress")
                        gatt?.discoverServices()
                    } else if (newState == BluetoothProfile.STATE_DISCONNECTED) {
                        Log.w("LitertPlugin", "GATT disconnected from Garmin watch")
                        if (!hasFinished) {
                            finish(gatt, batteryLevel)
                        }
                    }
                }

                override fun onMtuChanged(gatt: BluetoothGatt?, mtu: Int, status: Int) {
                    Log.i("LitertPlugin", "GATT onMtuChanged: mtu=$mtu, status=$status")
                    if (batteryChar != null && batteryLevel == null && gatt != null) {
                        gatt.readCharacteristic(batteryChar)
                    }
                    if (rxChar != null && gatt != null) {
                        gatt.setCharacteristicNotification(rxChar, true)
                        val descriptor = rxChar?.getDescriptor(UUID.fromString("00002902-0000-1000-8000-00805f9b34fb"))
                        if (descriptor != null) {
                            descriptor.value = BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE
                            gatt.writeDescriptor(descriptor)
                        }
                    }
                }

                override fun onServicesDiscovered(gatt: BluetoothGatt?, status: Int) {
                    Log.d("LitertPlugin", "onServicesDiscovered: status=$status")
                    if (status == BluetoothGatt.GATT_SUCCESS && gatt != null) {
                        isServicesDiscovered = true
                        for (service in gatt.services) {
                            Log.d("LitertPlugin", "Discovered GATT Service: ${service.uuid}")
                            for (char in service.characteristics) {
                                val uuidStr = char.uuid.toString().uppercase()
                                Log.d("LitertPlugin", "  Characteristic: $uuidStr (props=0x${Integer.toHexString(char.properties)})")
                                if (uuidStr.contains("2810") || uuidStr.contains("2811") || uuidStr.contains("2812")) {
                                    rxChar = char
                                } else if (uuidStr.contains("2820") || uuidStr.contains("2821") || uuidStr.contains("2822")) {
                                    txChar = char
                                } else if (uuidStr.contains("2A19")) {
                                    batteryChar = char
                                }
                            }
                        }

                        if (batteryChar != null) {
                            Log.i("LitertPlugin", "Discovered Battery Level characteristic (${batteryChar?.uuid}). Reading watch battery...")
                            gatt.readCharacteristic(batteryChar)
                        }

                        if (rxChar != null && txChar != null) {
                            val canWriteDefault = supportsProperty(txChar!!, BluetoothGattCharacteristic.PROPERTY_WRITE)
                            val canWriteNoResponse = supportsProperty(txChar!!, BluetoothGattCharacteristic.PROPERTY_WRITE_NO_RESPONSE)
                            Log.i("LitertPlugin", "TX properties: WRITE=$canWriteDefault, WRITE_NO_RESPONSE=$canWriteNoResponse, raw=0x${txChar!!.properties.toString(16)}")

                            writeQueue = if (canWriteDefault) {
                                Log.i("LitertPlugin", "Using ResponseWriteQueue with WRITE_TYPE_DEFAULT")
                                ResponseWriteQueue(gatt, txChar!!) { err ->
                                    Log.e("LitertPlugin", "ResponseWriteQueue error: $err")
                                    finish(gatt, batteryLevel, err)
                                }
                            } else if (canWriteNoResponse) {
                                Log.i("LitertPlugin", "Using NoResponseWriteQueue with WRITE_TYPE_NO_RESPONSE")
                                NoResponseWriteQueue(gatt, txChar!!, scope) { err ->
                                    Log.e("LitertPlugin", "NoResponseWriteQueue error: $err")
                                    finish(gatt, batteryLevel, err)
                                }
                            } else {
                                finish(gatt, batteryLevel, "Garmin TX characteristic has neither PROPERTY_WRITE nor PROPERTY_WRITE_NO_RESPONSE")
                                return
                            }

                            Log.i("LitertPlugin", "Found Garmin GFDI characteristic pair: RX ${rxChar?.uuid}, TX ${txChar?.uuid}. Requesting MTU 247...")
                            if (!gatt.requestMtu(247)) {
                                gatt.setCharacteristicNotification(rxChar, true)
                                val descriptor = rxChar?.getDescriptor(UUID.fromString("00002902-0000-1000-8000-00805f9b34fb"))
                                if (descriptor != null) {
                                    descriptor.value = BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE
                                    gatt.writeDescriptor(descriptor)
                                }
                            }
                        } else {
                            Log.w("LitertPlugin", "Garmin GFDI RX/TX characteristics not found on this device")
                            if (batteryChar != null) {
                                gatt.readCharacteristic(batteryChar)
                            } else {
                                finish(gatt, batteryLevel)
                            }
                        }
                    } else {
                        finish(gatt, null, "Failed to discover GATT services on watch (status $status)")
                    }
                }

                override fun onCharacteristicWrite(gatt: BluetoothGatt?, characteristic: BluetoothGattCharacteristic?, status: Int) {
                    writeQueue?.onWriteComplete(status)
                }

                override fun onDescriptorWrite(gatt: BluetoothGatt?, descriptor: BluetoothGattDescriptor?, status: Int) {
                    Log.d("LitertPlugin", "onDescriptorWrite: status=$status, desc=${descriptor?.uuid}")
                    if (status == BluetoothGatt.GATT_SUCCESS && txChar != null && gatt != null) {
                        Log.i("LitertPlugin", "Notifications enabled on Garmin RX. Sending MLR CLOSE_ALL...")
                        val closeAll = ByteBuffer.allocate(13).order(ByteOrder.LITTLE_ENDIAN)
                        closeAll.put(0.toByte()) // Handle 0
                        closeAll.put(5.toByte()) // RequestType.CLOSE_ALL_REQ
                        closeAll.putLong(2L)     // Client ID 2
                        closeAll.putShort(0)
                        writeQueue?.enqueue(closeAll.array())
                    }
                }

                override fun onCharacteristicChanged(gatt: BluetoothGatt?, characteristic: BluetoothGattCharacteristic?) {
                    val bytes = characteristic?.value ?: return
                    if (bytes.isEmpty()) return
                    lastActivityTime = System.currentTimeMillis()
                    Log.d("LitertPlugin", "RX NOTIFY [${bytes.size}b]: ${bytesToHex(bytes)}")

                    // Check for ML Control responses (handle 0)
                    if (bytes[0] == 0.toByte() && bytes.size >= 2) {
                        val msgType = bytes[1].toInt() and 0xFF
                        if (msgType == 6) { // CLOSE_ALL_RESP
                            Log.i("LitertPlugin", "Watch confirmed CLOSE_ALL_RESP. Registering GFDI service 1...")
                            val registerGfdi = ByteBuffer.allocate(13).order(ByteOrder.LITTLE_ENDIAN)
                            registerGfdi.put(0.toByte())
                            registerGfdi.put(0.toByte()) // RequestType.REGISTER_ML_REQ
                            registerGfdi.putLong(2L)     // Client ID 2
                            registerGfdi.putShort(1.toShort()) // Service 1 (GFDI)
                            registerGfdi.put(0.toByte()) // reliable = false
                            writeQueue?.enqueue(registerGfdi.array())
                        } else if (msgType == 1 && bytes.size >= 14) { // REGISTER_ML_RESP
                            val serviceCode = ByteBuffer.wrap(bytes, 10, 2).order(ByteOrder.LITTLE_ENDIAN).short
                            val status = bytes[12].toInt() and 0xFF
                            val handle = bytes[13].toInt() and 0xFF
                            Log.i("LitertPlugin", "Watch REGISTER_ML_RESP: service=$serviceCode, status=$status, handle=$handle")
                            if (status == 0) {
                                if (serviceCode == 1.toShort()) {
                                    gfdiHandle = handle
                                    triggerSyncSequence()
                                } else if (serviceCode == 0x2018.toShort() && activeTransfer?.garminFileHandle != null) {
                                    val transfer = activeTransfer!!
                                    fileTransferHandle = handle
                                    transfer.mlrTransportHandle = handle
                                    transferByMlrHandle[handle] = transfer
                                    Log.i("LitertPlugin", "FILE_TRANSFER_2 handle registered: $handle for file ${transfer.garminFileHandle}. Requesting stream...")
                                    val fileReq = ByteBuffer.allocate(6).order(ByteOrder.LITTLE_ENDIAN)
                                    fileReq.put(0.toByte())
                                    fileReq.put(0.toByte())
                                    fileReq.putShort(transfer.garminFileHandle!!.toShort())
                                    fileReq.put(0.toByte())
                                    fileReq.put(0.toByte())
                                    val packet = byteArrayOf(handle.toByte()) + fileReq.array()
                                    Log.i("LitertPlugin", "OFFLOAD TX: transportHandle=$handle, fileHandle=${transfer.garminFileHandle}, bytes=${bytesToHex(packet, 128)}")
                                    writeQueue?.enqueue(packet)
                                }
                            }
                        } else if (msgType == 3 || msgType == 4) { // CLOSE_HANDLE_REQ or CLOSE_HANDLE_RESP
                            val closedHandle = if (bytes.size >= 14) bytes[12].toInt() and 0xFF else bytes[bytes.size - 1].toInt() and 0xFF
                            Log.i("LitertPlugin", "Watch closed handle $closedHandle (msgType $msgType)")
                            val transfer = transferByMlrHandle.remove(closedHandle) ?: activeTransfer
                            if (transfer != null) {
                                markFileDescriptorSynced(transfer.fileDescriptor)
                                if (transfer.bytes.size() > 0) {
                                    val rawBytes = transfer.bytes.toByteArray()
                                    Log.i("LitertPlugin", "OFFLOAD COMPLETE for handle $closedHandle (${rawBytes.size} bytes captured). Inflating & Extracting FIT...")
                                    
                                    fun inflate(input: ByteArray): ByteArray? {
                                        var startIdx = 0
                                        while (startIdx < input.size && input[startIdx] == 0.toByte()) {
                                            startIdx++
                                        }
                                        val cleanInput = if (startIdx > 0) input.copyOfRange(startIdx, input.size) else input
                                        if (cleanInput.isEmpty()) return null

                                        for (nowrap in listOf(false, true)) {
                                            val inflater = java.util.zip.Inflater(nowrap)
                                            inflater.setInput(cleanInput)
                                            val baos = java.io.ByteArrayOutputStream(cleanInput.size * 3)
                                            val buf = ByteArray(8192)
                                            try {
                                                while (!inflater.finished()) {
                                                    val count = inflater.inflate(buf)
                                                    if (count == 0) {
                                                        if (inflater.needsInput() || inflater.needsDictionary()) break
                                                    }
                                                    baos.write(buf, 0, count)
                                                }
                                                inflater.end()
                                                val result = baos.toByteArray()
                                                if (result.isNotEmpty()) return result
                                            } catch (e: Exception) {
                                                inflater.end()
                                            }
                                        }
                                        return null
                                    }

                                    val inflated = inflate(rawBytes)
                                    val fit = (if (inflated != null) tryExtractCompleteFit(inflated) else null) ?: tryExtractCompleteFit(rawBytes)
                                    if (fit != null) {
                                        val b64 = Base64.encodeToString(fit, Base64.NO_WRAP)
                                        if (!fitFilesList.contains(b64)) {
                                            fitFilesList.add(b64)
                                            statFitsExtracted++
                                            lastProtocolProgressAt = System.currentTimeMillis()
                                            lastProgressDescription = "Extracted valid FIT activity (${fit.size}b)"
                                            Log.i("LitertPlugin", "SUCCESS: Extracted authentic .FIT activity (${fit.size} bytes) from offload stream (compressed ${rawBytes.size}b -> decompressed ${inflated?.size ?: 0}b)!")
                                        }
                                    } else {
                                        Log.d("LitertPlugin", "Stream payload (${rawBytes.size}b, inflated ${inflated?.size ?: 0}b) is non-FIT metadata. Skipping.")
                                    }
                                }
                                statOffloadsClosed++
                                lastProtocolProgressAt = System.currentTimeMillis()
                                fileTransferHandle = null
                                activeTransfer = null
                                startNextFileDownload()
                            }
                        }
                        return
                    }

                    // Check for incoming FileTransfer stream chunks
                    if (fileTransferHandle != null && (bytes[0].toInt() and 0xFF) == fileTransferHandle && bytes.size > 1) {
                        val streamChunk = bytes.copyOfRange(1, bytes.size)
                        val transfer = transferByMlrHandle[fileTransferHandle] ?: activeTransfer
                        if (transfer != null) {
                            if (transfer.bytes.size() == 0 && streamChunk.contentEquals(byteArrayOf(0, 0, 0))) {
                                Log.i("LitertPlugin", "OFFLOAD RX: Handshake [0,0,0] received. Stream starting...")
                                return
                            }
                            transfer.bytes.write(streamChunk)
                            lastProtocolProgressAt = System.currentTimeMillis()
                            lastProgressDescription = "Received offload chunk (${streamChunk.size}b, accumulated ${transfer.bytes.size()}b)"
                            Log.i("LitertPlugin", "OFFLOAD RX: transportHandle=$fileTransferHandle, chunk=${streamChunk.size}b (accumulated ${transfer.bytes.size()}b), head=${bytesToHex(streamChunk.take(32).toByteArray())}")
                        }
                        return
                    }

                    // Feed payload to GarminCobsDecoder for GFDI messages
                    if (gatt != null && bytes.size > 1) {
                        val handle = bytes[0].toInt() and 0xFF
                        if (handle == gfdiHandle || (handle and 0x80) != 0) {
                            val payload = bytes.copyOfRange(1, bytes.size)
                            val decodedList = garminCobsDecoder.receiveBytes(payload)
                            for (decoded in decodedList) {
                                processDecodedGfdiMessage(decoded, gatt)
                            }
                        }
                    }
                }

                override fun onCharacteristicRead(gatt: BluetoothGatt?, characteristic: BluetoothGattCharacteristic?, status: Int) {
                    Log.d("LitertPlugin", "onCharacteristicRead: char=${characteristic?.uuid}, status=$status")
                    if (status == BluetoothGatt.GATT_SUCCESS && characteristic != null) {
                        val valBytes = characteristic.value
                        if (valBytes != null && valBytes.isNotEmpty()) {
                            batteryLevel = valBytes[0].toInt() and 0xFF
                            Log.i("LitertPlugin", "Garmin watch battery read: $batteryLevel%")
                        }
                    }
                }
            }

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                gattRef = device.connectGatt(activity, false, gattCallback, BluetoothDevice.TRANSPORT_LE)
            } else {
                gattRef = device.connectGatt(activity, false, gattCallback)
            }

            scope.launch {
                val startTime = System.currentTimeMillis()
                while (!hasFinished) {
                    kotlinx.coroutines.delay(1000)
                    val elapsed = System.currentTimeMillis() - startTime
                    val noProgressFor = System.currentTimeMillis() - lastProtocolProgressAt

                    if (!isConnected && elapsed > 8000L) {
                        finish(gattRef, null, "BLE connection timed out. Garmin watch did not respond. Check Bluetooth range and ensure Garmin Connect app is disconnected.")
                        break
                    }

                    val isQueueDrained = fileListFinished && pendingFilesQueue.isEmpty() && activeTransfer == null

                    if (isQueueDrained && noProgressFor > 3000L && elapsed > 5000L) {
                        Log.i("LitertPlugin", "All file enumeration and downloads finished cleanly (${fitFilesList.size} FIT files captured).")
                        if (batteryChar != null && batteryLevel == null) {
                            gattRef?.readCharacteristic(batteryChar)
                            kotlinx.coroutines.delay(1000)
                        }
                        finish(gattRef, batteryLevel)
                        break
                    }

                    if (isConnected && noProgressFor > 30_000L) {
                        Log.w("LitertPlugin", "Sync stalled: no protocol progress for 30s at '$lastProgressDescription'")
                        if (batteryChar != null && batteryLevel == null) {
                            gattRef?.readCharacteristic(batteryChar)
                            kotlinx.coroutines.delay(1000)
                        }
                        finish(gattRef, batteryLevel)
                        break
                    }

                    if (elapsed > 10 * 60_000L) {
                        Log.w("LitertPlugin", "Reached 10-minute safety limit. Remaining queue: ${pendingFilesQueue.size}")
                        finish(gattRef, batteryLevel)
                        break
                    }
                }
            }

        } catch (e: SecurityException) {
            invoke.reject("Bluetooth permission denied: ${e.message}")
        } catch (e: Exception) {
            invoke.reject("Failed to connect to Garmin: ${e.message}")
        }
    }

    private fun startNativeSpeechInternal() {
        activity.runOnUiThread {
            try {
                if (speechRecognizer != null) {
                    speechRecognizer?.destroy()
                    speechRecognizer = null
                }
                latestSpeechTranscript = ""
                speechRecognizer = SpeechRecognizer.createSpeechRecognizer(activity)
                val intent = Intent(RecognizerIntent.ACTION_RECOGNIZE_SPEECH).apply {
                    putExtra(RecognizerIntent.EXTRA_LANGUAGE_MODEL, RecognizerIntent.LANGUAGE_MODEL_FREE_FORM)
                    putExtra(RecognizerIntent.EXTRA_PARTIAL_RESULTS, true)
                    putExtra(RecognizerIntent.EXTRA_MAX_RESULTS, 3)
                }

                speechRecognizer?.setRecognitionListener(object : RecognitionListener {
                    override fun onReadyForSpeech(params: Bundle?) {
                        Log.i("LitertPlugin", "Native SpeechRecognizer ready")
                    }
                    override fun onBeginningOfSpeech() {}
                    override fun onRmsChanged(rmsdB: Float) {}
                    override fun onBufferReceived(buffer: ByteArray?) {}
                    override fun onEndOfSpeech() {}
                    override fun onError(error: Int) {
                        Log.w("LitertPlugin", "Speech recognition error code: $error")
                    }
                    override fun onResults(results: Bundle?) {
                        val matches = results?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)
                        if (!matches.isNullOrEmpty()) {
                            latestSpeechTranscript = matches[0]
                            Log.i("LitertPlugin", "Final Speech Recognition result: $latestSpeechTranscript")
                        }
                    }
                    override fun onPartialResults(partialResults: Bundle?) {
                        val matches = partialResults?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)
                        if (!matches.isNullOrEmpty()) {
                            latestSpeechTranscript = matches[0]
                            Log.i("LitertPlugin", "Partial Speech Recognition result: $latestSpeechTranscript")
                        }
                    }
                    override fun onEvent(eventType: Int, params: Bundle?) {}
                })

                speechRecognizer?.startListening(intent)
                val ret = JSObject()
                ret.put("started", true)
                pendingSpeechInvoke?.resolve(ret)
                pendingSpeechInvoke = null
            } catch (e: Exception) {
                Log.e("LitertPlugin", "Failed to start SpeechRecognizer", e)
                pendingSpeechInvoke?.reject("Speech recognition failed: ${e.message}")
                pendingSpeechInvoke = null
            }
        }
    }

    @Command
    fun startSpeechRecognition(invoke: Invoke) {
        val hasPermission = androidx.core.content.ContextCompat.checkSelfPermission(
            activity,
            android.Manifest.permission.RECORD_AUDIO
        ) == PackageManager.PERMISSION_GRANTED

        pendingSpeechInvoke = invoke
        if (hasPermission) {
            startNativeSpeechInternal()
        } else {
            audioPermissionLauncher.launch(android.Manifest.permission.RECORD_AUDIO)
        }
    }

    @Command
    fun stopSpeechRecognition(invoke: Invoke) {
        activity.runOnUiThread {
            try {
                speechRecognizer?.stopListening()
                val ret = JSObject()
                ret.put("transcription", latestSpeechTranscript.trim())
                invoke.resolve(ret)
            } catch (e: Exception) {
                val ret = JSObject()
                ret.put("transcription", latestSpeechTranscript.trim())
                invoke.resolve(ret)
            } finally {
                speechRecognizer?.destroy()
                speechRecognizer = null
            }
        }
    }

    @Command
    fun checkCalendarPermission(invoke: Invoke) {
        val granted = androidx.core.content.ContextCompat.checkSelfPermission(
            activity,
            android.Manifest.permission.READ_CALENDAR
        ) == PackageManager.PERMISSION_GRANTED
        val ret = JSObject()
        ret.put("granted", granted)
        invoke.resolve(ret)
    }

    @Command
    fun requestCalendarPermission(invoke: Invoke) {
        val granted = androidx.core.content.ContextCompat.checkSelfPermission(
            activity,
            android.Manifest.permission.READ_CALENDAR
        ) == PackageManager.PERMISSION_GRANTED
        if (granted) {
            val ret = JSObject()
            ret.put("granted", true)
            invoke.resolve(ret)
            return
        }
        pendingCalendarPermissionInvoke = invoke
        calendarPermissionLauncher.launch(android.Manifest.permission.READ_CALENDAR)
    }

    @Command
    fun getCalendarEvents(invoke: Invoke) {
        val args = invoke.parseArgs(GetCalendarEventsArgs::class.java)
        val granted = androidx.core.content.ContextCompat.checkSelfPermission(
            activity,
            android.Manifest.permission.READ_CALENDAR
        ) == PackageManager.PERMISSION_GRANTED

        if (!granted) {
            invoke.reject("Calendar permission not granted")
            return
        }

        scope.launch {
            try {
                val startMs = if (args.startTimeEpochMs > 0) args.startTimeEpochMs else (System.currentTimeMillis() - 7 * 86400000L)
                val endMs = if (args.endTimeEpochMs > 0) args.endTimeEpochMs else (System.currentTimeMillis() + 7 * 86400000L)

                val builder = CalendarContract.Instances.CONTENT_URI.buildUpon()
                ContentUris.appendId(builder, startMs)
                ContentUris.appendId(builder, endMs)
                val uri = builder.build()

                val projection = arrayOf(
                    CalendarContract.Instances.EVENT_ID,
                    CalendarContract.Instances.TITLE,
                    CalendarContract.Instances.DESCRIPTION,
                    CalendarContract.Instances.EVENT_LOCATION,
                    CalendarContract.Instances.BEGIN,
                    CalendarContract.Instances.END,
                    CalendarContract.Instances.ALL_DAY,
                    CalendarContract.Instances.CALENDAR_DISPLAY_NAME,
                    CalendarContract.Instances.DISPLAY_COLOR
                )

                val eventsList = mutableListOf<JSObject>()
                val cursor = activity.contentResolver.query(
                    uri,
                    projection,
                    null,
                    null,
                    "${CalendarContract.Instances.BEGIN} ASC"
                )

                cursor?.use { c ->
                    val idCol = c.getColumnIndex(CalendarContract.Instances.EVENT_ID)
                    val titleCol = c.getColumnIndex(CalendarContract.Instances.TITLE)
                    val descCol = c.getColumnIndex(CalendarContract.Instances.DESCRIPTION)
                    val locCol = c.getColumnIndex(CalendarContract.Instances.EVENT_LOCATION)
                    val beginCol = c.getColumnIndex(CalendarContract.Instances.BEGIN)
                    val endCol = c.getColumnIndex(CalendarContract.Instances.END)
                    val allDayCol = c.getColumnIndex(CalendarContract.Instances.ALL_DAY)
                    val calNameCol = c.getColumnIndex(CalendarContract.Instances.CALENDAR_DISPLAY_NAME)
                    val colorCol = c.getColumnIndex(CalendarContract.Instances.DISPLAY_COLOR)

                    while (c.moveToNext()) {
                        val item = JSObject()
                        val eventId = if (idCol >= 0) c.getLong(idCol).toString() else ""
                        val title = if (titleCol >= 0) c.getString(titleCol) ?: "Untitled Event" else "Untitled Event"
                        val desc = if (descCol >= 0) c.getString(descCol) ?: "" else ""
                        val loc = if (locCol >= 0) c.getString(locCol) ?: "" else ""
                        val begin = if (beginCol >= 0) c.getLong(beginCol) else 0L
                        val end = if (endCol >= 0) c.getLong(endCol) else 0L
                        val allDay = if (allDayCol >= 0) c.getInt(allDayCol) == 1 else false
                        val calName = if (calNameCol >= 0) c.getString(calNameCol) ?: "Device Calendar" else "Device Calendar"
                        val color = if (colorCol >= 0) String.format("#%06X", 0xFFFFFF and c.getInt(colorCol)) else "#10B981"

                        item.put("id", eventId)
                        item.put("title", title)
                        item.put("description", desc)
                        item.put("location", loc)
                        item.put("startTime", begin)
                        item.put("endTime", end)
                        item.put("isAllDay", allDay)
                        item.put("calendarName", calName)
                        item.put("eventColor", color)
                        eventsList.add(item)
                    }
                }

                val ret = JSObject()
                val jsArray = JSArray()
                for (ev in eventsList) {
                    jsArray.put(ev)
                }
                ret.put("events", jsArray)
                invoke.resolve(ret)
            } catch (e: Exception) {
                Log.e("LitertPlugin", "Failed to query calendar events", e)
                invoke.reject("Failed to query calendar events: ${e.message}")
            }
        }
    }

    private fun findGarminUsbDevice(): UsbDevice? {
        val usbManager = activity.getSystemService(Context.USB_SERVICE) as? UsbManager ?: return null
        val devices = usbManager.deviceList.values
        if (devices.isEmpty()) {
            return null
        }
        for (dev in devices) {
            if (dev.vendorId == 0x091E) {
                return dev
            }
            try {
                val prodName = dev.productName?.lowercase() ?: ""
                val mfgName = dev.manufacturerName?.lowercase() ?: ""
                if (prodName.contains("garmin") || prodName.contains("forerunner") || prodName.contains("vivo") || prodName.contains("fenix") ||
                    mfgName.contains("garmin")) {
                    return dev
                }
            } catch (_: Exception) {
                // On Android 12+, reading productName or manufacturerName without permission can throw SecurityException
            }
            for (i in 0 until dev.interfaceCount) {
                val iface = dev.getInterface(i)
                if (iface.interfaceClass == UsbConstants.USB_CLASS_STILL_IMAGE ||
                    (dev.vendorId == 0x091E && iface.interfaceClass == 0xFF)) {
                    return dev
                }
            }
        }
        // Fallback: On mobile phones via OTG cable/adapter, usually exactly 1 USB device is connected
        if (devices.size == 1) {
            return devices.first()
        }
        return null
    }

    @Command
    fun scanUsbMtpDevices(invoke: Invoke) {
        val usbManager = activity.getSystemService(Context.USB_SERVICE) as? UsbManager
        if (usbManager == null) {
            val ret = JSObject()
            ret.put("isAttached", false)
            ret.put("hasPermission", false)
            ret.put("deviceName", "")
            ret.put("status", "USB service not available")
            invoke.resolve(ret)
            return
        }

        val dev = findGarminUsbDevice()
        val ret = JSObject()
        if (dev != null) {
            val hasPerm = usbManager.hasPermission(dev)
            var name = "Garmin Watch"
            if (hasPerm) {
                try {
                    name = dev.productName ?: "Garmin Watch"
                } catch (_: Exception) {}
            }
            ret.put("isAttached", true)
            ret.put("deviceName", name)
            ret.put("vendorId", dev.vendorId)
            ret.put("productId", dev.productId)
            ret.put("hasPermission", hasPerm)
            ret.put("devicePath", dev.deviceName)
            ret.put("status", if (hasPerm) "Garmin $name connected and ready for MTP sync" else "Garmin connected. Permission required.")
        } else {
            ret.put("isAttached", false)
            ret.put("hasPermission", false)
            ret.put("deviceName", "")
            ret.put("status", "No Garmin USB device connected")
        }
        invoke.resolve(ret)
    }

    @Command
    fun requestUsbMtpPermission(invoke: Invoke) {
        val usbManager = activity.getSystemService(Context.USB_SERVICE) as? UsbManager
        if (usbManager == null) {
            invoke.reject("USB service not available")
            return
        }

        val dev = findGarminUsbDevice()
        if (dev == null) {
            invoke.reject("No Garmin USB device found. Connect watch via USB cable.")
            return
        }

        if (usbManager.hasPermission(dev)) {
            val ret = JSObject()
            ret.put("granted", true)
            invoke.resolve(ret)
            return
        }

        val permissionAction = "${activity.packageName}.USB_PERMISSION"
        val flags = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            PendingIntent.FLAG_MUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        } else {
            PendingIntent.FLAG_UPDATE_CURRENT
        }
        // Android 14 (API 34+) requires PendingIntents with FLAG_MUTABLE to have an explicit package set
        val intent = Intent(permissionAction).apply {
            setPackage(activity.packageName)
        }
        val pendingIntent = PendingIntent.getBroadcast(activity, 0, intent, flags)

        val filter = IntentFilter(permissionAction)
        val receiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context?, intent: Intent?) {
                if (intent?.action == permissionAction) {
                    try {
                        activity.unregisterReceiver(this)
                    } catch (_: Exception) {}
                    val granted = intent.getBooleanExtra(UsbManager.EXTRA_PERMISSION_GRANTED, false)
                    val ret = JSObject()
                    ret.put("granted", granted)
                    invoke.resolve(ret)
                }
            }
        }

        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                activity.registerReceiver(receiver, filter, Context.RECEIVER_NOT_EXPORTED)
            } else {
                activity.registerReceiver(receiver, filter)
            }
            usbManager.requestPermission(dev, pendingIntent)
        } catch (e: Exception) {
            try {
                activity.unregisterReceiver(receiver)
            } catch (_: Exception) {}
            Log.e("LitertPlugin", "Failed to request USB permission: ${e.message}", e)
            invoke.reject("Failed to request USB permission: ${e.message}")
        }
    }

    @Command
    fun syncUsbMtpDevice(invoke: Invoke) {
        val args = invoke.parseArgs(SyncUsbMtpArgs::class.java)
        val usbManager = activity.getSystemService(Context.USB_SERVICE) as? UsbManager
        if (usbManager == null) {
            invoke.reject("USB service not available")
            return
        }

        val dev = findGarminUsbDevice()
        if (dev == null) {
            invoke.reject("No Garmin USB device connected. Ensure watch is connected via USB cable.")
            return
        }

        if (!usbManager.hasPermission(dev)) {
            invoke.reject("USB permission not granted. Call requestUsbMtpPermission first.")
            return
        }

        CoroutineScope(Dispatchers.IO).launch {
            try {
                val connection: UsbDeviceConnection? = usbManager.openDevice(dev)
                if (connection == null) {
                    withContext(Dispatchers.Main) {
                        invoke.reject("Could not open USB connection to Garmin watch.")
                    }
                    return@launch
                }

                val mtp = MtpDevice(dev)
                if (!mtp.open(connection)) {
                    connection.close()
                    withContext(Dispatchers.Main) {
                        invoke.reject("Failed to initialize MtpDevice session with Garmin watch. Ensure watch is unlocked and in MTP mode.")
                    }
                    return@launch
                }

                val storageIds = mtp.storageIds
                if (storageIds == null || storageIds.isEmpty()) {
                    mtp.close()
                    connection.close()
                    withContext(Dispatchers.Main) {
                        invoke.reject("No MTP storage found on Garmin watch. Ensure watch is unlocked.")
                    }
                    return@launch
                }

                val outputDir = File(activity.filesDir, "garmin_fits").apply { mkdirs() }
                val existingHashes = (args.existingHashes ?: emptyList()).toHashSet()
                val forcePull = args.forcePullAll ?: false

                val fitFilesBase64 = mutableListOf<String>()
                val fitFileNames = mutableListOf<String>()
                var totalBytes = 0L

                for (storageId in storageIds) {
                    var rootHandles = mtp.getObjectHandles(storageId, 0, 0)
                    if (rootHandles == null || rootHandles.isEmpty()) {
                        rootHandles = mtp.getObjectHandles(storageId, 0, -1)
                    }
                    if (rootHandles == null) continue

                    val targetFolders = mutableListOf<Int>()
                    var garminFolderHandle: Int? = null

                    for (h in rootHandles) {
                        val info = mtp.getObjectInfo(h) ?: continue
                        val name = info.name.trim()
                        if (name.equals("GARMIN", ignoreCase = true)) {
                            garminFolderHandle = h
                            break
                        } else if (name.equals("ACTIVITY", ignoreCase = true) ||
                                   name.equals("MONITOR", ignoreCase = true) ||
                                   name.equals("SLEEP", ignoreCase = true)) {
                            targetFolders.add(h)
                        }
                    }

                    if (garminFolderHandle != null) {
                        val subHandles = mtp.getObjectHandles(storageId, 0, garminFolderHandle)
                        if (subHandles != null) {
                            for (sh in subHandles) {
                                val info = mtp.getObjectInfo(sh) ?: continue
                                val name = info.name.trim().uppercase()
                                if (name == "ACTIVITY" || name == "MONITOR" || name == "SLEEP") {
                                    targetFolders.add(sh)
                                }
                            }
                        }
                    }

                    for (folderHandle in targetFolders) {
                        val fileHandles = mtp.getObjectHandles(storageId, 0, folderHandle) ?: continue
                        for (fileHandle in fileHandles) {
                            val fileInfo = mtp.getObjectInfo(fileHandle) ?: continue
                            val name = fileInfo.name
                            if (!name.endsWith(".FIT", ignoreCase = true) && !name.endsWith(".fit", ignoreCase = true)) {
                                continue
                            }
                            if (fileInfo.compressedSize <= 0) continue

                            val destFile = File(outputDir, name)
                            var imported = false
                            try {
                                imported = mtp.importFile(fileHandle, destFile.absolutePath)
                            } catch (e: Exception) {
                                Log.w("LitertPlugin", "MTP importFile failed for $name: ${e.message}")
                            }

                            var fileBytes: ByteArray? = null
                            if (imported && destFile.exists() && destFile.length() > 0) {
                                fileBytes = destFile.readBytes()
                            } else {
                                try {
                                    val raw = mtp.getObject(fileHandle, fileInfo.compressedSize)
                                    if (raw != null && raw.isNotEmpty()) {
                                        destFile.writeBytes(raw)
                                        fileBytes = raw
                                    }
                                } catch (e: Exception) {
                                    Log.w("LitertPlugin", "MTP getObject failed for $name: ${e.message}")
                                }
                            }

                            if (fileBytes != null && fileBytes.isNotEmpty()) {
                                val md = MessageDigest.getInstance("SHA-256")
                                val hashStr = md.digest(fileBytes).joinToString("") { "%02x".format(it) }
                                if (forcePull || !existingHashes.contains(hashStr)) {
                                    val b64 = Base64.encodeToString(fileBytes, Base64.NO_WRAP)
                                    fitFilesBase64.add(b64)
                                    fitFileNames.add(name)
                                    totalBytes += fileBytes.size
                                }
                            }
                        }
                    }
                }

                mtp.close()
                connection.close()

                withContext(Dispatchers.Main) {
                    val ret = JSObject()
                    ret.put("success", true)
                    ret.put("fileCount", fitFilesBase64.size)
                    ret.put("totalBytes", totalBytes)
                    ret.put("statusMessage", "Successfully synced ${fitFilesBase64.size} file(s) via USB MTP (${totalBytes / 1024} KB)")

                    val filesArray = JSArray()
                    for (b64 in fitFilesBase64) {
                        filesArray.put(b64)
                    }
                    ret.put("fitFiles", filesArray)

                    val namesArray = JSArray()
                    for (n in fitFileNames) {
                        namesArray.put(n)
                    }
                    ret.put("fileNames", namesArray)

                    invoke.resolve(ret)
                }
            } catch (e: Exception) {
                Log.e("LitertPlugin", "Error syncing USB MTP device", e)
                withContext(Dispatchers.Main) {
                    invoke.reject("USB MTP sync error: ${e.message}")
                }
            }
        }
    }
}
