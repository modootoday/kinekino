// Screen bodies for the studio frame (included by studio.rs).

fn screen_tsx(screen: &str) -> Option<String> {
    Some(match screen {
        "studio-overview" => overview(),
        "studio-skip" => skip(),
        "studio-subtitles" => subtitles(),
        "studio-ocr" => ocr(),
        "studio-correction" => correction(),
        "studio-convert" => convert(),
        "studio-subject" => subject(),
        "studio-short" => short(),
        "studio-short-recommend" => short_recommend(),
        "studio-short-batch" => short_batch(),
        "studio-raw-frame" => raw_frame(),
        "studio-snapshot" => snapshot(),
        "studio-cards" => cards(),
        "studio-asset2d" => asset2d(),
        "studio-projection" => projection(),
        "studio-projection-package" => projection_package(),
        "studio-download" => download(),
        "studio-models" => models(),
        _ => return None,
    })
}

fn overview() -> String {
    let main = format!(
        "{}{}{}{}{}{}",
        head("AI 제작", ""),
        stage("flex: 1", ""),
        transport("00:00 / 23:42", 2, &["volume", "fullscreen"]),
        strip(10, "", &[(2, 6, "ks-range"), (86, 7, "ks-range")], None),
        ruler(&["00:00", "05:00", "10:00", "15:00", "20:00", "23:42"]),
        table(&[90, 120, 20, 120, 0, 90, 70], &["구간", "시작", "", "끝", "", "", ""], &[
            cells(&["오프닝", &input("00:32"), "–", &input("01:58"), "", &btn("", "play", "미리보기"), &btn("", "", "수정")]),
            cells(&["엔딩", &input("22:05"), "–", &input("23:35"), "", &btn("", "play", "미리보기"), &btn("", "", "수정")]),
        ], None),
    ) + &p("ks-small", "예시 구간 · 분석 전");
    let insp = c(&[
        &sec("", &row(&c(&[&i("segments"), "<h3 style=\"font-size: 16px\">구간 감지</h3>"]))),
        &sec("감지 방식", &sel("음악 + 화면 반복")),
        &sec("구간 설정", &c(&[
            &f("오프닝", &sel("수동")), &row(&c(&[&input("00:32"), "–", &input("01:58")])),
            &f("엔딩", &sel("수동")), &row(&c(&[&input("22:05"), "–", &input("23:35")])),
        ])),
        &sec("추가 옵션", &c(&[&f("회차별 확인", &c(&[GROW, &toggle(true)])), &f("재생 중 분석", &c(&[GROW, &toggle(false)]))])),
        &col("", &c(&[&btn("ks-primary ks-wide", "play", "구간 분석 시작"), &btn("ks-wide", "plus", "수동 구간 지정")])),
    ]);
    frame(0, &main, &insp, &queue(0, &[], false))
}

const MARKS: [&str; 6] = ["00:00", "05:00", "10:00", "15:00", "20:00", "23:40"];
const JOBS: [(&str, &str, &str); 2] = [("시청 기록 저장", "1화 · 23:40", "대기 중"), ("구간 분석", "1화 · 오프닝/엔딩", "대기 중")];

fn chip(class: &str, text: &str) -> String {
    format!(r#"<span className="ks-chip-tag {class}">{text}</span>"#)
}
fn h(title: &str, crumb: &str, about: &str) -> String {
    let t = if crumb.is_empty() { title.to_owned() } else { format!("{title} › {crumb}") };
    head_col(&t, about)
}
fn insp_title(icon: &str, title: &str) -> String {
    sec("", &row(&c(&[&i(icon), &format!("<h3 style=\"font-size: 15px\">{title}</h3>")])))
}

fn skip() -> String {
    let main = c(&[
        &h("구간 감지", "오프닝 엔딩 제어", "오프닝과 엔딩 구간을 감지하고 건너뛰기 설정을 관리합니다."),
        &stage("flex: 1", r#"<div className="ks-overlay" style="right: 12px; bottom: 12px"><span>3초 후 오프닝 건너뛰기</span><span className="ks-btn ks-outline">이번에는 보기</span></div>"#),
        &transport("00:28 / 23:40", 2, &["subtitles", "fullscreen"]),
        &ruler(&MARKS),
        &strip(12, "", &[(3, 7, "ks-range"), (86, 7, "ks-range-violet")], Some(2)),
        &row(&c(&[&chip("", "OP 00:32 - 01:58"), GROW, &chip("ks-chip-violet", "ED 22:05 - 23:35")])),
        &p("ks-muted", "감지된 구간 목록 (2개)"),
        &table(&[110, 100, 100, 90, 0, 120], &["유형", "시작 시간", "종료 시간", "길이", "상태", "작업"], &[
            cells(&["● 오프닝", "00:32", "01:58", "01:26", &chip("", "사용자 확인"), &c(&[&btn("", "", "구간 수정"), &i("delete")])]),
            cells(&["● 엔딩", "22:05", "23:35", "01:30", &chip("ks-chip-violet", "후보"), &c(&[&btn("", "", "구간 수정"), &i("delete")])]),
        ], None),
    ]);
    let insp = c(&[
        &sec("구간 건너뛰기 설정", &c(&[&p("ks-l", "오프닝 설정"), &sel_c("ks-amber", "오프닝 · 자동"), &p("ks-l", "엔딩 설정"), &sel_c("ks-violet", "엔딩 · 수동"), &p("ks-small", "자동은 확인된 구간만 적용")])),
        &sec("재생 옵션", &c(&[&f("다음화 자동 재생", &c(&[GROW, &toggle(false)])), &f("회차별 확인", &c(&[GROW, &toggle(true)]))])),
        &sec("추가 설정", &c(&[&p("ks-l", "건너뛰기 후 동작"), &sel("다음 구간부터 일반 재생"), &p("ks-l", "미리 알림 표시"), &sel("3초 전")])),
        &sec("최근 작업 기록", &note("", "info", "<span>건너뛰었습니다 · <b className=\"ks-hl\">되돌리기</b></span><span className=\"ks-small\">오프닝 00:32 - 01:58 (2분 전)</span>")),
        &row(&c(&[&btn("ks-outline ks-grow", "", "구간 수정"), &btn("ks-primary ks-grow", "", "현재 회차 저장")])),
    ]);
    frame(0, &main, &insp, &queue(2, &JOBS, true))
}

fn subtitles() -> String {
    let main = c(&[
        &h("자막", "음성 인식", "음성을 인식해 자막을 만들고 시간과 문장을 다듬습니다."),
        &tabs(&SUB_TABS, 0),
        &stage("flex: 1", r#"<div className="ks-caption"><span>오늘 밤에는 별이 정말 많네.</span></div>"#),
        &transport("03:12 / 23:40", 14, &["volume", "fullscreen"]),
        r#"<div className="ks-wave-wrap">"#, r#"<div className="ks-wave ks-wave-green"></div>"#, "</div>",
        &row(&c(&[&btn("", "clip", "분할"), &btn("", "merge", "병합"), &btn("", "plus", "추가"), &btn("", "delete", "삭제"), &btn("", "undo", "되돌리기"), &btn("", "redo", "다시 실행")])),
        &table(&[40, 90, 90, 0, 0], &["#", "시작", "끝", "인식 문장", "교정안"], &[
            cells(&["12", "03:10.2", "03:12.8", "오늘 밤에는 별이 정말 많내.", &input("오늘 밤에는 별이 정말 많네.")]),
            cells(&["13", "03:13.0", "03:15.4", "저기 호수 건너편까지 가 볼까?", &input("저기 호수 건너편까지 가 볼까?")]),
            cells(&["14", "03:15.9", "03:18.1", "응, 조금만 더 걷자.", &input("응, 조금만 더 걷자.")]),
        ], Some(0)),
    ]);
    let insp = c(&[
        &insp_title("microphone", "인식 설정"),
        &sec("모델", &c(&[&sel("음성 인식 · 기본"), &f("언어", &sel("한국어")), &f("화자 구분", &c(&[GROW, &toggle(true)]))])),
        &sec("문장 처리", &c(&[&f("최대 글자 수", &input("32")), &f("최소 길이(초)", &input("0.8")), &check(true, "문장 부호 보정"), &check(false, "효과음 표기")])),
        &sec("정확도", &c(&[&slider(70), &p("ks-small", "높을수록 느리지만 정확합니다")])),
        &btn("ks-primary ks-wide", "play", "자막 인식 시작"),
    ]);
    frame(1, &main, &insp, &queue(0, &[], false))
}

fn ocr() -> String {
    let main = c(&[
        &h("자막", "화면 OCR", "화면에 새겨진 글자를 찾아 자막으로 옮깁니다."),
        &tabs(&SUB_TABS, 1),
        &stage("flex: 1", r#"<div className="ks-caption"><span>호수 마을에 어서 오세요</span></div><div className="ks-box" style="left: 28%; right: 28%; bottom: 8px; height: 34px"></div>"#),
        &transport("05:41 / 23:40", 24, &["volume", "fullscreen"]),
        &p("ks-muted", "감지된 글자 (3개)"),
        &table(&[90, 90, 0, 70, 90], &["시작", "끝", "감지된 글자", "신뢰도", "상태"], &[
            cells(&["05:40.1", "05:43.0", "호수 마을에 어서 오세요", "96%", &chip("", "확인")]),
            cells(&["07:02.4", "07:04.2", "마을 회관", "88%", &chip("ks-chip-gray", "후보")]),
            cells(&["11:20.0", "11:22.5", "등대까지 2km", "71%", &chip("ks-chip-violet", "재확인")]),
        ], Some(0)),
    ]);
    let insp = c(&[
        &insp_title("scan-text", "OCR 설정"),
        &sec("감지 영역", &c(&[&sel("화면 하단 자막 영역"), &f("X", &input("28%")), &f("너비", &input("44%"))])),
        &sec("인식", &c(&[&f("언어", &sel("한국어 + 영어")), &f("간격(프레임)", &input("6")), &slider(55)])),
        &sec("글자 교정", &c(&[&p("ks-l", "선택한 글자"), &input("호수 마을에 어서 오세요"), &check(true, "맞춤법 후보 표시")])),
        &btn("ks-primary ks-wide", "scan-text", "OCR 시작"),
    ]);
    frame(1, &main, &insp, &queue(0, &[], false))
}

fn correction() -> String {
    let main = c(&[
        &h("자막", "교정", "인식된 자막의 오탈자와 띄어쓰기를 고칩니다."),
        &tabs(&SUB_TABS, 2),
        &stage("flex: 0 0 220px", r#"<div className="ks-caption"><span>오늘 밤에는 별이 정말 많네.</span></div>"#),
        &table(&[40, 0, 0, 160, 120], &["#", "원문", "교정안", "근거", "작업"], &[
            cells(&["12", "오늘 밤에는 별이 정말 <b className=\"ks-err\">많내</b>.", "오늘 밤에는 별이 정말 <b className=\"ks-hl\">많네</b>.", "맞춤법", &c(&[&btn("ks-primary", "", "수락"), &btn("", "", "거절")])]),
            cells(&["18", "<b className=\"ks-err\">어 떻게</b> 알았어?", "<b className=\"ks-hl\">어떻게</b> 알았어?", "띄어쓰기", &c(&[&btn("ks-primary", "", "수락"), &btn("", "", "거절")])]),
            cells(&["25", "등대 까지 걸어가자.", "등대<b className=\"ks-hl\">까지</b> 걸어가자.", "띄어쓰기", &c(&[&btn("ks-primary", "", "수락"), &btn("", "", "거절")])]),
        ], Some(0)),
        &row(&c(&[&p("ks-muted", "교정 후보 3개 · 수락 0"), GROW, &btn("", "", "모두 거절"), &btn("ks-primary", "", "모두 수락")])),
    ]);
    let insp = c(&[
        &insp_title("text-document", "교정 옵션"),
        &sec("검사 항목", &c(&[&check(true, "맞춤법"), &check(true, "띄어쓰기"), &check(false, "문체 통일"), &check(true, "고유명사 보호")])),
        &sec("고유명사", &c(&[&input("등대, 호수 마을"), &p("ks-small", "쉼표로 구분")])),
        &sec("적용 범위", &c(&[&radio(true, "현재 회차"), &radio(false, "모든 회차")])),
        &btn("ks-primary ks-wide", "refresh", "다시 검사"),
    ]);
    frame(1, &main, &insp, &queue(1, &[("자막 교정", "1화 · 교정 후보 3개", "검토 대기")], false))
}

fn cue(n: &str, t: &str, text: &str) -> String {
    format!(r#"<div className="ks-card ks-cue"><span className="ks-small">{n} · {t}</span><span>{text}</span></div>"#)
}

fn convert() -> String {
    let ass = c(&[&p("ks-muted", "ASS (원본)"), &cue("12", "0:03:10.20 → 0:03:12.80", "{\"{\\\\pos(640,620)}\"}오늘 밤에는 별이 정말 많네."), &cue("13", "0:03:13.00 → 0:03:15.40", "{\"{\\\\i1}저기 호수 건너편까지 가 볼까?{\\\\i0}\"}"), &cue("14", "0:03:15.90 → 0:03:18.10", "응, 조금만 더 걷자.")]);
    let srt = c(&[&p("ks-muted", "SRT (변환 결과)"), &cue("12", "00:03:10,200 → 00:03:12,800", "오늘 밤에는 별이 정말 많네."), &cue("13", "00:03:13,000 → 00:03:15,400", "{\"<i>저기 호수 건너편까지 가 볼까?</i>\"}"), &cue("14", "00:03:15,900 → 00:03:18,100", "응, 조금만 더 걷자.")]);
    let main = c(&[
        &h("자막", "변환", "자막 형식을 바꾸고 스타일 손실을 미리 확인합니다."),
        &tabs(&SUB_TABS, 3),
        &stage("flex: 0 0 200px", r#"<div className="ks-caption"><span>저기 호수 건너편까지 가 볼까?</span></div>"#),
        &format!(r#"<div className="ks-row ks-grow" style="align-items: flex-start">{}{}</div>"#, col("flex: 1", &ass), col("flex: 1", &srt)),
        &note("ks-warn", "warning", "<span>위치 지정(pos 태그) 2개와 글꼴 스타일 1개는 SRT에서 유지되지 않습니다.</span>"),
    ]);
    let insp = c(&[
        &insp_title("refresh", "변환 설정"),
        &sec("형식", &c(&[&f("원본", &sel("ASS")), &f("대상", &sel("SRT"))])),
        &sec("옵션", &c(&[&check(true, "기울임 태그 유지"), &check(false, "위치 정보 주석으로 남기기"), &f("인코딩", &sel("UTF-8"))])),
        &sec("시간 보정", &c(&[&f("오프셋(ms)", &input("0")), &f("프레임 속도", &sel("23.976"))])),
        &btn("ks-primary ks-wide", "export", "변환 후 내보내기"),
    ]);
    frame(1, &main, &insp, &queue(0, &[], false))
}

fn tools(list: &[&str]) -> String {
    let mut s = String::from(r#"<div className="ks-col ks-rail" style="flex: 0 0 34px; gap: 6px">"#);
    for (k, n) in list.iter().enumerate() {
        let on = if k == 0 { " ks-on" } else { "" };
        s.push_str(&format!(r#"<span className="ks-tool{on}">{}</span>"#, i(n)));
    }
    s + "</div>"
}
fn track(icon: &str, label: &str, body: &str) -> String {
    format!(r#"<div className="ks-row ks-track">{}<span className="ks-l" style="flex: 0 0 60px">{label}</span><div className="ks-grow ks-trackbody">{body}</div></div>"#, i(icon))
}

fn subject() -> String {
    let view = c(&[
        &tools(&["selection-pointer", "brush-add", "eraser", "mask-exclude", "hand", "zoom-in"]),
        &stage("flex: 1", r#"<div className="ks-box-cyan" style="left: 44%; top: 34%; width: 14%; height: 46%"></div><div className="ks-overlay" style="left: 44%; top: 26%"><span>피사체 1 · 94%</span></div>"#),
    ]);
    let main = c(&[
        &h("피사체", "인식과 분할", "인물과 사물을 찾아 마스크로 분리하고 장면을 따라 추적합니다."),
        &row(&c(&[&seg(&["원본", "마스크", "합성"], 0), GROW, &btn("", "eye", "외곽선"), &btn("", "refresh", "다시 계산")])),
        &format!(r#"<div className="ks-row ks-grow" style="align-items: stretch">{view}</div>"#),
        &transport("08:14 / 23:40", 35, &["volume", "fullscreen"]),
        &strip(12, "", &[(30, 22, "ks-range"), (62, 6, "ks-range-violet")], Some(35)),
        &row(&c(&[&chip("", "추적 08:02 - 13:10"), GROW, &chip("ks-chip-violet", "재확인 필요 14:40")])),
    ]);
    let insp = c(&[
        &insp_title("subject", "피사체 목록"),
        &table(&[24, 0, 60], &["", "이름", "신뢰도"], &[
            cells(&[&check(true, ""), "피사체 1 · 인물", "94%"]),
            cells(&[&check(false, ""), "피사체 2 · 인물", "81%"]),
            cells(&[&check(false, ""), "배경 · 건물", "67%"]),
        ], Some(0)),
        &sec("탐지", &c(&[&f("모델", &sel("피사체 분할 · 기본")), &f("최소 신뢰도", &slider(60))])),
        &sec("분할", &c(&[&f("가장자리 다듬기", &slider(40)), &check(true, "머리카락 보정")])),
        &sec("추적", &c(&[&f("범위", &sel("현재 장면")), &check(true, "가림 후 다시 찾기")])),
        &row(&c(&[&btn("ks-outline ks-grow", "", "마스크 저장"), &btn("ks-primary ks-grow", "play", "추적 시작")])),
    ]);
    frame(2, &main, &insp, &queue(0, &[], false))
}

fn short() -> String {
    let crop = r#"<div className="ks-crop"><div className="ks-grid"></div></div>"#;
    let views = format!(
        r#"<div className="ks-row ks-grow" style="align-items: stretch; gap: 12px">{}<div className="ks-col" style="flex: 0 0 210px">{}{}</div></div>"#,
        stage("flex: 1", crop),
        p("ks-muted", "9:16 결과"),
        pic("", "flex: 1", r#"<div className="ks-caption"><span>별이 정말 많네</span></div>"#)
    );
    let main = c(&[
        &h("숏폼", "세로 영상 만들기", "장면을 골라 세로 화면으로 다시 구성합니다."),
        &views,
        &transport("03:10 / 23:40", 13, &["volume"]),
        &ruler(&["03:00", "03:10", "03:20", "03:30", "03:40", "03:50"]),
        &track("track-video", "영상", &strip(8, "", &[(10, 50, "ks-range")], Some(14))),
        &track("track-subtitle", "자막", r#"<div className="ks-clips"><span style="left: 12%; width: 18%">오늘 밤에는…</span><span style="left: 34%; width: 22%">저기 호수…</span></div>"#),
        &track("track-audio", "오디오", r#"<div className="ks-wave" style="height: 26px"></div>"#),
    ]);
    let insp = c(&[
        &insp_title("clip", "숏폼 설정"),
        &sec("화면", &c(&[&f("비율", &sel("9:16")), &f("해상도", &sel("1080 × 1920")), &f("길이", &input("00:40"))])),
        &sec("구도", &c(&[&f("추적 대상", &sel("피사체 1")), &f("부드러움", &slider(60)), &check(true, "격자 표시")])),
        &sec("자막", &c(&[&f("위치", &sel("하단")), &f("크기", &slider(50)), &check(true, "강조 단어 색상")])),
        &btn("ks-primary ks-wide", "export", "숏폼 만들기"),
    ]);
    frame(3, &main, &insp, &queue(0, &[], false))
}

fn short_recommend() -> String {
    let cand = |n: &str, t: &str, score: &str, why: &str, kind: &str, on: bool| {
        let c2 = if on { " ks-on" } else { "" };
        format!(r#"<div className="ks-card ks-cand{c2}"><div className="ks-row">{}<div className="ks-col ks-grow" style="gap: 4px"><span><b>후보 {n}</b> · {t}</span><span className="ks-small">근거: {why}</span></div><span className="ks-big">{score}</span>{}</div></div>"#,
            pic(kind, "flex: 0 0 120px; height: 68px", ""), btn(if on { "ks-primary" } else { "" }, "", "선택"))
    };
    let main = c(&[
        &h("숏폼", "추천 장면", "대사, 음악, 화면 변화를 보고 숏폼으로 쓸 만한 장면을 추천합니다."),
        &stage("flex: 0 0 240px", ""),
        &strip(12, "", &[(12, 4, "ks-range"), (46, 5, "ks-range"), (78, 4, "ks-range")], None),
        &cand("1", "03:10 - 03:50", "92", "대사 밀도 높음 · 배경음 상승 · 밤하늘 전경", "", true),
        &cand("2", "10:42 - 11:20", "85", "인물 클로즈업 · 감정 대사", "sunset", false),
        &cand("3", "18:05 - 18:40", "78", "장면 전환 많음 · 웃음 반응", "day", false),
    ]);
    let insp = c(&[
        &insp_title("clip", "추천 조건"),
        &sec("길이", &c(&[&f("최소", &input("00:20")), &f("최대", &input("00:60"))])),
        &sec("가중치", &c(&[&f("대사", &slider(70)), &f("음악", &slider(50)), &f("화면 변화", &slider(40))])),
        &sec("제외", &c(&[&check(true, "오프닝/엔딩 제외"), &check(false, "자막 없는 장면 제외")])),
        &btn("ks-primary ks-wide", "refresh", "다시 추천"),
    ]);
    frame(3, &main, &insp, &queue(0, &[], false))
}

fn short_batch() -> String {
    let main = c(&[
        &h("숏폼", "일괄 만들기", "여러 회차에서 같은 규칙으로 숏폼을 만듭니다."),
        &table(&[24, 70, 0, 90, 90, 110], &["", "회차", "추천 장면", "길이", "후보", "상태"], &[
            cells(&[&check(true, ""), "1화", "03:10 - 03:50 · 밤하늘 전경", "00:40", "3개", &chip("", "준비됨")]),
            cells(&[&check(true, ""), "2화", "07:22 - 07:58 · 호숫가 대화", "00:36", "2개", &chip("", "준비됨")]),
            cells(&[&check(true, ""), "3화", "12:01 - 12:45 · 등대 장면", "00:44", "4개", &chip("ks-chip-gray", "분석 전")]),
            cells(&[&check(false, ""), "4화", "-", "-", "-", &chip("ks-chip-gray", "분석 전")]),
            cells(&[&check(true, ""), "5화", "15:30 - 16:02 · 마을 축제", "00:32", "2개", &chip("ks-chip-violet", "재확인")]),
        ], Some(0)),
        &format!(r#"<div className="ks-card" style="flex: 0 0 auto"><h3>계획 요약</h3>{}{}{}{}</div>"#,
            f("대상 회차", "4개"), f("만들 숏폼", "4개 · 9:16 · 1080 × 1920"), f("예상 시간", "약 12분"), f("저장 위치", "회차 폴더 / shorts")),
    ]);
    let insp = c(&[
        &insp_title("clip", "일괄 규칙"),
        &sec("선택 규칙", &c(&[&radio(true, "회차별 최고 점수 1개"), &radio(false, "점수 80 이상 모두")])),
        &sec("출력", &c(&[&f("비율", &sel("9:16")), &f("자막", &sel("포함")), &check(true, "회차 번호 표시")])),
        &btn("ks-primary ks-wide", "play", "4개 만들기 시작"),
    ]);
    frame(3, &main, &insp, &queue(0, &[], false))
}

fn raw_frame() -> String {
    let fr = |label: &str, kind: &str, on: bool| {
        let c2 = if on { " ks-on" } else { "" };
        format!(r#"<div className="ks-col ks-frame{c2}" style="flex: 1">{}<span className="ks-small">{label}</span></div>"#, pic(kind, "height: 110px", ""))
    };
    let main = c(&[
        &h("스냅샷", "원본 프레임", "자막과 효과 없이 원본 프레임을 그대로 저장합니다."),
        &stage("flex: 1", ""),
        &transport("12:04.18 / 23:40", 51, &["previous", "next"]),
        &row(&c(&[&fr("이전 · 12:04.14", "", false), &fr("현재 · 12:04.18", "", true), &fr("다음 · 12:04.22", "", false)])),
        &p("ks-muted", "저장 대기 (2)"),
        &table(&[110, 0, 110, 90], &["시간", "파일 이름", "크기", "상태"], &[
            cells(&["12:04.18", "ep01_120418.png", "1920 × 1080", &chip("", "대기")]),
            cells(&["03:10.20", "ep01_031020.png", "1920 × 1080", &chip("ks-chip-gray", "저장됨")]),
        ], None),
    ]);
    let insp = c(&[
        &insp_title("image", "프레임 정보"),
        &sec("파일", &c(&[&f("해상도", "1920 × 1080"), &f("프레임", "17,386"), &f("시간", "12:04.18"), &f("색 공간", "BT.709")])),
        &sec("저장", &c(&[&f("형식", &sel("PNG")), &check(false, "자막 포함"), &check(true, "회차 폴더에 저장")])),
        &btn("ks-primary ks-wide", "image", "현재 프레임 저장"),
    ]);
    frame(4, &main, &insp, &queue(0, &[], false))
}

fn snapshot() -> String {
    let main = c(&[
        &h("스냅샷", "꾸미기", "프레임에 제목과 글자를 얹어 이미지로 저장합니다."),
        &stage("flex: 1", r#"<div className="ks-box" style="left: 10%; top: 12%; width: 46%; height: 22%"></div><div className="ks-overlay ks-title-text" style="left: 11%; top: 15%"><span>별이 쏟아지는 밤</span></div>"#),
        &strip(10, "", &[(40, 10, "ks-range")], Some(45)),
    ]);
    let sw = |col: &str| format!(r#"<span className="ks-swatch" style="background: {col}"></span>"#);
    let insp = c(&[
        &insp_title("image", "글자 설정"),
        &sec("내용", &c(&[&input("별이 쏟아지는 밤"), &f("글꼴", &sel("Noto Sans KR Bold")), &f("크기", &slider(60))])),
        &sec("색상", &row(&c(&[&sw("#ffffff"), &sw("#f0b43c"), &sw("#4fd6e0"), &sw("#bca0eb"), &sw("#ff6b6b"), &sw("#1a1c20")]))),
        &sec("효과", &c(&[&check(true, "그림자"), &check(false, "외곽선"), &f("투명도", &slider(100))])),
        &sec("레이어", &c(&[&row(&c(&[&i("eye"), "제목 글자", GROW, &i("lock")])), &row(&c(&[&i("eye"), "원본 프레임", GROW, &i("lock")]))])),
        &btn("ks-primary ks-wide", "export", "이미지 저장"),
    ]);
    frame(4, &main, &insp, &queue(0, &[], false))
}

fn cards() -> String {
    let item = |n: &str, kind: &str, on: bool| {
        let c2 = if on { " ks-on" } else { "" };
        format!(r#"<div className="ks-row ks-frame{c2}"><span className="ks-small">{n}</span>{}</div>"#, pic(kind, "flex: 0 0 64px; height: 80px", ""))
    };
    let list = c(&[&item("1", "", false), &item("2", "", true), &item("3", "sunset", false), &item("4", "day", false), &btn("", "plus", "추가")]);
    let main = c(&[
        &h("카드뉴스", "만들기", "장면과 글을 묶어 카드뉴스로 만듭니다."),
        &row(&c(&[&btn("", "fullscreen", "전체 보기"), &btn("", "copy", "복제"), &btn("", "delete", "삭제"), GROW, &p("ks-muted", "2 / 4")])),
        &format!(r#"<div className="ks-row ks-grow" style="align-items: stretch; gap: 12px"><div className="ks-col" style="flex: 0 0 90px; gap: 8px">{list}</div>{}</div>"#,
            stage("flex: 1", r#"<div className="ks-card-text"><span className="ks-big">별빛 여행</span><span>한 장면에 담긴 이야기</span></div><div className="ks-overlay" style="right: 10px; bottom: 10px"><span>2 / 4</span></div>"#)),
    ]);
    let insp = c(&[
        &insp_title("cards", "카드 설정"),
        &sec("플랫폼", &row(&c(&[&chip("ks-on", "영상 공유"), &chip("", "사진 공유")]))),
        &sec("비율", &seg(&["1:1", "4:5", "9:16", "16:9"], 1)),
        &sec("템플릿", &c(&[&seg(&["제목형", "인용형", "목록형"], 0), &f("글꼴", &sel("Noto Sans KR")), &f("강조 색", &row(&c(&[&format!(r#"<span className="ks-swatch" style="background: #f0b43c"></span>"#), &format!(r#"<span className="ks-swatch" style="background: #ffffff"></span>"#)])))])),
        &sec("텍스트", &c(&[&input("별빛 여행"), &input("한 장면에 담긴 이야기")])),
        &note("ks-warn", "info", "<span>글자 수가 많으면 작은 화면에서 잘릴 수 있습니다.</span>"),
        &row(&c(&[&btn("ks-outline ks-grow", "", "미리보기"), &btn("ks-primary ks-grow", "export", "모두 저장")])),
    ]);
    frame(5, &main, &insp, &queue(0, &[], false))
}

fn asset2d() -> String {
    let view = c(&[
        &tools(&["selection-pointer", "brush-add", "eraser", "hand", "zoom-in"]),
        &pic("checker", "flex: 1", r#"<div className="ks-box-blue" style="left: 34%; top: 10%; width: 30%; height: 78%"></div><div className="ks-figure" style="left: 38%; top: 16%; width: 22%; height: 70%"></div><span className="ks-pivot" style="left: 48%; top: 52%"></span>"#),
    ]);
    let layer = |name: &str, on: bool| {
        let c2 = if on { " ks-on" } else { "" };
        format!(r#"<div className="ks-row ks-frame{c2}">{}{}<span>{name}</span>{GROW}{}</div>"#, i("eye"), pic("checker", "flex: 0 0 36px; height: 24px", ""), i("lock"))
    };
    let main = c(&[
        &h("2D 에셋", "알파 추출", "프레임에서 캐릭터를 떼어 투명 배경 이미지로 만듭니다."),
        &row(&c(&[&seg(&["원본", "알파", "합성"], 1), GROW, &p("ks-muted", "100%"), &p("ks-muted", "08:24.120")])),
        &format!(r#"<div className="ks-row ks-grow" style="align-items: stretch">{view}</div>"#),
        &row(&c(&[&col("flex: 1; gap: 4px", &c(&[&p("ks-muted", "레이어"), &layer("캐릭터", true), &layer("그림자", false), &layer("배경", false)])), &col("flex: 0 0 220px", &c(&[&p("ks-muted", "원본 프레임"), &pic("room", "height: 96px", "")]))])),
    ]);
    let insp = c(&[
        &insp_title("layers", "추출 설정"),
        &sec("알파 추출", &c(&[&f("임계값", &slider(55)), &f("가장자리", &slider(30)), &f("번짐 제거", &slider(20))])),
        &sec("기준점", &c(&[&row(&c(&[&radio(true, "중앙"), &radio(false, "발 아래"), &radio(false, "직접 지정")])), &row(&c(&[&input("X 512"), &input("Y 980")]))])),
        &sec("경계 상자", &row(&c(&[&input("120"), &input("40"), &input("880"), &input("1024")]))),
        &sec("내보내기", &c(&[&f("크기", &sel("1,024 px")), &check(true, "PNG (알파 포함)"), &check(true, "기준점 JSON"), &check(false, "마스크 따로 저장")])),
        &row(&c(&[&btn("ks-outline ks-grow", "", "현재 저장"), &btn("ks-primary ks-grow", "export", "에셋 저장")])),
    ]);
    frame(6, &main, &insp, &queue(0, &[], false))
}

const PROJ_TABS: [(&str, &str); 2] = [("cube", "투영 만들기"), ("folder", "패키지 확인")];

fn projection() -> String {
    let planes = r#"<div className="ks-planes"><span className="ks-pic ks-pic-day" style="left: 8%; top: 18%"></span><span className="ks-pic" style="left: 20%; top: 14%"></span><span className="ks-pic ks-pic-sunset" style="left: 32%; top: 10%"></span><span className="ks-pic ks-pic-room" style="left: 44%; top: 6%"></span></div>"#;
    let main = c(&[
        &h("3D 투영", "깊이 레이어", "장면을 깊이별 평면으로 나눠 3D 공간에 배치합니다."),
        &tabs(&PROJ_TABS, 0),
        &row(&c(&[GROW, &seg(&["정면", "측면", "위", "자유"], 3)])),
        &format!(r#"<div className="ks-col ks-grow ks-space">{planes}</div>"#),
        &row(&c(&[&p("ks-l", "카메라 시점"), &format!(r#"<div className="ks-col" style="flex: 1">{}</div>"#, slider(62))])),
    ]);
    let insp = c(&[
        &insp_title("cube", "깊이 맵 설정"),
        &sec("깊이 모델", &c(&[&sel("깊이 추정 · 기본"), &p("ks-l", "깊이 범위"), &r#"<div className="ks-depth"></div>"#.to_owned(), &row(&c(&[&p("ks-small", "가까움"), GROW, &p("ks-small", "멀리")]))])),
        &sec("레이어", &c(&[&f("레이어 수", &input("4")), &f("간격", &slider(45))])),
        &sec("좌표계", &c(&[&f("위쪽 축", &sel("Y 위")), &f("단위", &sel("미터")), &f("원점", &sel("화면 중앙"))])),
        &note("ks-warn", "warning", "<span>가장 먼 레이어는 깊이 신뢰도가 낮아 흔들릴 수 있습니다.</span>"),
        &row(&c(&[&btn("ks-outline ks-grow", "", "미리보기"), &btn("ks-primary ks-grow", "export", "내보내기")])),
    ]);
    frame(7, &main, &insp, &queue(0, &[], false))
}

fn projection_package() -> String {
    let f2 = |icon: &str, name: &str| format!("<span>{}{name}</span>", i(icon));
    let tree = format!(r#"<div className="ks-tree">{}{}{}{}{}{}</div>"#,
        f2("folder", "scene/"), f2("file-image", "  layer_01.png"), f2("file-mask", "  layer_01_mask.png"), f2("file-depth", "  depth.png"), f2("file-json", "  projection.json"), f2("file-json", "  scene.gltf"));
    let main = c(&[
        &h("3D 투영", "패키지 확인", "내보낼 파일 묶음을 확인하고 빠진 항목을 고칩니다."),
        &tabs(&PROJ_TABS, 1),
        &format!(r#"<div className="ks-row ks-grow" style="align-items: stretch; gap: 12px">{}<div className="ks-col" style="flex: 0 0 240px">{}{}{}</div></div>"#,
            pic("room", "flex: 1", ""), p("ks-muted", "파일 구성"), tree, pic("", "height: 90px", "")),
        &p("ks-muted", "검증 결과 (6개 중 1개 확인 필요)"),
        &table(&[150, 0, 100], &["항목", "내용", "결과"], &[
            cells(&["좌표계", "Y 위 · 미터", &chip("", "통과")]),
            cells(&["레이어 수", "4개 · 마스크 4개", &chip("", "통과")]),
            cells(&["깊이 맵", "1920 × 1080 · 16비트", &chip("", "통과")]),
            cells(&["텍스처 크기", "레이어 3이 2의 거듭제곱이 아님", &chip("ks-chip-violet", "확인 필요")]),
        ], Some(3)),
    ]);
    let insp = c(&[
        &insp_title("folder", "패키지 설정"),
        &sec("형식", &c(&[&f("장면", &sel("glTF 2.0")), &f("텍스처", &sel("PNG"))])),
        &sec("좌표", &c(&[&f("위쪽 축", &sel("Y 위")), &f("단위", &sel("미터"))])),
        &sec("포함", &c(&[&check(true, "마스크"), &check(true, "깊이 맵"), &check(false, "원본 프레임")])),
        &note("ks-warn", "warning", "<span>텍스처 1개가 권장 크기가 아닙니다. 저장 시 자동으로 늘립니다.</span>"),
        &btn("ks-primary ks-wide", "export", "패키지 저장"),
    ]);
    frame(7, &main, &insp, &queue(1, &[("3D 패키지", "1화 · scene", "대기 중")], false))
}

fn download() -> String {
    let main = c(&[
        &h("영상 저장", "", "선택한 회차를 보관용 파일로 저장합니다."),
        &p("ks-muted", "별빛 여행 · 3화"),
        &stage("flex: 1", ""),
        &transport("00:00 / 23:40", 0, &["volume", "fullscreen"]),
        &p("ks-muted", "트랙 선택"),
        &table(&[24, 90, 0, 120], &["", "종류", "트랙", "상태"], &[
            cells(&[&check(true, ""), "영상", &sel("1080p · H.264"), &chip("", "저장 가능")]),
            cells(&[&check(true, ""), "오디오", &sel("한국어 · AAC 2ch"), &chip("", "저장 가능")]),
            cells(&[&check(false, ""), "자막", &sel("한국어 · ASS"), &chip("ks-chip-gray", "선택 안 함")]),
        ], None),
    ]);
    let insp = c(&[
        &insp_title("export", "저장 방식"),
        &sec("", &c(&[&radio(true, "원본 그대로 · 다시 인코딩하지 않음"), &radio(false, "호환성 우선 · MP4로 변환"), &radio(false, "용량 우선 · 화질 조절")])),
        &sec("출력 설정", &c(&[&f("컨테이너", &sel("MKV")), &f("파일 이름", &input("별빛 여행 - 03화")), &f("저장 위치", &input("동영상 / kinekino"))])),
        &sec("호환성 확인", &c(&[&note("", "info", "<span>MKV는 일부 기기에서 재생되지 않을 수 있습니다.</span>"), &note("ks-warn", "warning", "<span>권한이 있는 영상만 저장하세요.</span>")])),
        &btn("ks-primary ks-wide", "export", "저장 시작"),
    ]);
    frame(8, &main, &insp, &queue(0, &[], false))
}

fn models() -> String {
    let cat = |name: &str, n: &str, open: bool| {
        let arrow = if open { "chevron-down" } else { "forward" };
        format!(r#"<div className="ks-row ks-cat">{}{}<span>{name}</span><span className="ks-small">({n})</span></div>"#, i(arrow), i("model"))
    };
    let model = format!(r#"<div className="ks-row ks-frame ks-on ks-model">{}<div className="ks-col ks-grow" style="gap: 2px"><span>음성 인식 · 기본</span><span>{}{}</span></div>{}</div>"#,
        check(true, ""), chip("", "설치됨"), chip("ks-chip-gray", "1.5 GB"), btn("", "", "사용 중"));
    let main = c(&[
        &h("모델 관리", "", "AI 기능에 쓰는 모델을 내려받고 이 컴퓨터에 맞게 설정합니다."),
        &row(&c(&[&format!(r#"<span className="ks-input ks-grow">{}모델 이름, 기능, 크기 검색</span>"#, i("search")), &sel("전체")])),
        &format!(r#"<div className="ks-col ks-grow ks-card" style="gap: 6px">{}{}{}{}{}{}{}</div>"#,
            cat("음성 인식", "2", true), model, cat("화면 OCR", "1", false), cat("피사체 분할", "2", false), cat("깊이 추정", "1", false), cat("텍스트", "2", false), cat("장면 추천", "1", false)),
    ]);
    let insp = c(&[
        &insp_title("model", "모델 정보"),
        &sec("", &row(&c(&[&chip("", "설치됨"), &chip("ks-chip-gray", "음성 인식"), GROW, &btn("", "", "삭제")]))),
        &sec("파일", &c(&[&f("이름", "음성 인식 · 기본"), &f("크기", "1.5 GB"), &f("저장 위치", "모델 폴더"), &f("라이선스", "공개 모델")])),
        &sec("실행", &c(&[&f("양자화", &sel("Q4 (권장)")), &f("실행 장치", &sel("자동 (GPU 우선)")), &f("메모리 한도", &input("4 GB"))])),
        &btn("ks-outline ks-wide", "refresh", "업데이트 확인"),
    ]);
    frame(9, &main, &insp, &queue(0, &[], false))
}
