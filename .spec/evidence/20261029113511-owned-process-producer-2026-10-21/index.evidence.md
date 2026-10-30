---
id: evidence:kinekino/20261029113511
title: owned-process 생산자와 root 등록 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# owned-process 생산자와 root 등록 검증

2026-10-21 Windows의 중앙 heavy/shared snapshot 배정에서 실행했다. 제품 helper 전환 및 실제3category CLI/mpv gate가 아니다. 실제 사용한 child는 패키지의 숨김 Rust 시험 executable뿐이다. 제품 GUI·미디어·브라우저·AI 실행0이다.

## 입력과 최소 lock

root 기준 `e5aa0b8baee5ffc5c6c4d8fe5e8af9532f14415f`와 최신 A11 r3 integrated11/protected snapshot의 union1347을 원문 대조했다. root live1387, 원 Cargo/lock, 신규19 부재를 보존했다. 격리 public-p3-parent-r1 snapshot에만 r2 source19 및 승인한 workspace/dev-dependency 등록 patch를 적용했다.

첫 offline check만 --locked를 빼고 실행했다. lock 변경은 local owned-process package와 root dependency edge뿐이며 registry version/source/checksum 변경0이다. 이후 모든 Cargo 명령에 --offline --locked/-j1과 절대 --manifest-path를 사용했다. 등록 snapshot Cargo SHA는 `3531B4ADBC62525CB04B4FD46E8B364099F33FD819769EA1349AE7CD017C1BD0`, lock은 `6209C8C556B4508F98D4F87656336CC10D35AA3DA5759CEF76570A92E1BE93CF`다. root live Cargo/lock/index는 수정하지 않았다.

## 실제 결과와 보존된 수리

| 입력 | 검증 | 실제 결과 / runner elapsed |
|---|---|---|
| r2 source19 3F31…511A8 | default/support all-target check | exit0 /11109·1328ms |
| r2 | pure integration state_machine | 8pass/실패0/무시0/제외0,0.06초 /8828ms |
| r2 | Windows 목록 | 8개 이름 확인, 시험 실행0 /6543ms |
| r2 | Windows --ignored 실제 | 8pass/실패0/무시0/제외0,8.43초 /9061ms |
| r2 | default strict 원 실패 | unnecessary_mut_passed /101·769ms |
| r3 source19 775D…CAA2 | default strict | exit0 /1664ms |
| r3 | support strict 원 실패 | 의도한 parent-exit 자극의 zombie_processes /101·1092ms |
| r4 source19 7BE3…C285 | default/support all-target strict | exit0 /453·1148ms |
| r4 | package fmt | exit0 /190ms |
| r4 | parent-exit/outside sentinel exact 재검증 | 1pass/실패0/무시0/제외7,4.09초 /6264ms |

고유 행동 회귀는16개다. r4 exact1은 r2 Windows8에 포함되므로 더하지 않는다. r4에서 전체16개를 다시 실행했다고 하지 않는다. r2/r3 원 source와 원 실패/정상 로그 및 당시 바이너리를 보존했다.

r3은 CreateProcessW의 불필요한 mutable 참조와 변수 선언만 launch1파일에 수정했다. r4는 test-support 전용 child의 의도한 descendant/parent-exit 자극을 작은 함수로 분리하고 그 함수에만 이유 있는 expect를 뒀다. 모든 branch에 wait를 넣어 회귀의 반례를 없애지 않았고 production 광역 allow는 없다. r4 최종 source19 pin은 `7BE399D706B9C0D36FE3CB9C5336808375DDBE11B556DE181AE76012586CC285`다. r2 대비 수정2경로 외17은 그대로다.

## 관측 범위와 정리

실제 Windows 시험은 원 process/thread handle과 own nested Job readback, EOF writer 비상속, parent exit 후 active family, 별도 Job의 sentinel 생존, Unknown·부분 startup·미확인 Drop·늦은 resume 및 실제 argv 왕복을 단언했다. syscall/FS 자체의 hard wall-clock 상한을 검증한 것이 아니다. 성공한 fixture는 temp raw를 지우므로 내부 모든 PID/creation/stream의 영구 이력을 보존했다고 확대하지 않는다. test log/assertion과 외부 runner receipt/PID를 구분한다.

첫 마감에서 snapshot 보호1345/root1387/최종19 양쪽 및 등록 Cargo/lock 차이0을 확인했다. 합성1366 manifest pin은 `E5F26781B8E067614E87BC5201FD3A730AE28F474EC114A18DF8DEB5A9B6988B`다. 실제 r4 Windows 시험 바이너리는1062400bytes/SHA `D24DEE3CB022A26A3D11742D38EFF8541185725E7B829CB43CCFBC136073BDA5`, Rust child는293376bytes/SHA `4C71114DF2DC85A43270A8739BE629B016320DDFAC454A8F4E51D9BF57649BC7`다. EEA9 runner는422912bytes이며 원 profile/affinity3/BelowNormal/기존 cache를 유지했다.

첫 마감12 receipt는10zero·원1012로 모두 terminal이었다. 기록 child PID54432는 원 Cargo 종료 뒤 다른 node.exe로 재사용됐고 새 생성 시각·경로·부모·명령을 sidecar에 남겼다. 번호만 보고 사용자 프로세스를 종료하지 않았다. matching 시험 executable process는0이었다. 원 process 부재와 PID 번호 재사용을 구분한다.

## root dev-dependency 추가 호환

첫 명시 반환 후 다른 heavy 착수 전에 root default/native all-target strict2를 별도 재배정받았다. 이는 Windows JobObjects/FileSystem feature unification 영향을 확인하기 위한 추가 compile 검증이다. 원 r4 closeout은 보존하고 registration-root-compat에 별도 receipts/최종 보호 기록을 뒀다. default92986ms/native136574ms 모두 exit0, 원 handle67648 terminal0과 기록4PID 부재를 확인했다. snapshot1345/root1387/source19·등록 Cargo/lock·binary 대조 차이0 후 heavy/shared를 다시 PM에 명시 반환했다. 추가 closeout SHA는 `AB9031D2A13AE61AF901CC07BCB4DA6170239031F6E56C62B9AEB7FFD43448AE`다. 전체 제품 시험·GUI/CLI 재실행은 하지 않았다.

원 근거는 `runtime/quality-review/owned-process-producer-r2`, r3, r4 및 r4/registration-root-compat다. 등록 이후 helper 이관과 root 채택은 PM 소유다. root .gitignore의 책임 경로 추가나 root package stage/force-add는 이 담당이 수행하지 않는다.
