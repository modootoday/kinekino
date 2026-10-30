---
id: plan:kinekino/20261029113542
title: 제품 VAD host 활성화 잔여 gate
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 VAD host 활성화 잔여 gate

2026-10-23 중앙이 최종22를 root `5a322e037e624e5d1ab4caaaa4c36d49953b4878`로 채택했다. 정상 SDK·CLI의 metadata/fmt/strict 및 실제73(일반 CLI3 포함)은 [검증 정본](../../../sots/verification/ai/vad-product-host-2026-10-23.md)의 범위로 완료했다. 아래 순서1~2는 이 gate의 이력이며 반복 실행 배정이 아니다. 공유 cache buildscript 별칭2의 원 producer는 미해결/all protected=false를 유지한다. Media는 명시 반환했고 현재 추가 heavy 권한이 없다.

현재 제품은 `PRODUCT_RELEASE=None`이며 ready 전 NeedsInput 거부까지만 실제 입증했다. 다음 작업은 승인 record·bundle·실제 audio2/native 인수의 소스 계획 준비다. 신규 Cargo·worker 실행 파일 빌드·runtime load·추론·다운로드·네트워크·GUI는 새 중앙 배정 전 보류한다. [공통 계약](../../../sots/player/application/analysis/vad-product-host.md)과 final-source22/정확 preimage를 기준으로 하며 과거 stage를 자동 재적용하지 않는다.

1. PM 독립 읽기 검토: 정상 root module/기존 private helper의 test-only 책임, 원 new3/legacy16 호환, std TryFrom/full profile/manifest/worker 결속, per-request audio/destination guard, scope 확인 전 I/O 차단, CLI가 thin adapter인지 확인한다. SDK가 채택한 root·worker Cargo/test6와 before13을 대조하고 자동 예전 union 재적용을 금지한다.
2. 중앙 명시 default gate: 기존 S/T에서 정확21 적용·Windows filtered offline metadata/lock/의존성 kind3/ORT0·정상 root lib/bin/default strict를 검증한다. WAV parser body 변경0/재노출2, normal SDK/API/CLI module 실제 연결을 타입으로 확인한다. 신규8/편입 issuer3와 기존190 중 직접 영향·original host shutdown/CLI·guard·admission·legacy16 회귀를 실제 등록에서 중복 제거해 실행한다. 다른 source-only 후보/GUI 검사로 대체하지 않는다.
3. 제품 입력 인수: 고정 HF revision/license/precision/model pin, 실제 dedicated native worker의 bytes/SHA/build/ABI, 전체 release manifest와 dependency/notice 경로·pins·load phase, 검증된 Windows baseline/명시 system module 목록을 담당 정본과 대조한다. private Rust record의 빈 값을 운영·fixture JSON으로 자기승인하지 않는다. 기술 fixture와 생산 권한을 구별하고 생성 이미지·시각 승인을 모델 품질이나 배포 승인으로 전용하지 않는다.
4. 중앙 명시 실제 승인 경로 gate: normal `player-cli --serve-jobs`의 opt-in ready와 공유 SDK jobs.submit/watch/get을 통해 서로 다른 audio ID2·destination2를 처리한다. 원 WAV/PCM/descriptor·모델/worker/manifest pin, 실제 결과/error/no-overwrite/cancel/retry와 원 process owner/EOF/family/cleanup을 수집한다. configured22 test harness의 성공이나 NeedsInput만으로 실제 native/model 지원을 완료 처리하지 않는다.
5. prepared WAV 외 미디어·원격 입력은 기존 Rust media 처리/입출력 SDK의 변환·provenance/import 계약을 연결한다. 현재 per-request WAV 주소 경로는 첫 수직 범위이며 최종 제품을 단일 샘플/형식으로 축소하지 않는다. 사용자 고정 실제 미디어 품질 검토는 제품 연결 뒤 마지막에 수행한다. ASR·화자분리·OP/ED·비전·제작/검색·에셋 범위는 전체 목표에 계속 남긴다.

## 다음 positive gate의 준비물과 책임

[기존 cache 제한 탐색](../../../sots/player/infrastructure/inference/research-followups/vad-native-cache-inventory-2026-10-23.md)을 마쳤다. exact package/archive 부재는 조사 범위에 한정하며 추가 후보 탐색을 종료했다. runtime/coordination/vad-native-cache-inventory-r1/technical-input-evidence-map.json이 SDK의 실제 입력·원 핀·문서와 미완료 native/audio2 연결을 인계하는 자료다. 새 중앙 실행 배정이나 issuer 권한으로 사용하지 않는다.

소형 문서 실제 취득 결과는 [원 bytes 인계](../../../sots/player/infrastructure/inference/research-followups/vad-release-document-bytes-2026-10-23.md)를 따른다. 고정 HF license와 ORT source LICENSE/TPN은 이제 정확 bytes/SHA로 공급할 수 있다. DirectML HTML을 license로 채택하지 않았고 실제 native payload/notice mapping·전용 worker·audio2 기술 gate는 남긴다.

기존 실제 입력의 정확 경로·원 pin·현존 metadata와 누락은 [단일 준비표](../../../sots/player/infrastructure/inference/research-followups/vad-positive-input-readiness-2026-10-23.md)를 정본으로 사용한다. 모델/대화WAV/음악WAV와 역사적 native 도구·전용 default worker/fixture를 구별했으며 원고지·전용 native worker·manifest·audio2 제품 binding을 준비 완료로 표시하지 않는다.

공식 e71 LICENSE 본문 존재·ORT1.31 source commit·native terms와 Windows 문서 조사를 마쳤다. [책임별 파일 준비안](../../../sots/player/infrastructure/inference/research-followups/vad-official-license-sources-2026-10-23.md)은 작은 license text 취득/원 bytes pin, native payload 고지 대응, 기술 gate를 별도 배정할 구체 근거다. PM이 직접 배정하며 별도 독립 배포 담당이나 새 사용자 법률 승인 체계를 전제하지 않는다. 기존 실측 허용과 제품 record에 필요한 정확 파일/검증·중앙 자원 배정은 구별한다.

아래는 소스 경로를 실제 읽어 확인한 전제와 잔여 계획이며 실행 성공이나 배포 승인이 아니다. 값이 없으면 placeholder·fixture·caller JSON으로 채우지 않는다.

| 준비물 | 확인한 기존 소비 경로 | 담당·인수 조건 |
|---|---|---|
| 독립 제품 선택 | `application/audio_vad/product_release.rs`의 private ProductReleasePins/NativePins → ConfiguredVadRelease | PM/배포 담당이 승인한 HF revision·license·precision·worker·manifest pins와 근거를 인계한 뒤 Media가 최소 Rust record delta를 준비한다. 캐시/기술 fixture를 승인으로 사용하지 않는다. |
| 실제 bundle 전체 | `vad_product_host/release.rs`의 manifest snapshot·full profile·artifact set·파일별 guard·ParentSelectedRelease | worker/model/license/dependency/notice 실제 bytes·SHA·크기·재배포 근거가 모두 일치해야 한다. startup/runtime phase·System32 목록·최소 Windows build를 임의 확장하지 않는다. worker 자신의 hash를 worker 안에 재귀 삽입하지 않는다. |
| 요청별 실제 입력2 | `ProductVadHostOptions::plan` 및 ProductInputFactory | 서로 다른 원 미디어/opaque audio ID2·destination2, WAV 원 SHA·PCM·sample 수·descriptor/CAS·입력 provenance를 공통 SDK가 산출한다. 현재 prepared PCM16 mono16k WAV 밖 입력은 공통 미디어 어댑터 후속으로 유지한다. |
| 명시 실행 환경 | 중앙 execution-leases와 cache-unresolved2 보고 | PM이 캐시2 처리·재사용 여부와 fresh 자원/원 S/T·writer admission을 결정한다. 기존73 반환이나 WSL 정리 자체를 새 실행 권한으로 간주하지 않는다. |
| 실제 CLI consumer | 기존 일반 CLI BIN pin 주입·owned-process·localhost helper | 새 최소 Rust consumer는 별도 소유 delta로 준비한다. normal CLI test=false와 libtest를 구별하며 original owner/deadline·raw/stdin EOF/family/terminal·오류/정리 영수증을 보존한다. fixture로 생산 권한을 발급하지 않는다. |

## 다음 소스/검증 분리 순서

1. PM 인계된 record/bundle 근거와 기존 exact source의 교집합·소유권을 먼저 비교한다. 독립 승인 정보가 준비되기 전에는 PRODUCT_RELEASE를 None에서 바꾸지 않는다. 승인 record 작성과 새 positive consumer는 별도 manifest/preimage로 동결하고 기존22·실패3·cacheunknown2를 보존한다.
2. 새 배정 뒤에만 normal SDK/CLI의 fmt/test/strict·metadata/lock/feature 및 실제 BIN pin을 다시 고정한다. 새 worker 빌드/ORT load/model 추론이 필요한 단계는 그 범위를 중앙이 명시 배정해야 한다. 이전 configured fixture 성공을 생산 worker/native 증거로 승계하지 않는다.
3. 정상 host ready에서 VAD 지원을 실제 조회한 뒤 공통 SDK jobs.submit/watch/get으로 입력2를 처리한다. 두 결과의 원 입력/worker/model/manifest binding·출력 SHA·저장 경로와 no-overwrite를 확인한다. 서로 다른 WAV pin 시험4나 현재 NeedsInput 거부를 이 positive2 대신 세지 않는다.
4. 실제 취소/재시도·잘못된 원 SHA/설치 파일 변조·기존 destination 충돌·원 EOF/owner/cleanup을 별도 case로 검증한다. 새 cutoff/cleanup ACK를 만들거나 GUI로 우회하지 않는다. 실패는 원 최초 오류와 정리 오류를 함께 보존한다. fault case가 모델 실행을 요구하면 별도 실행 범위에 포함한다.
5. 실제 ABI/loaded closure/Windows 기준과 기술 결과를 인수한 뒤 사용자 모델 품질 검토로 넘긴다. 품질 검토 전 기술 성공과 배포 권한을 구별하며 전체 미디어·AI 기능 범위를 유지한다.

modoo.today `modootoday/software-qa/code-change-review-verification`의 실제 관측별 verdict 원칙을 적용해 현재73·미검증 positive2·승인 전 record를 분리했다. 본문/필수 참조의 정확 SHA와 적용 근거는 위 검증 정본을 따른다. 이번 갱신은 read-only source 조사와 계획 작성뿐이며 Cargo/프로세스/새 다운로드 실행0이다.
