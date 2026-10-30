use super::{parser::parse, render::render, *};

#[test]
fn surrounding_and_multiple_separator_blank_lines_preserve_text_indentation() {
    let input = "\n\n1\n00:00:01,000 --> 00:00:02,000\n  첫 줄  \n\t둘째 줄\n\n\n \t\n2\n00:00:03,000 --> 00:00:04,000\n  끝  \n\n\n";
    let parsed = parse(input, SubtitleFormat::Srt, SpeakerPolicy::Omit).unwrap();
    assert_eq!(parsed.cues.len(), 2);
    assert_eq!(parsed.cues[0].text, "  첫 줄  \n\t둘째 줄");
    assert_eq!(parsed.cues[1].text, "  끝  ");
    assert!(parsed.losses.is_empty());
    assert_eq!(
        parse(
            &render(&parsed, SubtitleFormat::Vtt),
            SubtitleFormat::Vtt,
            SpeakerPolicy::Omit
        )
        .unwrap()
        .cues,
        parsed.cues
    );
}

#[test]
fn unicode_multiline_crlf_bom_and_large_hours_roundtrip() {
    let source = "\u{feff}1\r\n100:00:00,001 --> 100:00:01,999\r\n한글 日本語 &amp; &lt;표시&gt;\r\n둘째 줄\r\n\r\n";
    let original = parse(source, SubtitleFormat::Srt, SpeakerPolicy::Omit).unwrap();
    assert!(original.losses.is_empty());
    let vtt = render(&original, SubtitleFormat::Vtt);
    let reparsed = parse(&vtt, SubtitleFormat::Vtt, SpeakerPolicy::Omit).unwrap();
    assert_eq!(original.cues, reparsed.cues);
    let srt = render(&reparsed, SubtitleFormat::Srt);
    assert_eq!(
        parse(&srt, SubtitleFormat::Srt, SpeakerPolicy::Omit)
            .unwrap()
            .cues,
        original.cues
    );
}

#[test]
fn rejects_negative_reversed_unsorted_empty_and_overflow_times() {
    for source in [
        "1\n-00:00:01,000 --> 00:00:02,000\na",
        "1\n00:00:02,000 --> 00:00:01,000\na",
        "1\n00:60:00,000 --> 01:01:00,000\na",
        "1\n00:00:00,000 --> 00:00:00,000\na",
        "1\n00:00:00.000 --> 00:00:01.000\na",
        "1\n00:00:00,000 --> 00:00:01,000",
        "1\n18446744073709551615:00:00,000 --> 18446744073709551615:00:01,000\na",
        "1\n00:00:02,000 --> 00:00:03,000\na\n\n2\n00:00:01,000 --> 00:00:02,000\nb",
    ] {
        assert!(
            parse(source, SubtitleFormat::Srt, SpeakerPolicy::Omit).is_err(),
            "{source}"
        );
    }
}

#[test]
fn reports_voice_style_settings_identifiers_notes_and_overlap() {
    let source = "WEBVTT\n\nSTYLE\n::cue { color: red; }\n\nNOTE comment\nnot a cue\n\nfirst\n00:01.000 --> 00:03.000 align:start\n<v 화자><b>안녕</b></v>\n\n00:02.000 --> 00:04.000\n둘";
    let parsed = parse(source, SubtitleFormat::Vtt, SpeakerPolicy::Prefix).unwrap();
    assert_eq!(parsed.cues[0].text, "[화자] 안녕");
    for code in [
        "metadata_removed",
        "cue_identifier_removed",
        "position_style_removed",
        "speaker_structure_removed",
        "markup_removed",
        "overlap_present",
    ] {
        assert!(parsed.losses.iter().any(|loss| loss.code == code));
    }
    assert_eq!(parsed.cues[1].in_us.0, 2_000_000);
}

#[test]
fn rejects_missing_header_unclosed_markup_and_non_numeric_srt_index() {
    assert!(
        parse(
            "00:01.000 --> 00:02.000\na",
            SubtitleFormat::Vtt,
            SpeakerPolicy::Omit
        )
        .is_err()
    );
    assert!(
        parse(
            "WEBVTT\n\n00:01.000 --> 00:02.000\n<v a",
            SubtitleFormat::Vtt,
            SpeakerPolicy::Omit
        )
        .is_err()
    );
    assert!(
        parse(
            "cue\n00:00:01,000 --> 00:00:02,000\na",
            SubtitleFormat::Srt,
            SpeakerPolicy::Omit
        )
        .is_err()
    );
}
