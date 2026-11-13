package ai.decentralized.app

/** Logical /ai/task/1 contract shared with the Rust node. */
internal object TaskProtocol {
    const val VERSION = 1
    const val OP_ADD = 1

    const val ERROR_INVALID_ID = 1
    const val ERROR_UNSUPPORTED_VERSION = 2
    const val ERROR_UNSUPPORTED_OPERATION = 3
    const val ERROR_OVERFLOW = 4

    const val KEY_VERSION = "version"
    const val KEY_ID = "id"
    const val KEY_OPERATION = "operation"
    const val KEY_LEFT = "left"
    const val KEY_RIGHT = "right"
    const val KEY_RESULT = "result"
    const val KEY_ERROR = "error"

    data class Request(
        val version: Int,
        val id: Long,
        val operation: Int,
        val left: Long,
        val right: Long,
    ) {
        fun execute(): Response {
            if (id <= 0L) return failure(ERROR_INVALID_ID)
            if (version != VERSION) return failure(ERROR_UNSUPPORTED_VERSION)
            if (operation != OP_ADD) return failure(ERROR_UNSUPPORTED_OPERATION)

            return try {
                Response(VERSION, id, Math.addExact(left, right), null)
            } catch (_: ArithmeticException) {
                failure(ERROR_OVERFLOW)
            }
        }

        private fun failure(code: Int) = Response(VERSION, id, null, code)
    }

    data class Response(
        val version: Int,
        val id: Long,
        val result: Long?,
        val error: Int?,
    )

    fun errorName(code: Int): String = when (code) {
        ERROR_INVALID_ID -> "invalid_task_id"
        ERROR_UNSUPPORTED_VERSION -> "unsupported_task_version"
        ERROR_UNSUPPORTED_OPERATION -> "unsupported_task_operation"
        ERROR_OVERFLOW -> "integer_overflow"
        else -> "unknown_task_error"
    }
}
