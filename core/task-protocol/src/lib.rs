use serde::{Deserialize, Serialize};

pub const TASK_VERSION: u8 = 1;
pub const OP_ADD: u8 = 1;

pub const ERROR_INVALID_ID: u8 = 1;
pub const ERROR_UNSUPPORTED_VERSION: u8 = 2;
pub const ERROR_UNSUPPORTED_OPERATION: u8 = 3;
pub const ERROR_OVERFLOW: u8 = 4;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TaskRequest {
    pub version: u8,
    pub id: i64,
    pub operation: u8,
    pub left: i64,
    pub right: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TaskResponse {
    pub version: u8,
    pub id: i64,
    pub result: Option<i64>,
    pub error: Option<u8>,
}

pub fn execute_task(request: TaskRequest) -> TaskResponse {
    let error = if request.id <= 0 {
        Some(ERROR_INVALID_ID)
    } else if request.version != TASK_VERSION {
        Some(ERROR_UNSUPPORTED_VERSION)
    } else if request.operation != OP_ADD {
        Some(ERROR_UNSUPPORTED_OPERATION)
    } else {
        None
    };

    if let Some(error) = error {
        return TaskResponse {
            version: TASK_VERSION,
            id: request.id,
            result: None,
            error: Some(error),
        };
    }

    match request.left.checked_add(request.right) {
        Some(result) => TaskResponse {
            version: TASK_VERSION,
            id: request.id,
            result: Some(result),
            error: None,
        },
        None => TaskResponse {
            version: TASK_VERSION,
            id: request.id,
            result: None,
            error: Some(ERROR_OVERFLOW),
        },
    }
}
