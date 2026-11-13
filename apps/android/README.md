# Android app

The Kotlin/Jetpack Compose settings screen is in `app/`. It saves local preferences for opt-in, CPU budget, memory budget, daily network budget, Wi-Fi-only, and charging-only behavior. Participation defaults to off. An opt-in user can enqueue a fixed local `20 + 22` demonstration task through WorkManager. The worker re-checks participation and budgets before executing; turning participation off cancels the queued demo task. WorkManager defers it until the selected unmetered-network/charging constraints and a non-low battery condition are met.

The foreground P2P node uses libp2p mDNS to discover and connect to opted-in peers on the same LAN. Android acquires a Wi-Fi multicast lock only on platform versions that require it and releases it when the node stops. mDNS reveals the node's Peer ID to the local network and does not provide internet-wide discovery or NAT traversal.

The local task uses the same versioned fields and operation/error codes as Rust's `/ai/task/1` protocol. Its arithmetic now crosses a narrow JNI boundary into the shared Rust protocol crate; the Rust P2P node itself is not connected. This remains a local scheduler proof: it does not receive P2P work, run an AI model, enforce an exact CPU percentage, or account for daily network use. Android's background-work restrictions mean the node must not assume it can run continuously; use scheduled work for deferrable jobs and a visible, user-started foreground service only for work that must remain active. No attestation or biometric check is implemented.

The Android Gradle build compiles and packages the JNI library for `arm64-v8a`, `armeabi-v7a`, and `x86_64`. Local builds need Rust stable, Android NDK, and `cargo-ndk` 4.1.2; CI installs these before assembling the APK.

Each successful CI run uploads its debug APK as `decentralized-ai-debug-apk-<commit-sha>` for 14 days. Download it from that run's Actions artifacts when preparing a same-Wi-Fi, two-device mDNS demonstration; it is a debug build, not a Play Store release.
