# 2026-10-29 임시 비공개 이관 적용

이 clone은 다른 단말용 전달본이다. 아래 원 호스트의 <원 호스트 저장소> 및 D 정책은 역사 기록으로 보존한다. 이 clone의 빌드 경로는 clone 루트/runtime/build-cache 아래 native·meari-native·resource-runner·tmp로 고정하고 D 드라이브를 가정하지 않는다. 전역 설정은 변경하지 않는다.
원 .spec 정본은 이관 사본 docs/spec에 대응한다. 담당별 소유권·private overlay·미검증·보류는 docs/continuation/HANDOFF.md를 읽는다. 사용자 요청은 이 임시 PRIVATE 저장소 업로드만 허용하며 기존 구현 goal은 paused다. 원 저장소/Meari push, 추론·다운로드·수동 GUI·WARP 보류는 유지한다. Python maintenance는 사용자가 명시 요청한 정리용 기존 예외다.

---
# 최상위 프로젝트 규칙

- **2026-10-26 사용자 후속 지시 — Windows 검증 호스트:** 이 호스트는 D 드라이브 없는 NVMe SSD 환경이다. 이 호스트의 빌드 산출물·증분 캐시·빌드 임시 파일은 실제 `<원 호스트 저장소>/runtime/build-cache` 아래로 통합한다. 아래 2026-10-25의 실제 D 경로 의무와 HDD I/O 전제는 이 호스트에 적용하지 않는다. 제품 workspace는 `native`, 독립 Meari workspace는 `meari-native`, 실행기 bootstrap은 `resource-runner`로 빌드 문맥을 분리하고 `tmp`를 임시 경로로 사용한다. 프로젝트 Cargo 설정 및 runner의 `CARGO_TARGET_DIR`·명시 `--target-dir`·`TMP`·`TEMP`를 해당 문맥과 일치시킨다. 전역 Cargo/Rustup·다른 프로젝트는 변경하지 않는다. 원 호스트의 D 정책·역사 manifest/FileID·원 증거는 보존한다. 검증 중 producer 독점 및 무거운 작업 1개 중앙 배정은 유지하며, 병렬도 증가는 새 호스트 실측 후 별도로 판단한다.

- **2026-10-25 사용자 지시: 이 프로젝트의 빌드 산출물·증분 캐시·빌드 임시 파일은 D 드라이브의 비공개 `runtime/build-cache` 아래로 통합한다.** C 캐시 우선 사용이라는 이전 자원 정책보다 이 지시가 우선한다. 로컬 Cargo 설정뿐 아니라 담당 runner의 `CARGO_TARGET_DIR`·명시 `--target-dir`·`TMP`·`TEMP`도 실제 D 경로를 사용하도록 맞춘다. 기존 캐시는 필수 증거·소스·실제 표본을 구분해 정리·이전하고 경로/파일 식별 변경을 검증 기록에 반영한다. HDD I/O와 무거운 빌드는 중앙에서 직렬 배정한다. 전역 Rustup/Cargo 설정과 무관한 프로젝트는 임의로 변경하지 않는다.

- Git 변경은 기능 맥락과 검증 단계별로 명시 경로만 stage하여 커밋한다. 미완성·미검증 범위를 커밋 설명과 로컬 정본에 구분한다. 공유 root index의 stage/commit은 PM이 직렬 배정하고 독립 submodule은 담당이 소유한다. 사용자 요청 없이 push하지 않는다.
- `.gitignore`는 기본 거부·최소 공개 경계를 유지하되 공개 전용 소스·테스트·계약·필수 자산은 책임별 디렉터리로 허용한다. 새 파일마다 승인·예외를 추가하는 목록으로 확장하지 않는다. runtime, 모델, 캐시, 개인 미디어, 비밀정보, 운영 스크립트와 로컬 `.spec`/`.agents`는 공개 디렉터리와 분리하고 커밋에서 제외한다. 새 공개 디렉터리는 제품 필수성·재배포 권한·민감정보를 검토한다. `git add .`/`git add -A`와 비공개 경계의 광범위 해제는 사용하지 않는다.

- **모든 담당 세션은 modoo.today skills에서 작업 도메인에 맞는 스킬을 탐색하고 본문·필요 참조를 읽어 실제 설계·구현·검증에 적용한다.** 스킬 이름만 언급하거나 무관한 검색 결과를 채택하지 않는다. 담당 정본/검증 근거에 정확한 스킬 출처와 적용 판단을 간결하게 기록한다. 사용자 지시·Rust 정책·소유권·자원/보류 범위를 유지하며 스킬의 타 언어 예제를 새 개발 코드로 도입하지 않는다.

- UI 변경은 [프로젝트 시각 검토 규칙](.agents/rules/README.md)을 따른다. 기준 시안의 세부 요소 확대·1:1 대응 검토와 실제 실행 재검토 없이 시각 일치를 완료로 표시하지 않는다. 제품 전용 디자인 에셋은 직접 작성한 SVG로 공통 관리한다. 사용자 후속 지시에 따라 공통 아이콘은 Meari의 공식 upstream/공식 Rust·React 배포 우선 카탈로그를 사용하며, 출처·버전·해시·라이선스를 보존한다. Pretendard와 필요한 fallback도 Meari 공통 자산 계약으로 소비하고 검증 전 기존 자산을 삭제하지 않는다.

- UI/UX 설계는 2026-10-18 사용자 승인 상태다. 생성 이미지의 불일치한 디테일은 복제하지 않고 공통 Rust 컴포넌트로 구현한 후 검토한다. 외부 도구는 동등한 기능과 검증 수준이면 Rust 구현을 우선한다. 모델은 HF revision·라이선스·정밀도를 고정하고 실제 미디어 스파이크로 검증한다.
- **2026-10-24 사용자 결정: AI 결과의 품질 승인 주체는 사용자이며, 실제 제품 GUI의 사용자 테스트 과정에서 검증·승인한다.** 개발 담당은 먼저 공통 Rust SDK/CLI 기능·오류·취소·저장과 자동 품질 검사를 검증하고, GUI에 원본/후보 비교·필요한 재생·수정·명시적 판정 및 이력 확인을 연결한다. CLI 판정·합성 테스트·모델 실측 성공을 실제 사용자 품질 승인으로 계상하지 않는다. 검토 가능한 GUI와 실제 표본이 준비되기 전에는 개발 미완료로 기록하고 사용자 승인 대기를 차단 조건으로 삼지 않는다. 이 결정은 즉시 수동 GUI 시험·추론·다운로드를 재개하는 지시가 아니다.

- **제품은 GUI `player.exe`와 AI 친화 CLI `player-cli.exe`를 제공한다. 공통 Rust API/SDK를 기준으로 설계하며 양쪽이 동일한 application 유스케이스·domain 정책을 사용한다.** 기능을 GUI 이벤트나 CLI 명령에만 구현하지 않는다. CLI는 GUI 조작 자동화에 의존하지 않는다.
- **모든 기능의 CLI 구현·실제 검증을 GUI 단계의 선행 조건으로 한다.** source.example.test의 카테고리·작품·회차·외부 영상 제공자 이동·광고차단·영상 처리/재생부터 자막·제작·AI 모델 입출력까지 공통 SDK와 CLI로 구현하고 실제 지원·오류·취소·저장 근거를 남긴다. CLI 미지원 경로를 GUI로 대체하지 않으며 전체 CLI 인수 전 새 GUI 기능 확장·수동 시각 검토를 진행하지 않는다.
- **2026-10-19 후속 사용자 지시로 Meari를 소비하는 React/TSX 기반 Web UI 제작을 병행한다.** 네이티브 UI 담당이 승인된 제품 UI 선언·공통 디자인 자산을 작성하고 Meari 담당이 변환·실행 소비자 연결을 지원한다. 이 UI 제작 재개는 CLI 기능 검증을 GUI로 대체하거나 미지원 SDK 기능을 지원으로 표시하는 근거가 아니다. 수동 시각 인수와 기능 검증은 계속 구별하고 자원은 중앙에서 배정한다.
- **Meari React Router 호환을 기준으로 Web/Native의 URL·경로·탭·페이지·데이터 전달을 공유한다.** 공식 API별 실제 지원과 미지원·미검증을 구별하고 독자 router를 전체 호환으로 표시하지 않는다. 라우팅은 공통 Rust SDK의 기능 정책을 복제하지 않으며 typed host 계약으로 연결한다.
- **Meari의 제품·환경별 Native/Web 기능은 공통 플러그인 계약으로 확장한다.** Meari 담당은 최소 확장 SDK·등록/호출·수명 계약을, 제품 UI 담당은 구체 Rust 플러그인과 Native/Web 어댑터를 소유한다. 제품 정책은 공통 application/domain에 유지하며 모든 기능을 Meari 본체에 넣지 않는다. 플러그인의 target별 지원·오류·취소·정리와 버전 호환성을 실제 검증한다.
- **기능 점검과 완료 판정은 공통 Rust SDK 시험과 CLI로 수행한다. GUI는 UI/UX·레이아웃·입력·포커스·접근성·공통 API 연결 및 결과/오류 표시 정합성을 검토한다.** 같은 저장·변환·CAS·취소 기능 검사를 GUI에서 반복하지 않으며 GUI 자동화로 CLI 미지원 기능을 우회하지 않는다. GUI에 고유한 창·DOM·렌더 관측은 해당 범위만 입증하고 기능 검증과 구분한다.
- **SDK/API/CLI 핵심 규격·구현·자동 기능 검증을 먼저 완성한다.** 그 전에 수동 GUI 클릭·스크린샷 반복·화면별 시각 인수를 선행 gate로 두지 않는다. GUI는 공통 typed client를 사용하는 얇은 표시·입력 계층으로 구현하고, 시각 검토는 핵심 기능 통합 뒤 묶어서 수행한다. 중복 검사·반복 상태 보고를 줄여 토큰과 자원을 절약한다.
- **Source Player는 가제다.** 표시 제품명·출력 파일명·패키지 메타데이터는 한 책임의 제품 식별 설정에서 관리한다. 프로토콜·프로젝트·작업 ID는 표시 이름에 의존하지 않는다. 이름 변경은 데이터·설정 마이그레이션을 포함한다.

사용자가 지정한 단일책임원칙, 모듈화, 계층형 디렉토리 구조를 모든 구현과 설계에 우선 적용한다.

- **모든 개발 코드는 Rust로 통일한다.** 제품 코드뿐 아니라 새 테스트·벤치마크·개발 도구에도 적용한다. Python/JavaScript/PowerShell로 새 애플리케이션 정책이나 검증 로직을 구현하지 않는다.
- 사용자가 명시 요청한 **React/TSX UI 선언 및 Meari 변환 입력·산출물**은 허용한다. 이 범위를 별도 JavaScript/TypeScript 애플리케이션 정책·SDK·검증 도구 구현으로 확대하지 않는다. 재생·저장·AI·취소 등 기능 정책은 공통 Rust application/domain에 둔다.
- 외부 추론 엔진·모델·운영체제 도구는 Rust의 명시적 어댑터로 연결한다. 외부 도구 사용을 프로젝트 개발 언어의 예외로 확대하지 않는다.
- 기존 비 Rust 코드/스크립트는 Rust 이관 대상으로 추적한다. 대체 동작 검증 전 임의 삭제하거나 사용자 보류 범위(DPI·VM)를 변경하지 않는다. 현재 제품의 WebView 주입 JavaScript도 이관/최소화 설계 대상으로 명시한다.

- `src/domain`: 정책과 값 객체. UI, 브라우저 엔진, 운영체제 API에 의존하지 않는다.
- `src/application`: 유스케이스와 계층 간 계약. 도메인에 의존한다.
- `src/infrastructure`: WebView2, 광고 필터 엔진, 프로세스 등 외부 기술의 어댑터.
- `src/presentation`: Rust UI와 사용자 입력. 플랫폼 API를 직접 호출하지 않는다.
- `src/main.rs`: 조립과 실행만 담당한다.
- 정책 판단, COM 이벤트 처리, UI 렌더링, 플레이어 프로세스 제어를 한 모듈에 섞지 않는다.
- 공통 이름의 거대 `utils` 모듈 대신 기능의 책임을 드러내는 이름을 사용한다.
- 외부 URL과 웹 메시지는 신뢰하지 않는다. 브라우저 데이터로 임의 프로그램이나 셸을 실행하지 않는다.
- 변경 후 `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`를 실행한다.
- **2026-10-29 사용자 지시: 모든 문서는 specfold(`spec` CLI)로 저장소 루트 `.spec/`에서 관리한다.** 설계 정본은 `sot`, 결정은 `adr`, 작업은 `plan`, 검증 기록은 `evidence` 타입이다. 옛 `docs/spec/` 경로는 `.spec/_aliases.tsv`로 해석되며, 아래의 옛 `.spec/sots`·`docs/spec` 언급은 이 구조로 읽는다. 컨벤션 정본은 `sot:kinekino/conventions`, 상태 정본은 `sot:kinekino/project-status`, 사용법은 `guide:kinekino/docs-specfold`. 관측 사실과 설계 및 미검증 항목을 구별한다.
- Design, planning and maintenance documents are managed only with the spec CLI. Before changing code, run `spec context <path>`. Create documents with `spec new <type> --governs <path> --title ...` and change status or relations with `spec set`; never write a spec file name by hand. Before committing, run `spec check --staged` (or `cargo xtask spec-check`).
- **`.spec`도 단일책임원칙·모듈화·계층형 디렉토리 구조를 최상위 규칙으로 적용한다.** 공통 규격, 기능별 계약, 결정, 계획, 관측 근거, 디자인 자산을 책임별로 분리하고 링크로 연결한다.
- **UI/UX 설계의 최우선 품질 기준은 일관성이다.** 공통 프레임·토큰·아이콘·용어·상태·입력·저장 동작을 공유하며 화면마다 임의로 다르게 정하지 않는다.
- 제작 중인 이미지 시안의 기준 경로는 `.spec/artifacts/design/`다. 검토 버전·화면 ID·상태를 기록하고 이미지의 장식/오류를 기능 규격으로 승격하지 않는다.
- 현재 시안 작업을 완료하고 **사용자 승인을 받은 뒤** `.spec` 기존 문서 전체 재배치를 수행한다. 승인 전에는 전체 재배치를 실행하지 않고 책임별 목표 구조·기존→신규 경로 매핑·링크 검증·복구 계획을 준비한다.
