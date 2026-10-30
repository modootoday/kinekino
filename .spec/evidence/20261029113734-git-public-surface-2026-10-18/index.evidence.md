---
id: evidence:kinekino/20261029113734
title: 로컬 Git 도입과 추적 허용 범위
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 로컬 Git 도입과 추적 허용 범위

후속 사용자 지시: Rust↔React 변환 라이브러리는 부모의 소스 추적 대상이 아니라 별도 저장소/서브모듈이어야 한다. 현재 `/tools/rust-react-bridge/`를 부모 ignore에서 명시적으로 제외해 공통 tools 허용 목록에 섞이지 않도록 했다. 독립 저장소의 commit·원격 URL과 실제 gitlink/.gitmodules 등록은 별도 단계다. 폴더에 `.git`이 있다는 이유만으로 서브모듈 등록 완료라고 부르지 않는다. 이 변경은 부모 index·원격·commit을 수정하지 않았다.

사용자 지시로2026-10-18 프로젝트 루트를 Git `main`으로 초기화했다. 원격·업로드·초기 커밋은 없다. 원래 nested engine repository는 변경하지 않았다. 사용자 전역 Git 설정도 변경하지 않았다.

`.gitignore`는 기본 거부와 명시적 허용 목록이다. root Cargo/README/AGENTS, 제품 Rust src/tests, Rust tool별 Cargo/build.rs/README/src/tests, 현재 제품의 include_*에 필요한 font/license/SVG/4개 PNG/필터/브라우저 script만 후보로 허용한다. `.spec`·`.agents`·runtime·scripts·모든target·모델/tokenizer fixture·실행 증거·미사용 데모 자산·환경/인증 키 파일은 로컬에 보존하면서 추적 제외한다. 문서 정본과 실행 증거를 삭제하거나 공개 저장소에 포함했다는 뜻이 아니다. 새 필수 자산을 추가할 때 허용 목록을 함께 검토해야 한다.

관측 시점 후보360개/22,954,430bytes였다. 민감 확장자와 대표 runtime/spec/target/model/미사용PNG 경로를 `git check-ignore -v`로 확인했다. 현재 허용된 Rust/TOML에서 대표 비밀키 패턴을 검색했을 때 파일이 검출되지 않았으나 포괄적 비밀정보·공개 적합성 감사나 향후 파일의 안전 보증은 아니다. `.gitattributes`는 Rust/설정/문서의LF와 PNG/TTF binary를 명시했으며 기존 working files를 대량 변환하지 않았다.
