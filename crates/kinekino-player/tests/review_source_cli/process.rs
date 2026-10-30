use super::fixture::{Fixture, sha};
use serde_json::Value;
use std::{
    fs::{self, File},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
pub fn call(f: &Fixture, label: &str, request: &Value, ok: bool, code: Option<&str>) -> Value {
    call_with_options(f, label, request, ok, code, &[])
}
pub fn call_with_options(
    f: &Fixture,
    label: &str,
    request: &Value,
    ok: bool,
    code: Option<&str>,
    host_options: &[String],
) -> Value {
    let input = f.root.join(format!("{label}.request.json"));
    let output = f.root.join(format!("{label}.stdout.json"));
    let error = f.root.join(format!("{label}.stderr"));
    fs::write(&input, serde_json::to_vec_pretty(request).unwrap()).unwrap();
    let executable = crate::cli_exe::kinekino_cli_exe();
    let before = sha(&fs::read(executable).unwrap());
    let mut child = OwnedChild(
        Command::new(executable)
            .args([
                "--input",
                input.to_str().unwrap(),
                "--json",
                "--review-root",
                f.evidence.to_str().unwrap(),
                "--user-review-root",
                f.user.to_str().unwrap(),
            ])
            .args(host_options)
            .stdin(Stdio::null())
            .stdout(File::create(&output).unwrap())
            .stderr(File::create(&error).unwrap())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            child.0.kill().unwrap();
            child.0.wait().unwrap();
            panic!(
                "{label}: owned CLI exceeded 30s; evidence {}",
                f.root.display()
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(
        sha(&fs::read(executable).unwrap()),
        before,
        "CLI producer changed"
    );
    fs::write(f.root.join(format!("{label}.process.json")),serde_json::to_vec_pretty(&serde_json::json!({"pid":child.0.id(),"exit_code":status.code(),"cli_sha256":before,"host_options":host_options,"input":input,"output":output,"stderr":error})).unwrap()).unwrap();
    let result: Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    assert_eq!(result["ok"], ok, "{label}: {result}");
    assert_eq!(status.success(), ok, "{label}: {result}");
    if let Some(code) = code {
        assert_eq!(result["error"]["code"], code, "{label}: {result}");
    }
    f.assert_originals();
    result
}
