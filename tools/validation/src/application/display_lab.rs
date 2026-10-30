use crate::{
    domain::display_lab,
    infrastructure::{display_driver, display_geometry},
};
use anyhow::Result;
use serde_json::{Value, json};
use std::path::Path;

pub fn execute(root: &Path, action: &str) -> Result<Value> {
    match action {
        "inventory" => display_geometry::observe(),
        "install" => display_driver::install(root),
        "add" => {
            display_driver::add(&display_lab::monitor())?;
            Ok(json!({"requested_add_id":display_lab::MONITOR_ID,"requires_os_observation":true}))
        }
        "remove" => {
            display_driver::remove(display_lab::MONITOR_ID)?;
            Ok(
                json!({"requested_remove_id":display_lab::MONITOR_ID,"requires_os_observation":true}),
            )
        }
        _ => anyhow::bail!("display-driver install|add|remove"),
    }
}
