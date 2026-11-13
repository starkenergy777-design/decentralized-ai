package ai.decentralized.app

import android.content.SharedPreferences
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.State
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.lifecycle.Observer
import androidx.work.WorkInfo
import androidx.work.WorkManager
import kotlin.math.roundToInt

class MainActivity : ComponentActivity() {
    private val p2pNodeState = mutableStateOf(P2pNodeUiState())
    private lateinit var p2pNodeController: P2pNodeController

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        p2pNodeController = P2pNodeController(this) { p2pNodeState.value = it }
        val preferences = getSharedPreferences(ResourcePolicy.PREFERENCES, MODE_PRIVATE)
        setContent {
            MaterialTheme {
                ResourceSettingsScreen(preferences, p2pNodeController, p2pNodeState)
            }
        }
    }

    override fun onStop() {
        p2pNodeController.stop()
        super.onStop()
    }

    override fun onDestroy() {
        p2pNodeController.close()
        super.onDestroy()
    }
}

@Composable
private fun ResourceSettingsScreen(
    preferences: SharedPreferences,
    p2pNodeController: P2pNodeController,
    p2pNodeState: State<P2pNodeUiState>,
) {
    val context = LocalContext.current
    val workManager = remember(context) { WorkManager.getInstance(context) }
    var taskStatus by remember { mutableStateOf("ჯერ დემო ამოცანა არ გაშვებულა") }
    val workInfo = remember(workManager) {
        workManager.getWorkInfosForUniqueWorkLiveData(ResourcePolicy.DEMO_WORK_NAME)
    }
    DisposableEffect(workInfo) {
        val observer = Observer<List<WorkInfo>> { infos ->
            val latest = infos.firstOrNull() ?: return@Observer
            taskStatus = when (latest.state) {
                WorkInfo.State.ENQUEUED, WorkInfo.State.BLOCKED ->
                    "რიგშია — ელოდება არჩეულ პირობებს"
                WorkInfo.State.RUNNING -> "ლოკალური დემო ამოცანა სრულდება"
                WorkInfo.State.SUCCEEDED ->
                    "დასრულდა — შედეგი ${latest.outputData.getLong(DemoTaskWorker.KEY_RESULT, 0L)}"
                WorkInfo.State.FAILED ->
                    "ვერ შესრულდა — ${latest.outputData.getString(DemoTaskWorker.KEY_WORKER_ERROR)
                        ?: TaskProtocol.errorName(latest.outputData.getInt(DemoTaskWorker.KEY_ERROR, 0))}"
                WorkInfo.State.CANCELLED -> "გაუქმებულია"
            }
        }
        workInfo.observeForever(observer)
        onDispose { workInfo.removeObserver(observer) }
    }

    var participating by remember {
        mutableStateOf(preferences.getBoolean(ResourcePolicy.PARTICIPATING, false))
    }
    var cpuBudget by remember {
        mutableIntStateOf(preferences.getInt(ResourcePolicy.CPU_BUDGET_PERCENT, 10))
    }
    var memoryBudget by remember {
        mutableIntStateOf(preferences.getInt(ResourcePolicy.MEMORY_BUDGET_MIB, 256))
    }
    var networkBudget by remember {
        mutableIntStateOf(preferences.getInt(ResourcePolicy.NETWORK_BUDGET_MIB_PER_DAY, 256))
    }
    var wifiOnly by remember {
        mutableStateOf(preferences.getBoolean(ResourcePolicy.WIFI_ONLY, true))
    }
    var chargingOnly by remember {
        mutableStateOf(preferences.getBoolean(ResourcePolicy.CHARGING_ONLY, true))
    }
    val p2pState by p2pNodeState

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text("Decentralized AI", style = MaterialTheme.typography.headlineMedium)
        Text("მოწყობილობის რესურსების პარამეტრები", style = MaterialTheme.typography.titleMedium)
        Text(
            "მონაწილეობა გამორთულია ნაგულისხმევად. ადგილობრივი დემო ამოცანა გაეშვება მხოლოდ თქვენი ჩართვის შემდეგ.",
            style = MaterialTheme.typography.bodyMedium,
        )

        HorizontalDivider()
        PolicySwitch(
            title = "გამოთვლებში მონაწილეობა",
            checked = participating,
            onCheckedChange = {
                participating = it
                preferences.edit().putBoolean(ResourcePolicy.PARTICIPATING, it).apply()
                if (!it) {
                    ContributionScheduler.stop(context)
                    p2pNodeController.stop()
                }
            },
        )

        HorizontalDivider()
        Text("P2P კვანძი", style = MaterialTheme.typography.titleMedium)
        Text(
            "ჩართვის შემდეგ ტელეფონი გახსნის ქსელურ კვანძს. Wi-Fi-ისა და დატენვის შეზღუდვები მოქმედებს; კვანძი მუშაობს მხოლოდ ამ ეკრანის წინა პლანზე ყოფნისას და არჩეული პირობების დაცვით.",
            style = MaterialTheme.typography.bodyMedium,
        )
        Button(
            onClick = {
                if (p2pState.phase == P2pNodePhase.RUNNING) {
                    p2pNodeController.stop()
                } else {
                    p2pNodeController.start(wifiOnly, chargingOnly, cpuBudget > 0)
                }
            },
            enabled = participating && cpuBudget > 0 && p2pState.phase != P2pNodePhase.STARTING &&
                p2pState.phase != P2pNodePhase.STOPPING,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text(if (p2pState.phase == P2pNodePhase.RUNNING) "კვანძის გაჩერება" else "P2P კვანძის დაწყება")
        }
        Text(p2pState.message, style = MaterialTheme.typography.bodyMedium)
        p2pState.peerId?.let { Text("Peer ID: $it", style = MaterialTheme.typography.bodySmall) }
        p2pState.listenAddress?.let {
            Text("მოსმენის მისამართი: $it", style = MaterialTheme.typography.bodySmall)
        }
        Text(
            "ეს სატესტო კვანძია: მისამართი შეიძლება NAT-ის გამო გარედან მიუწვდომელი იყოს; აღმოჩენა/რელე ჯერ არ გვაქვს. ამ ეტაპზე კვანძები ასრულებენ მხოლოდ მცირე არითმეტიკულ პროტოკოლს — AI მოდელის განაწილებული inference ჯერ არ არის ჩართული.",
            style = MaterialTheme.typography.bodySmall,
        )
        Text(
            "დღიური ქსელის ბიუჯეტის ზუსტი აღრიცხვა ჯერ არ მოქმედებს P2P კვანძზე; ქსელის ხარჯის შესაზღუდად დატოვეთ Wi-Fi-only ჩართული.",
            style = MaterialTheme.typography.bodySmall,
        )

        BudgetSlider(
            title = "CPU-ის სამიზნე ბიუჯეტი",
            value = cpuBudget,
            range = 0..50,
            steps = 9,
            valueLabel = "$cpuBudget%",
            onValueChange = {
                cpuBudget = it
                p2pNodeController.updatePolicy(wifiOnly, chargingOnly, it > 0)
            },
            onValueChangeFinished = {
                preferences.edit().putInt(ResourcePolicy.CPU_BUDGET_PERCENT, cpuBudget).apply()
            },
        )
        BudgetSlider(
            title = "მეხსიერების რბილი ლიმიტი",
            value = memoryBudget,
            range = 128..1024,
            steps = 6,
            valueLabel = "$memoryBudget MiB",
            onValueChange = { memoryBudget = it },
            onValueChangeFinished = {
                preferences.edit().putInt(ResourcePolicy.MEMORY_BUDGET_MIB, memoryBudget).apply()
            },
        )
        BudgetSlider(
            title = "ქსელის დღიური ბიუჯეტი",
            value = networkBudget,
            range = 64..2048,
            steps = 14,
            valueLabel = "$networkBudget MiB",
            onValueChange = { networkBudget = it },
            onValueChangeFinished = {
                preferences.edit().putInt(ResourcePolicy.NETWORK_BUDGET_MIB_PER_DAY, networkBudget).apply()
            },
        )

        HorizontalDivider()
        PolicySwitch(
            title = "მხოლოდ ულიმიტო ქსელზე (მაგ. Wi-Fi)",
            checked = wifiOnly,
            onCheckedChange = {
                wifiOnly = it
                preferences.edit().putBoolean(ResourcePolicy.WIFI_ONLY, it).apply()
                p2pNodeController.updatePolicy(
                    wifiOnly = it,
                    chargingOnly = chargingOnly,
                    computeEnabled = cpuBudget > 0,
                )
            },
        )
        PolicySwitch(
            title = "მხოლოდ დატენვისას",
            checked = chargingOnly,
            onCheckedChange = {
                chargingOnly = it
                preferences.edit().putBoolean(ResourcePolicy.CHARGING_ONLY, it).apply()
                p2pNodeController.updatePolicy(
                    wifiOnly = wifiOnly,
                    chargingOnly = it,
                    computeEnabled = cpuBudget > 0,
                )
            },
        )
        Spacer(Modifier.height(8.dp))
        Button(
            onClick = {
                ContributionScheduler.enqueueLocalDemo(context, wifiOnly, chargingOnly)
            },
            enabled = participating && cpuBudget > 0 && memoryBudget >= 64,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text("ადგილობრივი დემო: 20 + 22")
        }
        Text(taskStatus, style = MaterialTheme.typography.bodyMedium)
        Text(
            "ეს მხოლოდ უსაფრთხო, ადგილობრივი ტესტ-ამოცანაა. P2P კვანძი სატესტო არითმეტიკულ მოთხოვნებს იღებს; დღიური ქსელის ბიუჯეტის ზუსტი აღრიცხვა ჯერ არ არის ჩართული.",
            style = MaterialTheme.typography.bodySmall,
        )
        Text(
            "CPU-ის პროცენტი არის სამიზნე და არა Android-ის მიერ გარანტირებული ზუსტი დატვირთვა. Scheduler მოითხოვს არადაბალ ბატარეას და გამოიყენებს Wi-Fi/დატენვის შეზღუდვებს.",
            style = MaterialTheme.typography.bodySmall,
        )
    }
}

@Composable
private fun PolicySwitch(
    title: String,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Text(title, modifier = Modifier.weight(1f))
        Switch(checked = checked, onCheckedChange = onCheckedChange)
    }
}

@Composable
private fun BudgetSlider(
    title: String,
    value: Int,
    range: IntRange,
    steps: Int,
    valueLabel: String,
    onValueChange: (Int) -> Unit,
    onValueChangeFinished: () -> Unit,
) {
    Column {
        Text(title, style = MaterialTheme.typography.titleSmall)
        Text(valueLabel, style = MaterialTheme.typography.bodyMedium)
        Slider(
            value = value.toFloat(),
            onValueChange = { onValueChange(it.roundToInt()) },
            onValueChangeFinished = onValueChangeFinished,
            valueRange = range.first.toFloat()..range.last.toFloat(),
            steps = steps,
        )
    }
}
