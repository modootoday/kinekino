---
id: evidence:kinekino/20261029114829
title: contracts·publication·자막의 실제 패키지 분리
status: active
created: 2026-10-29
updated: 2026-10-29
---
# contracts·publication·자막의 실제 패키지 분리

2026-10-19. SDK 담당 세션이 부분 변경을 인계받아 구현과 검증을 완료했다. [최초 미검증 인계](../../architecture/package-extraction-owner-handoff-2026-10-19.md)와 이번 범위를 구분한다. 조사 agent는 최종 구현·빌드 담당이 아니다. 새 AI 추론·다운로드·벤치마크는 실행하지 않았다. **현재 완료 근거는 CLI와 비GUI 검사이며, 순수 분리 고정 GUI의 실제 검토는 UI 담당이 진행 중이다.**

## 구현 경계

- media-contracts는 기존 ApiError/ErrorCode와 DecimalU64를 소유한다. Request/API_VERSION/dispatch는 SDK에 남는다.
- media-file-publication은 하나의 덮어쓰기 없는 publication 구현을 소유한다. 자막·frame_capture·review가 같은 구현을 사용한다.
- media-subtitles는 SRT/VTT의 domain/application/파일 adapter를 수직으로 소유한다. GUI/WebView/engine/root API 역의존은 없다. 기존 공개 경로는 reexport shim으로 유지한다.

root workspace의 명시 members/default-members는 root와 3개 crate다. Meari gitlink와 독립 tool은 별도 workspace다. lock 변경은 로컬 3개 node/path 참조 추가이며 registry 버전 변경은 없다. workspace 선언이 없던 tool 5개 manifest에 resolver3의 독립 workspace를 추가했다. root 오소속 오류 원문을 보존하고 metadata --offline --locked 성공을 확인했다. 추론·모델 코드는 바꾸지 않았다.

기존 subtitles 하위 구현 사본은 root에 남지만 mod에서 참조하지 않는다. **컴파일되는 정본은 새 crate 하나다.** 불필요한 사본과 옛 source 링크를 정리하고 필요한 호환 shim만 유지하는 후속 작업으로 추적한다. 사본을 별도 정책 정본으로 취급하지 않는다.

## 검사와 실제 실행

근거는 `runtime/quality-review/package-extraction-r1/`이다. before에 부분 작업 상태의 src/tests/examples/Cargo/lock과 최초 fmt exit1 원문을 보존했다. 담당 세션이 import 순서를 수정했다. phase1 contracts/publication check/test/clippy, phase2 workspace fmt/test **155 pass/9 ignored**, strict all-targets Clippy, 양쪽 bin/examples build가 모두 exit0이다. 낮은 우선순위/-j1/testthread1을 사용했으며 기존 작업을 종료하지 않았다.

공통 publisher의 실제 FFmpeg 추가 회귀 3개도 통과했다. 성공 PNG/typed client, timeout/overwrite/hardlink, 경쟁/oversize를 검증했다. 표준 suite의 9 ignored를 성공으로 재분류하지 않는다.

고정 CLI의 실제 UTF8 한글/일본어 SRT→VTT→SRT는 exit0/0이다. 손실 미확인은 needs_input/exit3이며 출력하지 않았다. 명시 allow_loss+prefix는 exit0, 기존 출력 거부는 revision_conflict/exit5다. 입력 SHA 전후와 전체 request/stdout/stderr·출력은 cli-conversion에 보존했다.

고정 importer로 기존 실제 모델 출력 9개를 새 input-store/technical-store에 재실행해 exit0을 확인했다. 원 raw의 needs_repair, 명시 scripted 편집/채택, 새 CLI receipt 재조회, source/raw SHA 불변을 검증했다. 품질 승인 false/사용자 verdict0/새 추론0이다. 미디어 렌더나 project-current 증거가 아니다. 새 CLI로 옛 S1/S2 저장 schema1/2와 receipt3도 exit0으로 읽었다. 저장 형식을 자동 이관하지 않았다. 실제9 stdout/stderr·legacy S1/S2·source 전후·lock diff·독립 tool metadata 원문은 logs에 보존한다.

고정 GUI SHA `75c8a2d1be79fa27d71d05040081d9bbc85eeb25b751d77cd336728f9055020b`, CLI `a22a78611936402c5d0862748a71e3cbf0af3d92b466979835f06618260dded1`, importer `323fe6286b7bad67bb5b94b288c001572346033519b59027689785be0199b549`다. source-at-audit/pins와 함께 UI 담당에게 전달했다. 후속 focus/layout 소스는 이 순수 분리 실행에 귀속하지 않고 별도 checkpoint로 검사한다. CLI 검사는 실제 GUI 검토를 대체하지 않는다.

## 공개 소스와 남은 범위

후속 실제 GUI 기준 검토는 [r168 관측](../presentation/package-extraction-baseline-ui-2026-10-19.md)에 기록됐다. 같은 고정 SHA에서 SRT→VTT·덮어쓰기 거부·손실4개 명시 확인 후 변환과 이전 S1/S2 저장물·receipt3 복원을 확인했다. 전체9 GUI·새 편집/재채택·별도 재시작·전체 시각 일치로 확대하지 않는다. focus 수정은 [별도 SDK 검사](focus-sdk-gate-2026-10-19.md)와 [r169 실제 검토](../presentation/settings-focus-return-review-2026-10-19.md)를 따른다.

.gitignore는 새 3개 crate의 Cargo.toml/src/**/*.rs만 명시 허용했다. 실제 20개 파일의 공개 여부와 target/runtime model/.env/credentials/evidence 거부, 기존 Meari 160000 gitlink를 대조했다. stage/commit/push는 하지 않았다. 기존 examples ignore와 .gitattributes는 별도 문제이며 이번 허용 범위에 포함하지 않았다.

전체 compiler inputs나 loaded process memory attestation은 아니다. proposals/review/playback/catalogue 후속 추출과 tools 기능화·공개 API화는 별도 단계다. 다음 순서는 순수 분리 GUI baseline, focus 단독 검사·고정 bin 인계, projects의 실제 source/work snapshot 저장·CAS·재시작 구현이다. 후속 proposals 추출은 기능 검증 checkpoint에서 진행하며 기존 proposal receipt의 project_current_revision_verified=false를 유지한다.
