---
id: adr:kinekino/20261029113528
title: "ADR-015: 원본 임베딩 참조 실행의 고정 관측 게이트"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# ADR-015: 원본 임베딩 참조 실행의 고정 관측 게이트

상태: accepted, 2026-10-18. 원본 Candle의 첫 실제 실행은 일반80%게이트를 통과했으나 준비 제안60% 조건은 충족하지 못했다. [최초 관측](../../sots/verification/ai/embedding-reference.md)을 수정하거나 성능 비교로 승격하지 않고, 후속 실행의 조건을 코드로 강제한다.

`observe-reference`는 별도 Kind::Reference로 CPU가 유한0≤CPU<60%·여유RAM≥12GiB일 때만 worker를 시작한다. 원 preflight.ready와 앱 목록을 보존하며 사용자 앱을 종료하지 않는다. 일반 measure의 조건을 바꾸거나 과거 실행의 purpose/eligibility를 덮어쓰지 않는다.

허용 worker는 최종 sentence-embedding-reference EXE SHA256 `9047a6c1e0f59043674b81ea1e8f6c6a3c6a3f1f6bc29626bb647d4936816860`, 선보존 manifest SHA256 `7a2b4c5e279f313ca1a68f399dc4e3c48c8763e0ab9cc6403ae2baf8dcf67754`에 한정한다. label/exe 뒤에는 `run --model-dir ABS_FIXED_DIR --fixture ABS_FIXED_FIXTURE --output ABS_NEW_FILE --threads 4`의 정확한 순서·개수만 허용한다. 다른 flags·추가 인자·threads·inspect 명령을 거부한다.

모델 디렉토리는 workspace의 고정 MiniLM revision 경로와 canonical 일치해야 한다. 원본 weights와 fixture SHA를 실행 전에 확인하고, 출력은 results 바로 아래의 새 절대 경로만 허용한다. worker 자체도 metadata7파일·tokenizer·모델 shape/hash·CPU F32·환경 RAYON_NUM_THREADS=4를 검증하며 기존 파일을 덮어쓰지 않는다. root 단일 worker·BelowNormal·오프라인·600초 제한·선보존 orchestrator 규칙은 유지한다.

출력은 original_embedding_correctness_not_performance와 performance_comparison_eligible=false를 기록한다. 허용 인자·자원 경계를 검증한 것은 모델 품질·변환 전체 동등성·성능 통과가 아니다. CPU<60%는 실행 전 표본에 대한 admission 조건이며 다른 사용자 프로세스를 제한하는 OS 전체 hard cap이 아니다.
