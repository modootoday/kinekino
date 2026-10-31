//! Shared product tokens/components and viewing layout plus the derived player layout,
//! delivered through the settings window's asset path (pinned Noto, declared icons).
use kinekino_settings_ui::{Styles, prepare_with};

const STYLES: [(&str, &str); 4] = [
    (
        "crates/kinekino-player/src/presentation/web_ui/styles/tokens.css",
        include_str!("../../kinekino-player/src/presentation/web_ui/styles/tokens.css"),
    ),
    (
        "crates/kinekino-player/src/presentation/web_ui/styles/components.css",
        include_str!("../../kinekino-player/src/presentation/web_ui/styles/components.css"),
    ),
    (
        "crates/kinekino-player/src/presentation/web_ui/styles/viewing.css",
        include_str!("../../kinekino-player/src/presentation/web_ui/styles/viewing.css"),
    ),
    (
        "crates/kinekino-player-ui/src/player.css",
        include_str!("player.css"),
    ),
];

macro_rules! asset {
    ($path:literal) => {
        (
            $path,
            include_bytes!(concat!("../../../", $path)).as_slice(),
        )
    };
}
/// Every icon the stylesheets name must be declared, used here or not.
const ICONS: [(&str, &[u8]); 21] = [
    asset!("assets/design/icons/brand.svg"),
    asset!("assets/design/window/minimize.svg"),
    asset!("assets/design/window/maximize.svg"),
    asset!("assets/design/window/restore.svg"),
    asset!("assets/design/window/close.svg"),
    asset!("assets/design/icons/folder.svg"),
    asset!("assets/design/icons/search.svg"),
    asset!("assets/design/icons/previous.svg"),
    asset!("assets/design/icons/play.svg"),
    asset!("assets/design/icons/pause.svg"),
    asset!("assets/design/icons/next.svg"),
    asset!("assets/design/icons/volume.svg"),
    asset!("assets/design/icons/volume-zero.svg"),
    asset!("assets/design/icons/list.svg"),
    asset!("assets/design/icons/cards.svg"),
    asset!("assets/design/icons/fullscreen.svg"),
    asset!("assets/design/icons/more-vertical.svg"),
    asset!("assets/design/icons/start-mark.svg"),
    asset!("assets/design/icons/mascot-mark.svg"),
    asset!("assets/design/icons/mascot-color.svg"),
    asset!("assets/design/icons/mascot-small.svg"),
];

pub fn prepare() -> Result<Styles, String> {
    prepare_with(&STYLES, &ICONS)
}
