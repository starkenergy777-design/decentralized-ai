# Android app

The initial Kotlin/Jetpack Compose settings screen is in `app/`. It saves local preferences for opt-in, CPU budget, memory budget, daily network budget, Wi-Fi-only, and charging-only behavior. Participation defaults to off.

These preferences do not yet start or enforce background compute. The next work is a bounded scheduler that reads the policy, pauses on disallowed device conditions, and always supports immediate cancellation. Android's background-work restrictions mean the node must not assume it can run continuously; use scheduled work for deferrable jobs and a visible, user-started foreground service only for work that must remain active. The shared Rust networking core will be integrated later through a reviewed JNI/FFI boundary. No attestation or biometric check is implemented.
