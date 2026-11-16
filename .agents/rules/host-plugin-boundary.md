---
paths:
  - "crates/**"
  - "src/**"
---
# 호스트·플러그인 경계 [AGENTS.md, ADR 20261030200000-adr-0011-media-source-plugin, sot:kinekino/conventions]
- 호스트 코드에는 사이트별 용어를 쓰지 않는다. 쓰는 말은 "하위 목록이 있는 항목", "재생 가능한 항목"뿐.
- 시리즈·에피소드(회차) 해석, 사이트 탐색은 플러그인이 맡는다. 비공개 플러그인은 저장소 밖에 있으며 저장소에서 참조하지 않는다.
- 계층: domain(정책) / application(유스케이스·포트) / infrastructure(어댑터) / presentation(UI). 상세 `sot:kinekino/conventions`, 검사 `cargo xtask lint-structure`.
- 외부 URL·웹 메시지는 신뢰하지 않는다. 새 개발 코드는 Rust만.
