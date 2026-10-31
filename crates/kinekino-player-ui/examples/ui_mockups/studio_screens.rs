// Screen bodies for the studio frame (included by studio.rs).

fn screen_tsx(screen: &str) -> Option<String> {
    Some(match screen {
        "studio-overview" => overview(),
        "studio-skip" => skip(),
        "studio-subtitles" => subtitles(),
        "studio-ocr" => ocr(),
        "studio-correction" => correction(),
        "studio-convert" => convert(),
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
