//! One fixed existing source, metadata only. No model, registry, PTS approval or GUI.
use crate::{infrastructure as io, json};
use std::{
    fs::{self, File, OpenOptions},
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
const SOURCE_SHA: &str = "7CFC8B8C6EEB19685048A289ACA208A01356F5082B470FB9F4D933FE4B2087EF";
pub(super) fn guards(path: &Path) -> Result<Vec<File>, String> {
    let canonical = io::ordinary(path)?;
    let mut ancestors: Vec<_> = path.ancestors().collect();
    ancestors.reverse();
    let mut result = Vec::new();
    for entry in ancestors {
        let directory = entry != path;
        let file = OpenOptions::new()
            .read(true)
            .share_mode(if directory { 3 } else { 1 })
            .custom_flags(if directory { 0x02200000 } else { 0x00200000 })
            .open(entry)
            .map_err(|e| e.to_string())?;
        let metadata = file.metadata().map_err(|e| e.to_string())?;
        if metadata.file_attributes() & 0x400 != 0 || metadata.is_dir() != directory {
            return Err("ordinary guarded file/directory required".into());
        }
        result.push(file);
    }
    if io::ordinary(path)? != canonical {
        return Err("path changed during custody acquisition".into());
    }
    Ok(result)
}
struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
fn bounded_probe(tool: &Path, source: &Path, output: &Path, frames: bool) -> Result<(), String> {
    let stdout = output.join(if frames {
        "frames.stdout.json"
    } else {
        "streams.stdout.json"
    });
    let stderr = output.join("streams.stderr");
    let started = Instant::now();
    let mut command = Command::new(tool);
    command.args([
        "-v",
        "error",
        "-protocol_whitelist",
        "file",
        "-f",
        "mov",
        "-enable_drefs",
        "0",
        "-probesize",
        "1048576",
        "-analyzeduration",
        "1000000",
        "-threads",
        "1",
    ]);
    if frames {
        command.args([
            "-select_streams",
            "v:0",
            "-read_intervals",
            "16%26",
            "-show_frames",
            "-show_streams",
            "-show_entries",
            "frame=pts,best_effort_timestamp,duration,key_frame:stream=index,time_base",
        ]);
    } else {
        command.args(["-show_program_version","-show_entries","stream=index,codec_type,codec_name,time_base,start_pts,start_time,duration_ts,duration,sample_rate,initial_padding:format=start_time,duration"]);
    }
    command.args(["-of", "json"]).arg(source);
    use std::os::windows::process::CommandExt;
    command
        .creation_flags(0x08000000)
        .stdin(Stdio::null())
        .stdout(File::create(&stdout).map_err(|e| e.to_string())?)
        .stderr(File::create(&stderr).map_err(|e| e.to_string())?);
    io::write_new(
        &output.join("command.txt"),
        format!(
            "{command:?}\nmax_response_bytes=65536\nmax_stderr_bytes=65536\ndeadline_ms=15000\n"
        )
        .as_bytes(),
    )?;
    let mut child = OwnedChild(command.spawn().map_err(|e| e.to_string())?);
    let result = loop {
        let large = fs::metadata(&stdout).map_err(|e| e.to_string())?.len() > 65536
            || fs::metadata(&stderr).map_err(|e| e.to_string())?.len() > 65536;
        if large || started.elapsed() > Duration::from_secs(15) {
            break Err(if large {
                "response exceeded accepted bound"
            } else {
                "metadata deadline elapsed"
            }
            .to_string());
        }
        if let Some(status) = child.0.try_wait().map_err(|e| e.to_string())? {
            break if status.success() {
                Ok(())
            } else {
                Err(format!("ffprobe exit={:?}", status.code()))
            };
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    if result.is_err() {
        let _ = child.0.kill();
        let _ = child.0.wait();
    }
    io::write_new(
        &output.join("process.txt"),
        format!(
            "pid={}\nresult={result:?}\nelapsed_ms={}\nterminal=true\n",
            child.0.id(),
            started.elapsed().as_millis()
        )
        .as_bytes(),
    )?;
    result?;
    if fs::metadata(&stdout).map_err(|e| e.to_string())?.len() > 65536
        || fs::metadata(&stderr).map_err(|e| e.to_string())?.len() > 65536
    {
        return Err("final metadata response exceeded bound".into());
    }
    let bytes = fs::read(&stdout).map_err(|e| e.to_string())?;
    let parsed = if frames {
        json::parse_observation(&bytes)?
    } else {
        json::parse(&bytes)?
    };
    if parsed.field("streams")?.array()?.is_empty() {
        return Err("no stream metadata".into());
    }
    if frames {
        let (count, first, last) = frame_bounds(&parsed)?;
        io::write_new(&output.join("FRAMES.txt"),format!("requested_read_interval=16%26\nframes={count}\nfirst_pts={first}\nlast_pts={last}\npts_strictly_increasing=true\nseek_preroll_may_precede_request=true\npcm_mapping_verified=false\n").as_bytes())?;
    }
    Ok(())
}
fn frame_bounds(parsed: &json::Value) -> Result<(usize, i64, i64), String> {
    let frames = parsed.field("frames")?.array()?;
    if frames.is_empty() || frames.len() > 600 {
        return Err("bounded frame count missing or exceeded".into());
    }
    let mut first = None;
    let mut previous = None;
    for frame in frames {
        let json::Value::Literal(pts) = frame.field("pts")? else {
            return Err("integer frame PTS required".into());
        };
        let pts: i64 = pts.parse().map_err(|_| "invalid signed frame PTS")?;
        let json::Value::Literal(best) = frame.field("best_effort_timestamp")? else {
            return Err("integer best effort timestamp required".into());
        };
        let _: i64 = best.parse().map_err(|_| "invalid best effort timestamp")?;
        if previous.is_some_and(|previous| pts <= previous) {
            return Err("frame PTS not strictly increasing".into());
        }
        first.get_or_insert(pts);
        previous = Some(pts);
    }
    Ok((frames.len(), first.unwrap(), previous.unwrap()))
}
pub fn run(args: &[String]) -> Result<(), String> {
    run_mode(args, false)
}
pub fn frames(args: &[String]) -> Result<(), String> {
    run_mode(args, true)
}
fn run_mode(args: &[String], frames: bool) -> Result<(), String> {
    if args.len() != 3 {
        return Err("expected sprite-metadata ROOT TRUSTED_FFPROBE NEW_RECEIPT_DIR".into());
    }
    let root = io::ordinary(Path::new(&args[0]))?;
    let source = root.join("runtime/ai-spike/source/sprite-fright-sprite-source-r1/source.mp4");
    let tool = io::ordinary(Path::new(&args[1]))?;
    let output = PathBuf::from(&args[2]);
    if !io::ordinary(output.parent().ok_or("output parent missing")?)?
        .starts_with(root.join("runtime/verification"))
    {
        return Err("new private verification directory required".into());
    }
    let _source_guards = guards(&source)?;
    let _tool_guards = guards(&tool)?;
    if fs::metadata(&source).map_err(|e| e.to_string())?.len() != 102392846 {
        return Err("fixed source byte length mismatch".into());
    }
    let source_identity = io::identity(&source)?;
    let tool_identity = io::identity(&tool)?;
    let source_sha = io::hash(&source)?;
    let tool_sha = io::hash(&tool)?;
    if source_sha != SOURCE_SHA {
        return Err("fixed original source content mismatch".into());
    }
    fs::create_dir(&output).map_err(|e| e.to_string())?;
    io::write_new(&output.join("INPUT.txt"),format!("source={}\nsource_sha256={source_sha}\nsource_identity={source_identity}\nffprobe={}\nffprobe_sha256={tool_sha}\nffprobe_identity={tool_identity}\nmetadata_only={}\nactual9_binding=false\nquality_approved=false\n",source.display(),tool.display(),!frames).as_bytes())?;
    let result = bounded_probe(&tool, &source, &output, frames);
    let unchanged =
        io::identity(&source)? == source_identity && io::identity(&tool)? == tool_identity;
    io::write_new(&output.join("RETURN.txt"),format!("probe={result:?}\nguarded_identity_unchanged={unchanged}\nverified_timeline=false\nframe_pts_observed={}\npcm_mapping_verified=false\n",frames&&result.is_ok()).as_bytes())?;
    result?;
    if !unchanged {
        return Err("guarded source/tool identity changed".into());
    }
    println!(
        "fixed existing Sprite content verified; bounded stream metadata captured; timeline unverified"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frame_observation_refuses_missing_duplicate_or_invented_timestamps() {
        let valid=json::parse_observation(br#"{"frames":[{"pts":-1,"best_effort_timestamp":-1},{"pts":0,"best_effort_timestamp":0}]}"#).unwrap();
        assert_eq!(frame_bounds(&valid).unwrap(), (2, -1, 0));
        for bytes in [br#"{"frames":[]}"#.as_slice(),br#"{"frames":[{"pts":null,"best_effort_timestamp":0}]}"#,br#"{"frames":[{"pts":0}]}"#,br#"{"frames":[{"pts":1,"best_effort_timestamp":1},{"pts":1,"best_effort_timestamp":1}]}"#] {
            assert!(frame_bounds(&json::parse(bytes).unwrap()).is_err());
        }
    }
    #[test]
    fn ordinary_file_guard_denies_mutation_then_releases_without_changing_content() {
        let directory =
            std::env::temp_dir().join(format!("sprite-metadata-guard-{}", std::process::id()));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("source.txt");
        fs::write(&path, b"independent guard fixture").unwrap();
        let before = io::hash(&path).unwrap();
        let retained = guards(&path).unwrap();
        assert!(OpenOptions::new().write(true).open(&path).is_err());
        assert!(fs::remove_file(&path).is_err());
        drop(retained);
        drop(OpenOptions::new().write(true).open(&path).unwrap());
        assert_eq!(io::hash(&path).unwrap(), before);
        let retained_again = guards(&path).unwrap();
        assert!(OpenOptions::new().write(true).open(&path).is_err());
        drop(retained_again);
    }
}
