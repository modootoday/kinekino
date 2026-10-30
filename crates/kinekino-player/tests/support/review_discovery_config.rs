//! Optional explicit operator configuration for discovery; absent configuration retains the default case.
pub fn frame_options() -> Vec<String> {
    let fields = [
        ("DISCOVERY_FRAME_FFMPEG", "--frame-ffmpeg"),
        ("DISCOVERY_FRAME_FFMPEG_SHA256", "--frame-ffmpeg-sha256"),
        ("DISCOVERY_FRAME_FFPROBE", "--frame-ffprobe"),
        ("DISCOVERY_FRAME_FFPROBE_SHA256", "--frame-ffprobe-sha256"),
    ];
    let values = fields.map(|(variable, _)| std::env::var_os(variable));
    if values.iter().all(Option::is_none) {
        return Vec::new();
    }
    fields
        .into_iter()
        .zip(values)
        .flat_map(|((variable, flag), value)| {
            let value =
                value.unwrap_or_else(|| panic!("Incomplete discovery configuration: {variable}"));
            [
                flag.to_owned(),
                value
                    .into_string()
                    .expect("Discovery tool configuration must be Unicode"),
            ]
        })
        .collect()
}
