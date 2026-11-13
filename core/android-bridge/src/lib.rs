use std::{
    collections::HashMap,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicI64, Ordering},
    },
    thread,
    time::Duration,
};

use decentralized_ai_p2p::{Multiaddr, NodeInfo, execute_task, run_node};
use decentralized_ai_task_protocol::TaskRequest as ProtocolTaskRequest;
use jni::{
    EnvUnowned,
    errors::ThrowRuntimeExAndDefault,
    objects::JClass,
    sys::{JNI_FALSE, JNI_TRUE, jboolean, jint, jlong, jlongArray, jstring},
};
use tokio::sync::oneshot;

struct ManagedNode {
    stop: oneshot::Sender<()>,
    thread: thread::JoinHandle<()>,
}

static NEXT_NODE_ID: AtomicI64 = AtomicI64::new(1);
static NODES: OnceLock<Mutex<HashMap<i64, ManagedNode>>> = OnceLock::new();

fn nodes() -> &'static Mutex<HashMap<i64, ManagedNode>> {
    NODES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn start_managed_node(peer_address: Option<String>) -> Result<(i64, NodeInfo), String> {
    let dial_address = peer_address
        .filter(|address| !address.trim().is_empty())
        .map(|address| {
            address
                .parse::<Multiaddr>()
                .map_err(|error| format!("invalid peer multiaddress: {error}"))
        })
        .transpose()?;
    let (stop_tx, stop_rx) = oneshot::channel();
    let (startup_tx, startup_rx) = std::sync::mpsc::channel();
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
            if let Err(error) =
                runtime.block_on(run_node(
                    dial_address,
                    None,
                    Some(startup_tx),
                    Some(stop_rx),
                ))
            {
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
    let id = NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed);
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
        },
    );
    Ok((id, info))
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
) -> jstring {
    unowned_env
        .with_env(|env| -> jni::errors::Result<jstring> {
            let peer_address = peer_address.try_to_string(env)?;
            let message = match start_managed_node(Some(peer_address)) {
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
