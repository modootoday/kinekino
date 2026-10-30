---
id: evidence:kinekino/20261029113525
title: 실제 회차의 제공자 이동 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 회차의 제공자 이동 관측

2026-10-25, PM의 Codex 내장 브라우저 Browser Use read-only 조사. 제품 GUI/SDK 기능 시험의 대체가 아니다. source283-r3 DDBF의 실제 airing1은 provider 범위 검사 NeedsInput으로 멈췄고, 이 조사는 원인 대조다.

## 원 SDK 근거

원 episode DOM artifact6364ad13a211e35e2dc4b7ebc0a2d25385585752db365a1df3b0b6bc5aba14b1, body169209B/SHA8a90485f9c4b9a49654d7a1909684d27a6272e65fb679e7df2b2c79bd655cde6. requested/final 모두 `https://source.example.test/e/명탐정%20프리큐어%201화`, redirect0. SDK가 읽은 props.playing.body.url은 `https://michealcdn.com/video/4259f88ec5599c3e58266f349beceb45`다. 기존 PlayerDocumentLocation의 media.example.test/video/32hex 정책이 이를 거부했다. media_verified=false이며 HLS 검증·mpv 재생 성공은 없다.

## PM 실제 브라우저 관측

1. agent-created hidden IAB tab1에서 위 michealcdn 원 URL을 직접 열었다. 제목/AX는 `404 Not Found`, 본문은 requested URL not found였다. 보안 경고나 CAPTCHA는 없었고 우회하지 않았다.
2. 같은 tab1에서 원 source 1화로 이동했다. 제목은 명탐정 프리큐어 1화, AX에 회차 보기의 `바로 보기` 버튼69가 있었다.
3. 버튼69를 1회 클릭했다. 원 회차 탭은 그대로였으며 tab inventory에서 새 tab2가 확인됐다. 실제 URL은 `https://frame.example.test/tv-show.php?v=c0e169bcb9de1e0ca99e7df83db6d658`, 제목은 같은 회차였다.
4. tab2 AX에는 `OHLI` iframe, 위와 동일한 michealcdn/video 원 URL, Video Player/player, Volume90%, Play 버튼이 표시됐다. 광고 영역도 별도로 있었다. Play는 누르지 않았으며 실제 재생·음성·HLS·광고차단 성공을 주장하지 않는다.
5. PM이 만든 tab1/tab2를 모두 닫고 IAB tab inventory=[]를 확인했다. 광고 링크·가입·설치·공유·다운로드 조작0. SDK/AI 사용자 저장소 쓰기0.

## 해석과 후속 계약

단독 URL의404만으로 제공자 전체가 죽었다고 단정할 수 없다. 같은 URL이 wrapper iframe 안에서는 플레이어를 표시했으므로 중간 페이지/요청 문맥 차이를 확인해야 한다. Referer 또는 cookie가 원인인지는 미관측이며 추론이다. 헤더를 임의 생성하거나 provider URL을 다시 쓰는 근거로 사용하지 않는다.

정확한 원 episode→실제 watch popup(frame.example)→iframe(michealcdn) 관계를 공통 Rust SDK/CLI의 브라우저 관측·출처 계약으로 재현해야 한다. 명시 host/path 허용은 필요성·같은 원 회차·실 navigation 관계를 검증하고 추가한다. 광범위 host 허용, URL 치환, 직접 HTTP 우회 또는 페이지의 설치 광고를 기능으로 채택하지 않는다. 기존 HLS/원 job/취소/저장/원 native owner 정책을 유지한다.

원 SDK evidence: %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/evidence/product-review-route283-airing-r1/catalog/. 실패 gate: product-review-route283-airing-r1-gate. PM 도구 근거는 이 채팅의 cua_repl createBrowserTab/getAXState/goto/click69/listTabs/getTab2/close이다. 실제 CLI 재현과 후계 실행은 아직 미검증이다.
## 기존 Rust flow 대조 정정

SDK가 실제 호출 경로를 재확인했다. BrowserEpisodeResolveJobs::run은 player_target의 선언 target 검증 뒤 session.activate를 호출하며, 기존 watch-now 클릭→player_document_ready(wrapper)→child_document_ready(exact target)→network HLS/lineage/wire Referer 관측 경로가 이미 있다. 이번 거부는 activation 이전이므로 wrapper 흐름이 실행되지 않은 것이다. PM/SDK의 초기 의심을 기존 wrapper 포트 누락으로 확정하지 않는다. 새 click/JS/typed port를 추가하기보다 정확한 새 host의 동일 shape 계약과 기존 navigation::player_page 연결을 최소 후계로 검증한다.

PM raw DOM에는 type=button/class=watch-now/바로 보기 텍스트가 있고 AX에는 대상 버튼1개가 있었다. 실제 클릭 전 tab1, 클릭 후 tab2 신규 생성은 관측했지만 IAB의 HTTP redirect/header/COM opener 식별은 수집하지 않았다. 이를 SDK의 원 lineage 근거로 대신하지 않는다. 기존 고정 JS actuator의 Rust 이관 추적도 유지한다.

## provider288 실제 후계 관측

### 진단 후계294의 동일 첫 회차

전송 진단 범위: PM이 원 trace의 truncated=true와 마지막 event elapsed_us=12891870, fixture elapsed_us=46529986을 확인했다. 따라서 초기 약12.9초의 요청·ACK 구간은 포함하지만 전체 실행 후반의 전송 기록은 없다. 이번 성공으로 과거 ACK 원인이나 transport 문제 해결을 확정하지 않는다. 원 native observations에는 connected=true/volume_percent=0이 있으므로 실제 음량 청취 성공도 주장하지 않는다. bounded trace의 명시 truncation 동작과 전체 관측 충족 여부는 구별한다.

PM이 product-review-observed-transport-airing-r1-gate receipt exit0/47642ms·stdout1pass를 직접 읽었다. 같은 명탐정 프리큐어1화의 resolve-get은 completed/manifest_verified=true이며 inspect는 원 Referer absent를 보존한다. open 응답의 completion=unobserved 자체는 재생 성공 근거가 아니다. 별도 native diagnostic은 mpv의 h2641920x1080/d3d11va·gpu-next·wasapi stereo 초기화, frame.png 저장과 시간 진행을 담고 있다. 동시에 audio device underrun 및 dropped frames도 있어 짧은 CPU2 제한 실행의 성능 한계로 기록한다. 실제 음량 청취·장시간 품질·나머지 카테고리·현재 정상 GUI 왕복은 별도 미검증이다. 나머지3 actual과 원 cleanup/transport trace는 SDK가 확인한다. 이번 성공만으로 이전 transport Timeout의 원인을 해결했다고 단정하지 않는다.

source `63C98DC5A5895812FF0A85ACA400662EECA5E69711F635C6259A832BCB834880`의 같은 첫 airing은 149230ms/exit101, `Timeout: Episode resolution budget expired`, media_verified=false로 종료했다. 전체67 회귀와 정상 bins 빌드 성공은 이 실제 실패를 대체하지 않는다. 뒤3 표본은 실행하지 않았다.

원 evidence 루트는 %USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/evidence/product-review-observed-provider-airing-r1/ 이다. PM이 직접 읽은 catalog capture `1b0f2c12…`는 원 episode→frame.example navigation200과 광고차단 installed=true/requested68/blocked3을, `46f60c97…`는 원 child frame/loader 범위의 michealcdn `/m3/…` GET/XHR 응답200/text/plain57322B를 보존한다. wire Referer는 absent이며 추측해 보충하지 않는다. SDK는 실제 frame의 scripts.php?v=7 선언과 master.txt의 EXTM3U 본문을 확인 중이다. 최종 HLS 계약 충족·mpv 재생은 아직 입증되지 않았다.

PM이 직접 읽은 routed-gui-airing-first/routed-final.jsonl의 native_routed schema2는 cleanup_complete=true/first_error=null, execution unacknowledged0, local host joined, provider completed/browser_worker_exited=true, dispatcher joined/faults0을 기록한다. normal_os_exit=true/success=true는 해당 종료 보고의 성공이며 회차 해석 성공을 뜻하지 않는다. 시험 도구의 configured-owner cleanup incomplete 문구는 원 실패로 보존하되 실제 정리 실패로 계상하지 않는다. SDK가 기능 결과의 기대 completed jobs 수와 정리 판정의 혼합 여부를 대조하고 별도 진단 후계를 작성한다.

SDK의 원 trace 대조에서 michealcdn GET/Script `/player/assets/scripts.php?v=7` 응답200과 POST/XHR `/player/index.php?data=<sameid>&do=getVideo` 응답200은 관측됐으나 capture_issued=false였다. PM이 현재 response_purpose 소스를 직접 읽어 두 목적의 player 판정이 media-cdn host에 고정된 것을 확인했다. SDK는 원 secure origin 정책을 공유하는 최소2leaf 후계를 작성한다. 기존 method/resource/path·lineage·본문·HLS·Referer 검증을 유지하며 이 분류 함수에 새로운 query 정책이나 요청 생성 기능을 넣지 않는다. 후계 실행 전이므로 원인 대조와 수정 배정 상태이며 해결 완료는 아니다.
