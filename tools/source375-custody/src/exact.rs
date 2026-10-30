//! Run only frozen exact cases on unique original Cargo test producers.
use crate::{
    infrastructure as io,
    json::{self, Value},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};

struct Case {
    target: String,
    name: String,
    ignored: bool,
}
struct Producer {
    exe: PathBuf,
    files: Vec<(PathBuf, String, String)>,
    package: String,
    features: String,
    fresh: bool,
}
fn yes(value: &Value) -> bool {
    matches!(value,Value::Literal(s) if s=="true")
}
fn cases(plan: &Value, group: &str) -> Result<Vec<Case>, String> {
    let mut cases = Vec::new();
    let mut seen = BTreeSet::new();
    for row in plan.array()? {
        if row.field("group")?.text()? != group {
            continue;
        }
        let target = row.field("target")?.text()?.to_string();
        let name = row.field("name")?.text()?.to_string();
        if !seen.insert((target.clone(), name.clone())) {
            return Err("duplicate expected case".into());
        }
        cases.push(Case {
            target,
            name,
            ignored: yes(row.field("ignored")?),
        });
    }
    let expected = match group {
        "product" => 145,
        "meari-core" => 18,
        "meari-native" => 26,
        "meari-os" => 1,
        _ => return Err("unknown frozen group".into()),
    };
    if cases.len() != expected {
        return Err("group case count differs".into());
    }
    Ok(cases)
}
fn producers(
    path: &Path,
    wanted: &BTreeSet<String>,
    cache: &Path,
) -> Result<BTreeMap<String, Producer>, String> {
    let mut output = BTreeMap::new();
    for line in fs::read_to_string(path).map_err(|e| e.to_string())?.lines() {
        if !line.starts_with('{') {
            continue;
        }
        let row = json::parse(line.as_bytes())?;
        if row.field("reason")?.text()? != "compiler-artifact" {
            continue;
        }
        let target = row.field("target")?.field("name")?.text()?;
        if !wanted.contains(target) || !yes(row.field("profile")?.field("test")?) {
            continue;
        }
        let Ok(executable) = row.field("executable")?.text() else {
            continue;
        };
        let exe = io::ordinary(Path::new(executable))?;
        if !exe.starts_with(cache) {
            return Err("producer outside approved build cache".into());
        }
        if output.contains_key(target) {
            return Err(format!("multiple original producers: {target}"));
        }
        let mut files = Vec::new();
        for path in [exe.clone(), exe.with_extension("pdb")] {
            io::ordinary(&path)?;
            files.push((path.clone(), io::hash(&path)?, io::identity(&path)?));
        }
        output.insert(
            target.to_string(),
            Producer {
                exe,
                files,
                package: row.field("package_id")?.text()?.into(),
                features: format!("{:?}", row.field("features")?),
                fresh: yes(row.field("fresh")?),
            },
        );
    }
    if output.len() != wanted.len() {
        return Err("missing expected original producer".into());
    }
    Ok(output)
}
fn unchanged(producer: &Producer, content: bool) -> Result<(), String> {
    for (path, sha, metadata) in &producer.files {
        if io::identity(path)? != *metadata || content && io::hash(path)? != *sha {
            return Err(format!("original producer changed: {}", path.display()));
        }
    }
    Ok(())
}
fn invoke(
    producer: &Producer,
    args: &[String],
    directory: &Path,
    label: &str,
) -> Result<String, String> {
    unchanged(producer, false)?;
    let metadata = |phase: &str| -> Result<(), String> {
        let mut rows = String::from("path\tfileid\tbytes\tcreation100ns\tmtime100ns\n");
        for (path, _, _) in &producer.files {
            rows.push_str(&format!("{}\t{}\n", path.display(), io::identity(path)?));
        }
        io::write_new(
            &directory.join(format!("{label}.{phase}.tsv")),
            rows.as_bytes(),
        )
    };
    metadata("before")?;
    let started = Instant::now();
    let child = Command::new(&producer.exe)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let child_pid = child.id();
    io::write_new(
        &directory.join(format!("{label}.spawn.txt")),
        format!("launcher_pid={}\nchild_pid={child_pid}\nexecutable={}\nargs={args:?}\nterminal=false\n", std::process::id(), producer.exe.display()).as_bytes(),
    )?;
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    metadata("after")?;
    io::write_new(&directory.join(format!("{label}.stdout")), &output.stdout)?;
    io::write_new(&directory.join(format!("{label}.stderr")), &output.stderr)?;
    io::write_new(
        &directory.join(format!("{label}.receipt.txt")),
        format!(
            "launcher_pid={}\nchild_pid={child_pid}\nexecutable={}\nargs={args:?}\nexit_code={:?}\nelapsed_ms={}\nterminal=true\n",
            std::process::id(),
            producer.exe.display(),
            output.status.code(),
            started.elapsed().as_millis()
        )
        .as_bytes(),
    )?;
    unchanged(producer, false)?;
    if !output.status.success() {
        return Err(format!(
            "first failed invocation {label}, exit={:?}",
            output.status.code()
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| e.to_string())
}
fn one_pass(text: &str) -> bool {
    text.lines().any(|line| line == "running 1 test")
        && text.contains("test result: ok. 1 passed; 0 failed; 0 ignored;")
}
pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() != 6 {
        return Err(
            "expected exact PLAN CARGO_JSON GROUP NEW_LOG_DIR CACHE_ROOT SOURCE_RECEIPT".into(),
        );
    }
    let plan_path = Path::new(&args[0]);
    if io::hash(plan_path)? != "082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464" {
        return Err("exact190 pin mismatch".into());
    }
    let plan = json::parse(&fs::read(plan_path).map_err(|e| e.to_string())?)?;
    let cases = cases(&plan, &args[2])?;
    let wanted = cases.iter().map(|c| c.target.clone()).collect();
    let cache = io::ordinary(Path::new(&args[4]))?;
    let producers = producers(Path::new(&args[1]), &wanted, &cache)?;
    let directory = Path::new(&args[3]);
    fs::create_dir(directory).map_err(|e| e.to_string())?;
    let source_receipt = io::ordinary(Path::new(&args[5]))?;
    let source_pin = io::hash(&source_receipt)?;
    io::copy_verified(
        &source_receipt,
        &directory.join("source-receipt.txt"),
        &source_pin,
    )?;
    let mut provenance = String::from(
        "target\tpath\tsha256\tfileid\tbytes\tcreation100ns\tmtime100ns\tpackage_id\tfeatures\tfresh\n",
    );
    for (target, producer) in &producers {
        for (path, sha, identity) in &producer.files {
            provenance.push_str(&format!(
                "{target}\t{}\t{sha}\t{identity}\t{}\t{}\t{}\n",
                path.display(),
                producer.package,
                producer.features,
                producer.fresh
            ));
        }
    }
    io::write_new(
        &directory.join("producers-before.tsv"),
        provenance.as_bytes(),
    )?;
    let result = (|| {
        for (target, producer) in &producers {
            let listing = invoke(
                producer,
                &["--list".into()],
                directory,
                &format!("list-{target}"),
            )?;
            let ignored = invoke(
                producer,
                &["--list".into(), "--ignored".into()],
                directory,
                &format!("ignored-{target}"),
            )?;
            for case in cases.iter().filter(|c| c.target == *target) {
                let needle = format!("{}: test", case.name);
                if listing.lines().filter(|line| *line == needle).count() != 1
                    || ignored.lines().any(|line| line == needle) != case.ignored
                {
                    return Err(format!("registration/ignored mismatch: {}", case.name));
                }
            }
        }
        io::write_new(
            &directory.join("registration.txt"),
            format!(
                "group={}\nexpected_registered={}\nignored_verified=true\n",
                args[2],
                cases.len()
            )
            .as_bytes(),
        )?;
        for (index, case) in cases.iter().enumerate() {
            let mut invocation = vec![
                "--exact".into(),
                case.name.clone(),
                "--nocapture".into(),
                "--test-threads=1".into(),
            ];
            if case.ignored {
                invocation.push("--ignored".into());
            }
            let label = format!("exact-{:03}", index + 1);
            let output = invoke(&producers[&case.target], &invocation, directory, &label)?;
            if !one_pass(&output) {
                return Err(format!("not one passing exact test: {label}"));
            }
            io::write_new(
                &directory.join(format!("{label}.passed")),
                format!("{}\t{}\t{}\n", args[2], case.target, case.name).as_bytes(),
            )?;
            println!("{} exact {}/{} passed", args[2], index + 1, cases.len());
        }
        Ok(())
    })();
    let mut integrity = Ok(());
    for producer in producers.values() {
        if let Err(error) = unchanged(producer, true) {
            integrity = Err(error);
            break;
        }
    }
    io::write_new(&directory.join("group-return.txt"),format!("group={}\nexpected={}\nexecution_result={result:?}\nproducer_integrity={integrity:?}\nsource_receipt={}\nsource_receipt_sha256={source_pin}\n",args[2],cases.len(),source_receipt.display()).as_bytes())?;
    result?;
    integrity?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn successful_exit_without_single_execution_is_not_a_pass() {
        assert!(one_pass(
            "running 1 test\ntest result: ok. 1 passed; 0 failed; 0 ignored; 4 filtered out;\n"
        ));
        for text in [
            "running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored;",
            "running 1 test\ntest result: ok. 0 passed; 0 failed; 1 ignored;",
            "running 1 test\ntest result: FAILED. 0 passed; 1 failed; 0 ignored;",
        ] {
            assert!(!one_pass(text));
        }
    }
}
