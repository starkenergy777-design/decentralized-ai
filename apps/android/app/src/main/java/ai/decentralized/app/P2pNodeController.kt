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
    val connectedPeer: String? = null,
    val taskStatus: String? = null,
    val taskResult: String? = null,
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

    private var currentUiState = P2pNodeUiState()

    private var wifiOnly = true
    private var chargingOnly = true
    private var computeEnabled = true

    @Volatile
    private var activeNodeHandle: Long? = null

    private val eventPoll = object : Runnable {
        override fun run() {
            val handle = activeNodeHandle
            if (!desiredRunning || handle == null) return
            try {
                val event = NativeTaskBridge.pollNodeEvent(handle)
                if (event.isNotEmpty()) consumeNodeEvent(event)
            } catch (error: Exception) {
                updateUi { it.copy(message = error.message ?: "კვანძის სტატუსის წაკითხვა ვერ მოხერხდა") }
            }
            if (desiredRunning) mainHandler.postDelayed(this, 500)
        }
    }

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
        sendDemoAddition: Boolean,
    ) {
        if (closed) return
        this.wifiOnly = wifiOnly
        this.chargingOnly = chargingOnly
        this.computeEnabled = computeEnabled
        policyFailure()?.let {
            publish(P2pNodeUiState(P2pNodePhase.ERROR, it))
            return
        }
        if (sendDemoAddition && peerAddress.isBlank()) {
            publish(
                P2pNodeUiState(
                    P2pNodePhase.ERROR,
                    "დემო შეკრების გასაგზავნად შეიყვანეთ სხვა კვანძის მისამართი.",
                ),
            )
            return
        }
        desiredRunning = true
        mainHandler.removeCallbacks(policyMonitor)
        mainHandler.postDelayed(policyMonitor, 10_000)
        publish(
            P2pNodeUiState(
                phase = P2pNodePhase.STARTING,
                message = "P2P კვანძი ეშვება…",
                taskStatus = if (sendDemoAddition) {
                    "20 + 22 მოთხოვნა გაიგზავნება კავშირის დამყარების შემდეგ"
                } else {
                    null
                },
            ),
        )
        executor.execute {
            if (!desiredRunning) return@execute
            try {
                val identitySeed = AndroidPeerIdentityStore.loadOrCreate(context)
                val response = try {
                    NativeTaskBridge.startNode(peerAddress.trim(), sendDemoAddition, identitySeed)
                } finally {
                    identitySeed.fill(0)
                }
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
                        } else if (sendDemoAddition) {
                            "კვანძი მუშაობს; peer-თან კავშირისა და 20 + 22 პასუხის მოლოდინშია"
                        } else {
                            "კვანძი მუშაობს; პირდაპირი დაკავშირების მცდელობა გაიგზავნა"
                        },
                        taskStatus = if (sendDemoAddition) {
                            "ველოდები peer-ის პასუხს"
                        } else {
                            null
                        },
                        peerId = parts[2],
                        listenAddress = dialableListenAddress(parts[3]),
                    ),
                )
                mainHandler.post(eventPoll)
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
        mainHandler.removeCallbacks(eventPoll)
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
        mainHandler.removeCallbacks(eventPoll)
        executor.execute {
            activeNodeHandle?.let { NativeTaskBridge.stopNode(it) }
            activeNodeHandle = null
        }
        executor.shutdown()
    }

    private fun publish(state: P2pNodeUiState) {
        mainHandler.post {
            currentUiState = state
            onStateChanged(state)
        }
    }

    private fun updateUi(transform: (P2pNodeUiState) -> P2pNodeUiState) {
        mainHandler.post {
            val state = transform(currentUiState)
            currentUiState = state
            onStateChanged(state)
        }
    }

    private fun consumeNodeEvent(event: String) {
        if (event == "node_closed") {
            stop("P2P კვანძი მოულოდნელად გაითიშა")
            return
        }
        val fields = event.split('|', limit = 4)
        val state = when (fields.firstOrNull()) {
            "connected" -> currentUiState.copy(
                message = "peer-თან კავშირი დამყარდა",
                connectedPeer = fields.getOrNull(1),
            )
            "disconnected" -> currentUiState.copy(
                message = "peer-თან კავშირი დაიხურა",
                connectedPeer = null,
            )
            "connection_failed" -> currentUiState.copy(
                message = "peer-თან კავშირი ვერ დამყარდა; შეამოწმეთ მისამართი და ქსელი",
                connectedPeer = null,
            )
            "task_sent" -> currentUiState.copy(
                taskStatus = "დემო ამოცანა გაიგზავნა (ID ${fields.getOrNull(1)})",
            )
            "task_failed" -> currentUiState.copy(
                message = "დემო ამოცანა ვერ შესრულდა ან პასუხი ვერ დადასტურდა",
                taskStatus = "შეამოწმეთ peer-ის მისამართი და კავშირის მდგომარეობა",
            )
            "task_received" -> currentUiState.copy(
                taskStatus = "მიღებულია peer-ის ამოცანა (ID ${fields.getOrNull(1)})",
            )
            "task_response" -> {
                val result = fields.getOrNull(2).orEmpty()
                val errorText = fields.getOrNull(3).orEmpty()
                val resultText = when {
                    errorText.isEmpty() && result.isNotEmpty() -> result
                    errorText.isNotEmpty() && errorText.toIntOrNull() != null ->
                        "შეცდომა: ${TaskProtocol.errorName(errorText.toInt())}"
                    else -> "პასუხი არასწორია"
                }
                currentUiState.copy(
                    message = "დისტანციური ამოცანა დასრულდა",
                    taskStatus = "დემო შეკრება (ID ${fields.getOrNull(1)})",
                    taskResult = resultText,
                )
            }
            else -> currentUiState
        }
        currentUiState = state
        onStateChanged(state)
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
