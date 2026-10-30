---
id: evidence:kinekino/20261029114554
title: Native guarded owner/read-context union23 Rust gate · 2026-10-21
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native guarded owner/read-context union23 Rust gate · 2026-10-21

root56d9e87d8fcc1209798a5790c70f72493f81e626와 Meari98f6a5c4c72ad07a40ac0eae7f7f113926609fae, owned-process19 r4 및 검증 Cargo/lock이 합류한 반환 snapshot에서 B7 r2+C8을 검증했다. 실제 mpv/media/browser/GUI·production issuer 활성화·CLI 결과 wire·skip 소비 구현0이다. 근거는 `runtime/quality-review/sdk-guarded-union23-r1/`다.

## 입력과 최소 수리

| 산출물 | SHA256 |
|---|---|
| 원 C8 union23 | E7DCCD4819A70E2FB6EC076AE9A6F1307226CC4A4437C11A4AE241960417D031 |
| 최종 r2 union23 | 96BF4421451E4203377DC660C38BAA826FC3A5588934050C67008D43F8180351 |
| r1→r2 delta1 | BD8C65C3893E269E8137CED86C1544D234744A147468E98EB3828584262C2490 |
| integrated23-r2 | EE09974E3A8E8D86FCDFB4D7BAF794EEB7016C5D9EFC420E18CCC0C6EA7D3FA0 |

원 A11 r3/B7 r2/C8 pin은 보존했다. B7 원 후보에 A11 r3 queue/tests 정확2를 정규화한 base17과 C8 source8의 union23이며, 실제 적용은 **동일7 보호/기존 변경15/신규1**이다. before23 기존22/부재1, payload16 기존15/부재1을 분리했다. producer 반환1366 대조0 후 additive patch check/apply0, payload16만 frozen bytes/현재 mtime으로 합류했다. 후보의 옛 Cargo/lock은 복사하지 않았다.

첫 실제 compile은173189ms0/guarded35pass였다. B7 lifecycle 첫 실행은2pass/11fail,69ms101이며 모두 fixture에서 생성하지 않은 catalog 하위 디렉토리를 CatalogFiles::open해 NotFound가 난 동일 원인이었다. 원 소스/로그/receipt를 보존하고 lifecycle_tests.rs 한 곳에서 create_dir 후 open하도록 r2로 분리했다. 제품 정책/권한/assertion 완화0, 다른22 byte 불변이다. 보정 뒤 default 실제 재컴파일32151ms0, native 실제 컴파일239349ms0을 확인했다.

## 실제 결과

최종 r2 실제 시험명은 **default119/native119, 중복을 제외한 고유119**다. 이119 안에 bounded Rust child helper2가 포함돼 **행동 단언 시험명117**이다. 119+helper2 또는238개의 고유 성공이라고 보고하지 않는다. 최초 실패 실행과 수리 전 성공 반복도 최종 고유 수에 더하지 않는다.

| 양 구성에서 실제 선택한 범위 | 구성당 수 |
|---|---:|
| guarded protocol/fence | 35 |
| B7 lifecycle | 13(helper1 포함) |
| application guarded | 7 |
| parent binding | 14 |
| parent router | 15 |
| status projection | 3 |
| native control 전체 | 7(기본5/옵션1/mute-wire1) |
| receive observation | 9 |
| 기존 playback session SDK | 3 |
| owned startup failure/recovery | 2 |
| replacement/EOF/recovery | 10(helper1 포함) |
| 닫힌 IPC worker join | 1 |

B7 신규 행동 단언12와 C8 신규12가 실제 통과했다. 기존 관련 회귀 및 helper는 별도로 구별한다. 작성 예상118과의 차이는 전체 control 필터의 mute-wire1이며 실제 이름으로119로 정정했다. 0selected/목록 성공0, 전체 suite 반복은 이번 gate 범위가 아니다.

default/native all-target strict Clippy는 실제 Checking 뒤 각각26743/62966ms0, cargo fmt --all -- --check3214ms0이다. Rust child/worker와 synthetic protocol driver는 실제 mpv·sampled media·landing/품질 증거가 아니다.

## 보호·실행 소유·반환

root 비입력1391와 원 입력16, snapshot 비입력1351(동일7 포함), 최종 source/integrated23·producer19·바이너리 hash delta0을 운영 반환 전에 확인했다. Cargo.toml SHA3531B4ADBC62525CB04B4FD46E8B364099F33FD819769EA1349AE7CD017C1BD0, Cargo.lock SHA6209C8C556B4508F98D4F87656336CC10D35AA3DA5759CEF76570A92E1BE93CF를 root/snapshot에서 보존했다. UI13/자막8/CLI11/harness6/Meari405도 비입력 보호에 포함된다.

fixed runner EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D/conservative2/affinity3/BelowNormal/descendant Job, 절대 manifest/기존 native target/snapshot cwd/offline/locked/-j1/thread1을 사용했다. 원 handle terminal 회수 후 다음 own 실행을 시작했다. 32receipt 모두 terminal=31zero+원 fixture101 하나 보존, 기록64PID 부재, own/build/evidence process0, index empty다. 운영 슬롯은 상세 문서보다 먼저 PM에 명시 반환했다.

default libtest94990848bytes/SHA35F201873E37AE2E6D14AEF1CFD0AC0959BCDBBD0088156303739BC10B422FEB, native156502528bytes/SHAB856DDE97DF05390CD8C403162CFABB94D1AD508CA3F183144D6386ACD821C1C다. 원 r1 바이너리 식별도 별도로 보존했다. 이번 gate의 CLI executable 실행0이며 과거 CLI 캐시를 새 실행 근거로 사용하지 않았다.

## 계약 판정과 미완료

원 logical scope 전달, foreign host guard/result 거절, own closed submit 거절과 own closed read 보존, Active 해제 후 Observe wait, cleanup/permit drain, prepare 전 원 word 고정/잠금 없는 revoke, 원 admission/source/receipt를 유지하는 clone 공유 최초1회 context, 과거 binding/router read·foreign/expired owner·단일 consume·64slot 예산을 Rust seam에서 검증했다. read-context는 idle Weak owner만 보유하며 raw native read의 decorated-context 우회도 거절한다.

NativeEpisodeEngine::new는 여전히 disabled이고 CLI skip registry false/기존 port Unknown·general control이 남아 있다. 다음은 Trusted standalone owner, 같은 view application 소비, bounded CLI ticket wire의 별도 세 책임이며 단순 supported=true로 바꾸지 않는다. GUI parent origin/focus/actual surface와 실제 media/manual skip·Auto/AI는 이 gate로 지원되지 않는다.

다음 순수 SDK gate는 default canonical 실제 시험과 native all-target 타입 검사를 구분해 비용을 줄일 수 있다. native feature·OS 기능 gate는 별도 유지하고, 이번 이미 실행한 양 구성 결과와 다음 타입 검사 결과를 혼동하지 않는다. 중앙 copy/commit은 PM 소유다.
