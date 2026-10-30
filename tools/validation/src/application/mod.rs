mod benchmark;
mod cleanup_vm;
mod display_lab;
mod dpi_report;
mod preflight;
mod resource_report;
mod soak;

pub fn run(args: Vec<String>) -> anyhow::Result<()> {
    let root = std::env::current_dir()?;
    let result = match args.first().map(String::as_str) {
        Some("preflight") => preflight::inspect(&root)?,
        Some("cleanup-vm") => {
            cleanup_vm::execute(&root, args.get(1).map(String::as_str).unwrap_or("plan"))?
        }
        Some("dpi-report") => dpi_report::summarize(std::path::Path::new(
            args.get(1)
                .ok_or_else(|| anyhow::anyhow!("missing DPI log"))?,
        ))?,
        Some("display-driver") => {
            display_lab::execute(&root, args.get(1).map(String::as_str).unwrap_or_default())?
        }
        Some("bench") => {
            benchmark::execute(&root, args.get(1).map(String::as_str).unwrap_or_default())?
        }
        Some("soak") => soak::summarize(&args)?,
        Some("resources") => resource_report::summarize(std::path::Path::new(
            args.get(1)
                .ok_or_else(|| anyhow::anyhow!("missing resource log"))?,
        ))?,
        _ => anyhow::bail!(
            "usage: preflight | bench llama|mistral | soak PATH web|native SECONDS [live]"
        ),
    };
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}
