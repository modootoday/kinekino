---
id: evidence:kinekino/20261029114228
title: 수동 OP/ED skip 코어 실제 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 수동 OP/ED skip 코어 실제 검증

2026-10-20. 증거: `runtime/verification/manual-skip-core-r2/`. [코어 계약](../../player/application/manual-segment-skip.md). 이 검증은 코어와 기존 control/status 회귀이며 host17·SDK/CLI skip 지원 검증이 아니다.

## 입력과 최소 수리

PM이 PNG gate 반환 후 public-p3-parent-r1 snapshot 및 단일 heavy를 배정했다. 최신 PNG/SDK/Meari 입력 위에 frozen r2 신규9를 정확히 적용하고 domain/application mod2에 skip_segments 등록만 더했다. PNG snapshot_export 등록은 보존했다. 적용 전 신규9 absent·등록2 원문을 before/와 before11.json에 보존했다. integrated11.json SHA E278957140A4D6F0850487C937DA07C99FC660137E3360AA26CD7ACF4A02B2ED, frozen r2 AAB8F0753960A38E20C9A8A97C4D051F1C33F623113A2B90B1AAB943B88C86A7은 그대로 남겼다.

첫 코어15와 기존 control4/status3이 통과한 뒤 default strict가 policy.rs 중첩 if의 collapsible_if 경고1건으로101/9780ms 종료했다. 원 stderr/receipt를 보존하고 allow/expect 없이 동등한 if-let 체인으로만 바꿨다. r3는 frozen r2 전체를 복사한 뒤 policy.rs만 교체했으며 작업 중 host17·응답 Deserialize 변경을 섞지 않았다.

최종 frozen `runtime/coordination/manual-segment-skip-r3/source11.json` SHA **9C1BE6D389B41E9E23D089E49058F88CC6A383A068AC53C8A301641069EF15FE**, repair.diff SHA97D23C1671C11EE16FAEE6DFABE3803C8DECB208D8E34F2F43607FB7E8C29F73. 등록2는 PNG를 보존한 integrated11-r3.json으로 구분하며 SHA **35A0605279B8FB27EA35224B3A4D7C0406F7F6CECC7DED1404EF2B6842D57AC5**다.

## 실제 결과

| 검사 | 결과 | 원 receipt 시간 |
| --- | --- | ---: |
| r2 코어 | 15 passed,0 failed/ignored,417 filtered | 65382ms |
| 기존 playback_control | 4 passed | control 묶음93663ms |
| 기존 playback_status | 3 passed | 같은 묶음 |
| 첫 default strict | 101, 위 Clippy1건 | 9780ms |
| r3 default all-target strict | 0 | 40105ms |
| r3 native feature all-target strict | 0 | 46290ms |
| r3 코어 재확인 | 15 passed,0 failed/ignored,417 filtered | 26279ms |
| Cargo fmt check | 0 | 1502ms |

코어 domain4/application11=15의 실제 실행을 확인했다. r2/r3 반복을 별개30개로 합산하지 않는다. 기존 회귀7은 정책이 동등한 Clippy 수리 전 r2에서 실행됐고, 최종 r3 코어15/양 strict/fmt와 구분한다. 신규 flag 취소 회귀는 accepted/deferred/error 뒤 TooLateOrUnobserved·추가 observe/submit0을 확인한다. revision 정확히+1 및 stale flag 취소 상태 불변은 그 회귀의 직접 assert 밖이라는 읽기 검토 한계를 유지한다. 실행15가 제품 큐 제거를 입증하지 않는다.

기존 instance-checked seek/ControlReceipt를 재사용하며 Native 어댑터의 seekUnknown/미인가 관측 차단이 남아 있다. fake 포트의 call0/1, 한 pass/역탐색/명시 새 pass·override·취소/접수·불확정 오류 구분을 시험했으며 실제 mpv seek/landing, 새 CLI 명령, durable 저장/재개, host tick/자동 타이머 또는 AI 검출은 검증하지 않았다.

## 소유권·보호

지정 fixed runner SHA EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D를 직접 `--profile conservative2`로 사용했다. 모든 receipt의 admission accepted, mask3/논리CPU2/BelowNormal16384/descendants=true, 기존 native cache/offline/locked/-j1/testthreads1이다. build4나 -j2를 사용하지 않았다.

최종 integrated source11 차이0·보호609 차이0이다. 보호에는 최신 PNG, SDK58/producer394, 자막/기존 공통 소스가 포함됐다. host17 경로는 snapshot에 없고, 실제 코어 contracts는 Serialize-only r3로 유지됐다. `final-source11.json`, `final-source-differences.json=[]`, `final-protected-differences.json=[]`, `final-receipts.json`, `final-owned.json=[]`에 남겼다. 원 실패 포함 receipt7 모두 terminal, launcher/child14 PID 부재다. 실제 최종 root lib test binary 경로/해시는 final-test-binary.json에 기록했다.

heavy와 snapshot은 PM에 명시 반환했다. snapshot은 PNG+core r3를 유지하며 rootlive/index/stage/commit은 이 세션이 바꾸지 않았다. 제품 코어 통합은 PM 후속이고 host17는 별도 source-only 핀·읽기 검토 대상이다. 모델/GUI/추가 native OS 실행은 없었다.

스킬 적용: modoo.today requirement-to-testable-stories로 단순 DTO 미러 대신 실패·경계·seek 호출수·상태 전이를 시험했다. code-change-review-verification의 최초 실패 보존, 동등 수리의 별도 핀, 소스·실행·제품 지원의 분리를 적용했다. Rust·SRP와 중앙 자원 배정을 유지했다.
