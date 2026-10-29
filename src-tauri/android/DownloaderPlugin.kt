package com.aetherforge.downpour

import android.app.Activity
import android.content.ContentValues
import android.os.Environment
import android.provider.MediaStore
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import dev.ffmpegkit_maintained.ytdlp.YtDlp
import dev.ffmpegkit_maintained.ytdlp.YtDlpRequest
import java.io.File
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean

@InvokeArg
class DownloadArgs {
    lateinit var url: String
    lateinit var quality: String
    var playlist: Boolean = false
}

@TauriPlugin
class DownloaderPlugin(private val activity: Activity) : Plugin(activity) {
    private val worker = Executors.newSingleThreadExecutor()
    private val busy = AtomicBoolean(false)
    @Volatile private var phase = "idle"
    @Volatile private var progress = 0f
    @Volatile private var message = ""

    @Command
    fun start(invoke: Invoke) {
        var claimed = false
        try {
            val args = invoke.parseArgs(DownloadArgs::class.java)
            val uri = android.net.Uri.parse(args.url)
            if (uri.scheme !in listOf("https", "http") ||
                uri.userInfo != null ||
                !(uri.host == "youtube.com" || uri.host == "youtu.be" ||
                  (uri.host?.endsWith(".youtube.com") == true))) {
                invoke.reject("A valid YouTube link is required.")
                return
            }
            if (args.quality !in listOf("best", "720", "480")) {
                invoke.reject("Unsupported Android quality.")
                return
            }
            if (!busy.compareAndSet(false, true)) {
                invoke.reject("A download is already in progress.")
                return
            }
            claimed = true
            phase = "running"
            progress = 0f
            message = "Preparing embedded yt-dlp…"
            worker.execute {
                try {
                    YtDlp.init(activity.applicationContext)
                    val cacheRoot = activity.getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS)
                        ?: activity.filesDir
                    val temp = File(cacheRoot, "downpour/" + System.nanoTime())
                    check(temp.mkdirs()) { "Could not create a temporary download folder." }
                    try {
                        val selector = if (args.quality == "best") "best[ext=mp4]"
                            else "best[ext=mp4][height<=?" + args.quality + "]"
                        val request = YtDlpRequest(args.url)
                            .setOutputTemplate(File(temp, "%(title).180B [%(id)s].%(ext)s").absolutePath)
                            .addOption("-f", selector)
                        if (!args.playlist) request.addOption("--no-playlist")
                        val response = YtDlp.execute(request) { pct, _, _ ->
                            progress = pct.coerceIn(0f, 100f)
                            message = "Downloading…"
                        }
                        check(response.isSuccess) { "yt-dlp failed (exit " + response.exitCode + ")." }
                        val files = temp.walkTopDown()
                            .filter { it.isFile && !it.name.endsWith(".part") }
                            .toList()
                        check(files.isNotEmpty()) { "No completed video was produced." }
                        message = "Saving to Downloads…"
                        for (file in files) publishDownload(file)
                        progress = 100f
                        message = "Saved to Downloads / Downpour."
                        phase = "complete"
                    } finally {
                        temp.deleteRecursively()
                    }
                } catch (e: Exception) {
                    message = e.message ?: "Android download failed."
                    phase = "error"
                } finally {
                    busy.set(false)
                }
            }
            invoke.resolve(JSObject().apply { put("started", true) })
        } catch (e: Exception) {
            if (claimed) {
                busy.set(false)
                phase = "error"
            }
            invoke.reject(e.message ?: "Could not start Android download.")
        }
    }

    @Command
    fun status(invoke: Invoke) {
        invoke.resolve(JSObject().apply {
            put("status", phase)
            put("progress", progress.toDouble())
            put("message", message)
        })
    }

    private fun publishDownload(source: File) {
        val resolver = activity.contentResolver
        val values = ContentValues().apply {
            put(MediaStore.MediaColumns.DISPLAY_NAME, source.name)
            put(MediaStore.MediaColumns.MIME_TYPE, "video/mp4")
            put(MediaStore.MediaColumns.RELATIVE_PATH, Environment.DIRECTORY_DOWNLOADS + "/Downpour")
            put(MediaStore.MediaColumns.IS_PENDING, 1)
        }
        val uri = resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, values)
            ?: error("Could not create a Downloads entry.")
        try {
            resolver.openOutputStream(uri)?.use { out ->
                source.inputStream().use { input -> input.copyTo(out) }
            } ?: error("Could not write the downloaded file.")
            values.clear()
            values.put(MediaStore.MediaColumns.IS_PENDING, 0)
            resolver.update(uri, values, null, null)
        } catch (e: Exception) {
            resolver.delete(uri, null, null)
            throw e
        }
    }
}
