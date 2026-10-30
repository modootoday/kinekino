use serde::Serialize;

#[derive(Default, Serialize)]
pub struct Scenario {
    pub dpi: u64,
    pub window_samples: usize,
    pub normal_window: bool,
    pub fullscreen_bounds_match: bool,
    pub controls_hidden: bool,
    pub controls_visible: bool,
    pub panel_open: bool,
    pub panel_closed: bool,
    pub episode_count_max: u64,
    pub native_playing_samples: usize,
    pub native_paused_samples: usize,
    pub native_errors: usize,
    pub selected_episodes: Vec<u64>,
    pub volumes: Vec<f64>,
}

impl Scenario {
    pub fn geometry_matches(&self, actual_dpi: u64, scale: f64, primary: bool) -> bool {
        actual_dpi == self.dpi && (scale - self.dpi as f64 / 96.).abs() < 0.001 && !primary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requires_actual_app_dpi_scale_and_nonprimary_monitor() {
        let scenario = Scenario {
            dpi: 144,
            ..Default::default()
        };
        assert!(scenario.geometry_matches(144, 1.5, false));
        assert!(!scenario.geometry_matches(96, 1.5, false));
        assert!(!scenario.geometry_matches(144, 1., false));
        assert!(!scenario.geometry_matches(144, 1.5, true));
    }
}
