use super::files;
use crate::domain::display_lab;
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::{fs::OpenOptions, io::Write, path::Path, process::Command};

pub fn install(root: &Path) -> Result<Value> {
    let folder = root.join("runtime/display-driver-review/v0.3.1");
    let authorization =
        files::json(&root.join("runtime/display-driver-review/certificate-registration.json"))?;
    ensure!(
        authorization["thumbprint"] == display_lab::DRIVER_CERTIFICATE
            && authorization["rootPresent"] == 1
            && authorization["publisherPresent"] == 1,
        "exact user-approved certificate registration missing"
    );
    for (name, expected) in [
        (
            "VirtualDisplayDriver.dll",
            "c244f897784a7745e0eb251bc617b3e02d884e8452e4eaaa97b780829c10f808",
        ),
        (
            "virtualdisplaydriver.cat",
            "352eda1e7bcf5f2389ff152a2215b058f5c2fec9a777719dfff54f1717639d21",
        ),
    ] {
        ensure!(
            files::hash(&folder.join(name))? == expected,
            "reviewed driver data changed"
        );
    }
    let device_tool = root.join("runtime/display-driver-review/devcon.exe");
    ensure!(
        files::hash(&device_tool)?
            == "77e87e7c4e23d6b9a4f5d86d15ed9ec61a16a242fb299c390af6f179225bfa05",
        "reviewed Microsoft device tool changed"
    );
    let before = Command::new(&device_tool)
        .args(["findall", r"Root\VirtualDisplayDriver"])
        .output()?;
    ensure!(
        before.status.success()
            && String::from_utf8_lossy(&before.stdout).contains("No matching devices found."),
        "a matching driver already exists or inventory failed; do not duplicate"
    );
    let output = Command::new(device_tool)
        .arg("install")
        .arg(folder.join("VirtualDisplayDriver.inf"))
        .arg(r"Root\VirtualDisplayDriver")
        .output()?;
    let result = json!({"exit_code":output.status.code(),"stdout":String::from_utf8_lossy(&output.stdout),
        "stderr":String::from_utf8_lossy(&output.stderr),"reboot_performed":false,
        "scope":"reviewed Rust driver only; no display topology, scale or security mode modification"});
    files::write(
        &root.join("runtime/display-driver-review/installation.json"),
        &result,
    )?;
    Ok(result)
}

fn send(request: &Value) -> Result<()> {
    let data = serde_json::to_vec(request)?;
    ensure!(data.len() < 4096, "driver IPC message too large");
    let mut pipe = OpenOptions::new()
        .write(true)
        .open(r"\\.\pipe\virtualdisplaydriver")?;
    pipe.write_all(&data)?;
    Ok(())
}

pub fn add(monitor: &display_lab::Monitor) -> Result<()> {
    send(&json!({"Add":[monitor]}))
}
pub fn remove(id: u32) -> Result<()> {
    send(&json!({"Remove":[id]}))
}
