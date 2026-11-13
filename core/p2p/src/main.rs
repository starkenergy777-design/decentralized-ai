use std::error::Error;

use decentralized_ai_p2p::{OP_ADD, TASK_VERSION, TaskRequest, run_node};
use libp2p::Multiaddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut args = std::env::args().skip(1);
    let dial_address = args
        .next()
        .map(|address| address.parse::<Multiaddr>())
        .transpose()?;
    let rendezvous_server = std::env::var("DAI_RENDEZVOUS_ADDR")
        .ok()
        .filter(|address| !address.trim().is_empty())
        .map(|address| address.parse::<Multiaddr>())
        .transpose()?;
    let pending_task = match args.next().as_deref() {
        Some("--add") => {
            let left = args
                .next()
                .ok_or("--add requires two integer operands")?
                .parse()?;
            let right = args
                .next()
                .ok_or("--add requires two integer operands")?
                .parse()?;
            Some(TaskRequest {
                version: TASK_VERSION,
                id: 1,
                operation: OP_ADD,
                left,
                right,
            })
        }
        Some(other) => return Err(format!("unknown option: {other}").into()),
        None => None,
    };

    run_node(
        dial_address,
        pending_task,
        None,
        None,
        None,
        None,
        rendezvous_server,
    )
    .await
}

#[cfg(test)]
mod tests {
    use decentralized_ai_p2p::{ERROR_OVERFLOW, OP_ADD, TASK_VERSION, TaskRequest, execute_task};

    #[test]
    fn addition_returns_checked_result() {
        let response = execute_task(TaskRequest {
            version: TASK_VERSION,
            id: 7,
            operation: OP_ADD,
            left: 20,
            right: 22,
        });

        assert_eq!(response.version, TASK_VERSION);
        assert_eq!(response.id, 7);
        assert_eq!(response.result, Some(42));
        assert_eq!(response.error, None);
    }

    #[test]
    fn addition_overflow_is_rejected() {
        let response = execute_task(TaskRequest {
            version: TASK_VERSION,
            id: 8,
            operation: OP_ADD,
            left: i64::MAX,
            right: 1,
        });

        assert_eq!(response.id, 8);
        assert_eq!(response.result, None);
        assert_eq!(response.error, Some(ERROR_OVERFLOW));
    }
}
