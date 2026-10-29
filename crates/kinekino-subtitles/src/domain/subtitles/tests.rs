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

#[test]
fn ass_output_escapes_and_times() {
    let doc = super::parser::parse(
        "1\n00:00:01,250 --> 01:02:03,999\nhi {x}\nnext\n",
        SubtitleFormat::Srt,
        SpeakerPolicy::Omit,
    )
    .unwrap();
    let out = super::render::render(&doc, SubtitleFormat::Ass);
    assert!(out.starts_with("[Script Info]\n"));
    assert!(out.contains("Dialogue: 0,0:00:01.25,1:02:03.99,Default,,0,0,0,,hi (x)\\Nnext\n"));
    assert!(matches!(
        super::parser::parse("", SubtitleFormat::Ass, SpeakerPolicy::Omit),
        Err(SubtitleError::Invalid(_))
    ));
}

const ASS_IN: &str = "\u{feff}[Script Info]\r\nScriptType: v4.00+\r\n\r\n[V4+ Styles]\r\nFormat: Name, Fontname\r\nStyle: Default,Noto Sans\r\n\r\n[Events]\r\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\r\nDialogue: 0,0:00:05.00,0:00:06.50,Default,,0,0,0,,{\\an8}later, with comma\r\nComment: 0,0:00:00.00,0:00:01.00,Default,,0,0,0,,note\r\nDialogue: 0,0:00:01.25,1:02:03.99,Default,Ann,0,0,0,,hi\\Nnext\r\nDialogue: 0,0:00:07.00,0:00:08.00,Default,,0,0,0,,{\\p1}\r\n";

#[test]
fn ass_input_parses_dialogue_drops_markup_and_reorders() {
    let doc = super::parser::parse(ASS_IN, SubtitleFormat::Ass, SpeakerPolicy::Prefix).unwrap();
    let cues: Vec<_> = doc
        .cues
        .iter()
        .map(|c| (c.in_us.0, c.out_us.0, c.text.as_str()))
        .collect();
    assert_eq!(
        cues,
        vec![
            (1_250_000, 3_723_990_000, "[Ann] hi\nnext"),
            (5_000_000, 6_500_000, "later, with comma")
        ]
    );
    let codes: Vec<_> = doc.losses.iter().map(|l| l.code.as_str()).collect();
    for code in [
        "position_style_removed",
        "markup_removed",
        "metadata_removed",
        "speaker_structure_removed",
        "cues_reordered",
        "overlap_present",
    ] {
        assert!(codes.contains(&code), "{code} missing from {codes:?}");
    }
    let omit = super::parser::parse(ASS_IN, SubtitleFormat::Ass, SpeakerPolicy::Omit).unwrap();
    assert_eq!(omit.cues[0].text, "hi\nnext");
    let srt = super::render::render(&omit, SubtitleFormat::Srt);
    assert!(srt.contains("00:00:01,250 --> 01:02:03,990"), "{srt}");
}

#[test]
fn ass_input_rejects_malformed() {
    let ok =
        "[Script Info]\n[Events]\nFormat: Start, End, Text\nDialogue: 0:00:01.00,0:00:02.00,x\n";
    assert!(super::parser::parse(ok, SubtitleFormat::Ass, SpeakerPolicy::Omit).is_ok());
    for bad in [
        ok.replace("[Script Info]\n", ""),
        ok.replace("Format: Start, End, Text\n", ""),
        ok.replace("Start, End, Text", "Text, Start, End"),
        ok.replace("0:00:01.00", "0:00:01.0"),
        ok.replace("0:00:01.00", "0:61:01.00"),
        ok.replace("0:00:02.00", "0:00:00.50"),
        ok.replace(",x", ","),
    ] {
        assert!(
            matches!(
                super::parser::parse(&bad, SubtitleFormat::Ass, SpeakerPolicy::Omit),
                Err(SubtitleError::Invalid(_))
            ),
            "{bad}"
        );
    }
}
