use anyhow::Result;
use std::path::Path;

pub fn benchmark(engine: &str, model: &Path) -> Result<(&'static str, Vec<String>)> {
    let (executable, mut args): (_, Vec<String>) = match engine {
        "llama" => (
            "llama-b11716-cpu/llama-bench.exe",
            [
                "-t",
                "4",
                "-ngl",
                "0",
                "-p",
                "128",
                "-n",
                "64",
                "-d",
                "0,128",
                "-b",
                "128",
                "-ub",
                "128",
                "-r",
                "3",
                "--prio",
                "-1",
                "--poll",
                "0",
                "--offline",
                "-o",
                "json",
            ]
            .map(str::to_owned)
            .to_vec(),
        ),
        "mistral" => (
            "mistralrs-v0.9.4-cpu/mistralrs.exe",
            [
                "bench",
                "--cpu",
                "--token-source",
                "none",
                "--max-model-len",
                "2048",
                "--paged-attn",
                "off",
                "--prompt-len",
                "128",
                "--depth",
                "128",
                "--gen-len",
                "64",
                "--iterations",
                "3",
                "--warmup",
                "1",
            ]
            .map(str::to_owned)
            .to_vec(),
        ),
        _ => anyhow::bail!("engine must be llama or mistral"),
    };
    args.extend([
        if engine == "llama" { "-m" } else { "-f" }.to_owned(),
        model.to_string_lossy().into_owned(),
    ]);
    Ok((executable, args))
}
