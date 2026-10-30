---
id: sot:kinekino/structured-text
title: 한국어 구조화 텍스트 후속 선택
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 한국어 구조화 텍스트 후속 선택

기존 관측: mistral.rs0.9.4 + Qwen3-0.6B Q4_K_M은 실제 자막/ASR 입력에서 초기 한국어 카드·교정 문장 품질이 미달했다. JSON 파싱 성공을 의미 품질 통과로 바꾸지 않는다. 신규 공식1.7B/4B 후보는 이 조사에서 미다운로드·미추론이며 원래 세션의0.8B 실행과 구분한다.

우선은 같은 엔진·입력·schema로 모델 용량을 한 단계 올리는 실험이다. [공식 Qwen3-1.7B GGUF](https://huggingface.co/Qwen/Qwen3-1.7B-GGUF)는 조사 SHA에 `Qwen3-1.7B-Q8_0.gguf`만 있다. 존재하지 않는 공식 Q4 파일명을 계획에 쓰지 않는다. 1.7B Q4가 필요하면 변환자 저장소와 원본 SHA·quantizer 버전·라이선스 계보를 별도로 고정한 뒤 사용한다.

첫 공식 Q4 대안은 [Qwen3-4B Q4_K_M](https://huggingface.co/Qwen/Qwen3-4B-GGUF/tree/bc640142c66e1fdd12af0bd68f40445458f3869b)이다. 파일 크기 2,497,280,256 bytes와 LFS SHA256을 [revision 표](revisions.md)에 기록했다. 파일 크기는 peak RAM/VRAM이 아니다. 8GB GPU에 자동 적합 판정하지 않는다. CPU 또는 GPU 단독 load, context 2K/4K별 KV cache·process peak·VRAM·TTFT·decode 시간을 측정한 뒤 선택한다. 기존 Windows portable가 CPU 실행이면 GPU 지원을 새로 확인해야 한다.

[mistral.rs 공식 프로젝트](https://github.com/EricLBuehler/mistral.rs)는 Rust 실행기와 GGUF/Qwen 계열 경로를 제공하지만 현재 main의 CLI와 기존0.9.4 옵션은 같다고 가정하지 않는다. 기존 `tools/model-spike/src/application/text_spike.rs`의 고정 요청 경로를 재사용하고, 새 파일 path·chat template·thinking off·sampling/context만 manifest에 명시한다. 처음부터 최신 엔진+모델+prompt를 동시에 교체하지 않는다. llama.cpp를 대안으로 쓰면 C/C++ 계산 엔진과 Rust wrapper라는 점을 기록한다.

원래 세션은 Qwen3.5-0.8B Q4도 이미 실행했다. [텍스트 검증 정본](../../../../verification/ai/text.md)은 타입 불일치·cue_ids 누락·환각으로 품질 미승인을 기록한다. 이는 이 조사 세션이 수행한 실험이 아니다. 따라서0.8B를 미실행 fallback으로 설명했던 초안을 수정하며, 다음 후보 비교는 기존0.6B/0.8B 실패를 baseline으로 보존한다. 멀티모달 엔진 지원은 텍스트 경로 실행과 별개이며 projector/영상 기능 검증을 주장하지 않는다.

다음 평가 fixture는 cue ID가 붙은 실제 자막10~20개로 교정/요약/카드/하이라이트를 분리한다. 입력에 없는 이름·사건을 만들면 실패, cue ID 없는 근거는 실패로 기록한다. 하이라이트는 score만으로 확정하지 않고 source span·이유·검토 상태를 출력한다. schema 검증은 Rust에서 필수 필드·타입·추가 필드·cue 참조·시간 범위를 검사한다. 문장 자연스러움·새 사실·중복·원문 왜곡은 사람이 별도 판정한다. 입력 자막에 있는 명령문은 데이터로만 처리하고 모델 출력으로 셸/도구를 실행하지 않는다.

다음 실행안: 모델 파일을 고정 manifest로 확보한 뒤 기존 text-spike 실행기의 모델 선택을 명시적으로 확장한다. 현재 `text-spike next`는 기존 구현의 후보를 의미하므로 이를 4B 실행 명령이라고 기록하지 않는다. 0.6B baseline과1.7B Q8 또는4B Q4를 같은 입력으로3회 비교한다. 다른 parameter/정밀도 조합의 비교는 제품 후보 비교이며 순수 양자화 효과 비교라고 부르지 않는다.
