---
id: evidence:kinekino/20261029114231
title: C0 선행 소비 exact9 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# C0 선행 소비 exact9 검증

2026-10-21 PM 독립감사·단일 heavy/shared gate 배정으로 수행했다. base rootfaa4e96의 before9 기존8/신규 tests/native_skip_cli.rs1 부재를 유지하고, root3032의 SDK P0 r3 계약을 의존했다. 최종 overlay는 **runtime/development/native-export-consumer-r2**, manifest `runtime/coordination/native-export-consumer-r1/revision-r2/source9.json`, SHA256 **5DCE6851AEED982EE3FA5938CEA42725E8FFA30DC521077585FA8C9F0E319901**다. r1/원 source9·before9·수리 실패를 별도 보존했다.

## 실제 결과와 원 실행

| 구성·필터 | 실제 고유 시험 |
|---|---:|
| Native application::skip_segments broad | 31 = 기존26+신규5 |
| 같은 Native libtest의 native_result_export broad | 18 = 기존 SDK P0 영향 |
| Native integration native_skip_cli | 6 = 신규 strict DTO/client/지원 차단 |
| 합계 | 55, 신규11 |

모두 최종 r2 source에 대한 원 실행이며 repeated/subfilter 중복 합산0이다. Native library49는 같은 fresh binary·같은 snapshot cwd를 사용했고 integration6은 같은 최종 pin/구성으로 별도 빌드했다. 실제 선택 이름은 runtime/verification/native-export-consumer-r1/actual-tests.json과 원 stdout에 있다. 시험 파일명이 native_skip_cli여도 실제 player-cli.exe/OS child/브라우저·매체를 실행한 시험이 아니다. default runtime0이며 default/native all-target strict -D warnings와 workspace fmt도 통과했다.

원7receipt 모두 terminal, 6 exit0 + 원 default strict101 하나를 보존했다. 원 r1에서 service.rs의 cfg(test) 모듈 뒤 production item이 있어 items_after_test_module이 발생했다. r2는 같은 모듈/단언을 파일 끝으로 옮긴 기계수리1/불변8이다. production/getter/DTO/권한 동작 변경0·broad allow0이며 r2 독립감사와 양 strict/fmt 뒤 같은 최종 pin 시험을 수행했다. 원 exec terminal/receipt를 회수한 뒤 다음 실행으로 진행했다.

Native libtest는162947584 bytes/SHA256 **F502D77E148FE63B4F0D42FB96315463162FB12C5FFAFE1A22DA51FD8478B9B3**다. 원 hash 최종 불변 및 artifacts/native-final.exe 사본을 보존했다. integration 사본과 hash도 artifacts/integration-final.exe 및 integration-binary-sha.json에 있다. 기존 resource-runner EEA9…88D/conservative2/2CPU·기존 snapshot public-p3-parent-r1/target native·offline/locked/-j1/thread1을 유지했다.

## 검증된 소비 경계

공식 SDK v1 codec/Origin/Annotation/preparation·고정 issuer/list/read·원 Exported::into_parts·CoreError/compact encoder를 재사용했다. 독자 production codec/ref/결과 map/권한 mint/legacy fallback0이다. Getter는 binding과 revision/pass scalar·선택1 annotation만 읽고 with_session의 inspect().binding 전체 clone을 제거했다. 기존 evaluate/예약/취소/override 및 current_required/동일 session 잠금 의미를 유지했다. submit 뒤는 state만 읽으며 과거 annotation을 재구성하지 않는다.

원 SDK Ok/ref가 있는 synthetic post-admit core 오류에서 admitted_with_error/ref/원 annotation·target을 보존했다. no_submission_returned는 미접수/seek0 보장이 아니며 취소에 reference/target을 만들지 않는다. 조회는 현재 session entry/revision/route를 선택하지 않고 원 issuer에 origin을 전달한다. foreign ref는 source 조회 전에 거절되고 원 SourceExpired를 전달한다. 실제 P0 default Unsupported에서 legacy observe/submit0와 revision 불변을 확인했으며 fake admission 성공으로 대체하지 않았다.

strict DTO/공식 codec→context→availability 순서를 새4 dispatch에 적용하고 legacy9 순서는 유지했다. client는 같은 command constants/context/원 오류를 전달하며 자동 retry0이다. 유한 SDK error/data를 bounded bytes로 인코딩한 뒤만 Value로 변환하고 전체 segments/무제한 details/remediation을 새 결과에 복사하지 않는다.

## 지원 한계·남은 작업

**wire 지원은 false**다. registry export_enabled는 private false, schema도 false이며 전역 COMMANDS·CLI 조립/전체 transport 출력은 연결하지 않았다. 이는 dispatch 지원을 차단한다는 뜻이며 신뢰된 Rust typed 경계까지 모두 차단됐다는 뜻이 아니다. typed 경계는 실제 코드로 구현했고 production producer 미구현을 dummy 성공으로 대체하지 않았다.

실제 SDK P1 row/actual SubmitInput target stamp·P2 C8 forwarding·Weak lifetime/consume/응답 유실 복구·원 production Observe→submit positive는 이 소비 gate의 성공 범위가 아니다. post-admit projection/issuer는 crate-private synthetic seam이며 실제 mpv/ACK/landing 검증이 아니다. 안전한 wire activation은 별도 registry 작은 delta와 CLI2/공유출력6 인수를 요구한다.98299/98304/item1024×64의 P0 계약/기존 max-bound 시험과 최종 CLI compact/pretty/envelope/client allocation 검증을 구분한다.

## 보호와 반환

실행 전 SDK P0 protected-snapshot→integrated5-r3 path-key1400(Meari412 포함)와 before9를 대조해 차이0 후 exact9만 적용했다. 최종 copied9 bytes/hash/UTC mtime를 integrated9-r2.json에 기록했다. snapshot의 기존 입력8은 선언 delta로 보호 제외하고 final9는 별도 overlay/snapshot 대조했다. tracked root1028과 SDK P0 최종 전체 root1440(미추적 Rust/source·library 포함)을 별도로 보호했고 P0 원r1/r2/r3 bundle도 보호했다. 원 before9·최종 소스·비입력 root/snapshot·SDK/Meari/Cargo/lock 대조 차이0, index0이다. P1/UI 미완성 overlay 합성0이다.

원7실행 기록14PID의 현재 부재 및 모든 원 handle terminal을 확인했다. PID 숫자 재사용은 원 executable/creation과 구분하며 다른 프로세스 부재를 새 반환 조건으로 삼지 않는다. kill0이다. 상세 문서 작성 전에 PM에 heavy/shared를 명시 반환했고 이후 추가 Cargo/shared 실행0이다. 실행 정본은 runtime/verification/native-export-consumer-r1/{closeout.json,receipts.json,actual-tests.json,protection-differences.json,final-source9.json,handoff.md}다. root 채택/stage/commit은 PM 책임이며 push0이다.

적용 근거는 modoo.today plan/plan-authoring·code-change-review-verification/verdict-table의 본문을 재사용한 정확 입력/의존·권한/원 오류·관측/설계/실행·수리/반환 구분이다. [소비 계획](../../../plans/player/native-result-export-consumer-c0-review.md)과 공식 C0 정본을 따른다. 전체 담당 goal 및 실제 CLI/매체 통합은 진행 중이다.
