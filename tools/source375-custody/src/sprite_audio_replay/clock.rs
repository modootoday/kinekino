#[derive(Debug, PartialEq, Eq)]
pub(super) struct Clock {
    pub frames: u64,
    pub first_pts: i64,
    pub end_pts: i128,
    pub samples: u64,
}
fn field<'a>(line: &'a str, name: &str) -> Result<&'a str, String> {
    line.split_whitespace()
        .find_map(|token| token.strip_prefix(name))
        .ok_or_else(|| format!("missing ashowinfo {name}"))
}
pub(super) fn parse(log: &str) -> Result<Clock, String> {
    let mut result = Clock {
        frames: 0,
        first_pts: 0,
        end_pts: 0,
        samples: 0,
    };
    for line in log.lines().filter(|line| {
        line.contains("ashowinfo") && line.split_whitespace().any(|token| token.starts_with("n:"))
    }) {
        let number: u64 = field(line, "n:")?.parse().map_err(|_| "bad frame number")?;
        let pts: i64 = field(line, "pts:")?.parse().map_err(|_| "bad signed PTS")?;
        let samples: u64 = field(line, "nb_samples:")?
            .parse()
            .map_err(|_| "bad sample count")?;
        if result.frames >= 50000
            || number != result.frames
            || samples == 0
            || field(line, "fmt:")? != "s16"
            || field(line, "channels:")? != "1"
            || field(line, "rate:")? != "16000"
        {
            return Err("ashowinfo format/count differs from fixed filter contract".into());
        }
        if result.frames == 0 {
            result.first_pts = pts;
        } else if i128::from(pts) != result.end_pts {
            return Err(format!(
                "audio PTS discontinuity frame={number} previous_end={} next_pts={pts}",
                result.end_pts
            ));
        }
        result.end_pts = i128::from(pts) + i128::from(samples);
        result.samples = result
            .samples
            .checked_add(samples)
            .ok_or("sample sum overflow")?;
        result.frames += 1;
    }
    if result.frames == 0 {
        return Err("no audio frame timestamps observed".into());
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn continuity_uses_integer_pts_and_requires_every_frame() {
        let a = "[Parsed_ashowinfo_2] n:0 pts:-2 pts_time:-0.000125 fmt:s16 channels:1 rate:16000 nb_samples:2";
        let b =
            "[Parsed_ashowinfo_2] n:1 pts:0 pts_time:0 fmt:s16 channels:1 rate:16000 nb_samples:3";
        assert_eq!(
            parse(&format!("{a}\n{b}")).unwrap(),
            Clock {
                frames: 2,
                first_pts: -2,
                end_pts: 3,
                samples: 5
            }
        );
        for bad in [
            b.replace("pts:0 ", "pts:1 "),
            b.replace("n:1 ", "n:2 "),
            b.replace("rate:16000", "rate:44100"),
            b.replace("nb_samples:3", "nb_samples:0"),
        ] {
            assert!(parse(&format!("{a}\n{bad}")).is_err());
        }
        assert!(parse("no timestamps").is_err());
    }
}
