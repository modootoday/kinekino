//! Single pinned existing source replay; this never constructs product Verified evidence.
mod clock;
mod process;
use crate::{
    infrastructure as io,
    sprite_metadata::guards,
    sprite_pcm::{self, Pcm},
};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};
pub fn run(args: &[String]) -> Result<(), String> {
    run_mode(
        args,
        true,
        "09948D4CDD0650DA6FF5A87577469F2A218DC2615AE379F8F734D24C49DE0F73",
    )
}
pub fn run_default(args: &[String]) -> Result<(), String> {
    run_mode(
        args,
        false,
        "09948D4CDD0650DA6FF5A87577469F2A218DC2615AE379F8F734D24C49DE0F73",
    )
}
pub fn run_legacy(args: &[String]) -> Result<(), String> {
    run_mode(
        args,
        true,
        "71414777922D05F2639A5CF4A0F9A405F169809CF459000A6F976C9ADD61F060",
    )
}
fn run_mode(args: &[String], trace: bool, tool_pin: &str) -> Result<(), String> {
    if args.len() != 3 {
        return Err("expected sprite-audio-replay ROOT FIXED_FFMPEG NEW_RECEIPT_DIR".into());
    }
    let root = io::ordinary(Path::new(&args[0]))?;
    let tool = io::ordinary(Path::new(&args[1]))?;
    let output = PathBuf::from(&args[2]);
    if !io::ordinary(output.parent().ok_or("output parent missing")?)?
        .starts_with(root.join("runtime/verification"))
    {
        return Err("private new receipt directory required".into());
    }
    let source = root.join("runtime/ai-spike/source/sprite-fright-sprite-source-r1/source.mp4");
    let pcm = root.join("runtime/ai-spike/assets/sprite-clips-r1/full-mono-16k.wav");
    let _source = guards(&source)?;
    let _pcm = guards(&pcm)?;
    let _tool = guards(&tool)?;
    let mut inputs = String::new();
    let mut identities = vec![];
    for (path, expected) in [
        (
            &source,
            "7CFC8B8C6EEB19685048A289ACA208A01356F5082B470FB9F4D933FE4B2087EF",
        ),
        (
            &pcm,
            "872987BC1C6296B33241E833F5A315E8BD656B2D3ED05D14034231CAD4AE51A4",
        ),
        (&tool, tool_pin),
    ] {
        let identity = io::identity(path)?;
        let hash = io::hash(path)?;
        if hash != expected {
            return Err(format!(
                "fixed replay input hash differs: {}",
                path.display()
            ));
        }
        inputs.push_str(&format!(
            "path={}\nsha256={hash}\nidentity={identity}\n",
            path.display()
        ));
        identities.push((path.clone(), identity));
    }
    let temporary_parent = io::ordinary(&root.join("runtime/build-cache/tmp"))?;
    let temporary = temporary_parent.join(output.file_name().ok_or("receipt name required")?);
    fs::create_dir(&output).map_err(|e| e.to_string())?;
    fs::create_dir(&temporary).map_err(|e| e.to_string())?;
    let raw = temporary.join("decoded.s16le");
    io::write_new(&output.join("INPUT.txt"),format!("{inputs}raw_output={}\ntrace={trace}\nsource_timebase_preserved_by_copyts={trace}\nfilter_timebase={}\nquality_approved=false\nactual9_binding=false\n",raw.display(),if trace{"1/16000"}else{"not_observed"}).as_bytes())?;
    let decode = process::decode(&tool, &source, &raw, &output, trace);
    let observed = if trace && decode.is_ok() {
        clock::parse(&fs::read_to_string(output.join("decode.stderr")).map_err(|e| e.to_string())?)
    } else {
        Err("not checked: decode failed or default recipe has no clock trace".into())
    };
    let matched = (|| {
        decode.as_ref().map_err(Clone::clone)?;
        if trace {
            let clock = observed.as_ref().map_err(Clone::clone)?;
            io::write_new(&output.join("CLOCK.txt"),format!("frames={}\nfirst_pts={}\nend_pts={}\nsamples={}\nfilter_timebase=1/16000\ninteger_continuity=true\n",clock.frames,clock.first_pts,clock.end_pts,clock.samples).as_bytes())?;
            if clock.first_pts != 0
                || clock.samples != 10078204
                || clock.end_pts != i128::from(clock.samples)
            {
                return Err(
                    "observed audio clock differs from original zero-origin sample timeline".into(),
                );
            }
        }
        let mut original = File::open(&pcm).map_err(|e| e.to_string())?;
        let whole = sprite_pcm::layout(&mut original)?;
        let bytes = fs::metadata(&raw).map_err(|e| e.to_string())?.len();
        if bytes != whole.bytes || bytes != 10078204 * 2 {
            return Err("replayed PCM byte/sample length differs".into());
        }
        sprite_pcm::compare(
            &mut original,
            whole,
            &mut File::open(&raw).map_err(|e| e.to_string())?,
            Pcm { start: 0, bytes },
            0,
        )
    })();
    let unchanged = identities
        .iter()
        .all(|(path, before)| io::identity(path).is_ok_and(|after| after == *before));
    io::write_new(&output.join("RETURN.txt"),format!("decode={decode:?}\nclock={observed:?}\npcm_data_match={matched:?}\nguarded_identity_unchanged={unchanged}\nproduct_verified_timeline=false\nquality_approved=false\nactual9_binding=false\n").as_bytes())?;
    decode?;
    matched?;
    if !unchanged {
        return Err("replay input identity changed".into());
    }
    io::write_new(
        &output.join("PCM-SHA.txt"),
        format!("raw_pcm_sha256={}\n", io::hash(&raw)?).as_bytes(),
    )?;
    println!(
        "current decoder PCM matches existing data; trace={trace}; product timeline not promoted"
    );
    Ok(())
}
