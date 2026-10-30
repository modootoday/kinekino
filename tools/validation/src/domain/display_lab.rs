// The reviewed driver's connector index must fit MaxMonitorsSupported (10).
pub const MONITOR_ID: u32 = 9;
pub const DRIVER_CERTIFICATE: &str = "5DFA7E68997ED52D4B10420407860D1C29B6F427";

#[derive(serde::Serialize)]
pub struct Mode {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
}

#[derive(serde::Serialize)]
pub struct Monitor {
    pub id: u32,
    pub modes: Vec<Mode>,
}

pub fn monitor() -> Monitor {
    Monitor {
        id: MONITOR_ID,
        modes: vec![
            Mode {
                width: 1920,
                height: 1080,
                refresh_rate: 60,
            },
            Mode {
                width: 3840,
                height: 2160,
                refresh_rate: 60,
            },
        ],
    }
}
