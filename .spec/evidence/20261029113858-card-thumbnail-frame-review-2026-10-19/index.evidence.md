---
id: evidence:kinekino/20261029113858
title: CARD 썸네일 외곽 프레임 실행 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CARD 썸네일 외곽 프레임 실행 검토

2026-10-19. 기준은 `.spec/artifacts/design/v2/studio/cards.png`다. 전체 화면과 (234,350,185,187) 영역의 4배 확대를 대조했다. 확대 산출물은 `runtime/review-runs/card-thumbnail-r130-4x.png`다. 시안의 선택 외곽선과 이미지 사이에는 어두운 여백이 있으나 r130에서는 이미지 가장자리에 선택선을 그렸다.

## 변경

공용 `selectable_thumbnail`이 footprint·외곽 프레임·이미지·키보드 focus를 분리한다. CARD의132 footprint 안에 TIGHT4 여백을 두고 선택선2/radius5를 적용한다. 번호 badge는 inset 이미지에 정렬한다. 선택 여부에 따라 이미지 크기를 바꾸지 않는다. 작은 Taffy 측정 영역에서는 음수 크기의 inset을 만들지 않는다. 카드 데이터와 선택 정책은 화면이 소유한다.

## 실제 관측

기존 가상 모니터에서 r131 `player.exe` PID108216을 실행했다. VM·DPI·모니터 구성은 변경하지 않았다.

- 1440: 세 thumbnail 모두 이미지와 외곽 프레임 사이 여백을 확인했다. 두 번째 thumbnail의 이미지 바깥 오른쪽 여백 (322,380)을 클릭해 선택2, canvas 제목·inspector 제목·2/3이 함께 바뀌었다. End로3을 선택해 외곽 amber와 별도 내부 focus 및3/3을 확인했다.
- 760×480: 가로 thumbnail strip에서 첫 thumbnail 오른쪽 여백 (190,235)을 클릭하고 Right로2를 선택했다. 선택 전후 크기 변화 없이 외곽선과 내부 focus가 구분됐다. 설정 drawer를 열고 Escape로 닫은 뒤 선택2가 유지됐다. drawer 안 제목 필드까지 스크롤해 확인하지는 않았다.
- Empty: thumbnail·선택선·번호·페이지 counter가 사라지고 빈 source 안내가 표시됐다. 연결되지 않은 toolbar는 비활성을 유지했다.
- caption으로 검토 player를 종료했다. 이후 CIM 조회에서 player.exe/mpv.exe가 없었다.

## 검증과 남은 범위

fmt 및 전체 cargo test 재시도는 통과했다. 최초 테스트는 SDK 병행 수정 중 `LocalHost::is_running` 부재로 컴파일 실패했으며 해당 소스를 수정하지 않고 소유 세션에 전달했다. 소유 세션 변경 후 같은 UI target의 전체 테스트가 통과했다. clippy/build도 수행했으며 최종 clippy 결과는 아래 후속 기록을 따른다.

최종 `cargo clippy --all-targets --target-dir runtime/parallel-build/ui -j 1 -- -D warnings`는 현재 공유 소스에서 exit0으로 통과했다. 전체 테스트 재시도 뒤 추가 Rust 수정은 없다.

이번 근거는 선택 프레임과 해당 입력 표본에 한정한다. LongLabels·980/1180·hover/pressed 전체 상태·스크린리더·임의 비율·실제 카드 문서·200카드 스크롤은 미검증이다. 공통 toolbar 면과 전체 inspector 등 CARD 전체 시각 일치를 완료로 표시하지 않는다.
