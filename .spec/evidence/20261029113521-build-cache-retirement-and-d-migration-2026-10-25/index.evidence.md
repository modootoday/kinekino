---
id: evidence:kinekino/20261029113521
title: 빌드 캐시 폐기와 D 이전 — 2026-10-25
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 빌드 캐시 폐기와 D 이전 — 2026-10-25

최신 사용자 지시가 C 우선 정책을 대체한다. 새 빌드 target은 `D:/workspace/@myso-kr/com.kinekino.player/runtime/build-cache/native`, TMP/TEMP는 `D:/workspace/@myso-kr/com.kinekino.player/runtime/build-cache/tmp`다. 전역 Rustup/CargoHome와 다른 프로젝트 환경은 변경하지 않는다. Root 정책 문서 commit은 d13cbfca140e41863b4d0bd27930be3f57a1123f이며 parent77fd의 Rust/TSX 바이트는 바뀌지 않았다. 기존 AI품질승인·appmod WIP는 그대로다.

## 실제 정리와 현재 보호

- PM incremental 정리:127,340파일/112,294,212,136 logicalB, 실제 free 증가99,412,389,888B(92.58GiB). [원 receipt](../../../evidence/environment/build-cache-incremental-cleanup-2026-10-25.json).
- SDK가 정확173 과거 gate EXE/PDB 목록과 필수5EXE 제외를 고정했고, PM이 사용자 명시 Python 예외로 삭제했다. logical72,246,441,984B, 실제 free 증가67,595,862,016B(62.95GiB). [원 receipt](../../../evidence/environment/build-cache-snapshot-cleanup-2026-10-25.json).
- 두 실제회수 합계167,008,251,904B(155.54GiB). 논리합계와 실제 free는 hardlink/압축/동시디스크사용으로 다를 수 있다. 이전 압축12개를 다시 별도 free로 중복 합산하지 않는다.
- PowerShell incremental 재귀삭제와 SDK173개 exact파일삭제 시도는 자동 승인 검토의 blocked by policy로 실행 전 거부됐다. SDK는 다른 도구로 우회하지 않았다. 이후 사용자 명시 Python 요청에 따라 PM이 일회 실행한 사실을 별도로 기록한다. 새 정리 개발도구는 작성하지 않았다.

원173 부재와 남은333의 bytes/mtime100ns/FileID를 SDK가 모두 대조했다. `runtime/verification/cache-custody-d-migration-r1/00-active-protection333-pre-migration.json` SHA `1BABADCBCE1C364B2C436E07D26DE6FEA9B66C3D34D5EC19F6BD9FB61AE5BB00`가 폐기 후 현재 권위다. C구본은 덮지 않았고 D메타 사본의SHA도 동일하다. 이333은 아직 저장위치 이전 전의 원identity다. PM snapshot preflight 최초mtime환산오류는 FILETIME1601과.NET0001 offset 혼동으로 삭제0 후 정확한621355968000000000 offset으로 전173 원100ns일치를 확인한 뒤 수행했다. 허용범위 완화는 없다.

폐기된과거 바이너리는 현재재실행가능 snapshot이 아니다. 당시 source·CargoJSON·원시험로그·해시는 역사 관측으로 유지하며 통과 이력을 취소하지 않는다. 현재실행파일 또는 새source의성공으로 이전하지 않는다. 보존은 Media 원CLI7AA/Media19pass 독립EXE, Meari r7 range/r8·r9 source_callbacks/r10 OS원실패EXE, R3원실패EXE, actual9/미디어/모델/역사verifier/mpv와 원소스·로그·manifest다. 추가PDB 영구보존은 필수가 아니다.

## 물리 이전 소유권과 검증 경계

PM이 C project-9dc296c4/native→D runtime/build-cache/native의 단일 robocopy를 소유한다. SDK는 이 두 경로의 복사·삭제·빌드를 병행하지 않는다. 현재 복사본은 완료 및 검증 전 실행 가능한 cache로 계상하지 않는다. 원C는 복사 검증 전 보존한다. COPY:DAT/DCOPY:DAT·단일IO·BelowNormal, noCargo 상태와 원 [복사 receipt](../../../evidence/environment/native-cache-copy-2026-10-25.json)를 따르며 실제 결과는 그 완료 후 기록한다.

SDK 후속 검증은 기존333 중 Cnative 직접파일26의 새Dpath/bytes/mtime/FileID/원내용SHA 및 실제 볼륨을 새epoch에 연결한다. 원FI는덮지않고 old→new를기록한다. 현재C의debug/player.exe는hardlink로관측됐으므로단순copy후동일FI관계를가정하지않는다. logicalcopy/실저장량/기존alias관계를분리하며옛C가그대로200GB남은상태를이전완료로표시하지않는다. 구Cprefix호환junction은별도경로검증과실D저장확인뒤에만가능하다.

나머지소스mirror는D runtime/build-cache/source-mirrors/public-p3-parent-r1를제안하며본체Rust캐시와달리읽기소스/원preimage를보존한다. 기존Csource path는역사경로라정리중임의실행하지않는다. source의절대경로가달라지면projectpackageID/생성입력변경에따른정상재빌드를허용하고지문위장/mtime강제/이전성공강제재사용0. actual9/미디어/브라우저프로필/evidence는빌드잔여물로삭제하지않으며원파일identity를훼손하는일괄이동도별도검토없이하지않는다. 이미D에있는333 항목266개는중복복사하지않는다.

## 다음 동일입력 검증 준비

신규D gate `runtime/verification/product-meari-os-joint375-d-r5-gate`의source375 SHA `A09B51ACBC77645FD12D8488B23021173780851E267A8BA5103D34F827294D39`는원41DB+UIcatalog source1 C445C58B4BFFAB71F2DE94FF7D091D6D16E9359E2222437AF6BAE24DF36C2728다. 수동기술5버튼제거외auto progression/typedprops/SDK registry·원assert/190082C계획은불변이다. D target/tmp를명시하고RootHEADd13·새WIP/보호후계·fontclosure7stage/5상속·원소스preimage를재대조한다. 실제Cargoproducer/list190/정상bins는PM복사완료·원검증반환뒤배정이며아직0이다.

`.cargo/config.toml`뿐아니라runner의 CARGO_TARGET_DIR·명시--target-dir·TMP/TEMP 및Meari BuildRequest.target_dir도D로통일해야한다. Cargo밖에서실행하는libtest/GUIworker도부모process의D TMP/TEMP를상속시킨다. 옛Cgate스크립트는증거전용이며재실행하지않는다. 모든시험은원CargoJSON/직접EXE/PDB 그룹전후hash·매실행metadata·단독writer를유지하고, 실패/부분결과를성공cache로취급하지않는다.

## 적용 스킬

## 09:16 UTC 후속 관측과 원격 범위 수정

PM 복사는 robocopy exit1로 완료됐다. SDK가 보호26개 총7,528,194,853B의 D 실제 SHA256/크기/mtime을 원333 권위와 대조해 26/26 일치를 확인했다. C 원본의 크기/mtime/FileID도 원기록과 같았으며, D 검증 후 메타데이터를 다시 확인했다. 새 볼륨 identity는 `runtime/verification/cache-custody-d-migration-r1/05-volumes.json`, old→new26은 `02-native26-d-verified.json`에 보존했다. 후계333 `03-active-protection333-d-epoch.json` SHA는 `9C66DE1D0DCDC50F6E3FD78EB070AC1DB10DE1A912CE208C7878F2A3191C0A5C`다. 나머지307행은 그대로 유지했다. C 원본 삭제·junction·추가복사·Cargo는 수행하지 않았고 hardlink topology 보존은 입증하지 않았다.

최신 사용자 결정에 따라 기존 빌드 캐시는 build-host으로 전송하지 않는다. 대상 `<원 호스트 저장소>`에서 새 빌드하며 로컬26 검증은 원격 선행 gate가 아니다. [SDK 원격 인계](../../../plans/player/remote-sdk-handoff-2026-10-25.md)에 전송 제외/별도 필수자료/소스375 및 exact190/재현 순서와 미검증 범위를 기록했다. 현재 PC WARP는 변경하지 않는다. 이전 절의 로컬 D target은 현재 PC 정책이며 대상 PC의 C root와 구분한다.

### 스킬 적용 기록

modoo.today 검색후 `modootoday-free/build-performance-plus/build-cache-correctness-audit` 본문을SDK가직접읽었다. version0.1.0/SKILL.md4809B/SHA `598e01f3a75a076c5f8b40994fa1450fe5b9bcf0633c356ec696f9806592818b`, 추가필수참조없음. 적용: producer/consumer·source/toolchain/env/생성입력 경계를유지하고,폐기·이전·실제복원검증과역사성공을구분한다. 이번삭제는사용자스토리지정리이며캐시문제진단을위한공유캐시삭제대조시험이아니다. 로컬복사완료만으로원격복원·Web/OS·제품품질성공을주장하지않는다. 새source필수checks와수정후실제재빌드를건너뛰지않는다.
