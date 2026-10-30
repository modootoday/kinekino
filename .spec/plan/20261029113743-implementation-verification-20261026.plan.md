---
id: plan:kinekino/20261029113743
title: 구현·검증 goal — 새 호스트 source375
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 구현·검증 goal — 새 호스트 source375

2026-10-26후속필수의존성예외: PCM exact재현에필요한역사FFmpeg6.1을사용자의최소의존성설치승인으로project-local격리확보했다. 원exe714147…pin일치/라이선스·URL·archiveSHA와실제재현은 [영수증](../../evidence/environment/ffmpeg61-replay-dependency-20261026.md)을따른다. 이는모델/미디어다운로드·추론/GUI/WARP/push보류해제가아니며아래이전crates-only실행범위와구분한다.

2026-10-26 사용자 요청으로 active goal 등록. 중앙 PM이 B0 입력/custody/preimage 고정부터 source375 SDK/CLI→전체190→정상 player/player-cli bins 순으로 진행한다. 설계 완료는 개발 완료가 아니며 실제 AI 결과 검토/수정/판정/이력·사용자 품질 승인과 승인시안/실UI 요소별 시각 비교도 전체 이행 범위에 유지한다. 현재 goal의 SDK/190/bins는 그 선행 검증 단계다.

## 최신 권한·불변 경계

사용자 후속 승인: 필수 의존성이 필요하면 최소 설치를 진행한다. 이번에는 현재 Windows Rust build/test graph에 필요한 crates/필수 toolchain component의 부족분만 대상으로 하며 모델/AI미디어/추론자료 다운로드·WARP 변경·수동 GUI·push 보류는 해제하지 않는다. lock/toolchain을 임의 최신화하거나 전역 Cargo/Rustup 설정·타 프로젝트를 변경하지 않는다. 실제 설치 대상과 결과를 별도 기록한다. 이미 있는 MSVC/Windows SDK는 재설치하지 않는다.

최신 AGENTS의 C NVMe cache, heavy1/jobs1/conservative2, root index PM 직렬 소유·독립Meari 소유, Rust-only·원본/WIP 보존을 적용한다. 공유 canonical에 무조건 overlay하지 않고 fresh private source stage를 작성한다. 기존 원소스/원증거/manifest/FileID는 보존한다. 선택375와 canonical HEAD 빌드를 혼동하지 않는다. 실제 첫 실패와 미실행을 분리하고 assertion/guard/목록을 약화하지 않는다.

## B0 진행

최신B2/B3반환: private후계r9 final50~53 전체190/190·실제등록/ignored·producer무결성통과,normalbins56 player/player-cli2개빌드exit0(freshtrue)·CLI기본version/capabilities실행oktrue. root/Meari fmt/type/strict범위와원375/각후계실패를보존한다. sourcepin·원producer/정상bin출처·남은구현은 [실제190 정본](../../sots/verification/application/source375-datalab-first190-20261026.md) 및B2-B3-RETURN-r9.md를따른다. 다음은WIP/소유권별채택검토와F1 custody/association/durable저장/DTO·SDK/CLI이고전체AI/UI제품goal은계속active다. 아래과거163/190·빌드진행중은당시관측이다.

최신 관측(이하 과거 진행 기록과 구분): 원 A09 source375는67pass/68fail/122미실행으로보존했다. 원인과각실패를분리한private후계r8에서Product145/Core18 실제등록·ignored·exact163개및producer전후무결성통과. rootr8 fmt39/strict38 exit0이며root/Meari원type·fmt·strict근거도유지한다. Native26/OS1의no-run42가C/meari-native에서실행중이고실제27시험·정상bins는아직미완료다. r8은INPUT-R8.md SHAF9840E5E0B4D7FC4FC389352CB3AF42304DB8999B8B4FAE60A14BEB0FE21029A의6delta/부모chain으로고정하며canonical적용이나원375성공으로대체하지않는다. [실제190 정본](../../sots/verification/application/source375-datalab-first190-20261026.md)이각원로그·producer·실패/미실행을분리한다.

경로 설명 정정: stage에서생략한것은root .cargo이며package의기존.cargo설정은baseline closure에있다. 실제launcher env/명시target-dir/TMP/TEMP가각C cache문맥을고정한다. registry source/cache는시작때비어있었지만사용자승인으로필수locked Rust crates만확보했고이후전검증은offline이다. 두lock/MSVC/SDK/전역설정은불변이다. 아래과거의registry상태·no-run실행중/67pass상태를현재결과로오인하지않는다.

최신 결과: root/Meari fmt·type·strict 및 제품 no-run 통과. Product145 실제등록/ignored 대조 후67pass/68번째 route settle 실패를 보존했다. 남은 Product77/Meari45·정상bins gate는 미실행이다. [첫190 gate 결과](../../sots/verification/application/source375-datalab-first190-20261026.md)의 원실패/독립EXE-PDB·한정backtrace·다음조사 순서를 따른다. goal은 active이며 첫실패 때문에 전체목표를 축소하거나 완료로 표시하지 않는다.

원pin: source375 A09B51ACBC77645FD12D8488B23021173780851E267A8BA5103D34F827294D39, exact190 082C68B479E656A290F11A6CBA3AB0AF9AAE0C9651B265AA337767D454ADD464. root d13cbfca140e41863b4d0bd27930be3f57a1123f/Meari0844f6c24caf224617ca8abbb2dd75695a971b99.

private Rust tool runtime/development/source375-custody-r1는 std-only JSON frozen-manifest decoder, lexical target/source 경계, Windows OS certutil SHA256와 GetFileInformationByHandle 식별자, create_new/sync/copy검증을 책임별 모듈로 분리했다. 새 PS/Python/JS 검증 로직0. 처음 source payload의 private runtime 경로에 public target validator를 잘못 적용해 exit1·stage 생성0이었다. public destination 제한과 pinned private payload remap을 분리하고 회귀를 추가했다. 원실패 로그를 보존했다.

runtime/verification/source375-datalab-b0-r1에 source375 actual375/375 SHA 일치, exact unique190/Product145/Core18/Native26/OS1, canonical existing318/absent57/existing중different113을 기록했다. historical 원manifest·selected375 사본·기존318 preimage 및 old→new custody/fileID/bytes/creation/mtime/hash TSV를 보존했다. canonical writes0. 이는 원 호스트 역사 preimage 개수171/204와 다른 **현재 canonical의 실제** 관측이며 같은것으로 위장하지 않는다.

fresh stage는 runtime/development/source375-datalab-stage-r1. canonical src/tests/assets/packages와 Cargo/lock 등 source closure를 기준으로 원 파일을 덮지 않고 selected375를 사전에 선택해 복사한다. .git/.spec/.agents/runtime/target/node_modules/.cache는 기본 제외, 필요한 실제 fixture/resource는 명시 후속 closure로만 추가한다. baseline/selected 구분과 각 파일 hash/identity를 기록하며 stage 준비 완료 전 제품 Cargo는 실행하지 않는다. private stage에 .cargo를 복제하지 않으므로 launcher env/명시 target-dir/TMP/TEMP를 반드시 고정한다.

## 도구체인 관측과 다음 gate

rustc1.99.1/commit58cf8f9c35b355199727627d07ae34a4e2846d67·cargo1.99.1/5f94df478·stable-x86_64-pc-windows-msvc. VS2022 BuildTools/MSVC14.44.35207·Windows SDK10.0.26100.0 설치 경로가 존재한다. 현재 Rust registry source/cache는 비어 있고 index만 존재해 필수 dependency 확보가 필요하다. root lock SHA B2E3D540665FFC8E68D24D0A6C531D2B983658FB19B5ABBA525659BA1ACC371D, Meari lock23792484F39356E19904DBBB34B1496979E87793CF035491EF71F7C4134CC1EE. 경로존재/lock/hash/runner기능은 제품 정상빌드 성공이 아니다.

stage→root/Meari fmt→lock 고정의 최소 dependency 확보/실type→strict→no-run JSON producer→실등록190→동일producer exact190→정상bins/provenance 순서다. 각 그룹 실행 중 Cargo writer0와 전후 content/identity를 확인한다. 결과/원복/streams/writer 반환과 원 WIP/index를 기록한다. UI/AI·정상제품 사용자 인수는 별도 후속 단계이며 전체목표에서 누락하지 않는다.

진행 갱신: stage2103/selected375 준비 후 root/Meari fmt·type·strict Clippy가 모두 실제 exit0이었다. no-run/등록/실제190/정상bins는후속gate로남기며현재 root no-run이실행중이다. [B0/B1 근거](../../evidence/environment/source375-datalab-b0-20261026.md)에원receipt와도구체인/lock/custody pin을기록한다. 사용자후속정정에따라 정상bins에서전체개발완료로종료하지않고 실제AI결과검토/수정/판정·이력과UI시각인수까지 전체목표에유지한다. 이로써추론/수동GUI보류가자동해제되지는않으며검토가능한SDK/CLI·실표본·GUI준비전사용자승인대기를개발차단으로쓰지않는다.

스킬: modoo.today 검색 후 build-cache-correctness-audit v0.1.0 본문 SHA598e01f3a75a076c5f8b40994fa1450fe5b9bcf0633c356ec696f9806592818b 재열람. 실제producer/consumer/source환경identity·실패보존을 적용했다. 검색의 inference benchmark/demo 결과는 무관하여 채택하지 않았다. 같은 세션에서 읽은 shared-checkout(bf93d292…)의 타WIP/index/branch 보존을 유지한다.
