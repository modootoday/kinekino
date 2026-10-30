---
id: sot:kinekino/models
title: 모델과 자원 관리
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 모델과 자원 관리

구성·상태별 형태는 [요소별 시각 계약](models-visual.md)을 적용한다.

## 시각화

- [HF 모델과 양자화](../../../../artifacts/design/v2/studio/models-refined.png)

이미지는 배치 참고이며 아래 요소 계약과 공통 필드·상태 규칙이 구현 기준이다.

상태: UI/UX 설계 승인(2026-10-18). 생성 이미지 디테일은 실제 Rust 공통 컴포넌트로 검토한다. 구현 완료와 구분한다. [공통 계약](../sot-020-ui-contract.md)의 프레임·입력 검증·상태·접근성·저장·Rust 계층 규칙을 상속한다. 기본/hover/pressed/selected/disabled/focus는 모든 조작 요소에 적용한다.

## 요소 계약

| 요소 ID | 요소 | 표시와 동작 |
|---|---|---|
| MODEL-01 | 검색/필터 | ASR/text/OCR/피사체/depth 및 speaker_segmentation/speaker_embedding 등 작업 종류·설치 상태·엔진·장치별 후보 |
| MODEL-02 | 저장소 | HF repo·revision·파일목록, 명시적 allowlisted 포맷 |
| MODEL-03 | 라이선스 | 링크·용도·허용 여부·검토 상태 |
| MODEL-04 | 정밀도 | FP16/Q8/Q4 등 실제 지원 artifact만, 호환 미상 비활성 |
| MODEL-05 | 용량 | 파일 bytes·추가 캐시·확인 시점, 가중치와 실행메모리 구분 |
| MODEL-06 | 장치 | 자동/CPU/검증 GPU, backend 지원 여부 |
| MODEL-07 | 다운로드 | 필요한 파일만·진행·취소·해시 검증·원자적 준비 |
| MODEL-08 | 게이트 모델 | 약관·로그인은 사용자 처리, OS credential 참조 |
| MODEL-09 | 준비/해제 | 사용 작업 표시, idle unload, 준비 실패 이유 |
| MODEL-10 | 제거 | 사용 중이면 차단/대기, 모델과 cache 경로/크기 요약 |
| MODEL-11 | 절약 모드 | 무거운 작업1개·재생 우선·초안 설정 |
| MODEL-12 | 예산 | RAM/VRAM/worker limit, 실제 가용량과 측정 여부 |
| MODEL-13 | 품질 비교 | 같은 입력·revision·양자화·품질·시간·메모리 함께 |
| MODEL-14 | 진단 | 비밀 없는 요약, remote code 실행 금지 |
| MODEL-15 | 기본 모델 | 작업별 선택, 앱 시작 필수 로딩 아님 |

## 상태와 오류

미설치·다운로드·검증·준비·사용 중·오류·지원 안 함을 구별한다. Q4라는 이름만으로 메모리 절약을 보장하지 않는다. 다른 앱 종료·전원/보안 설정 변경은 이 화면의 동작에 포함하지 않는다.

## 저장 계약

브랜치/태그를 검색 입력으로 받더라도 실제 파일 준비 전에 immutable commit revision으로 resolve하고 표시·저장한다. main 같은 가변 ref 자체를 설치 고정 revision으로 보관하지 않는다.

ModelRef(repo,revision,files[{path,hash,size}],license,format,quantization,engine,device_constraints), ResourceProfile and measured evidence separately.

## 인수 기준

손상 파일·디스크 부족·revision 변경·미지원 GPU·양자화 호환·중단 복구·사용 중 제거.

각 요소 ID의 유스케이스는 application에, 검증 정책/값 객체는 domain에, OS·추론·파일·프로세스는 infrastructure에 분리한다. presentation은 입력과 렌더링을 담당한다. 모델 미선정/encoder·플랫폼 지원 여부 미확정 항목은 완료로 표시하지 않고 해당 기능을 비활성 또는 NeedsInput으로 제공한다.

