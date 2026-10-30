---
id: sot:kinekino/model-catalog-reads
title: 공통 모델 카탈로그 조회 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 공통 모델 카탈로그 조회 계약

2026-10-27 후속: [models.verify 파일 검증](model-file-verification.md)을 별도 bounded checkpoint 경로로 구현했다. inspect의 present_unverified 의미와 read-only/no-admission 정책은 유지한다. 아래 verify 미구현 설명은 이전 조회-only checkpoint의 상태다. install/remove는 계속 미구현이다.

2026-10-27. [generic manifest 설계](model-manifest.md)의 metadata 조회를 공통 Rust domain/application과 fixed-root infrastructure로 구현한다. `models.list`/`models.inspect`만 해당한다. install/verify/remove, 엔진 준비/추론·품질 승인과 구별한다. 미구현 작업을 카탈로그 JSON으로 활성화하지 않는다.

`PlayerSdk::with_model_catalog`가 read-only port를 주입한다. 기본 SDK는 unconfigured Unsupported를 유지한다. CLI `--model-root ABS_DIR`는 다른 local store와 조합 가능하지만 remote endpoint와 섞지 않는다. 존재하는 ordinary local root만 읽으며 디렉터리/카탈로그/모델 파일을 만들지 않는다. root/catalog.json은256KiB 이하 UTF8 JSON이며 `model-catalog/1`,models최대64·각파일최대128이다. `model-manifest/1`은 model_id/task/repo_id/immutable40hex revision·files(path/decimal bytes/SHA256)·allowlisted format·quantization·source_model/conversion·tokenizer/config 참조·license/ref·languages·runtime/min_version·선언 측정/evaluation 참조를 소유한다. 누락된 관측은 null이며 만들어 채우지 않는다.

manifest의 경로는 portable ASCII 상대 경로이며 traversal/절대경로/역슬래시/Windows reserved name·case alias·file/directory prefix collision을 거절한다. tokenizer/config는 선언된 파일에 정확히 포함되어야 한다. mutable main/latest revision·중복 model/revision·unknown fields·라이선스 URL의 credential/query를 거절한다. 포맷은 safetensors/GGUF/ONNX 선언만 허용하나, metadata가 포맷 내용이나 실행 안전성을 검증한 것은 아니다. README·remote code를 읽어 실행하지 않는다.

`models.list` payload는 빈 object 또는 task filter(null 허용)다. summary의 local_files는 not_checked다. `models.inspect`는 model_id와 immutable revision을 정확히 지정하고 원 manifest와 파일별 관측을 반환한다. 경로는 `artifacts/<model_id>/<revision>/<declared path>`에 한정한다. 각 원 directory/file handle을 관측 동안 보유하고 reparse를 거절한다. 목록의 SHA는 이번 요청에 읽은 catalog bytes의 SHA다. 여러 파일 관측 전체의 원자 snapshot이나 응답 후 수명은 보장하지 않는다.

상태는 missing/present_unverified/size_mismatch와 observed_bytes다. 같은 크기의 손상·잘못된 모델도 present_unverified이며 SHA 검사 완료로 표시하지 않는다. 설치됨/가용/엔진 호환/실제 dtype·RAM/VRAM·권리 허용·품질 승인을 추론하지 않는다. 응답은 integrity/declared_measurements/license_permission=not_verified, runtime_admitted=false,quality_approved=false를 유지한다. catalog 포트 가용성은 연결 상태이며 올바른 모델이 설치됐다는 의미가 아니다.

revision/idempotency/timeout context는 read-only 명령에서 거절한다. catalog 변경을 최신 모델 자동선택·재시도·다운로드·원격접속·실행으로 연결하지 않는다. GUI는 같은 DTO를 표시하는 후속 소비자이며 아직 연결/시각 인수 전이다.

검증은 synthetic manifest/네 바이트 stub·누락/크기불일치·고정 CLI schema발견·재열기·readonly 불변으로 수행한다. 실제 HF model 지원이나 actual9 품질 근거로 계상하지 않는다. 기존 modoo.today QA 및 postcondition/replay 스킬의 선언/실제·원본 불변·fixture/실제 모델 분리를 적용한다. 도메인 검색의 무관한 training skill은 채택하지 않았다.
