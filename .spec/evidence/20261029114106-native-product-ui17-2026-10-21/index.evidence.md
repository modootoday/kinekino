---
id: evidence:kinekino/20261029114106
title: Native UI17 제품 조립 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native UI17 제품 조립 검증

2026-10-21 관측. root `062333dc6925b2ca32fe0b3420c780edd2381092`, Meari `2e0bf259d82937b612d4753cbce0cd97569699e4`의 actual430과 UI17을 별도 snapshot에 조립했다. root 채택·기본 화면 전환·전체 제품 완료는 별개다.

## 입력과 실제 수리

원 UI17 `3CDF8902C2B69E2B756B4017B2EB4941032F231B2C7AFFE9D7783757E9D35A98`의 첫 workspace fmt는 `presentation/interaction.rs does not exist`로 실패했다. `#[path]`로 로드되는 chrome.rs 및 하위 interaction.rs에 실제 디렉터리를 명시하는 경로2만 수리했다. 원 입력·실패 receipt/log를 보존했다. 직접 파일 fmt가 실제 module discovery까지 검증하지 못한 사례다.

최종 revision-r2 source17은 `8279976C4F86A937545867ADB6834B1A68614A8C7278C8461A25303A152335C3`, 조립1484 manifest는 `F57E59746F7E788B556CCB58E3DF9E42490058B69DE24538A6C7DC0E86F1F159`다. route2/Caption 소스는 합성하지 않았다.

## CPU 관측

`runtime/quality-review/ui-native-product-union17-r1/revision-r2`에 원 출력/receipt/소스를 보존했다. workspace fmt, default/native all-target Clippy `-D warnings`, Native libtest 단일 no-run 및 list가 성공했다. 실제 등록859 중 지정 disjoint9 필터의 **고유 regular95가 모두 통과**, ignored7은 CPU 단계 미실행이다. 고정 예상 명단의 누락·예상 외 통과·중복은0이다. 전체859 시험을 실행했다는 뜻이 아니다.

최종 BIN SHA256은 `478656B6FCDFCB5D87F3F9010697444E9207CEFEA6F19190D7DFDB686FFD1E41`이며 artifact 사본과 일치한다. 최종 receipt14 모두 terminal/exit0, 최초 fmt 실패1은 별도다. 원 conservative2/CPU2/affinity3/BelowNormal/Job/offline/locked/-j1/thread1을 유지했다.

## 실제 Viewer NoMedia 관측

CPU 반환 후 PM 별도 OS 배정으로 **동일 BIN을 재빌드 없이** 두 exact에 한 번씩 사용했다. `runtime/quality-review/ui-native-product-os2-r1`에 outer receipt와 closeout이 있다.

| 시나리오 | 실제 근거 | 결과 |
| --- | --- | --- |
| 정상 | 원 bind→pending reflection→실제 ProofObserved→Completion에서 ticket91 observed_disabled 1회→CloseIntent→InnerClose Complete/remaining0·owner1/epoch1/surface2→원 runtime 반환 | Closed, same_runtime=true, WorkersQuiescent(Closed), probe fault 없음 |
| bind 전 문서 변경 | 원 pending document revision1→2→`original frame request document changed before bind` 거절→원 preparation remaining0/retained stamp doc2→실패 반환 후 원 정리 | run_ok=false/lifecycle=None, WorkersQuiescent(Unconfirmed), 예상 거절 시나리오 통과 |

정상 family43728/creation134370197935375592, 변경 family51628/creation134370198119026428 모두 exit0·Empty·forced=false·stdin EOF=true다. outer 각각3146ms/2655ms exit0이며 recorded PID6은 최종 부재를 확인했다. 예상 거절을 창 Closed 성공으로 합치지 않는다.

원 inner 자료는 각각 `runtime/quality-review/ui-viewer-no-media-process/99940-1792546193534154300`과 `73156-1792546211900027000`이다. 예산20+2초/raw256KiB/trace80은 불변이다. 자동 재시도·사용자 앱 종료·모니터/VM/DPI 설정 변경은 없다. PM이 전달한 이후 사용자 메인 모니터 검증 허용 지시를 적용했다.

## 보호 및 남은 범위

CPU/OS 반환 시 root1476과 snapshot1484의 SHA 및 UTC mtime 차이0, root/index와 BIN이 유지되었다. 최초 보호 집계의 문자열 대 DateTime 비교 오류는 원 초안을 보존하고 UTC ticks로 정정했다. 실제 소스 차이로 오인해 복구하거나 시험을 반복하지 않았다. heavy/shared는 문서 마감 전에 명시 반환했다.

실제 NoMedia 준비·원 Completion·child-first close·실패 owner 보존의 관측이다. 정상 player caller의 사용자 종료, 실제 영상/커서/창 caption, 전체13화면·모든 탭 시각 일치, 사람 품질 승인은 미완료다. 제안 편집 제품 route는 [별도 계획](../../../plans/player/presentation/proposal-product-entry.md)과 source2 후보로 계속한다.
