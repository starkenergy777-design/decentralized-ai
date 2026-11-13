package ai.decentralized.app

import android.content.Context
import androidx.work.Data
import androidx.work.Worker
import androidx.work.WorkerParameters

/** Executes only a fixed, local demonstration task. It never runs peer-supplied code. */
class DemoTaskWorker(context: Context, parameters: WorkerParameters) : Worker(context, parameters) {
    override fun doWork(): Result {
        val preferences = applicationContext.getSharedPreferences(
            ResourcePolicy.PREFERENCES,
            Context.MODE_PRIVATE,
        )
        if (!preferences.getBoolean(ResourcePolicy.PARTICIPATING, false)) {
            return failure("participation_disabled")
        }
        if (preferences.getInt(ResourcePolicy.CPU_BUDGET_PERCENT, 10) == 0) {
            return failure("cpu_budget_is_zero")
        }
        if (preferences.getInt(ResourcePolicy.MEMORY_BUDGET_MIB, 256) < MIN_MEMORY_BUDGET_MIB) {
            return failure("memory_budget_too_low")
        }

        val requestKeys = setOf(KEY_VERSION, KEY_ID, KEY_OPERATION, KEY_LEFT, KEY_RIGHT)
        val values = inputData.keyValueMap
        if (values.keys != requestKeys ||
            values[KEY_VERSION] !is Int ||
            values[KEY_ID] !is Long ||
            values[KEY_OPERATION] !is Int ||
            values[KEY_LEFT] !is Long ||
            values[KEY_RIGHT] !is Long
        ) {
            return failure("invalid_task_request")
        }

        val response = TaskProtocol.Request(
            version = inputData.getInt(KEY_VERSION, -1),
            id = inputData.getLong(KEY_ID, 0L),
            operation = inputData.getInt(KEY_OPERATION, 0),
            left = inputData.getLong(KEY_LEFT, 0L),
            right = inputData.getLong(KEY_RIGHT, 0L),
        ).execute()
        val error = response.error
        if (error != null) {
            return Result.failure(
                Data.Builder()
                    .putInt(KEY_VERSION, response.version)
                    .putLong(KEY_ID, response.id)
                    .putInt(KEY_ERROR, error)
                    .build(),
            )
        }

        val result = response.result ?: return failure("missing_task_result")
        return Result.success(
            Data.Builder()
                .putInt(KEY_VERSION, response.version)
                .putLong(KEY_ID, response.id)
                .putLong(KEY_RESULT, result)
                .build(),
        )
    }

    private fun failure(reason: String): Result =
        Result.failure(Data.Builder().putString(KEY_WORKER_ERROR, reason).build())

    companion object {
        const val KEY_VERSION = TaskProtocol.KEY_VERSION
        const val KEY_ID = TaskProtocol.KEY_ID
        const val KEY_OPERATION = TaskProtocol.KEY_OPERATION
        const val KEY_LEFT = TaskProtocol.KEY_LEFT
        const val KEY_RIGHT = TaskProtocol.KEY_RIGHT
        const val KEY_RESULT = TaskProtocol.KEY_RESULT
        const val KEY_ERROR = TaskProtocol.KEY_ERROR
        const val KEY_WORKER_ERROR = "worker_error"
        private const val MIN_MEMORY_BUDGET_MIB = 64
    }
}
