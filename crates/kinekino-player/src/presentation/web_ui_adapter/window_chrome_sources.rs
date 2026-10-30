//! One closed source inventory shared by Viewing, Studio and JOB frame producers.
use meari::domain::build::BuildPath;
use std::collections::BTreeMap;

pub(crate) fn inventory() -> BTreeMap<BuildPath, String> {
    [
        (
            "components/window-chrome.tsx",
            include_str!("../web_ui/components/window-chrome.tsx"),
        ),
        (
            "components/window-chrome-frame.tsx",
            include_str!("../web_ui/components/window-chrome-frame.tsx"),
        ),
        (
            "components/disabled-window-caption-controls.tsx",
            include_str!("../web_ui/components/disabled-window-caption-controls.tsx"),
        ),
    ]
    .into_iter()
    .map(|(path, source)| {
        (
            BuildPath::try_from(path.to_owned()).expect("fixed product window chrome source path"),
            source.to_owned(),
        )
    })
    .collect()
}
