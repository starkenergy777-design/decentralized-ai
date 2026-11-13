package ai.decentralized.app

/** Narrow JNI boundary into the shared Rust task protocol crate. */
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
    }
}
