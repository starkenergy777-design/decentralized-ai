package ai.decentralized.app

import android.content.Context
import androidx.work.Constraints
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.workDataOf

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

