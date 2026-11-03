//! The headless engine double the player tests drive.
use kinekino_core::playback::{FakeSurface, RgbaFrame, VideoSurface};
use kinekino_core::port::error::PortResult;
use kinekino_player_ui::Engine;
use std::collections::{HashMap, VecDeque};

#[derive(Default)]
pub struct FakeEngine {
    pub surface: FakeSurface,
    pub loaded: Vec<String>,
    pub commands: Vec<String>,
    pub time: f64,
    pub subtitle: Option<String>,
    /// mpv `eof-reached` with keep-open: the file played to its end; a load clears it.
    pub eof: bool,
    /// Further numeric properties (e.g. `track-list/count`), read after the ones above.
    pub numbers: HashMap<String, f64>,
    /// Further text properties (e.g. `aid`, `track-list/0/type`), read after `sub-text`.
    pub strings: HashMap<String, String>,
    /// Text properties commands change, in order: each command takes the next entry
    /// (`(name, value)` pairs) and sets them in `strings`, like mpv answering `cycle aid`.
    pub after_command: VecDeque<Vec<(String, String)>>,
}
impl VideoSurface for FakeEngine {
    fn frame_ready(&mut self) -> PortResult<bool> {
        self.surface.frame_ready()
    }
    fn render(&mut self, frame: &mut RgbaFrame) -> PortResult<()> {
        self.surface.render(frame)
    }
}
impl Engine for FakeEngine {
    fn begin_load(&mut self, source: &str, _: f64) -> PortResult<()> {
        self.loaded.push(source.to_owned());
        self.eof = false;
        Ok(())
    }
    fn load_ready(&mut self) -> PortResult<bool> {
        Ok(true)
    }
    fn command(&mut self, args: &[&str]) -> PortResult<()> {
        self.commands.push(args.join(" "));
        self.strings
            .extend(self.after_command.pop_front().unwrap_or_default());
        Ok(())
    }
    fn property(&mut self, name: &str) -> Option<f64> {
        match name {
            "time-pos" => Some(self.time),
            "duration" => Some(1500.0),
            "eof-reached" => Some(if self.eof { 1.0 } else { 0.0 }),
            _ => self.numbers.get(name).copied(),
        }
    }
    fn text(&mut self, name: &str) -> Option<String> {
        if name == "sub-text" {
            return self.subtitle.clone();
        }
        self.strings.get(name).cloned()
    }
}

/// Applies `command` and waits for any plugin call it started.
#[allow(dead_code)]
pub fn go<E: kinekino_player_ui::Engine>(
    c: &mut kinekino_player_ui::Controller<E>,
    command: kinekino_player_ui::Command,
) {
    c.apply(command);
    c.finish();
}
