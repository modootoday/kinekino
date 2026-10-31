//! The headless engine double the player tests drive.
use kinekino_core::playback::{FakeSurface, RgbaFrame, VideoSurface};
use kinekino_core::port::error::PortResult;
use kinekino_player_ui::Engine;

#[derive(Default)]
pub struct FakeEngine {
    pub surface: FakeSurface,
    pub loaded: Vec<String>,
    pub commands: Vec<String>,
    pub time: f64,
    pub subtitle: Option<String>,
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
    fn load(&mut self, source: &str, _: f64) -> PortResult<()> {
        self.loaded.push(source.to_owned());
        Ok(())
    }
    fn command(&mut self, args: &[&str]) -> PortResult<()> {
        self.commands.push(args.join(" "));
        Ok(())
    }
    fn property(&mut self, name: &str) -> Option<f64> {
        match name {
            "time-pos" => Some(self.time),
            "duration" => Some(1500.0),
            _ => None,
        }
    }
    fn text(&mut self, name: &str) -> Option<String> {
        (name == "sub-text")
            .then(|| self.subtitle.clone())
            .flatten()
    }
}
