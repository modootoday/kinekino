---
id: evidence:kinekino/20261029114010
title: 데모 검토 크기 표시 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 데모 검토 크기 표시 검토

2026-10-19. 기준은 제품 시안의 새로운 기능이 아니라 [r113 실제 관측](studio-caption-cycle-review-2026-10-19.md)의 검토 host 혼동이다. 최대화된3840×2112 창에서도 선택기760×480이 남아 실제 현재 크기로 오해할 수 있었다.

## 변경

일반 상태는 egui content rect의 현재 논리 크기, 최대화 상태는 ‘최대화’를 표시한다. hover에는 현재 논리 크기와 마지막 preset을 구분한다. 메뉴 제목은 ‘검토 창 크기 프리셋’이다. 같은 preset을 선택해도 복원/크기 요청을 수행한다. 기존 ComboBox ID·caption hit 영역·시나리오 설정·모니터/DPI는 변경하지 않는다. 물리 pixel 측정값이나 DPI 검증 결과로 표시하지 않는다.

## 실제 실행

r116 PID97764,최종 빌드.1440×900에서 제목줄1440×900과 메뉴의 ‘검토 창 크기 프리셋’을 확인했다.980 선택 후 창980×760과 선택기980×760이 일치했다. 최대화 후 안정된 캡처3840×2112/원점3440,0에서 선택기가 ‘최대화’로 표시됐다. 메뉴의 기존980 항목을 다시 클릭하자980×760/원점3500,100으로 복원됐고 표시도980×760으로 갱신됐다. 이 캡처는 현재 고정 배율에서의 일치 확인이며 다른 배율 검증이 아니다.

Tab은 제목 링크→회차 예시→시나리오→크기 선택기로 진행하며 focus outline이 표시됐다. Space로 크기 메뉴를 열고 Escape로 닫을 수 있었다. hover 도움말의 실제 표시·키보드 항목 변경은 이번에 확인하지 않았다.

caption 닫기로 종료 후 player.exe/mpv.exe 조회가 비어 있었다. 기존 가상 모니터 위치3500,100을 사용했고 VM·DPI·모니터 구성은 변경하지 않았다. 최종 cargo fmt --check, cargo test(lib71 passed/1 ignored와 CLI/통합), cargo clippy --all-targets -- -D warnings, cargo build --bin player가 통과했다. 외부 미디어 의존 ignored 검사는 실행하지 않았다.

## 미검증

일반 제품 창·HMR 도구의 크기 UI를 변경한 것이 아니다. OS edge drag·다른 배율·다중 viewport·screen reader·모든 입력/전환 조합과 장시간 성능은 별도다.
