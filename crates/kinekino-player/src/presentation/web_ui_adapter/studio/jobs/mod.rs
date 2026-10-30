//! JOB route presentation only; SDK owns pagination, job state and record validation.
pub mod labels;
pub mod list;
pub mod selection;

/// Closed UI declarations; selecting a module never configures an SDK backend.
pub fn prepare() -> Result<
    (
        meari::domain::source_modules::SourceModuleGraph,
        std::collections::BTreeMap<meari::domain::build::BuildPath, String>,
    ),
    String,
> {
    use meari::domain::build::BuildPath;
    let path = |value: &str| BuildPath::try_from(value.to_owned()).expect("fixed JOB source path");
    let mut inventory = crate::presentation::web_ui_adapter::window_chrome_sources::inventory();
    inventory.extend([
        (
            path("components/window-caption-controls.tsx"),
            include_str!("../../../web_ui/components/window-caption-controls.tsx").into(),
        ),
        (
            path("studio/jobs/application.tsx"),
            include_str!("../../../web_ui/studio/jobs/application.tsx").into(),
        ),
        (
            path("studio/jobs/read-workspace.tsx"),
            include_str!("../../../web_ui/studio/jobs/read-workspace.tsx").into(),
        ),
        (
            path("studio/jobs/read-row.tsx"),
            include_str!("../../../web_ui/studio/jobs/read-row.tsx").into(),
        ),
        (
            path("studio/jobs/detail-panel.tsx"),
            include_str!("../../../web_ui/studio/jobs/detail-panel.tsx").into(),
        ),
        (
            path("studio/application.tsx"),
            include_str!("../../../web_ui/studio/application.tsx").into(),
        ),
        (
            path("studio/frame.tsx"),
            include_str!("../../../web_ui/studio/frame.tsx").into(),
        ),
        (
            path("studio/navigation.tsx"),
            include_str!("../../../web_ui/studio/navigation.tsx").into(),
        ),
        (
            path("studio/navigation-item.tsx"),
            include_str!("../../../web_ui/studio/navigation-item.tsx").into(),
        ),
        (
            path("studio/inspector-dock.tsx"),
            include_str!("../../../web_ui/studio/inspector-dock.tsx").into(),
        ),
        (
            path("studio/queue-footer.tsx"),
            include_str!("../../../web_ui/studio/queue-footer.tsx").into(),
        ),
        (
            path("components/notice.tsx"),
            include_str!("../../../web_ui/components/notice.tsx").into(),
        ),
    ]);
    let mut graph = meari::parse_module_graph(&path("studio/jobs/application.tsx"), &inventory)
        .map_err(|error| format!("{error:?}"))?;
    for entry in [
        "studio/jobs/read-workspace.tsx",
        "studio/jobs/detail-panel.tsx",
        "components/notice.tsx",
    ] {
        let child = meari::parse_module_graph(&path(entry), &inventory)
            .map_err(|error| format!("{error:?}"))?;
        for (key, module) in child.modules {
            if graph
                .modules
                .get(&key)
                .is_some_and(|previous| previous != &module)
            {
                return Err("JOB closed module inventory collision".into());
            }
            graph.modules.insert(key, module);
        }
    }
    graph.validate()?;
    Ok((graph, inventory))
}

#[cfg(test)]
mod tests;
