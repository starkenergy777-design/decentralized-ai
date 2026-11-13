package ai.decentralized.app

/** Logical /ai/task/1 contract shared with the Rust node and JNI bridge. */
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
    )

    data class Response(
        val version: Int,
        val id: Long,
        val result: Long?,
        val error: Int?,
    )

    fun responseFromNative(request: Request, values: LongArray): Response? {
        if (values.size != 4 || values[0] != VERSION.toLong() || values[1] != request.id) {
            return null
        }

        val error = values[3]
        if (error == 0L) return Response(VERSION, request.id, values[2], null)
        if (error !in ERROR_INVALID_ID.toLong()..ERROR_OVERFLOW.toLong() || values[2] != 0L) {
            return null
        }
        return Response(VERSION, request.id, null, error.toInt())
    }

    fun errorName(code: Int): String = when (code) {
        ERROR_INVALID_ID -> "invalid_task_id"
        ERROR_UNSUPPORTED_VERSION -> "unsupported_task_version"
        ERROR_UNSUPPORTED_OPERATION -> "unsupported_task_operation"
        ERROR_OVERFLOW -> "integer_overflow"
        else -> "unknown_task_error"
    }
}
