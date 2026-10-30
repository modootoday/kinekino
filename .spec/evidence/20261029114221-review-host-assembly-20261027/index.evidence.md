---
id: evidence:kinekino/20261029114221
title: Review 공통 host 조립·수명 재생성
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Review 공통 host 조립·수명 재생성

후속375 감사는2190파일 content/identity 동일·종료0/38460ms다. 소스 편집 없이 이번 영향 checkpoint를 고정했다.

2026-10-27 중앙 PM. 공통 assembly::review_playback::assemble_local은 trusted Rust LocalReviewSessionConfig와 bounded history를 받아 하나의 Arc<ReviewPlaybackSessions>를 반환한다. Windows CoCreateGuid는 infrastructure에서 호출하고 scope는 요청·저장 설정에서 받지 않는다. media open은 수행하지 않으며 각 host는 반환 service를 같은 PlayerSdk에 주입한다. 고정 Sprite admission을 일반 제품 기본값으로 넣지 않았다.

입력 runtime/verification/source375-review-host-assembly-input-r1/SOURCE.md SHA256 `394039FA403A5A65FDEC9CCCBA8568B3B6842E171E956F42E70F3248D3D83F95`; inventory `5FBA1C403755308AB69BE0F578B5C1304E1A6B02776824F850F269E9E603926A`,2190파일. 기존2088 입력의190/정상bins는 이 변경 전 checkpoint이며 승계하지 않는다. preimage는 runtime/verification/review-host-assembly-preimage-r1에 보존했다.

371 all-target native strict,372 actual no-run,373 actual1/CLI35(34.71초),374 공통 정책14,fmt --check가 통과했다. stdout/stderr/원 launcher receipts는 source375-datalab-b0-r1-logs의 각 번호가 소유한다. 실제 결과 runtime/verification/review-host-assembly-cli-actual-datalab-r1/RETURN.json SHA256 `D77479D0378FCF1278721B22655D16E3E9E39C9E84219B2562A10A091764DC6E`.

실제 CLI35는 새 공통 조립을 사용해 source-bound 수명/다섯 제어/오류/관측 경계 pause/정상 정리를 통과했다. CLI producer SHA256 `1EA3B34EA4320635FA439680E767F0800D8EE05510A453D4BDAC901DA3E9FC76`. 이후 같은 프로세스에서 새 host service를 만들고 이전 session status를 RevisionConflict로 거절했다. 새 host ordinal1과 이전 ordinal1은 다른 scope이며 이전 host도 새 참조를 거절했다. 새 Unverified revision1은 NeedsInput/child미생성/physical_closed/owner_finished로 종료했다. 이전 scope cd7c9face9f749cdb437117d436c02e5,새 scope382409cf1b5742e0957f3ebd18cdcbc0. GUID의 일반적인 고유성 보장·별도 프로세스 강제 종료 복구 전수 시험으로 확대하지 않는다.

새 조립 함수는 제품 기본 no-args/CLI host의 자동 설정이 아니다. trusted admission 공급·기본 host/transport 종료 연결·별도 OS 프로세스 재시작 행렬·전체CLI·GUI시각·AI품질은 남는다. 실제9 연결=false/quality_approved=false, headless CPU다. 수동GUI·추론·다운로드·WARP·push0, 기존WIP/소유권/원증거/C캐시/heavy1 유지. 적용 modoo.today QA 스킬의 실제/합성·수명/프로세스·부분/전체 구분은 선행 제어 검증의 출처·SHA 기록을 따른다.
