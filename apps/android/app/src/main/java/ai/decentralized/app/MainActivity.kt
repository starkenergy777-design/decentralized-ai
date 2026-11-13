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
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlin.math.roundToInt

private const val PREFERENCES = "resource_policy"

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val preferences = getSharedPreferences(PREFERENCES, MODE_PRIVATE)
        setContent {
            MaterialTheme {
                ResourceSettingsScreen(preferences)
            }
        }
    }
}

@Composable
private fun ResourceSettingsScreen(preferences: SharedPreferences) {
    var participating by remember {
        mutableStateOf(preferences.getBoolean("participating", false))
    }
    var cpuBudget by remember {
        mutableIntStateOf(preferences.getInt("cpu_budget_percent", 10))
    }
    var memoryBudget by remember {
        mutableIntStateOf(preferences.getInt("memory_budget_mib", 256))
    }
    var networkBudget by remember {
        mutableIntStateOf(preferences.getInt("network_budget_mib_per_day", 256))
    }
    var wifiOnly by remember {
        mutableStateOf(preferences.getBoolean("wifi_only", true))
    }
    var chargingOnly by remember {
        mutableStateOf(preferences.getBoolean("charging_only", true))
    }

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
            "მონაწილეობა გამორთულია ნაგულისხმევად. ეს პარამეტრები ჯერ მხოლოდ ადგილობრივად ინახება; ფონური გამოთვლები ჯერ არ მუშაობს.",
            style = MaterialTheme.typography.bodyMedium,
        )

        HorizontalDivider()
        PolicySwitch(
            title = "გამოთვლებში მონაწილეობა",
            checked = participating,
            onCheckedChange = {
                participating = it
                preferences.edit().putBoolean("participating", it).apply()
            },
        )

        BudgetSlider(
            title = "CPU-ის სამიზნე ბიუჯეტი",
            value = cpuBudget,
            range = 0..50,
            steps = 9,
            valueLabel = "$cpuBudget%",
            onValueChange = { cpuBudget = it },
            onValueChangeFinished = {
                preferences.edit().putInt("cpu_budget_percent", cpuBudget).apply()
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
                preferences.edit().putInt("memory_budget_mib", memoryBudget).apply()
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
                preferences.edit().putInt("network_budget_mib_per_day", networkBudget).apply()
            },
        )

        HorizontalDivider()
        PolicySwitch(
            title = "მხოლოდ Wi-Fi-ზე",
            checked = wifiOnly,
            onCheckedChange = {
                wifiOnly = it
                preferences.edit().putBoolean("wifi_only", it).apply()
            },
        )
        PolicySwitch(
            title = "მხოლოდ დატენვისას",
            checked = chargingOnly,
            onCheckedChange = {
                chargingOnly = it
                preferences.edit().putBoolean("charging_only", it).apply()
            },
        )
        Spacer(Modifier.height(8.dp))
        Text(
            "CPU-ის პროცენტი არის სასურველი ბიუჯეტი და არა Android-ის მიერ გარანტირებული ზუსტი დატვირთვა. შემდეგ ეტაპზე მუშა scheduler ამ პარამეტრებს მოწყობილობის მდგომარეობასთან ერთად გამოიყენებს.",
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
