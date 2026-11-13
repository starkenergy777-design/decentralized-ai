use decentralized_ai_task_protocol::{TaskRequest, execute_task};
use jni::{
    EnvUnowned,
    errors::ThrowRuntimeExAndDefault,
    objects::JClass,
    sys::{jint, jlong, jlongArray},
};

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
            let request = TaskRequest {
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
