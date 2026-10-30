---
id: evidence:kinekino/20261029114813
title: GUI·CLI 공통 SDK 통합 감사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# GUI·CLI 공통 SDK 통합 감사

2026-10-18. **읽기 전용 source 감사**와 root가 별도로 보존한 실행 관측을 구분한다. 이번 작업은 제품 source 수정·빌드·시험 실행·GUI 실행·모델/도구/기존 증거 변경 없이 수행했다. 작성 파일은 이 보고서와 runtime/coordination/sdk-integration-audit.md 두 개뿐이다. source와 기존 EXE가 동일하다는 보장은 하지 않는다. UI 세션이 presentation을 병행 수정하므로 아래 줄 번호와 화면 연결은 감사 시점의 source 기준이다. 시각 일치·전체 제품 완성·AI 품질 승인은 범위 밖이다.

## 결론과 근거 수준

- 등록 명령은52개다. source dispatch는 discovery3개를 제공하고 subtitles.convert·production.capture_frame 두 개를 adapter 조건부로 제공한다. 나머지47개는 명시적 unsupported다. 이름이나 schema 조회 가능성을 실행 지원으로 해석하면 안 된다.
- 정상 GUI·CLI 조립은 같은 PlayerSdk에 SubtitleFiles와 FfmpegFrameCapture를 넣는다. GUI의 실제 자막 변환은 동일 ApplicationClient/usecase로 연결된다. 프레임 추출의 GUI 실행 연결과 legacy playback의 공통 SDK 연결은 아직 없다.
- WorkspaceReadPort는 application 계약이나, Live provider는 Unsupported/빈 snapshot이다. DesignSample/로컬 UI 편집 상태와 실제 저장·분석·모델 설치·Job 결과를 구분한다. 검토한 제품 API/화면 경로에서 모델 스파이크를 실행 가능한 제품 AI 명령으로 광고하는 근거는 찾지 못했다.
- CLI 오류 request_id 보존의 비대칭, schema error catalog와 UTF8 길이 규칙의 차이, 오래된 SOT 상태·CLI 도움말 형태를 최소 후속 수정 대상으로 제시한다. 이번 감사에서 수정하지 않았다.

근거 A는 아래 source 경로/줄 번호의 정적 분기이며 실제 성공 측정이 아니다. 근거 B는 [root 통합 회귀 보고서](root-integration-regression-2026-10-18.md)와 그 저장 stdout을 읽은 관측이다. root는73tests/fmt/clippy와 EXE SHA54f483ef938aca6f5127615cdb0102e939aa979b3de7187055070027782773b9를 별도로 기록했다. 이 감사가 그 실행을 재수행하거나 이후 source까지 자동 연결한 것은 아니다.

읽은 [capabilities stdout](../../../evidence/player/sdk-integration-regression-2026-10-18-r1/capabilities.stdout)은 request_id=integration-capabilities-r1, discovery3+convert+frame supported, local_host_transport unsupported를 반환했다. FFmpeg/FFprobe6.1이 발견된 **그 호스트/그 실행**의 결과다. [subtitle schema stdout](../../../evidence/player/sdk-integration-regression-2026-10-18-r1/subtitle-schema.stdout)의 누락 error catalog도 source와 같았다. root source/EXE provenance 및 actual read-only 요청 관측은 root 보고서 소유로 유지한다.

## 조립·포트 연결

| 경계 | source 근거 | 현재 의미 |
| --- | --- | --- |
| 공개 library/facade | src/lib.rs:1–6; src/application/api/mod.rs:43–92 | application/domain/infrastructure 공개, PlayerSdk의 실제 보유 adapter는 subtitle storage와 frame capture 두 개 |
| 정상 GUI | src/main.rs:16–27 | 같은 SDK+실제 파일/FFmpeg adapter와 native subtitle path picker를 주입 |
| CLI | src/bin/player-cli/runner.rs:20–28 | GUI 자동화 없이 동일 SDK 조립·execute; GUI host 연결 없음 |
| typed client | src/application/api/client.rs:8–47 | capture_frame/convert_subtitles가 같은 Request dispatch로 정규화됨 |
| GUI 자막 변환 | src/presentation/studio/mod.rs:43–55,318–320; src/presentation/studio/subtitles.rs:85–105 | capability gate, 별도 worker, 같은 손실 확인 usecase. 경로 선택은 별도 application port |
| GUI read model | src/application/workspace_read/mod.rs:77–96 | WorkspaceReadPort 별도 계약; Live Unsupported이며 source/canvas/table/queue를 위조하지 않음 |
| demo | src/main.rs:10–13; src/presentation/design_preview.rs:26–30; src/application/workspace_read/preview.rs:37,63–64 | 별도 Preview provider/DesignSample. 정상 GUI adapter 주입 전 분기되며 demo를 live 기능 지원으로 세지 않음 |
| legacy 재생 | src/presentation/player_app.rs:14–28,117–125,210–219; src/presentation/transport.rs:23–100,274–278 | Browser/Mpv를 presentation이 직접 보유·호출. 일부 application pause/route/episode 정책을 사용하나 공개 SDK PlaybackPort/dispatch는 없음 |

## 명령별 실제 범위

S=source 구현 지원, C=adapter/입력 조건부 지원, U=등록은 있으나 Unsupported, D=DesignSample만 표시. GUI 열의 legacy는 source 호출 경로를 뜻하며 현재 EXE의 실제 재생 성공을 측정한 표시는 아니다. 모든 U는 capabilities.rs:58–61의 supported=false 및 queries.rs:61–65의 unavailable application adapter 분기로 연결된다. U에도 schema 조회 자체는 가능하지만 실행 payload 계약은 제공되지 않는다(schema.rs:36–38).

| 명령 | SDK·CLI | GUI 연결/표시 | 근거/제약 |
| --- | --- | --- | --- |
| version | S | SDK 상태 조회에 사용 | queries.rs:22–24; studio/sdk_status.rs:12 |
| capabilities | S | SDK 상태/변환 활성 판단 | queries.rs:25–28; studio/mod.rs:46–47 |
| schema | S | 전용 GUI 입력 화면은 없음 | queries.rs:29–33; 알려진52명령만 조회 |
| catalog.search | U | browser 탐색은 typed catalog 검색과 별개 | legacy 주소/사이트 표시를 CatalogPort로 세지 않음 |
| episodes.list | U | legacy 회차 목록/선택 | episode_panel.rs:4–22,39,80; 공통 query 아님 |
| episodes.resolve | U | legacy browser/source 확보 | typed resolve receipt 없음 |
| playback.open | U | legacy 웹/로컬 재생 시작 경로 | transport.rs:39–58; PlayerApp의 file_picker/browser |
| playback.status | U | legacy snapshot을 UI에서 읽음 | transport.rs:23–36 |
| playback.play | U | legacy toggle로 재생 의도 처리 | transport.rs:96–100,135–155; 독립 SDK play 아님 |
| playback.pause | U | legacy toggle/browser pause | 위와 동일; shared pause command 없음 |
| playback.seek | U | legacy seek | transport.rs:88–93; GUI seconds와 SDK 예정 integer us를 혼동하지 않음 |
| playback.engine | U | legacy native/web 전환 | transport.rs:39–84 |
| playback.set_volume | U | legacy volume | transport.rs:274–278 |
| playback.set_rate | U | 공통 SDK 연결 없음 | 현재 transport에서 rate 호출 근거 미확인 |
| playback.select_track | U | 공통 SDK 연결 없음 | 자막 편집 탭을 재생 track API로 세지 않음 |
| playback.previous | U | legacy 회차 버튼 | transport.rs:135; episode_panel.rs |
| playback.next | U | legacy 회차 버튼 | transport.rs:155; episode_panel.rs |
| playback.close | U | 내부 stop_native 경로는 별개 | player_app.rs:218; SDK close receipt 없음 |
| segments.detect | U | SEG 구성/D·미연결 실행 | live reader Unsupported; 실행 button 비활성 |
| segments.edit | U | UI 로컬 선택/D | domain revision 저장 아님 |
| segments.accept_skip | U | 실제 수락 API 없음 | D 제안을 확정 결과로 세지 않음 |
| segments.set_skip_policy | U | 공통 정책 command 없음 | legacy 재생과도 별개 |
| subtitles.transcribe | U | SUB 인식 구성/D·연결 필요 | 모델 스파이크는 등록 adapter 아님 |
| subtitles.diarize | U | SUB 분리 구성/D·연결 필요 | 실제 diarization receipt 없음 |
| subtitles.convert | C | **실제 공통 client 연결** | UTF8 SRT/VTT≤8MiB, loss gate, 새 파일. storage 없으면 U |
| subtitles.speakers | U | 화자 화면 구성/D | 읽기 API/실제 화자 근거 없음 |
| subtitles.align_speakers | U | 연결 필요 | 외부 forced-aligner 연구와 제품 통합을 구분 |
| subtitles.edit_cues | U | 시안/로컬 UI 상태 | 저장된 CueDocument revision 없음 |
| subtitles.edit_speakers | U | 시안/로컬 UI 상태 | 공통 변경 command 없음 |
| production.render_short | U | CLIP manual/recommend/batch 구성/D | 추천 목록·일괄 row를 실제 Job 결과로 세지 않음 |
| production.download_media | U | SAVE 구성/D | 실제 다운로드 command 없음 |
| production.capture_frame | C | **typed client는 있으나 GUI 호출 없음** | trusted FFmpeg/FFprobe, absolute local MP4/MKV/WebM→새 PNG |
| production.export_cards | U | CARD preset/ratio/template 구성/D | local UI 편집을 파일 export로 세지 않음 |
| production.segment_subject | U | OBJ/A2D 구성/D | 생성 alpha 자산은 실분할 관측 아님 |
| production.build_assets | U | A2D readonly canvas/D | mask edit/파일 저장 API 없음 |
| production.package_projection | U | PROJ 구성/D | 실제 package/artifact receipt 없음 |
| models.list | U | MODEL table/D·catalog 연결 필요 | root 로컬 model cache와 연결 안 됨 |
| models.inspect | U | metadata 항목/D·연결 필요 | 현재 실제 installed 상태 조회 아님 |
| models.install | U | 모델 다운로드 button 비활성 | system.rs:136 |
| models.verify | U | 실제 검증 command 없음 | 별도 Rust spike 검사와 구분 |
| models.remove | U | 실제 삭제 command 없음 | 사용자 모델 파일 변경 없음 |
| jobs.get | U | JOB table/D | SubtitleConversion.activity는 durable Job 조회가 아님 |
| jobs.submit | U | 전체 시작 button 비활성 | system.rs:65–66 |
| jobs.list | U | JOB/queue D | live queue 비어 있음 |
| jobs.watch | U | 이벤트 DTO만 존재 | api/events.rs:5–20; EventStream/NDJSON 미구현 |
| jobs.cancel | U | 실제 job 취소 미연결 | system.rs:162 |
| jobs.retry | U | 실제 job 재시도 미연결 | system.rs:162 |
| projects.create | U | 프로젝트 구상/D | 실제 ProjectStore command 없음 |
| projects.inspect | U | source/snapshot 표시와 별개 | 영속 project 조회 없음 |
| projects.save_revision | U | 실제 revision 저장 없음 | GUI 로컬 상태 유지와 구분 |
| projects.reconnect_source | U | 실제 reconnect command 없음 | browser 탐색과 별개 |
| projects.export | U | OUT 구성/D·비활성 | system.rs:181–209 |

표의 query/dispatcher는 src/application/api, GUI screen은 src/presentation/studio/screens 기준이다. DesignSample 경계 근거: screens/view.rs:20의 origin gate, studio/mod.rs:278–295의 샘플 큐 표시, screens/system.rs:93의 sample asset gate와136/162/181–209 비활성 실행, studio/inspector.rs:36–37,178–182의 비활성 실행. source 검토에서 이를 실제 모델 설치/Job/파일 성공으로 승격한 문구는 확인하지 못했다. 실제 화면의 현재 표시·시안 대응은 UI 담당의 실행 검토 범위다.

## 조건부 두 유스케이스

자막은 SubtitleFiles.available()를 사용하는 정상 entry에서 지원되며 Default SDK의 UnavailableStorage에서는 U다(storage.rs:5–16). GUI와 CLI는 convert::execute의 format/8MiB/원본 보존/손실 목록→allow_loss 명시 확인→publish_new를 공유한다(convert.rs:33–93). GUI는 context에 timeout/revision/idempotency를 넣지 않는다. CLI가 이를 넣으면 queries.rs:50–59에서 Unsupported다. GUI worker의 pending/receipt 표시는 durable jobs.submit/get/watch 구현 증거가 아니다. show_preview의 출력 위치/cue 수/bytes는 receipt 요약이지 원문·변환문 전체 read-preview API가 아니다(subtitles.rs:270–293).

프레임은 discover가 trusted local FFmpeg/FFprobe의 version까지 확인한 경우에만 C다(infrastructure/frame_capture/mod.rs:46–74). schema/dispatch 모두 같은 capability bool을 사용한다. source PTS decimal us, 첫 source PTS≥요청, MP4/MKV/WebM, PNG 신규 저장, network/HDR 제한은 별도 capture 계약이다. timeout 기본30000/최대120000ms(application/frame_capture.rs:76–81), expected_revision/idempotency는 U(queries.rs:35–42). 이는 동기 CapturedFrame receipt이며 JobId/ArtifactId를 돌려주는 전체 production/job 설계가 구현된 것은 아니다. GUI client에는 capture_frame이 있지만 presentation의 해당 호출을 찾지 못했고 FRAME/OUT 실행은 비활성이다. FFmpeg 지원 여부만으로 GUI 버튼이 연결됐다고 하지 않는다.

## CLI 입출력·도움말

runner.rs:45–125는 단일 command 또는 GROUP ACTION을 canonical dot command로 정규화한다. schema group/action도 처리한다. --input FILE|-는 **full Request JSON**이며1MiB 한도, unknown root/context field 거절, UTF8 JSON→같은 SDK를 사용한다(136–150). full JSON과 context flags를 함께 쓰면 거절하고, JSON command와 positional command가 함께 있으면 중복 오류다(83–88,109–112). payload는 object여야 한다(commands.rs:61). --json은 기본 compact envelope, --human은 pretty envelope이지 다른 정책/텍스트 결과가 아니다. 출력 실패는1, API 오류는 errors.rs:42–54의2/3/4/5/6/7/8/9/10/1이다.

--endpoint는 parser가 받지만 transport Unsupported3으로 끝난다(127–132). --wait, --job-id, watch NDJSON/host/detach는 구현하지 않았다. help는 현재 변환과 조건부 capture 지원을 올바르게 설명하고 과거의 “media commands unsupported” 문구는 없다(runner.rs:9–15; tests/cli_contract.rs:49–57). CLI assembly는 발견 단계에서 local FFmpeg version probe를 할 수 있어 discovery를 “외부 프로세스도 전혀 없는 조회”로 확대하지 않는다. 네트워크·모델 설치나 GUI 자동화를 수행하는 CLI 연결은 없다.

## 확인된 차이와 최소 후속 제안

### A-01 · P2 · validation 실패의 request_id가 CLI에서 소실

runner.rs:20–30은 parse 오류를 Response::failure("cli",error)로 만든다. parse는125에서 request.validate()?를 먼저 호출하므로, 정상 deserialize된 full Request의 유효한 request_id도 api_version/timeout/context validation 실패 시 잃는다. 반면 SDK mod.rs:70–82는 동일 오류에 request.context.request_id를 보존한다.

정적 재현 입력: context.request_id="audit-id", api_version="2", command="version", payload={}인 JSON. SDK 직접 execute의 오류 ID는audit-id, CLI --input 경로는cli가 된다. **이번 감사에서 실행한 재현은 아니다.** 최소 수정: parse 오류가 가능한 경우 parsed request_id를 같이 반환하거나, 정상 구성된 Request의 semantic validation은 SDK에 맡기고 endpoint/구문 오류에도 안전하게 얻은 ID를 보존한다. root 후속 회귀는 동일 잘못된 Request의 GUI client/SDK/CLI code·request_id 일치를 확인한다.

### A-02 · P2 · schema의 error catalog가 enum과 불일치

api/schema.rs:62의 error_codes9개에 source_expired/cancelled/internal이 없지만 errors.rs:14–18,50–52는 이를 안정 code/exit로 정의한다. client.rs:50–51의 invalid application response도 internal을 만든다. root의 저장 subtitle-schema stdout에도 같은9개 catalog가 있다. 이는 error.properties.code에 enum이 있어 응답 schema 검증이 바로 실패한다는 주장이 아니라 **제공한 code 목록이 불완전**하다는 차이다. 최소 수정: 단일 ErrorCode 목록에서 catalog를 생성하고 전 변형 serialize/exit/schema 목록 대조 회귀를 추가한다.

### A-03 · P2 · request/context 문자열 길이 의미가 schema와 다름

commands.rs:51,58–59는 String.len()의UTF8 **bytes**≤128을 검사한다. schema.rs:67의 JSON Schema maxLength128은Unicode 문자 수다. 예를 들어 request_id="가"×43은43문자/129bytes라 schema의 해당 제한은 통과하고 Rust는 거절한다. idempotency_key에도 같은 차이가 있다. expected_revision도schema 숫자 문자열 pattern만으로는 runtime u64 범위를 모두 설명하지 않는다. 최소 수정: ID의 허용 문자·byte/문자 단위를 정본에서 결정하고 schema의 표현/설명과 runtime을 일치시킨다. ASCII 제한을 임의 추가하는 것은 별도 정책 변경이므로 여기서 확정하지 않는다. root는Unicode 경계 및 u64 최대/초과를 확인한다.

### A-04 · P2 · SDK·CLI 정본의 구현 상태가 오래됨

application/api/sdk-contract.md:3,9,마지막 인수 문단은 API/CLI 생성·연동 시험이 전부 미수행이라고 쓴다. presentation/cli/automation-contract.md:3,11도 전체 미구현/모든 예시 비실행으로 표시한다. 현재 source의 두 bin(Cargo.toml:24–30), 공통 facade/두 파일 유스케이스 및 root의 실제 discovery 관측과 구분이 필요하다. SOT에는 이미 구현된 source/호스트 관측·계획 항목·미검증 인수를 분리하고 이 감사/root 회귀를 링크하는 것이 최소 수정이다. 아직 미구현인 local-runtime transport 설계는 그대로 미구현으로 둔다. 모든 목표 기능을 완료로 바꾸면 안 된다.

CLI 설계9,15–17의 --wait/--job-id/JSON+context flag 예시는 현재 실행 가능한 문법이 아니다. 현행 실행 예시는 full Request의 canonical --input 형식을 별도로 제공하고 나머지는 “계획”으로 유지한다. sdk-contract.md 기능표의 command_schema/jobs inspect와 실제 schema/jobs.get 이름도 안정 명령명/개념명 구분을 명확히 한다.

### A-05 · P2 · legacy playback은 아직 공통 facade 밖

player_app.rs:14–28,210–219 및 transport.rs:39–100,274–278의 직접 Browser/Mpv 제어가 남아 있는 반면 playback.*는 모두 U다. top-level 공통 GUI/CLI 유스케이스 요구의 남은 통합 간극이며 새로운 회귀나 “SDK가 playback 지원을 거짓 광고”한 문제는 아니다. 최소 후속 단위: application PlaybackPort/세션 계약에 기존 pause/route/episode 정책을 모으고 SDK dispatch와 GUI 둘 다 연결한 뒤 capabilities를 연다. GUI의 toggle/cycle을 play/pause 명령으로 단순 복사하면 두 번 play가 pause가 되는 의미 차이가 생길 수 있으므로 의도 기반 정책과 generation/us 변환을 먼저 정의한다. 임의 GUI 자동화/숨은 host 실행으로 CLI 격차를 메우지 않는다.

### A-06 · P2 · capture는 CLI/API 구현에 비해 GUI 실행 연결이 없음

api/client.rs:10–28의 typed capture와 정상 GUI의 adapter 주입은 있지만 presentation에 capture_frame/production.capture_frame 호출이 없다. FRAME 화면/inspector 구성이나 SDK capability의 supported를 실제 GUI capture 완료로 표시하면 안 된다. 최소 후속: GUI의 실제 local source/PTS·새 output 입력을 공통 CaptureFrame DTO로 만들어 같은 client를 worker에서 호출하고 receipt/error를 표시한다. 연결·실행 검사 전에는 현재 비활성 상태를 유지한다. 이번 감사는 해당 UI를 구현하거나 실행하지 않았다.

### A-07 · P2 · help Usage가 full JSON 단독 형식을 명확히 보여주지 않음

runner.rs:11의 Usage는 positional command가 필수이고 --input은 거기에 추가하는 옵션처럼 보인다. 실제로 full Request에 command가 있으면 `player-cli --input request.json`이 canonical 형식이고 command를 또 적으면111에서 거절된다. 최소 수정은 Usage를 “positional discovery/command”와 “full Request --input” 두 형태로 나누고 JSON+context flags/중복 command 금지를 명시하는 것이다. policy를 바꾸거나 중복 입력을 묵시적으로 우선하지 않는다.

## source 식별과 남은 검증

감사에서 읽은 source hash는 capabilities.rs=dd9793dcc54b75edc15a9ac7fbceda7fd124c7e8c235384218452fe125c205d3, queries.rs=49c4cb68234cf35566c84894e8247c0fca93e030cfa6a5787d462f1eb325ea51, schema.rs=bbe480921da28799d784df51ec77b41a1a1aa76aa9a61be06ee0f930136965f3, runner.rs=6f0045ac63e7b3c8a597e3124120186215140590bddd64eb1c7eeb072ba9f05e다. 이 값은 source 식별만 하며 root EXE와의 일치 증거를 대신하지 않는다.

root가 후속 수정 시 fmt/test/clippy와 read-only CLI 요청/invalid-context ID/schema 경계를 검증한다. 실제 CLI 파일 작업/legacy media 동작/GUI13화면 시각 인수/AI모델 parity는 별도 근거로 유지한다. 본 감사에서는 실행·빌드·모델 읽기/다운로드·Pages/VM/DPI/자원 설정·사용자 앱·부모 Git index 변경을 수행하지 않았다.
