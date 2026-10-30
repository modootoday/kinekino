use super::fixture::{Fixture, TestResult};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{os::windows::io::AsRawHandle, path::PathBuf, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::windows::named_pipe::ClientOptions,
};
use windows::Win32::{Foundation::HANDLE, System::Pipes::GetNamedPipeServerProcessId};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Native {
    pub properties: Value,
    pub paused: bool,
    pub position_us: u64,
    pub volume: f64,
    pub duration_us: u64,
    pub muted: bool,
}
pub fn observe(fixture: &Fixture) -> TestResult<Native> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "Cannot create native observation runtime")?;
    let properties = rt.block_on(async {
        tokio::time::timeout(Duration::from_secs(3), async {
            let mut pipe = ClientOptions::new()
                .open(&fixture.mpv_pipe)
                .map_err(|_| "Owned native observation pipe unavailable")?;
            let mut pid = 0;
            unsafe { GetNamedPipeServerProcessId(HANDLE(pipe.as_raw_handle()), &mut pid) }
                .map_err(|_| "Cannot verify native observation owner")?;
            if pid != fixture.mpv_pid {
                return Err("Native pipe process identity mismatch".into());
            }
            let mut properties = serde_json::Map::new();
            for (index, name) in [
                "path",
                "pause",
                "time-pos",
                "volume",
                "duration",
                "idle-active",
                "mute",
            ]
            .into_iter()
            .enumerate()
            {
                let id = index + 100;
                let wire = format!(
                    "{}\n",
                    json!({"command":["get_property",name],"request_id":id})
                );
                pipe.write_all(wire.as_bytes())
                    .await
                    .map_err(|_| "Native read-only query failed")?;
                loop {
                    let mut line = Vec::new();
                    loop {
                        let byte = pipe
                            .read_u8()
                            .await
                            .map_err(|_| "Native read-only response ended")?;
                        if byte == b'\n' {
                            break;
                        }
                        if line.len() >= 65536 {
                            return Err("Native response exceeds bound".into());
                        }
                        line.push(byte);
                    }
                    let response: Value = serde_json::from_slice(&line)
                        .map_err(|_| "Invalid native read-only response")?;
                    if response["request_id"] != id {
                        continue;
                    }
                    if response["error"] != "success" {
                        return Err("Native property unavailable".into());
                    }
                    properties.insert(name.into(), response["data"].clone());
                    break;
                }
            }
            Ok::<_, String>(Value::Object(properties))
        })
        .await
        .map_err(|_| "Native read-only deadline expired")?
    })?;
    let actual = properties["path"]
        .as_str()
        .ok_or("Native source identity absent")?;
    if PathBuf::from(actual).canonicalize().ok() != fixture.source_path.canonicalize().ok() {
        return Err("Native source changed; mutation/restoration refused".into());
    }
    if properties["idle-active"] != false {
        return Err("Owned native engine is idle".into());
    }
    let seconds = |name: &str| -> TestResult<u64> {
        let value = properties[name]
            .as_f64()
            .filter(|v| v.is_finite() && *v >= 0.)
            .ok_or("Native numeric property invalid")?;
        Ok((value * 1_000_000.).round() as u64)
    };
    Ok(Native {
        paused: properties["pause"].as_bool().ok_or("Native pause absent")?,
        position_us: seconds("time-pos")?,
        volume: properties["volume"]
            .as_f64()
            .filter(|v| v.is_finite() && (0. ..=100.).contains(v))
            .ok_or("Native volume invalid")?,
        duration_us: seconds("duration")?,
        muted: properties["mute"].as_bool().ok_or("Native mute absent")?,
        properties,
    })
}
