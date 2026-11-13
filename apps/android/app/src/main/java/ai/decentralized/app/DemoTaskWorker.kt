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

        val left = inputData.getLong(KEY_LEFT, 20L)
        val right = inputData.getLong(KEY_RIGHT, 22L)
        val sum = try {
            Math.addExact(left, right)
        } catch (_: ArithmeticException) {
            return failure("integer_overflow")
        }

        return Result.success(Data.Builder().putLong(KEY_RESULT, sum).build())
    }

    private fun failure(reason: String): Result =
        Result.failure(Data.Builder().putString(KEY_ERROR, reason).build())

    companion object {
        const val KEY_LEFT = "left"
        const val KEY_RIGHT = "right"
        const val KEY_RESULT = "result"
        const val KEY_ERROR = "error"
        private const val MIN_MEMORY_BUDGET_MIB = 64
    }
}

