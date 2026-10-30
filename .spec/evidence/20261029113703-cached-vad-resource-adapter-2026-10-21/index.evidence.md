---
id: evidence:kinekino/20261029113703
title: Cached VAD 자원 어댑터 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Cached VAD 자원 어댑터 검증

사용자 AI 재개 및 [중앙 자원 정책](../../../plans/player/ai/resource-allocation-2026-10-21.md)에 따른 첫 경량 CPU 경로다. Silero `onnx-community/silero-vad@e71cae966052b992a7eca6b17738916ce0eca4ec`/MIT·FP32 ONNX2243022B와 기존 mono16kPCM16 dialogue.wav25초를 고정했다. 코드/빌드·정상 추론1·협력 취소1의 제한 검증을 마쳤다. 사람 VAD 품질·제품 SDK/CLI AI 연결은 별도 미완료다.

## 구현과 소유권

원 owned-process WindowsFamily/Deadline/Supervisor를 재사용한다. ResourceProbe는 원 process/Job 양쪽 borrow만 하며 새 핸들·권한·상속·PID 재개방을 만들지 않는다. 원 child exit+원 JobEmpty 뒤 OS peak를 조회하고 borrow를 끝낸 뒤 원 finish를 별도로 인수한다. peak query 성공으로 cleanup을 대체하지 않으며 Incomplete/원 오류/owner를 보존한다. OS currentPrivate·peakWorkingSet·peakPagefile·JobPeak는 구별한다.

도구의 별도 bounded VAD entry는 CPU EP/intra2/inter1을 명시한다. 기존 role 기본4는 유지한다. 정상25초/782score와 취소 first-chunk handshake/partial1score를 분리해 원 요청/시각·loaded runtime pins·affinity/priority/원 binary·결과를 실제 보존했다. 취소 handshake는 성능 비교 대상이 아니며 fast completion/forced kill/status 문자열만으로 취소 성공을 표시하지 않는다.

## 실제 검증 범위

독립 source13 r1 388DE737을 읽기 감사로 인수한 뒤 source229+조립/입력4의 독립 S snapshot233을 byte/SHA로 인수했다. S의 상대 build root와 local dependency를 Cargo Windows-filtered metadata로 확인했고, Cargo가 tool local owned-process lock delta 및 별도 owned workspace lock을 생성했다. r2 B4E67A1C는 lock1만 변경하고 코드12 불변이다.

r2의 정상 실행파일 링크는 LNK2019 DMLCreateDevice1로 실패했다. 고정 ORT static library의 explicit-library 경로에서 필요한 기존 DirectML import library 연결이 누락됐다. 원 r2 source/로그/receipt를 보존하고 build.rs1만 수정한 r3 **6EC80C84C1929F857EFF457FA298243E57ED4FCEBAED83036C219C420DFEECEF**를 동결했다. DLL/ORT library/추론 CPU 설정·전역 RUSTFLAGS를 바꾸지 않았다.

최종 r3에서 owned/model-tool 양 all-target strict와 양 manifest fmt를 통과했고 실제 고유7을 통과했다. tool 등록74개 중 선택4(원 identity·782 frame/점수·fast completion 거부·원 취소 proof·strict request shape)와 owned invalid-handle1, ResourceProbe compile-fail doc2다. doc 원 stderr의 E0515/E0505를 확인해 owner escape/borrow중move가 거부되는 의도한 경계임을 인수했다. 이7은 정상/취소 실제 추론2와 별도다.

정상 BIN41708544B/SHA09F77408AEE8E866FF877AC2422F64B1E4315F88CA137E436FF4BD09ED3EC137의 build 및 비추론 preserve-build가 통과했다. build.rs 수집 선언227source+BIN1=228 row는 archived/current bytes/SHA 차이0이며 전체 실제 Cargo/registry/native compile closure가 아니다. 초기 snapshot233/lock뒤234와 분모를 구별한다. 원 build receipt27 terminal(25zero+원101둘)/고유62PID부재·보호/index 차이0 후 중앙에 슬롯을 반환했다.

## 실제 정상1과 협력 취소1

정상1은 inferred25초/782score/speech254, load0.1425539초/inference0.2355821초/worker0.4747055초/outer3593ms다. OS JobPeak27308032B·process WS peak42188800B·Pagefile peak26058752B·currentPrivate77824B를 구별했다. 실제 loaded DLL5 canonical/hash·CPU mask3/BelowNormal이 핀과 일치했다. 원 worker62532/creation134370157095029049는 exit0/Empty/forcedfalse/EOF이며 outer 포함3PID 부재·보호/index 차이0 후 슬롯을 반환했다.

취소1은 첫 chunk ready1077955187→원 parent signal1077955234→관측1077955296의 같은 request/monotonic boot ticks와 cancelled1frame이다. performance comparison=false이며 이 handshake의 RTF를 속도로 쓰지 않는다. JobPeak27140096B·WSpeak41349120B·Pagefilepeak25894912B·currentPrivate73728B, loaded DLL5 일치를 확인했다. 원 worker59336/creation134370158396451854는 exit0/Empty/forcedfalse/EOF, outer3803ms/포함3PID 부재·보호/index 차이0 후 슬롯을 반환했다. 재시도/다운로드0이며 4GiB는 관측 목표이지 hardcap이 아니다.

원 full metadata는 캐시 없는 타 플랫폼 nix0.31.3을 offline이 거부한101을 남겼다. 이후 Windows-filtered metadata0을 확인했으며 registry version/source/checksum drift나 실제 다운로드0이다. 원 실패를 없애거나 첫 시도 성공으로 바꾸지 않는다.

## 근거와 남은 범위

정확 source/before/delta/lock/계획은 `runtime/coordination/ai-resume-vad-r1/source-r{1,2,3}/`, 원 build logs/receipts/artifacts는 `runtime/verification/ai-vad-build-r1/`, 실제 정상/취소는 `runtime/verification/ai-vad-{normal,cancel}-r1/RESULT.md` 및 closeout/원 JSON에 있다. 초기 model/tool/native DLL/library pins와 실제 loaded DLL 관측을 구별한다. 전체 OS DLL closure·VRAM peak·사람 recall/오탐 및 제품 연결은 남아 있다.

기존23 실험·사람 품질 승인0·텍스트9 검토 패키지와 본 단계의 실행 분모를 합치지 않는다. 같은 Silero 정상/취소 재실행을 새 모델 역할로 더하지 않는다. tracked 공통 owned-process4와 private untracked model-spike tool/provenance를 분리하고, 불완전한 private 도구 일부만 공개 stage하지 않는다. 제품 승격 때 Rust package의 정확 공개 source/dependency/license closure와 공통 SDK/CLI input/result/error/cancel/provenance 계약 및 사람 교정·명시채택을 별도로 완성한다. Pages/VM/DPI·GUI 검토는 재개하지 않았다.

modoo.today inference-benchmark-workload-protocol 및 references/protocol-tables.md(SHA89f6ff32617d660be61433aac491db8f4e1fc1c9bb7b07b8a0c0cf80c1f0c4fd)를 적용해 cache 재사용·한 host/한 입력·시작/실패 분모·load/inference/전체시간과 취소 실험을 구분했다. 7개 구조 회귀를 모델 품질·추론 benchmark로 확대하지 않는다.
