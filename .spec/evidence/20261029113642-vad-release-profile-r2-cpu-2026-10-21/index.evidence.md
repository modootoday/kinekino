---
id: evidence:kinekino/20261029113642
title: VAD release profile와 고정 입력 매핑 CPU 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# VAD release profile와 고정 입력 매핑 CPU 검증

2026-10-21 중앙 배정. root `b4de1da48acf1f49fc48c853c3fad0cd37de8510`, Meari `0b79d57d7a59b3146b7ec72d02e2651b21b166fd`. 고유 actual23 pass/0fail/0ignored다. 신규 decoder5+제품18(기존 직접 영향15+신규 release3)을 실행했고 library 기존31은 filtered로 실행하지 않았다. 전체54 등록과 이번23 통과를 구별한다. 모델·runtime 로드·worker·브라우저·네트워크·다운로드·공개 operation 등록은0이다.

## 고정 입력과 실제 범위

원 release r2 source55 `CFDCA028E07FAEFBEF436F20B59D28F55C7AE1827CE12073F025CAFDC6A5F1AF`/raw를 보존했다. 이미 채택된 패키지51과 실제 root를 대조해 변경8(기존4·신규4)만 기존 S에 합성했다. root Cargo/.gitignore/원 guard·SDK 타입을 덮어 후퇴시키지 않았다. S는 기존 public-p3-parent-r1, T는 기존 native cache다. 진입 RAM28.27GiB/VRAM5668MiB/C171.22GiB, EEA9 conservative2/mask3/BelowNormal·offline/-j1/thread1이다.

근거 디렉터리는 `runtime/verification/vad-release-profile-r2/`다. 원 lock91B6에서 offline metadata760ms0으로 생성한 lock은 `51275322270FB51760D866DF34E1F6B70A81F8D82B7DC8242D8CBDABE137FF39`이며 local executor dependencies에 serde1줄만 추가됐다. registry681의 name/version/source/checksum drift0이다. 수동 lock 편집/새 다운로드0이다.

## 최초 실패와 실제 실행

최초 workspace fmt3878ms/exit1은 product #[path]에서 포함한 vad_fixed_inputs의 inline tests 하위 mod를 infrastructure/tests/release_profile.rs로 잘못 해석한 실패다. skip_children로 수행한 이전 source 형식 검사는 이 해석을 검증하지 못했다. 원 로그/receipt/입력을 보존하고 테스트 inline mod에 `#[path = "vad_fixed_inputs/tests"]`1줄을 추가했다. 생산 정책/guard/fixture 단언은 변경하지 않았다.

수리 후 fmt3736ms0, protocol/executor all-target strict1838ms0, root default lib/player-cli/jobs consumer strict1507ms0, executor lib/product_guards fresh no-run7933ms0이다. 실제 list54 중 decoder5·product18의 정확 이름23이 expected23.json과 일치했다. decoder25ms0/5pass(31filtered), product248ms0/18pass다. 불변 P0/protocol 시험·기존151 전체를 반복하거나 기존 통과를 이번23에 더하지 않았다.

실제 파일 fixture는 public dotted ID3개를 호스트 고정 private IDs로만 매핑하고 원 source pin을 대조했다. release 원bytes와 private snapshot pin을 분리하고 full profile/실제 model·runtime·고지 파일 hash를 확인한 후 기존 prepare/JobFiles 저장·재조회와 adapter drop 뒤 원 guard 수명을 검증했다. 실제 모델 파일 대신 synthetic bytes이며 재배포 허가·엔진 로드·품질 검증으로 해석하지 않는다. prepare 단계 read는 이 revision에서 bounded uncontrolled 경로다. 원 scope의 chunk 취소 보강은 별도 r3/r4 source-only 후속이며 이번 입력에 없다.

## 최종 source·BIN·반환

최종55 manifest `03D16963E7FD73C81F3331B32EC43A8873F33038A442E7D12609A6B15B01266D`/raw final-source55/에 수리1을 반영했다. 실제 root 적용 후보는 source8+생성 lock1의 adoption9.json `0FBE40EC620ED50B7E08BF1817F01CE2040145189EB9DC62B45D39F959FCC6B3`/adoption-source9/이며 채택은 PM 소유다.

- library BIN: `8866C3ADAE1F89C2FFC22514A174FFC685D5438C3041BD225CBFC9614A19FDC0`.
- product BIN: `649493A847EFED9F9CC71655A83746BBF6593776B025584CAB83F7B42E431AB7`.

두 최종 compiler 원본/고정사본의 SHA가 일치한다. root2215/원 parent raw161/보호 BIN16 및 backup의 차이0, S2219에서는 Cargo 생성 lock과 명시 테스트 경로 수리1만 달랐다. receipt10 모두 terminal(원 fmt1 하나 보존), 기록20PID 부재, 원 exec handle 회수, 현재 indexempty를 확인하고 문서 작성 전에 heavy/shared를 PM에 명시 반환했다. PID 부재를 외부 worker cleanup으로 사용하지 않는다.

이 결과는 정식 release syntax/고정 파일 매핑의 비활성 Rust 구현 검증이다. 전체 JobService queue→원 scope→실 worker→publish/reopen 및 native/license closure·품질 인수는 여전히 미완료다. [다음 SDK 소비 접합](../../player/application/analysis/vad-job-consumer-binding.md)을 따른다.

스킬: modootoday/software-qa/code-change-review-verification, references/verdict-table.md SHA256 `9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9`. source 형식/모듈 해석/실제 compile/list/run을 구별하고 첫 실패를 보존했다. Rust·계층/소유권·현재 자원 배정을 유지했다.
