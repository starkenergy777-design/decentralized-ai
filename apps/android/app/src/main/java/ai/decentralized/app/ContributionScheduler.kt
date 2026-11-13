package ai.decentralized.app

import android.content.Context
import androidx.work.Constraints
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.workDataOf
import java.util.UUID

internal object ContributionScheduler {
    fun enqueueLocalDemo(
        context: Context,
        wifiOnly: Boolean,
        chargingOnly: Boolean,
    ) {
        val constraints = Constraints.Builder()
            .setRequiresBatteryNotLow(true)
            .apply {
                if (wifiOnly) setRequiredNetworkType(NetworkType.UNMETERED)
                if (chargingOnly) setRequiresCharging(true)
            }
            .build()

        val request = OneTimeWorkRequestBuilder<DemoTaskWorker>()
            .setConstraints(constraints)
            .setInputData(
                workDataOf(
                    TaskProtocol.KEY_VERSION to TaskProtocol.VERSION,
                    TaskProtocol.KEY_ID to UUID.randomUUID().mostSignificantBits
                        .and(Long.MAX_VALUE).coerceAtLeast(1L),
                    TaskProtocol.KEY_OPERATION to TaskProtocol.OP_ADD,
                    DemoTaskWorker.KEY_LEFT to 20L,
                    DemoTaskWorker.KEY_RIGHT to 22L,
                ),
            )
            .build()

        WorkManager.getInstance(context).enqueueUniqueWork(
            ResourcePolicy.DEMO_WORK_NAME,
            ExistingWorkPolicy.REPLACE,
            request,
        )
    }

    fun stop(context: Context) {
        WorkManager.getInstance(context).cancelUniqueWork(ResourcePolicy.DEMO_WORK_NAME)
    }
}
