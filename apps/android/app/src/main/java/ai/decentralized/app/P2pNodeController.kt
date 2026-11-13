package ai.decentralized.app

import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.os.BatteryManager
import android.os.Handler
import android.os.Looper
import java.net.Inet4Address
import java.util.concurrent.Executors

enum class P2pNodePhase { STOPPED, STARTING, RUNNING, STOPPING, ERROR }

data class P2pNodeUiState(
    val phase: P2pNodePhase = P2pNodePhase.STOPPED,
    val message: String = "კვანძი გამორთულია",
    val peerId: String? = null,
    val listenAddress: String? = null,
)

/** Runs JNI startup and shutdown away from Android's main thread. */
class P2pNodeController(
    private val context: Context,
    private val onStateChanged: (P2pNodeUiState) -> Unit,
) {
    private val mainHandler = Handler(Looper.getMainLooper())
    private val executor = Executors.newSingleThreadExecutor()

    @Volatile
    private var desiredRunning = false

    @Volatile
    private var closed = false

    private var wifiOnly = true
    private var chargingOnly = true
    private var computeEnabled = true

    // Accessed only by executor tasks.
    private var activeNodeHandle: Long? = null

    private val policyMonitor = object : Runnable {
        override fun run() {
            if (!desiredRunning) return
            val failure = policyFailure()
            if (failure != null) {
                stop(failure)
            } else {
                mainHandler.postDelayed(this, 10_000)
            }
        }
    }

    fun start(
        wifiOnly: Boolean,
        chargingOnly: Boolean,
        computeEnabled: Boolean,
        peerAddress: String,
    ) {
        if (closed) return
        this.wifiOnly = wifiOnly
        this.chargingOnly = chargingOnly
        this.computeEnabled = computeEnabled
        policyFailure()?.let {
            publish(P2pNodeUiState(P2pNodePhase.ERROR, it))
            return
        }
        desiredRunning = true
        mainHandler.removeCallbacks(policyMonitor)
        mainHandler.postDelayed(policyMonitor, 10_000)
        publish(P2pNodeUiState(P2pNodePhase.STARTING, "P2P კვანძი ეშვება…"))
        executor.execute {
            if (!desiredRunning) return@execute
            try {
                val response = NativeTaskBridge.startNode(peerAddress.trim())
                val parts = response.split('|', limit = 4)
                if (parts.firstOrNull() != "ok" || parts.size != 4) {
                    if (desiredRunning) {
                        publish(
                            P2pNodeUiState(
                                P2pNodePhase.ERROR,
                                parts.getOrElse(1) { "კვანძის გაშვება ვერ მოხერხდა" },
                            ),
                        )
                    }
                    return@execute
                }

                val handle = parts[1].toLong()
                if (!desiredRunning) {
                    NativeTaskBridge.stopNode(handle)
                    return@execute
                }
                activeNodeHandle = handle
                publish(
                    P2pNodeUiState(
                        phase = P2pNodePhase.RUNNING,
                        message = if (peerAddress.isBlank()) {
                            "კვანძი მუშაობს — მხოლოდ ეკრანის წინა პლანზე"
                        } else {
                            "კვანძი მუშაობს; პირდაპირი დაკავშირების მცდელობა გაიგზავნა"
                        },
                        peerId = parts[2],
                        listenAddress = dialableListenAddress(parts[3]),
                    ),
                )
            } catch (error: Exception) {
                if (desiredRunning) {
                    publish(
                        P2pNodeUiState(
                            P2pNodePhase.ERROR,
                            error.message ?: "კვანძის გაშვება ვერ მოხერხდა",
                        ),
                    )
                }
            }
        }
    }

    fun updatePolicy(wifiOnly: Boolean, chargingOnly: Boolean, computeEnabled: Boolean) {
        this.wifiOnly = wifiOnly
        this.chargingOnly = chargingOnly
        this.computeEnabled = computeEnabled
        if (desiredRunning) {
            policyFailure()?.let { stop(it) }
        }
    }

    fun stop(finalMessage: String? = null) {
        desiredRunning = false
        mainHandler.removeCallbacks(policyMonitor)
        publish(P2pNodeUiState(P2pNodePhase.STOPPING, "P2P კვანძი ჩერდება…"))
        if (!closed) {
            executor.execute {
                activeNodeHandle?.let { NativeTaskBridge.stopNode(it) }
                activeNodeHandle = null
                publish(P2pNodeUiState(message = finalMessage ?: "კვანძი გამორთულია"))
            }
        }
    }

    fun close() {
        if (closed) return
        closed = true
        desiredRunning = false
        mainHandler.removeCallbacks(policyMonitor)
        executor.execute {
            activeNodeHandle?.let { NativeTaskBridge.stopNode(it) }
            activeNodeHandle = null
        }
        executor.shutdown()
    }

    private fun publish(state: P2pNodeUiState) {
        mainHandler.post { onStateChanged(state) }
    }

    private fun policyFailure(): String? {
        if (!computeEnabled) return "CPU-ის ბიუჯეტი 0%-ია; კვანძი გაშვებული ვერ იქნება."

        if (wifiOnly) {
            val connectivity = context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
            val network = connectivity.activeNetwork
            val capabilities = network?.let(connectivity::getNetworkCapabilities)
            if (capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) != true) {
                return "არჩეულია Wi-Fi-only რეჟიმი; დაუკავშირდით Wi-Fi-ს ან შეცვალეთ პარამეტრი."
            }
        }

        if (chargingOnly) {
            val batteryStatus = context.registerReceiver(
                null,
                IntentFilter(Intent.ACTION_BATTERY_CHANGED),
            )?.getIntExtra(BatteryManager.EXTRA_STATUS, -1)
            if (batteryStatus != BatteryManager.BATTERY_STATUS_CHARGING &&
                batteryStatus != BatteryManager.BATTERY_STATUS_FULL
            ) {
                return "არჩეულია მხოლოდ დატენვისას რეჟიმი; დატენეთ მოწყობილობა ან შეცვალეთ პარამეტრი."
            }
        }
        return null
    }

    private fun dialableListenAddress(address: String): String {
        if (!address.startsWith("/ip4/0.0.0.0/")) return address
        val connectivity = context.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
        val network = connectivity.activeNetwork ?: return address
        val localIp = connectivity.getLinkProperties(network)
            ?.linkAddresses
            ?.firstOrNull { it.address is Inet4Address && !it.address.isLoopbackAddress }
            ?.address
            ?.hostAddress
            ?: return address
        return address.replaceFirst("/ip4/0.0.0.0/", "/ip4/$localIp/")
    }
}
