---
id: evidence:kinekino/20261029114535
title: Routed native export B11 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Routed native export B11 검증

2026-10-21. root `a2d10fba0a89399768bd83c3456fe5ab6f95e16a`와 UI 최종r4/Meari412 baseline에 source11 `7212FA808837E363521134F80EB0F906FA71980DD1159D11355116075468DC06`을 합류했다. 수리/retry0이며 상세는 `runtime/quality-review/sdk-row-export-routed-r1/handoff.md`·원 로그/receipts/actual-tests/closeout에 있다.

Native 실제 고유184=기존174+신규10, helper2 포함/행동182가 통과했다. default runtime0이며 default/Native all-target strict와 workspace fmt도 통과했다. 첫 compile122189ms, strict default48848/native30242ms, fmt3180ms다. 같은 fresh Native binary와 snapshot cwd를 사용했다. 이전 gate 성공을 새 소스에 승계하거나 broad 필터를 중복 가산하지 않았다.

최종 binary SHA `A76A44C74224465B1C0D20C97632F23E7A9680B6B32945320105916FD96E830A`,159327232bytes. 원 B11/before·A9r2/row7 bundles·비입력 root1421/snapshot1381·UI13/library412/Cargo/lock·binary 보호 차이0,19 receipt 전 terminal/exit0·38기록PID 부재·본인handle0/index0 확인 직후 heavy/shared를 PM에 명시 반환했다. SDK가 root source/index/commit을 변경하지 않았다.

원 issuer/Weak binding/router/epoch token을 같은 원 read cell에 enqueue 전에1회 결속하고 postbind0·generic doublebind거절·원64/consume·resolver unlock 뒤 원 context read·close/foreign/expiry/Weak 수명 경계를 Rust로 확인했다. 실제 mpv/ACK/barrier·OS parent/GUI·미디어/CLI 성공은 아니다. 고정 binding list와 router 전체 list Unsupported를 구분한다. C0 공식 issuer/ref·metadata/target·bounded wire와 지원true는 [phase2 후속](../../../plans/player/native-result-export-phase2.md)으로 남긴다.
