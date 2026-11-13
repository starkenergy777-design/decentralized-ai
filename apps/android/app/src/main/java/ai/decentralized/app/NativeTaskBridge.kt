package ai.decentralized.app

/** JNI boundary into the shared task protocol and P2P node. */
class NativeTaskBridge {
    companion object {
        init {
            System.loadLibrary("decentralized_ai_android_bridge")
        }

        @JvmStatic
        external fun execute(
            version: Int,
            id: Long,
            operation: Int,
            left: Long,
            right: Long,
        ): LongArray

        @JvmStatic
        external fun startNode(peerAddress: String): String

        @JvmStatic
        external fun stopNode(handle: Long): Boolean
    }
}
