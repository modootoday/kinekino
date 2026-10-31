//! Headless half of the player window: the Meari document, the user's actions read back
//! from source state as `Command`s, and view updates as props (no remount).
use crate::catalog::BROWSE_ROWS;
use crate::source::{self, BUTTONS, Prop, View};
use meari::application::source_modules::ModuleProgram;
use meari::domain::{Value as MeariValue, build::BuildPath, component_state::StateValue};
use meari_native::domain::{Viewport, widgets::Action};
use meari_native::infrastructure::document::InteractiveDocument;
use std::collections::BTreeMap;

pub const VIEWPORT: Viewport = Viewport {
    width: 1280,
    height: 720,
    dark: true,
};
const ENTRY: &str = "player/player-window.tsx";

/// What the user asked for, in click order per control.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Open,
    Browse,
    Previous,
    PlayPause,
    Next,
    Mute,
    ToggleEpisodes,
    ToggleFullscreen,
    /// Esc: leave fullscreen (nothing when windowed).
    ExitFullscreen,
    More,
    Minimize,
    Maximize,
    Close,
    Episode(usize),
    /// 작품 탐색: open the listed row.
    BrowseItem(usize),
    BrowseMore,
    BrowseBack,
    /// Search the open source for the typed text (검색 button or Enter in the field).
    BrowseSearch(String),
    /// Seek target as a 0–1 fraction of the duration.
    Seek(f64),
    /// Volume 0–100.
    Volume(f64),
}

pub struct Session {
    episodes: usize,
    counts: BTreeMap<String, i64>,
    seek_at: String,
    volume_at: String,
    query_at: String,
}

pub fn meari_props(view: &View) -> BTreeMap<String, MeariValue> {
    source::props(view)
        .into_iter()
        .map(|(k, v)| {
            let v = match v {
                Prop::Bool(b) => MeariValue::Bool(b),
                Prop::Text(s) => MeariValue::String(s),
            };
            (k, v)
        })
        .collect()
}

fn state(document: &InteractiveDocument) -> Result<BTreeMap<String, StateValue>, String> {
    let projection = document.module_projection()?;
    let mut values = BTreeMap::new();
    for commit in projection.instances.values() {
        values.extend(commit.values.clone());
    }
    Ok(values)
}

fn count(value: Option<&StateValue>) -> i64 {
    match value {
        Some(StateValue::Number(n)) => n.as_f64().map_or(0, |n| n as i64),
        _ => 0,
    }
}

fn text(value: Option<&StateValue>) -> String {
    match value {
        Some(StateValue::String(s)) => s.clone(),
        _ => String::new(),
    }
}

fn command(button: &str) -> Command {
    match button {
        "open" | "menuOpen" => Command::Open,
        "browse" => Command::Browse,
        "previous" => Command::Previous,
        "playPause" => Command::PlayPause,
        "next" => Command::Next,
        "mute" => Command::Mute,
        "episodes" | "panelClose" | "menuEpisodes" => Command::ToggleEpisodes,
        "fullscreen" | "menuFullscreen" => Command::ToggleFullscreen,
        "more" => Command::More,
        "minimize" => Command::Minimize,
        "maximize" => Command::Maximize,
        "browseMore" => Command::BrowseMore,
        "browseBack" => Command::BrowseBack,
        _ => Command::Close,
    }
}

impl Session {
    /// Mounts the window for `view` (episode rows fixed until the next mount).
    pub fn mount(view: &View) -> Result<(Self, InteractiveDocument), String> {
        let tsx = source::build(&view.episodes);
        let entry = BuildPath::try_from(ENTRY.to_owned())?;
        let graph = meari::parse_module_graph(&entry, &BTreeMap::from([(entry.clone(), tsx)]))
            .map_err(|e| format!("{e:?}"))?;
        let program = ModuleProgram::mount(graph, meari_props(view), "kinekino-player".into(), 1)
            .map_err(|e| format!("{e:?}"))?;
        let styles = crate::styles::prepare()?;
        let document = InteractiveDocument::from_module_program_with_assets(
            program,
            &styles.css,
            VIEWPORT,
            styles.assets,
        )?;
        let session = Self {
            episodes: view.episodes.len(),
            counts: BTreeMap::new(),
            seek_at: String::new(),
            volume_at: String::new(),
            query_at: String::new(),
        };
        Ok((session, document))
    }

    /// Whether `view` still fits this mount (same episode rows); otherwise remount.
    pub fn fits(&self, view: &View) -> bool {
        view.episodes.len() == self.episodes
    }

    /// Controls used since the last call.
    pub fn commands(&mut self, document: &InteractiveDocument) -> Result<Vec<Command>, String> {
        let state = state(document)?;
        let mut out = Vec::new();
        let ids = BUTTONS
            .iter()
            .map(|b| (*b).to_owned())
            .chain((0..self.episodes).map(|i| format!("ep{i}")))
            .chain((0..BROWSE_ROWS).map(|i| format!("br{i}")));
        for id in ids {
            let now = count(state.get(&format!("{id}Count")));
            let seen = self.counts.insert(id.clone(), now).unwrap_or_default();
            for _ in seen..now {
                let index = |p: &str| id.strip_prefix(p).and_then(|i| i.parse().ok());
                out.push(match (index("ep"), index("br")) {
                    (Some(i), _) => Command::Episode(i),
                    (_, Some(i)) => Command::BrowseItem(i),
                    _ if id == "browseSearch" => Command::BrowseSearch(text(state.get("queryAt"))),
                    _ => command(&id),
                });
            }
        }
        let seek = text(state.get("seekAt"));
        if seek != self.seek_at {
            if let Ok(v) = seek.parse::<f64>() {
                out.push(Command::Seek((v / 1000.0).clamp(0.0, 1.0)));
            }
            self.seek_at = seek;
        }
        let volume = text(state.get("volumeAt"));
        if volume != self.volume_at {
            if let Ok(v) = volume.parse::<f64>() {
                out.push(Command::Volume(v.clamp(0.0, 100.0)));
            }
            self.volume_at = volume;
        }
        self.query_at = text(state.get("queryAt"));
        Ok(out)
    }

    /// The text typed in the search field (Enter searches it).
    pub fn query(&self) -> &str {
        &self.query_at
    }
}

pub fn path_of(document: &InteractiveDocument, id: &str) -> Result<String, String> {
    document
        .elements()
        .iter()
        .find(|e| e.id == id)
        .map(|e| e.path.clone())
        .ok_or_else(|| format!("player element {id} missing"))
}

/// Activates a control the way a click does (tests, smoke runs, keyboard shortcuts).
pub fn activate(document: &mut InteractiveDocument, id: &str) -> Result<(), String> {
    let path = path_of(document, id)?;
    document.dispatch(Action::Activate { path }).map(|_| ())
}
