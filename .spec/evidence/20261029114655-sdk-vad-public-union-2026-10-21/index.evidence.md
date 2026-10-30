---
id: evidence:kinekino/20261029114655
title: SDK·VAD 공개 소스 합성 CPU 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SDK·VAD 공개 소스 합성 CPU 검증

2026-10-21 root `03a1075921e2a8a9da0bf588f0c4eed4d0476ed7`, Meari `0b79d57d7a59b3146b7ec72d02e2651b21b166fd`에서 Media 공개 패키지51, SDK host 저장 panic3, 공통 VAD 요청4를 중복·override 없이 고유58로 합성했다. 원 manifest/raw와 UI 채택 입력을 보존했다. 증거는 `runtime/verification/sdk-vad-public-union58-r1/`에 있다.

## 입력·lock·첫 실패

원 합성 source58 `D28C7BF6…C1FC01`은 guard1을 변경 없이 상속하므로 실제 수정/신규57이다. 기존7/new51의 before를 현 root에서 재대조했다. root Cargo는 기존 dependency/features/patch를 유지하면서 local workspace/default-members3만 추가하고, executor standalone workspace를 제거한다. 실제 VAD operation이나 root product dependency는 등록하지 않았다.

원 root Cargo.lock `6209C8C556B4508F98D4F87656336CC10D35AA3DA5759CEF76570A92E1BE93CF`를 보존하고 할당된 S에서 Cargo offline metadata로만 새 lock을 생성했다. package692→695는 local3 추가이며 registry681의 name/version/source/checksum drift0이다. 생성 lock SHA256 `91B6EB00B46C3DA1F2C558DF942A1453BFFA16F79D6474FB328F599344CBDDEE`. standalone pruned lock 이식·수동 lock 편집·다운로드0이다. 이후 모든 compiler/check 명령은 locked로 실행했다.

첫 selected-package strict는2131ms/101이었다. host shutdown의 fallback wake 호출에 JobWake trait import가 없어 E0599가 발생했다. 원 로그·receipt·source58/부모를 보존하고 import1파일만 수정했다. allow나 검사 약화·정책 변경은 없다. 최종 source58 SHA256 `A2632DC112C9E7304536B211A532F0BC36A859A923FE3F0EC0E976B75B78650C`, 원문 final-source58/. Cargo 생성 lock을 포함한 인계 scope59 SHA256 `7B239C2C80380936D91881E0BB5E0F76BF8CED26AE64D32C83F93F6AC5E1B01A`, raw adoption-source59/와 before59.json을 별도로 보존했다. root 채택은 중앙 PM 책임이다.

## 실제 검증

수정 후 패키지4 all-target strict3491ms/0, default root lib/player-cli/jobs 소비자 strict14231ms/0, root+선택 package cargo fmt2431ms/0이다. 패키지 시험 fresh compile18153ms/0와 root libtest compile60514ms/0 뒤 원/fixed BIN6을 고정했다. 각각의 실제 --list를 package+target+full name 키로 기대 목록과 비교해 다음 고유155를 실행했다.

| package | target | 실제 결과 |
|---|---|---:|
| media-vad | contract |11pass|
| media-vad-protocol | contract |19pass|
| media-vad-executor | lib |31pass|
| media-vad-executor | product_guards |15pass|
| media-jobs | lib |75pass|
| kinekino-player | lib, application::audio_vad::tests만 |4pass|

합계155pass/0fail/0ignored다. root 나머지636개는 filtered이며 전체 root suite 통과라는 뜻이 아니다. 같은 contract target 이름도 package가 달라 중복 집계하지 않았다. inherited standalone 통과를 union 통과로 대체하지 않았다. 기대155 원 명단과 실제 list/결과는 gate JSON/log에 있다.

## 무엇을 검증했는가

공개된 세 Rust 패키지의 workspace 형식과 공통 jobs owned API 호환성, P0/protocol/executor의 기존 synthetic 계약·ordinary file guard, SDK shutdown 준비/최종 저장의 adapter unwind containment와 공통 요청 syntax를 검증했다. store panic 시 원 host/worker/session/lease를 carrier 밖으로 unwind-drop하지 않고 기존 최초 오류를 보존한다. 저장 복구 뒤 receipt가 생겨도 기록된 오류가 성공으로 바뀌지 않는다. 이 경계의 세 새 시험은 std thread와 synthetic lease/store를 사용하며 실제 Windows 추론 worker 종료가 아니다.

public 요청은 원 음원 expected_source와 derived WAV/PCM/descriptor를 구별하고 정식 release manifest를 private profile snapshot과 등치하지 않는다. 실제 release decoder/producer/trusted public→private ID mapping과 원 guard 연결은 별도 미인수다. JobService가 expected_source를 prepared.parameters에 자동 보존하지 않으므로 SDK-owned 내부 versioned envelope와 매 attempt 재admission이 후속으로 필요하다. 이 구현은 이번58에 없다.

## 보호·한계

원 runner receipt20은 모두 terminal이며 최초101 하나와 최종 zero19개를 보존했다. 기록40PID 부재, 원 exec 회수/live0이다. root2164/S2164, 원 parent raw119, index, 기존 CLI3E13/jobs1F2235/F7/DEF 및 새 fixed BIN6의 해시 차이0을 확인하고 heavy/shared를 문서 작성 전에 명시 반환했다. 새 lock은 선언된 유일 추가 Cargo 출력이다.

RAM29.49GiB/GPU5668MiB/C171.85GiB, 기존 S/T/cache·EEA9 conservative2/affinity3/BelowNormal·offline/locked/-j1/thread1을 유지했다. 실제 browser/network/GUI/모델 worker 실행0이며 허용된 ordinary file/lease 회귀만 실행했다. 공개 source 등록은 제품 AI 기능 활성화·외부 모델/runtime 재배포 승인이나 inference 품질 근거가 아니다. 임의 carrier Drop/destructor panic/abort, 전체 owned-host queue/retry/lease, 실제 VAD cleanup/EOF/quality 및 [전체 SDK·CLI 기능](../../../plans/player/plan-006-sdk-gui-cli.md)은 계속 남아 있다.
