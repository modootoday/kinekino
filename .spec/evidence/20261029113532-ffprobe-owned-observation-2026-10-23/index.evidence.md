---
id: evidence:kinekino/20261029113532
title: 고정 ffprobe 정보와 기존 매체 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 고정 ffprobe 정보와 기존 매체 관측

후속 원 leased regular-file fd 시험의 현재 부분 결과는 [별도 checkpoint](download-original-fd-checkpoint-2026-10-23.md)를 따른다. 아래 source2 미실행 표기는 이 source1 gate 종료 시점의 이력이다.

관측 시험 source1 채택 commit: `740b144edb75bfb93e1ce9fb370117e7c2dcce9f`.

2026-10-23. 고정 실제 ffprobe의 버전·빌드 구성·라이선스·입력 프로토콜과 기존 portrait MP4의 metadata를 원 owned-process consumer로 관측했다. **실제 관측5 pass, 별도 helper 회귀8 pass**, 최종 fmt/root all-target strict와 관측 target no-run도 통과했다. 이는 source2의 원 leased stdin/fd 매체·packet 검증이나 제품 download CLI 인수가 아니다.

## 소스와 실행 범위

root10d9a532132717904ed00af162aefcb94b080c14/library65ccf91840ef682857c53514b5abb1e5927e5884/lockE469A7204B9DC2AC82AC729E36FD8AC41449E1C382E9167A28A00A45C45789A2 기준이다. source1 manifest는 `runtime/development/ffprobe-observation-r1/final-source1.json` SHA1A0686E937D3E24FE91443697066AA8DF10015B9BA3542EFE758EF26C8BEBC4A다. 제품 미러의 기존 library7fb→채택65cc exact17과 신규 시험1만 union18로 반영했다(기존12/new6). 원 media source2 BB5B7656690025C0B9CA01C9A507CD9210CA2E2EAF2987AAA1139AA2F3509531, HTTP opener77DA7D46A9DA6F849F2730B3DBFE946A882663A98ABAF8785D869CD481EB4C96 및 UI/Media 후보를 합성하지 않았다.

gate는 `runtime/verification/ffprobe-observation-r1/`다. 기존 canonical 제품 S/T·runner EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D·offline/locked/j1/thread1/CPU2/BelowNormal/descendant Job을 사용했다. 초기 가용RAM20707880KiB/GPU6251MiB/C여유121654702080B/writer0을 보존했다. 새 target/clean/전역 touch/사용자 앱 종료/WSL/VM/DPI/모델/네트워크/새 media 생성/GUI 실행0이다.

## 실제 BIN과 등록

Cargo JSON의 새 관측 BIN은 fresh=false, 2078720B, SHA BDF51071BF0C0BA3F01D9696EC95F788FC8C7A09D7DAFBC697F0C431BE64244D다. 고정 사본 bin/ffprobe-observation-r1.exe를 실행했다. 실제 --list13, --ignored --list5를 보존하고 새 관측5와 기존 helper inline8을 구분했다. 전체 cargo test/workspace 전체 기능 실행이 아니다.

01 fmt exit0/5091ms, 02 root all-target strict exit0/42921ms, 03 no-run exit0/108588ms다. 08 version,09 buildconf,10 license,11 protocols,12 metadata는 각 실제 등록명 exact/ignored로1개씩 통과했고13 helper 필터는8개 통과했다. 조회5는 서로 다른 original child/deadline이며 정보4각10s/metadata15s와 cleanup2s를 원 helper가 적용했다. pin admission과 동기OS/file호출까지55s hard wall로 보장한다고 표시하지 않는다.

## 관측과 의미

| 관측 | 고정 실제 출력에서 확인한 사실 | 아직 입증하지 않는 범위 |
|---|---|---|
| 버전/구성 | FFprobe6.1-essentials_build-www.gyan.dev, gcc12.2.0. --enable-gpl/--enable-version3/--enable-static 및 외부 라이브러리 구성을 raw로 보존 | 대응 전체 소스/외부 의존성·보안·제품 배포 입장 완료가 아님 |
| 라이선스 | 실제 -L은 GPL version3 또는 이후 버전을 명시 | LGPL 빌드로 표시하지 않으며 제품 전체 라이선스를 추정하지 않음 |
| 프로토콜 | 실제 Input 구간의 fd=true | 원 sealed writer lease를 실제 fd로 읽은 검증이 아님 |
| 기존 portrait metadata | MP4/mov 계열,932830B,12.000000s. video0 h264720×1280, audio1 aac/48000Hz/6channels | packet count/전체 decode/A-V sync/품질/원 provider 다운로드 지원이 아님 |

실제 tool은 C:/ProgramData/chocolatey/lib/ffmpeg/tools/ffmpeg/bin/ffprobe.exe,82620928B/SHAefb014c8f39091c90f3616154fe20ab8470f5f943f6a48ec4fd30c479c7dd697다. 각 caller가 원 executable/ancestor guards와 hash/bytes를 확인하고 유지했다. metadata에는 기존 portrait SHA0fd9b2de74774b4d2dcea9f9b66466899b44aff1300b121d548e10af4c386483도 같은 방식으로 고정했다. PATH 탐색/다른 tool fallback0이다.

관측5 모두 실제 inner child_exit0/familyEmpty/forcedfalse/원 creation identity, 원 receipt 및 stdout/stderr10개 complete=true/limit_exceeded=false를 보존했다. stdin_eof_requested와 출력 EOF를 혼동하지 않았다. 원 family 종료 뒤 RawFile identity·byte-bounded read/export의 complete를 확인했다. helper가 보고한 child elapsed는 version953/config60/license56/protocol72/metadata76ms이며 최초 준비·hash·전체 wall/성능 비교가 아니다. 원 stdout/stderr·request.json·owned/receipt.json·observation.json이 직접 근거다.

## 첫 실패와 수정

07 version은 ffprobe spawn 전 evidence parent guard에서 NotFound로 실패했고 outer receipt exit101/11ms를 보존했다. 실행 cwd=S에서 상대 New-Item으로 빈 S/runtime/verification/ffprobe-observation-r1/observations만 생성하고 env의 D절대 parent를 만들지 않은 orchestration 오류다. source/BIN/guard를 완화하지 않고 D절대 parent를 명시 생성한08에서 같은 BIN으로 재실행했다. 빈 S directory도 삭제/숨김 없이 유지했다. source 수리revision은0이다.

이 gate의 **runner receipt는12개 전부 terminal,11 exit0+원07 exit101**이다. 중간 전달의13개 언급은 산술 오기이며 raw inventory runner-receipts-observed.json의12개가 정본이다. 04는 BIN 메타데이터이며 runner 실행이 아니다.

## 보호와 반환

root tracked1322/library tracked490의 byte 변경0, 양 index hash도 불변이다. root index B19DF659A4D64413DEF621B78C64EE154A709AB1294B33C0354924F458905F1D/library F92BE778CC38B6B727700177E87B25485BC3A95BCA5535D51767887E7C447272다. S4863→4869에서 기존12만 변경/new6, 제거0, byte동일4851의 UTC ticks mtime 차0이다. 변경bytes만 freshmtime으로 넣었고 부모의 과거 mtime을 복제하지 않았다.

T 기존exe571 중 변경4는 normal GUI/CLI와 각 deps hardlink alias다. 신규1은 실제 Cargo JSON 관측 BIN이다. JSON producer3와 각 동일 volume/FileID·hash alias2로5개 delta를 대응했다. 기존 normal4 실제 preimage 사본을 보존했고 새 normal GUI SHA3F1494C608B10919EC70D600AB1399EBF828FC69DA5FF6C4F74C8AA54C3CE66C/81094656B, CLI911F2DDCED4377E0CF2EF3F6B42034FE68AAC82EA0D9D683AD16CF15A26EB0BD/31848960B와 관측 BIN의 고정 사본3도 유지했다. 정상 GUI·제품 CLI 실행은0이다.

이전 고정 BIN61은 이번 사전/사후 hash 변경0이고 이전 owned-probe5도 그 prior gate의 핀과 현재 hash가 일치함을 재관측했다(후자5는 이번 build 이후 관측임을 JSON에 명시). T 제거0이다. historical4709 미귀속 alias2는 이전과 같은2D2094C73DC9FA25162BFA1B2466F7C5C5E0390ED79E637914F6E86D6BC6CCAA로 유지하며 all_cache_protected=false를 그대로 둔다. 이번 delta 귀속으로 과거 미귀속을 소급 해소하지 않는다.

final-process-observation에서 원 child5와 gate/build 프로세스는 부재이고 모든 본인 exec session/handle은 terminal이다. 원5 familyEmpty receipts가 종료의 직접 근거이며 PID 부재만으로 이를 대체하지 않는다. protection-source-summary/protection-binary-summary/전후 index·S/T/normal FileID·원 receipts를 보존하고 slot-return으로 명시 반환한다.

## 별도 입력 핀과 남은 범위

이 current metadata를 근거로 `runtime/development/download-real-probe-r1/input-portrait-r1.json`을 별도 source-only 동결했다: SHA C90B2C168EA25C78889531008D89DFED79C7CD60CB3EF6AC733B23B8B1275C25, duration min=max12000000us, video0h264/audio1aac다. 과거 probe로 current 관측을 대체하지 않았다. 기존 제작 clip의 source MKV hash/107.250~119.250s/번역 자막/로컬 폰트 근거는 입력 준비 문서를 따른다. private 기술 fixture이며 공개 media/font 번들·재배포 승인·권리 확대가 아니다.

다음 source2 actual3(원 leased regular stdin/fd 실제매체·손상거부·source취소)는 아직0이다. 실제 ffprobe child 취소/drop/unwind, production tool 입장, HTTP original source/runtime/resolver 물리ACK, provider header/auth/HLS/remux/resume/재리졸브/모드·batch/실제 product CLI 및 전체 자막·제작·AI 목표는 계속 남는다. HTTP opener의 새 leaf도 concretehost 없는 source-only다.

적용 스킬은 modootoday/software-qa/test-fixture-and-mock-build SKILL SHAa393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100이다. 계약 입력·상한·고립증거·원 정리·synthetic/actual 구분에 적용했다. code-change-review-verification SKILL SHA9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B와 verdict reference9D9619A829AE24D73645EF5F7C8126C8257CAEDDAC80800632866257008AD2A9를 원 실패/실제BIN·명단·지원/관측/품질·보호 한계 구분에 적용했다.
