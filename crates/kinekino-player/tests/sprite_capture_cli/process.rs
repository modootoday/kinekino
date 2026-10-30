use super::custody::Guard;
use serde_json::Value;
use std::{
    fs::{self, File},
    path::Path,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
struct Owned(Child);
impl Drop for Owned {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
pub fn call(
    root: &Path,
    label: &str,
    request: &Value,
    cli: &Guard,
    success: bool,
    code: Option<&str>,
) -> Value {
    let input = root.join(format!("{label}.request.json"));
    let stdout = root.join(format!("{label}.stdout.json"));
    let stderr = root.join(format!("{label}.stderr"));
    fs::write(&input, serde_json::to_vec_pretty(request).unwrap()).unwrap();
    let started = Instant::now();
    let mut child = Owned(
        Command::new(crate::cli_exe::kinekino_cli_exe())
            .args(["--input", input.to_str().unwrap(), "--json"])
            .stdin(Stdio::null())
            .stdout(File::create(&stdout).unwrap())
            .stderr(File::create(&stderr).unwrap())
            .spawn()
            .unwrap(),
    );
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(
            started.elapsed() < Duration::from_secs(40),
            "owned CLI timeout; {}",
            root.display()
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    cli.assert_unchanged();
    fs::write(root.join(format!("{label}.process.json")),serde_json::to_vec_pretty(&serde_json::json!({"pid":child.0.id(),"exit_code":status.code(),"elapsed_ms":started.elapsed().as_millis(),"cli":cli.evidence()})).unwrap()).unwrap();
    let result: Value = serde_json::from_slice(&fs::read(stdout).unwrap()).unwrap();
    assert_eq!(result["ok"], success, "{label}: {result}");
    assert_eq!(status.success(), success, "{label}: {result}");
    if let Some(code) = code {
        assert_eq!(result["error"]["code"], code, "{label}: {result}");
    }
    result
}
