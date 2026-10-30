---
id: evidence:kinekino/20261029113555
title: 실제 카드·세로 클립 제작 스파이크
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 카드·세로 클립 제작 스파이크

2026-10-18. 모델 결과 문구를 사용하지 않고 실제 배포 자막과 해당 구간 프레임으로 수동 제작 경로를 검증했다. 제품 GUI 편집기나 durable job 기능 완료를 뜻하지 않는다.

## 입력·모듈 책임

원본 Sintel 영상과 한국어 배포 자막의 해시는 [공통 제작 검증](production.md)의 원본과 같다. 카드1은 cue1/110초 프레임, 카드2는 cue2/113초 프레임이다. 한국어는 배포된 번역 자막 인용이며 원음 ASR 정답으로 사용하지 않는다.

Rust domain의 `production`은 선택 구간과 cue 교차·원점 이동·SRT 시간을 담당한다. infrastructure의 `production_export/cards`는 이미지/문구 배치, `short`는 출력 계약, `ffmpeg`는 명시적 외부 프로세스 호출을 담당한다. `typography`는 ab_glyph로 글꼴·줄바꿈·지원 글리프·넘침을 처리한다. Python/JS/PowerShell 제작 코드는 추가하지 않았다.

## 실제 출력과 수정

첫 실행은 [84836 manifest](../../../evidence/ai/model-spikes-2026-10-18/results/production-export-84836/manifest.json)에 보존했다. MP4를 끝까지 디코딩했지만 자막이 너무 크게 영상을 가렸고, FFprobe에서 챕터 유래 bin_data 트랙까지 총3개 stream을 확인했다. Windows OsStr 파일명도 JSON 문자열 대신 Windows 정수 배열로 직렬화됐다.

이후 폰트 크기·여백, 명시적 metadata/chapter 제외, UTF8 파일명 직렬화를 수정했다. [수정 후76968 manifest](../../../evidence/ai/model-spikes-2026-10-18/results/production-export-76968/manifest.json)와 probe·이미지를 별도 보존했다. [산출물 색인](../../../artifacts/spikes/2026-10-18/README.md)을 따른다.

| 산출물 | 직접 확인한 내용 |
| --- | --- |
| 2장 카드×2비율 | 1080×1080,1080×1350 PNG 총4개. 두 장의 선택 프레임·인용 문구·cue/시각·출처·페이지 번호를 같은 템플릿으로 렌더 |
| 세로 클립 | 원본107.250~119.250초 → 720×1280,12.000초,24fps·288프레임,H.264 yuv420p/AAC의2개 stream, 시작0초 |
| 자막 | cue1~3을 출력 기준0초로 이동. 마지막 cue3의 끝은 클립 경계에서 잘려 표시1.25초이며 읽기 품질은 별도 미승인 |
| 유효성 | FFprobe 크기/형식/길이/프레임 수 검사와 전체 FFmpeg null 디코딩 통과. 수정 미리보기에서 영상 가림 개선·한글 표시 확인 |

최종 MP4 SHA256은 `0fd9b2de74774b4d2dcea9f9b66466899b44aff1300b121d548e10af4c386483`다. 첫 버전은 삭제하지 않는다. 수정 후 워커의 로드 포함 wall 약7.50초, 100ms 표본 최대 private bytes464,703,488이었다. 한 번의 실행이며 빌드와 병행해 성능 순위나 노트북 보장으로 사용하지 않는다. 측정 전 CPU 약12.79%, 가용 RAM 약37.9GB, worker1개/BelowNormal/FFmpeg2threads를 적용했다.

## 남은 범위

로컬 설치 Malgun Gothic 글꼴을 사용하고 경로/해시만 기록했으며 글꼴 파일을 복사·번들하지 않았다. 현재 폰트·팔레트·배치는 연구용 템플릿이고 최종 UI 시안 승인이나 상용 폰트 재배포 권한을 뜻하지 않는다. 캔버스 크기는 앱 제작 프리셋 실험이며 플랫폼 규격·업로드 성공을 검증한 것이 아니다.

세로 출력은 원본 비율을 보존한 letterbox다. 자동 피사체 crop·추적·음악 선정·추천 구간·일괄 큐·트랜지션·플랫폼 게시를 실행하지 않았다. 전체 영상을 디코딩했지만 이 단계에서 음성을 직접 청취해 A/V sync를 승인하지 않았다. 원본 요청 PTS와 실제 디코딩 프레임의 오차 한계, 마지막 잘린 자막의 읽기 시간, HDR/회전/다중 트랙은 추가 검증 대상이다.

편집 가능한 공통 프로젝트 저장/불러오기, 게시문·대체텍스트 초안 묶음, 재렌더 왕복과 SDK·CLI·GUI 통합은 남아 있다. 제품 API가 이 도구의 렌더 기능을 지원한다고 표시하지 않는다. Rust unit7개·fmt·Clippy 검증은 도구 범위이며 제품 실제 기능 통합과 구별한다.
