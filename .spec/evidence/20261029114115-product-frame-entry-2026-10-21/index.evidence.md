---
id: evidence:kinekino/20261029114115
title: 고정 제품 frame 진입 CPU 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 고정 제품 frame 진입 CPU 검증

2026-10-21. root `fd98ec9ca193703c52d23a26408b7cb2bc14ec0d`, Meari `0b79d57d7a59b3146b7ec72d02e2651b21b166fd`/Core Schema v4 위에 source14 `712BE58D1D4EC66447295E0B0C4A643010EDE1E44829056781C549C9E1C35FFD`만 조립했다. 부모 frame5와 root before14는 일치했고, 별도 미완성 Card 후속은 합성하지 않았다.

## 관측

`runtime/quality-review/ui-product-frame-entry-r1`에 원 입력·출력·receipt·BIN 사본을 보존했다. workspace fmt, default/native all-target strict Clippy가 성공했다. 새 Core와 기존 유효 typed props/ReactNode slot 소비가 실제 컴파일됐다. 최종 Native libtest를 한 번 생성해 실제 등록873 중 다음 최소 필터만 실행했다.

| 실제 범위 | pass | 별도 ignored |
| --- | ---: | ---: |
| 새 workspace frame 계약 | 3 | 0 |
| 새 실제 제품 source/profile 조립 | 2 | 0 |
| 제품 caller 판정·출력 보존 영향 | 10 | 0 |
| ViewerHost reflection·정리 영향 | 10 | 1 |
| 기존 폰트·자산·스타일 소비 | 2 | 0 |
| 기존 Standalone NoMedia 준비 | 8 | 2 |
| 기존 문서 생성·소유권·영상 선언 | 9 | 0 |
| 고유 합계 | **44** | **3** |

새5와 기존39는 중복이 없다. 전체873 및 OS/GUI/미디어는 실행하지 않았다. BIN SHA256 `A2044714514A8D84D1BFAEA892F85F6E62A04EAC597B11763F1586E27BC96B4C`는 사본과 일치했다. 실패·수리 없이 원 source14 bytes 그대로 통과했다.

실제 제품 시청 소스에서 Viewing/Studio/Card inventory가 함께 등록되고 원 client/host와 단일 비활성 video semantic ID가 유지되는 문서 생성을 확인했다. 원 SDK attachment가 붙은 상태의 같은-slot checked 재투영과 명시 release/reattach/worker join을 확인했다. 잘못된 구성은 export 전에 원 client/host를 반환했다.

작은 child fixture의 실제 NativeDocumentHandle에서는 다른 등록 reference로 keyed workspace를 교체하고 host/scope 유지·revision1 증가·old export 취소·stale stamp 거부를 확인했다. 또 실제 callback preflight의 미등록 binding 거부 뒤 원 program/stamp/export와 SDK attachment가 유지되고 기존 queued callback의 ticket/ACK가 작동하는 것을 확인했다. 이는 미리 주입한 성공/실패 closure를 actual renderer 결과로 세지 않은 근거다.

## 보호 및 한계

root1508/snapshot1513 SHA·UTC mtime 차이0, root/index 불변, 원 입력·부모/BIN 사본 불변, receipt12 모두 terminal/exit0, 기록 PID24 전부 부재였다. 원 conservative2/CPU2/mask3/BelowNormal/descendant Job·기존 cache/offline/locked/-j1/thread1을 유지했다. 원 handle 회수·보호 대조 후 heavy/shared를 문서 마감 전에 반환했다.

제품 --native-viewer caller가 새 profile로 연결된 코드는 실제 타입 검사됐지만 정상 player.exe의 창 실행·사용자 종료를 검증한 것은 아니다. 기존 ViewerHost 시험은 레거시 영향 검증이며 새 frame의 실제 LocalHost/Native loop 소비를 증명하지 않는다. callback preflight 거부는 post-patch renderer recovery가 아니다. 현재 profile에 없는 controlled Dialog를 인위적으로 추가해 recovery 성공을 만들지 않았다.

사용자 Card 선택→원 selection identity→초기 inspect→scoped Card session/admission→수정/save/명시 accept/재열기와 active Card close ledger의 제품 연결, 실제 미디어·caption·전체13화면/탭 시각 검토는 미완료다. [진입 계획](../../../plans/player/presentation/proposal-product-entry.md)을 계속 따른다.
