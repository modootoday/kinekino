# kinekino 에이전트 규칙

이 파일과 `.agents/`가 단일 정본이다. `CLAUDE.md`(`@AGENTS.md` 한 줄)와 `.claude/{rules,skills,agents}`(링크)는 얇은 어댑터이며 내용을 따로 쓰지 않는다.
주제별 규칙은 `.agents/rules/`(색인 `.agents/rules/README.md`), 절차는 `.agents/skills/`, 역할 정의는 `.agents/agents/`에 있다.

## 항상 지키는 것
- 사용자가 지정한 단일책임원칙, 모듈화, 계층형 디렉터리 구조를 모든 구현과 설계에 우선 적용한다. 상세 `sot:kinekino/conventions`, 검사 `cargo xtask lint-structure`.
- 모든 개발 코드(제품, 테스트, 벤치마크, 개발 도구)는 Rust로 쓴다. 사용자가 요청한 React/TSX UI 선언과 Meari 변환 입출력만 예외다. 외부 엔진·모델·OS 도구는 Rust 어댑터로 연결한다.
- 계층: `domain`(정책·값 객체) / `application`(유스케이스·포트) / `infrastructure`(외부 기술 어댑터) / `presentation`(UI·입력) / `main.rs`(조립). 거대 `utils` 모듈 금지.
- 호스트 코드에는 사이트별 용어를 쓰지 않는다. 사이트 해석은 플러그인이 맡는다.
- 외부 URL과 웹 메시지는 신뢰하지 않는다. 브라우저 데이터로 프로그램이나 셸을 실행하지 않는다.
- 기능은 공통 Rust SDK와 CLI로 먼저 구현·검증하고, GUI는 같은 유스케이스를 쓰는 얇은 표시·입력 계층이다. GUI 자동화로 CLI 미지원 기능을 우회하지 않는다.
- AI 결과의 품질 승인은 사용자가 실제 GUI에서 한다. 자동 검사 통과를 사용자 승인으로 세지 않는다.

## Git
- 기능 맥락과 검증 단계별로 명시 경로만 stage해 커밋한다. `git add .`/`git add -A` 금지. push는 사용자 요청이나 승인된 루틴 자동 push 규칙(`.agents/rules/git-and-release.md`)에 따를 때만 한다.
- `main` 반영은 PR로만 한다: 토픽 브랜치 push → `main`으로 PR → Linux 필수 체크(`lint`, `test (ubuntu-26.04)`) 통과 후 사람이 merge commit으로 병합. 규칙 정의는 `.github/rulesets/main-protected.json`. `main` 직접 push·강제 push·이력 재작성은 하지 않고, 에이전트가 PR을 스스로 승인·병합하지 않는다. 근거 `adr:kinekino/20261116062843`.
- `.gitignore`는 기본 거부를 유지한다. runtime, 모델, 캐시, 개인 미디어, 비밀정보, 로컬 `.agents`/`.spec.local`은 커밋하지 않는다.

## 문서 (specfold)
- 모든 설계·계획·유지보수 문서는 `spec` CLI로 루트 `.spec/`에서 관리한다(sot, adr, plan, evidence). 코드를 바꾸기 전 `spec context <path>`, 생성은 `spec new <type> --governs <path> --title … --write`, 상태는 `spec set`. 파일명을 직접 쓰지 않는다. 커밋 전 `spec check --staged`(또는 `cargo xtask spec-check`).
- 컨벤션 `sot:kinekino/conventions`, 상태 `sot:kinekino/project-status`, 사용법 `guide:kinekino/docs-specfold`. 관측 사실, 설계, 미검증을 구별한다.

## 검증
- 변경 후 `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`.

## UI
- UI 변경은 [시각 검토 규칙](.agents/rules/visual-review.md)을 따른다. 시안 세부의 1:1 대응과 실제 실행 재검토 없이 시각 일치를 완료로 표시하지 않는다.
- 일관성이 최우선 기준이다. 시안 기준 경로는 `.spec/artifacts/design/`이며, 이미지의 장식·오류를 기능 규격으로 올리지 않는다.
- 공통 아이콘은 Meari 공식 카탈로그를, 글꼴은 Meari 공통 자산 계약을 쓰고 출처·버전·해시·라이선스를 보존한다.

## 스킬
- 작업 시작 시 modoo.today skills에서 도메인 스킬을 찾아 본문을 읽고 적용하며, 출처와 적용 판단을 정본이나 검증 근거에 한 줄로 남긴다. 다른 언어 예제를 코드로 들여오지 않는다. 절차: `.agents/skills/modoo-skill-lookup`.
