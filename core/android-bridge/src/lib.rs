use std::{
    collections::HashMap,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicI64, Ordering},
        mpsc::Receiver,
    },
    thread,
    time::Duration,
};

use decentralized_ai_p2p::{
    Multiaddr, NodeEvent, NodeInfo, OP_ADD, TASK_VERSION, execute_task, run_node,
};
use decentralized_ai_task_protocol::TaskRequest as ProtocolTaskRequest;
use jni::{
    EnvUnowned,
    errors::ThrowRuntimeExAndDefault,
    objects::{JByteArray, JClass},
    sys::{JNI_FALSE, JNI_TRUE, jboolean, jint, jlong, jlongArray, jstring},
};
use tokio::sync::oneshot;
use zeroize::Zeroizing;

struct ManagedNode {
    stop: oneshot::Sender<()>,
    thread: thread::JoinHandle<()>,
    events: Receiver<NodeEvent>,
}

static NEXT_NODE_ID: AtomicI64 = AtomicI64::new(1);
static NODES: OnceLock<Mutex<HashMap<i64, ManagedNode>>> = OnceLock::new();

fn nodes() -> &'static Mutex<HashMap<i64, ManagedNode>> {
    NODES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn start_managed_node(
    peer_address: Option<String>,
    send_demo_addition: bool,
    identity_seed: [u8; 32],
) -> Result<(i64, NodeInfo), String> {
    let identity_seed = Zeroizing::new(identity_seed);
    let dial_address = peer_address
        .filter(|address| !address.trim().is_empty())
        .map(|address| {
            address
                .parse::<Multiaddr>()
                .map_err(|error| format!("invalid peer multiaddress: {error}"))
        })
        .transpose()?;
    if send_demo_addition && dial_address.is_none() {
        return Err("enter a peer multiaddress before sending the demo task".to_owned());
    }
    let id = NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed);
    let pending_task = send_demo_addition.then_some(ProtocolTaskRequest {
        version: TASK_VERSION,
        id,
        operation: OP_ADD,
        left: 20,
        right: 22,
    });
    let (stop_tx, stop_rx) = oneshot::channel();
    let (startup_tx, startup_rx) = std::sync::mpsc::channel();
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let node_thread = thread::Builder::new()
        .name("decentralized-ai-p2p".to_owned())
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    eprintln!("p2p_runtime_start_failed={error}");
                    return;
                }
            };
            if let Err(error) = runtime.block_on(run_node(
                dial_address,
                pending_task,
                Some(startup_tx),
                Some(stop_rx),
                Some(event_tx),
                Some(*identity_seed),
            )) {
                eprintln!("p2p_node_failed={error}");
            }
        })
        .map_err(|error| format!("could not start node thread: {error}"))?;

    let info = match startup_rx.recv_timeout(Duration::from_secs(15)) {
        Ok(info) => info,
        Err(error) => {
            let _ = stop_tx.send(());
            let _ = node_thread.join();
            return Err(format!("node did not begin listening: {error}"));
        }
    };
    let Ok(mut registry) = nodes().lock() else {
        let _ = stop_tx.send(());
        let _ = node_thread.join();
        return Err("node registry is unavailable".to_owned());
    };
    registry.insert(
        id,
        ManagedNode {
            stop: stop_tx,
            thread: node_thread,
            events: event_rx,
        },
    );
    Ok((id, info))
}

fn take_node_event(id: i64) -> String {
    let Ok(mut registry) = nodes().lock() else {
        return "error|node registry is unavailable".to_owned();
    };
    let Some(node) = registry.get_mut(&id) else {
        return "error|node handle was not found".to_owned();
    };
    match node.events.try_recv() {
        Ok(NodeEvent::PeerConnected(peer_id)) => format!("connected|{peer_id}"),
        Ok(NodeEvent::PeerDisconnected(peer_id)) => format!("disconnected|{peer_id}"),
        Ok(NodeEvent::ConnectionFailed(peer_id)) => {
            format!("connection_failed|{}", peer_id.unwrap_or_default())
        }
        Ok(NodeEvent::TaskSent(id)) => format!("task_sent|{id}"),
        Ok(NodeEvent::TaskFailed) => "task_failed".to_owned(),
        Ok(NodeEvent::TaskReceived(id)) => format!("task_received|{id}"),
        Ok(NodeEvent::TaskResponse { id, result, error }) => format!(
            "task_response|{id}|{}|{}",
            result.map(|value| value.to_string()).unwrap_or_default(),
            error.map(|value| value.to_string()).unwrap_or_default()
        ),
        Err(std::sync::mpsc::TryRecvError::Empty) => String::new(),
        Err(std::sync::mpsc::TryRecvError::Disconnected) => "node_closed".to_owned(),
    }
}

fn stop_managed_node(id: i64) -> bool {
    let Ok(mut registry) = nodes().lock() else {
        return false;
    };
    let Some(node) = registry.remove(&id) else {
        return false;
    };
    let _ = node.stop.send(());
    node.thread.join().is_ok()
}

#[jni::jni_mangle("ai.decentralized.app.NativeTaskBridge")]
pub fn execute<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    version: jint,
    id: jlong,
    operation: jint,
    left: jlong,
    right: jlong,
) -> jlongArray {
    unowned_env
        .with_env(|env| -> jni::errors::Result<jlongArray> {
            let request = ProtocolTaskRequest {
                version: u8::try_from(version).unwrap_or(0),
                id,
                operation: u8::try_from(operation).unwrap_or(0),
                left,
                right,
            };
            let response = execute_task(request);
            let values: [jlong; 4] = [
                jlong::from(response.version),
                response.id,
                response.result.unwrap_or(0),
                jlong::from(response.error.unwrap_or(0)),
            ];
            let array = env.new_long_array(values.len())?;
            array.set_region(env, 0, &values)?;
            Ok(array.into_raw())
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[jni::jni_mangle("ai.decentralized.app.NativeTaskBridge")]
pub fn start_node<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    peer_address: jni::objects::JString<'caller>,
    send_demo_addition: jboolean,
    identity_seed: JByteArray<'caller>,
) -> jstring {
    unowned_env
        .with_env(|env| -> jni::errors::Result<jstring> {
            let peer_address = peer_address.try_to_string(env)?;
            if identity_seed.len(env)? != 32 {
                return Ok(env
                    .new_string("error|Android identity seed must contain exactly 32 bytes")?
                    .into_raw());
            }
            let mut seed_bytes = [0i8; 32];
            identity_seed.get_region(env, 0, &mut seed_bytes)?;
            let mut seed = [0u8; 32];
            for (target, source) in seed.iter_mut().zip(seed_bytes.iter()) {
                *target = *source as u8;
            }
            seed_bytes.fill(0);
            let message =
                match start_managed_node(Some(peer_address), send_demo_addition != JNI_FALSE, seed)
                {
                    Ok((id, info)) => {
                        format!("ok|{id}|{}|{}", info.peer_id, info.listen_address)
                    }
                    Err(error) => format!("error|{error}"),
                };
            Ok(env.new_string(message)?.into_raw())
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[jni::jni_mangle("ai.decentralized.app.NativeTaskBridge")]
pub fn poll_node_event<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    id: jlong,
) -> jstring {
    unowned_env
        .with_env(|env| -> jni::errors::Result<jstring> {
            Ok(env.new_string(take_node_event(id))?.into_raw())
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[jni::jni_mangle("ai.decentralized.app.NativeTaskBridge")]
pub fn stop_node<'caller>(
    _unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    id: jlong,
) -> jboolean {
    if stop_managed_node(id) {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

