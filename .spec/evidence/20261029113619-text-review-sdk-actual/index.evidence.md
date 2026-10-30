---
id: evidence:kinekino/20261029113619
title: 실제 텍스트 표본의 공통 SDK 검토 패키지
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 텍스트 표본의 공통 SDK 검토 패키지

2026-10-19. 기존 Gemma270·Llama1B·Qwen0.6의9개 실제 결과를 공통 검토 계약v1로 변환하고, 고정한 CLI 사본으로 실제 inspect/list를 실행했다. `ok=true`, `evidence_verified=true`, `review_ready=true`, `automatic_application_enabled=false`를 확인했다. review_ready는 자료를 검토할 수 있다는 뜻이며 의미 품질이나 모델 채택 통과가 아니다. 사용자 판정은 기록하지 않았다.

최신 package는 [r3](../../../artifacts/quality-review/sdk-text-results-r3/packages/659b3cb6757259d8e770f6d3c96c4a77dafbab43c6ea540bcc7e0c58e4eb0197.json),51,138바이트/SHA256 `659b3cb6757259d8e770f6d3c96c4a77dafbab43c6ea540bcc7e0c58e4eb0197`다. r2의 자원 값을 배치 전체 비용과 요청별 시간으로 명확히 구분했다. Gemma 예시는 batch36.0653998초, 교정 request11.4456266초이며, peak값은 모델 로드를 포함한 전체 worker tree 값이다. 공정한 모델 간 성능 순위를 뜻하지 않는다. r2 자료와 검증 이력은 그대로 보존한다.

r3도 실제 SDK inspect에서 같은9개 표본과 evidence_verified/review_ready true·automatic_application_enabled false를 확인했다. [r3 요청·응답·도구 보존](../../../evidence/player/model-review-root-actual-r3/)의 응답 SHA256은 `1a9b4e5aeea9f3d640f13e71d52c9b896e0c30100e2e8a4987df23a5d5c8718e`, exporter SHA256은 `dc2d69f627b244c6b445f3cdb652956658886ead68a6ca6433430b309260a259`다. exporter r3는 fmt·기존 viewer2개/producer3개 시험·all-targets strict Clippy·locked offline build를 통과했다. 누락된 자원 값을 만들어 넣지 않고 거부하는 회귀를 포함한다. 새 모델 추론·사용자 판정 기록은0회다.

## 고정 자료

개발 담당이9개 실제 출력과 원문을 직접 읽은 [내용 감사](text-review-content-audit.md)를 분리해 기록했다. 반복·잘림·근거 cue 불일치 등은 개발 선행 점검이며 사용자 판정 저장이나 모델 전체 승인으로 간주하지 않는다.

[실제 package](../../../artifacts/quality-review/sdk-text-results-r2/packages/98762d00f8b3bb3f2db9f82a395e29e42e5e48072a143f3a5ff65244ba2d1f78.json)는49,749바이트, SHA256 `98762d00f8b3bb3f2db9f82a395e29e42e5e48072a143f3a5ff65244ba2d1f78`다. 같은 review root의 objects는 실제 바이트 해시로 배치했다. 모델 manifest3개와 측정3개의 SHA를 고정해 읽었고, 원문/실제 출력/모델revision·정밀도·engine/자원/자동검사 저장값/검토 기준/알려진 문제를 연결했다.

원문 baseline은 `reference/source-text`, revision은 실제 원문 representation 자산 SHA다. 독립 사람 정답으로 간주하지 않는다. 검증된 시간 구간이 없는 텍스트9개는 모두 range=null이며 가짜 timestamp를 만들지 않았다. 교정 결과2개의 구조 실패도 그대로 노출한다.

## 실제 실행과 증거

[CLI 요청·응답·도구 EXE·소스 스냅샷·게이트 로그](../../../evidence/player/model-review-root-actual-r2/)를 보존했다. 사용한 CLI SHA256은 `f6804b5fdf347f8be3fe8e26fede52679cd06a8652b7fff3396447e76829dedf`, exporter SHA256은 `4919682569840f063640f50f9159383e7884bc48f8fc59de77a0b1ddfec75719`다. CLI 사본은 실행 전과 보존 후 같은 해시를 확인했다. 이는 파일 바이트 식별이며 loaded memory나 전체 compiler 입력 attestation은 아니다.

- reviews.inspect: 실제9개 item 수락·자산 검증·검토 준비 확인. 응답 SHA256 `a36a7470492b3f13d2ce310cfb2c66762143d5e8ceb8a32c7a8ac9f6b0b10b6b`.
- reviews.decisions: gemma270-correction에 빈 목록 반환. 응답 SHA256 `032d8bff170a495cc0c144ff9cc7c70e63849361bb16084ed3b8383a167d3353`.
- 두 CLI 명령 모두 exit0/stderr0. review root의 판정 파일0개를 별도 관측했다.

exporter 변경은 fmt·두 binary의 각2개 시험·all-targets strict Clippy·locked offline release build를 통과했다. content-addressed 기존 자산 변조 시 덮어쓰지 않는 회귀와, 과거 engine_id가 없을 때 당시 SHA로 구분하고 digest마저 없으면 거부하는 회귀를 포함한다.

첫 export r1은 과거 Llama/Qwen manifest에 engine_id가 없어 exit1로 거부됐다. 그 실패와 부분 디렉토리는 보존했다. 후속 r2는 당시 기록된 engine_sha256에서 명시적 content identifier를 만들고 근거를 input_conditions에 적었다. 현재 설치 파일을 과거 실행 파일로 귀속시키지 않았다. [producer receipt](../../../artifacts/quality-review/sdk-text-results-r2/export-receipt.json)의 sdk_inspection_executed=false는 export 시점의 사실이며, 후속 실제 검증은 위 별도 응답에 보존한다.

## 미연결 범위

공통 reviews.read_text_asset은 구현 후 고정 새 CLI `8a0daa08ea99f14a2242b0d31b8e3360ba66f0d7853fdedc89d566cfaa447f19`로 실제3개 자산을 조회했다. Gemma 잘린 JSON text/plain516B, Qwen 한글 text/plain261B, 원문 application/json2129B 모두 ok=true/exit0이다. 잘린 내용을 JSON 성공으로 바꾸거나 보정하지 않았다. [원문 요청·응답·CLI 사본](../../../evidence/player/model-review-text-root-actual-r1/)을 보존했다. 응답 SHA는 각각 `a92736559f53c818a7c54093ebe842029eca56ebbf60db3ef8ccbc913945d756`, `1a1ee6dc92f093920c283c8d55b09800eb1307a28322cd60b898cb9797b7ca03`, `bbb6ecabc90f3ef5c751ac157b56be1a5dabea64f7ae8bc1e17e8b383b83b53a`다. 이는 실제3자산의 전달 관측이며 모델 의미 품질이나 GUI 표시 인수는 아니다.

GUI main의 명시 --review-root 조립과 application launch 파싱 회귀를 추가했다. 기존 client와 LocalHost에 같은 ReviewStore를 주입하며, 미제공은 기존 unavailable을 유지한다. 해당 체크포인트의 fmt·기본 cargo test132통과/9ignored·all-targets strict Clippy·player compile check는 exit0이다. [두 변경 파일과 gate 원문](../../../evidence/verification/review-gui-wiring-2026-10-19-r1/)을 보존했다. 이는 전체 전이 compiler source attestation이나 이후 MODEL UI/transport 진단 변경의 최종 통합 결과가 아니다. GUI를 실제 실행해 확인한 것으로 표시하지 않는다. UI 소스에 package SHA나 `.spec` 경로를 고정하지 않는다.

[기존 읽기 전용 화면](text-review-bundle-actual.md)은 같은 실제 입력/출력을 보여 주지만 이 SDK 패키지의 판정 저장 UI가 아니다. 공통 bounded text-asset 읽기는 실제3자산을 확인했고, GUI는 이 유스케이스를 이용해 파일을 직접 읽지 않도록 연결 중이다. 실제 GUI backend 주입·비교·판정 입력의 실행 인수와 음성/화자/영상 전체 검토는 남아 있다. 사용자 판정이 없다는 사실을 개발 진행의 차단 조건으로 삼지 않는다. [공통 계약](../../player/application/api/model-review-contract.md)을 따른다.
