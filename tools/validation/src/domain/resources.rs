pub fn may_benchmark(free_ram: u64, cpu_percent: f64, validation_running: bool) -> bool {
    !validation_running
        && free_ram >= 6 * 1024 * 1024 * 1024
        && cpu_percent.is_finite()
        && (0.0..80.0).contains(&cpu_percent)
}

pub fn is_player_process(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    name == "player.exe" || (name.starts_with("kinekino-player") && name.ends_with(".exe"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn waits_for_playback_or_resource_pressure_without_closing_applications() {
        assert!(is_player_process("player.exe"));
        assert!(is_player_process("Player.EXE"));
        assert!(is_player_process("kinekino-player-soak.exe"));
        assert!(!is_player_process("player-cli.exe"));
        assert!(may_benchmark(8 << 30, 10.0, false));
        assert!(!may_benchmark(8 << 30, 10.0, true));
        assert!(!may_benchmark(4 << 30, 10.0, false));
        assert!(!may_benchmark(8 << 30, 90.0, false));
        assert!(!may_benchmark(8 << 30, f64::NAN, false));
    }
}
