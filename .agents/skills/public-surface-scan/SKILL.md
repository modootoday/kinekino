---
name: public-surface-scan
description: 공개 표면(README, docs, .spec, 커밋 메시지, --help 출력)을 점검·개선할 때. 비공개 흔적·낡은 문구·링크 오류를 찾는다. push 직전 스캔 순서도 여기에 둔다.
---
# 공개 표면 점검
1. 비공개 흔적(일상 점검): `git grep -n -i -F -f .local-status/private-terms.txt -- . ':!.spec.local'` (`-F` 필수, 2026-11-03) (목록 파일은 로컬 전용). 보조: 로컬 정밀 스캔 도구(`.agents/local/scan.md`, stdout만, 결과 파일 금지; tracked 파일명·내용).
2. `spec check` 0/0, README·docs의 플랫폼 상태 표기가 실제 구현과 맞는지 비교.
3. CLI `--help`·`version` 출력이 실제 명령과 맞는지 확인.
4. 발견마다: 위치 / 심각도(높음·중간·낮음) / 수정안. 수정은 worktree 안에서만.

## push 직전 필수 스캔 (2026-11-05 원 님 승인)
루틴 자동 push 직전, 아래를 **이 순서대로** 하고 하나라도 적중하면 push하지 않는다. git-and-release.md와 동일.
1. **커밋 메시지**: `git log origin/main..HEAD --format=%B` → `private-terms.txt` 고정 문자열(`-F`) 검사.
2. **추가 diff**: `git diff origin/main..HEAD`의 추가(+) 줄 검사.
3. **파일명**: `git diff --name-only origin/main..HEAD` 경로 검사. 필요 시 로컬 정밀 스캔 도구(`.agents/local/scan.md`, stdout만)로 tracked 파일명·내용을 한 번에 본다.
4. **이미지·바이너리**: 경로·파일명만 점검. 신규 바이너리 추가가 있으면 보고하고, 내용 바이트 검사는 하지 않는다.
5. 적중 1건이라도 있으면 **push 금지**. 오탐은 제외 근거를 한 줄 남긴다.
