use crate::application::native_playback::NativePlayback;
use serde_json::{Value, json};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
pub(super) type LatestPlayback = Arc<Mutex<NativePlayback>>;
fn publish(target: &LatestPlayback, value: NativePlayback) {
    if let Ok(mut latest) = target.lock() {
        *latest = value;
    }
}
use kinekino_core::platform::ipc::IpcStream;
use tokio::{io::AsyncWriteExt, sync::mpsc::Receiver};

pub(super) fn connect(
    pipe: String,
    commands: Receiver<Value>,
    updates: LatestPlayback,
    connection_state: std::sync::Arc<super::control::ConnectionState>,
) -> std::io::Result<std::thread::JoinHandle<()>> {
    std::thread::Builder::new()
        .name("player-mpv-ipc".into())
        .spawn(move || {
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(anyhow::Error::from)
                .and_then(|runtime| {
                    runtime.block_on(open(
                        pipe,
                        commands,
                        updates.clone(),
                        connection_state.clone(),
                    ))
                });
            // Retire the original CAS generation before publishing disconnected.
            let _ = connection_state.guarded.close();
            connection_state
                .connected
                .store(false, std::sync::atomic::Ordering::Release);
            if let Ok(mut observations) = connection_state.native_observation.lock() {
                observations.disconnect();
            }
            if let Err(error) = result {
                publish(
                    &updates,
                    NativePlayback {
                        error: Some(error.to_string()),
                        ..Default::default()
                    },
                );
            }
        })
}

async fn open(
    pipe: String,
    commands: Receiver<Value>,
    updates: LatestPlayback,
    state: std::sync::Arc<super::control::ConnectionState>,
) -> anyhow::Result<()> {
    for _ in 0..50 {
        if !state.live.load(std::sync::atomic::Ordering::Acquire) || state.guarded.fence.is_closed()
        {
            return Ok(());
        }
        if let Ok(connection) = crate::infrastructure::os_policy::ipc().connect_address(&pipe) {
            return run(connection, commands, updates, state).await;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    anyhow::bail!("mpv IPC connection timed out")
}

async fn run(
    connection: Box<dyn IpcStream>,
    mut commands: Receiver<Value>,
    updates: LatestPlayback,
    connection_state: std::sync::Arc<super::control::ConnectionState>,
) -> anyhow::Result<()> {
    if !connection_state
        .live
        .load(std::sync::atomic::Ordering::Acquire)
        || connection_state.guarded.fence.is_closed()
    {
        return Ok(());
    }
    let (reader, mut writer) = tokio::io::split(connection);
    let mut lines = super::message_reader::MessageReader::new(reader);
    let mut guarded_commands = connection_state
        .guarded
        .take_receiver()
        .map_err(protocol_error)?;
    let mut guarded = super::guarded_command::Worker::new();
    let mut request_ids = super::guarded_command::RequestIds::default();
    connection_state
        .native_observation
        .lock()
        .map_err(|_| anyhow::anyhow!("Native receive observations poisoned"))?
        .connect()
        .map_err(anyhow::Error::msg)?;
    let mut state = NativePlayback {
        connected: true,
        ..Default::default()
    };
    publish(&updates, state.clone());
    connection_state
        .connected
        .store(true, std::sync::atomic::Ordering::Release);
    let mut lifetime = tokio::time::interval(Duration::from_millis(100));
    for (id, property) in [
        "time-pos",
        "duration",
        "pause",
        "idle-active",
        "volume",
        "mute",
        "video-params",
        "audio-params",
        "fullscreen",
        "speed",
        "track-list",
        "aid",
        "vid",
        "sid",
        "seekable",
        "partially-seekable",
    ]
    .iter()
    .enumerate()
    {
        let limit =
            super::control::guarded_deadline(std::time::Instant::now()).map_err(protocol_error)?;
        let packet = super::guarded_command::Packet {
            wire: json!({"command":["observe_property",id+1,property],"request_id":request_ids.next().map_err(protocol_error)?}),
            deadline: connection_state
                .guarded
                .registry
                .pending_deadline()
                .map_err(protocol_error)?
                .unwrap_or(limit)
                .min(limit),
            permit: None,
            ticket: None,
        };
        // The same lifetime timer is reused by subscription, legacy and guarded writes.
        write_packets(
            &mut writer,
            vec![packet],
            &connection_state.guarded,
            &mut lifetime,
        )
        .await?;
    }
    loop {
        tokio::select! {
            _=lifetime.tick()=>{
                if guarded.tick(&connection_state.guarded, std::time::Instant::now()).map_err(protocol_error)? {
                    anyhow::bail!("Native queued or in-flight command deadline elapsed; original connection discarded");
                }
                if !connection_state.live.load(std::sync::atomic::Ordering::Acquire) {
                    break;
                }
            }
            message=lines.next()=>{
                let Some(message)=message? else { break; };
                connection_state.native_observation.lock().map_err(|_| anyhow::anyhow!("Native receive observations poisoned"))?.receive(&message,std::time::Instant::now()).map_err(anyhow::Error::msg)?;
                if matches!(message["event"].as_str(),Some("start-file"|"end-file"|"shutdown")) {
                    connection_state.guarded.fence.advance().map_err(protocol_error)?;
                }
                let generations = connection_state.native_observation.lock().map_err(|_|anyhow::anyhow!("Native receive observations poisoned"))?.generations();
                let packets = guarded.receive(&message,generations,&mut request_ids,&connection_state.guarded,std::time::Instant::now()).map_err(protocol_error)?;
                write_packets(&mut writer,packets,&connection_state.guarded,&mut lifetime).await?;
                connection_state.options.lock().map_err(|_| anyhow::anyhow!("Native option observations unavailable"))?.apply(&message);
                super::events::apply(&mut state,&message);
                publish(&updates,state.clone());
                if message["event"].as_str()==Some("shutdown") {break;}
            }
            command=commands.recv()=>{
                let Some(command)=command else { break; };
                let limit=super::control::guarded_deadline(std::time::Instant::now()).map_err(protocol_error)?;
                let packet=super::guarded_command::Packet {
                    wire:json!({"command":command,"request_id":request_ids.next().map_err(protocol_error)?}),
                    deadline:connection_state.guarded.registry.pending_deadline().map_err(protocol_error)?.unwrap_or(limit).min(limit),
                    permit:None,ticket:None,
                };
                write_packets(&mut writer,vec![packet],&connection_state.guarded,&mut lifetime).await?;
            }
            command=guarded_commands.recv(), if guarded.can_begin()=>{
                let Some(command)=command else { anyhow::bail!("Native guarded queue closed"); };
                let packets=guarded.begin(command,&mut request_ids,&connection_state.guarded,std::time::Instant::now()).map_err(protocol_error)?;
                write_packets(&mut writer,packets,&connection_state.guarded,&mut lifetime).await?;
            }
        }
    }
    state.connected = false;
    publish(&updates, state);
    Ok(())
}

pub(super) async fn write_packets<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    packets: Vec<super::guarded_command::Packet>,
    host: &super::guarded_command::GuardedQueue,
    lifetime: &mut tokio::time::Interval,
) -> anyhow::Result<()> {
    use super::guarded_command::{Failure, FailureStage};
    use crate::application::playback_session::GuardedSeekState;
    for packet in packets {
        let frame = format!("{}\n", packet.wire);
        let result = if host.fence.is_closed() {
            Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "Original native connection closed before writer poll",
            ))
        } else if std::time::Instant::now() >= packet.deadline
            || host
                .registry
                .pending_deadline()
                .map_err(protocol_error)?
                .is_some_and(|deadline| std::time::Instant::now() >= deadline)
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Native frame deadline elapsed before writer poll",
            ))
        } else {
            let write = writer.write_all(frame.as_bytes());
            tokio::pin!(write);
            loop {
                tokio::select! {
                    biased;
                    _=lifetime.tick()=>{
                        if host.fence.is_closed() {
                            break Err(std::io::Error::new(std::io::ErrorKind::Interrupted,"Original native connection closed during frame write"));
                        }
                        if host.registry.pending_deadline().map_err(protocol_error)?.is_some_and(|deadline|std::time::Instant::now()>=deadline) {
                            break Err(std::io::Error::new(std::io::ErrorKind::TimedOut,"Original guarded request deadline elapsed during frame write"));
                        }
                        if std::time::Instant::now() >= packet.deadline {
                            break Err(std::io::Error::new(std::io::ErrorKind::TimedOut,"Native frame write deadline elapsed"));
                        }
                    }
                    result=&mut write=>break result,
                }
            }
        };
        if let Some(ticket) = &packet.ticket {
            match &result {
                Ok(()) => {
                    host.registry
                        .transition(ticket, GuardedSeekState::WrittenAwaitingAck, None)
                        .map_err(protocol_error)?;
                }
                Err(error) => {
                    host.registry
                        .transition(
                            ticket,
                            GuardedSeekState::OutcomeUnobserved,
                            Some(Failure::io(
                                if error.kind() == std::io::ErrorKind::TimedOut {
                                    FailureStage::Deadline
                                } else {
                                    FailureStage::Write
                                },
                                error,
                            )),
                        )
                        .map_err(protocol_error)?;
                }
            }
        }
        // The original logical write permit survives the entire pinned write future.
        drop(packet.permit);
        if let Err(error) = &result {
            host.fail_connection(error).map_err(protocol_error)?;
        }
        result?; // A partial/failed frame discards this connection, never appends another frame.
    }
    Ok(())
}

fn protocol_error(error: crate::application::api::ApiError) -> anyhow::Error {
    anyhow::anyhow!(
        "Native guarded protocol {:?}: {}",
        error.code,
        error.message
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closed_owner_joins_before_any_connection_retry_or_error_observation() {
        let (_sender, commands) = tokio::sync::mpsc::channel(1);
        let updates = Arc::new(Mutex::new(NativePlayback::default()));
        let reader = updates.clone();
        let state = std::sync::Arc::new(super::super::control::ConnectionState::default());
        let worker = connect(
            r"\\.\pipe\synthetic-closed-owner-never-connect".into(),
            commands,
            updates,
            state,
        )
        .unwrap();
        worker.join().unwrap();
        let observation = reader.lock().unwrap();
        assert!(observation.error.is_none());
        assert!(!observation.connected);
    }
}
