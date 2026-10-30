---
id: adr:kinekino/20261029113522
title: 공식 encoder 참조 환경의 별도 pin과 추출 정책
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공식 encoder 참조 환경의 별도 pin과 추출 정책

상태: 채택, 2026-10-18.

## 배경

공식 Qwen encoder 비교에는 Torch CPU와 원본 Transformers/nagisa pin이 필요하다. 실제 Torch wheel은110859971B이고 torch_cpu.dll은262656512B여서 기존64MiB 취득·128MiB 추출 제한으로 처리할 수 없다. 과거 NumPy/분절 수치 증거의 환경을 수정해서는 안 된다.

## 결정

Rust domain에 고정 CPU wheel과26개 의존성의 버전·ABI·크기·SHA를 둔다. 일반 다운로드 한도를 늘리지 않고 정확한 Torch SHA를 가진 배포물에만 별도 archive 취득/추출 profile을 적용한다. 새57archive 환경을 구성하며 이전 manifest로 과거 archive를 다시 검증하고 구버전 Transformers/nagisa만 선택에서 제외한다. 공통 경로·CRC·중복 검사와 실패 보존은 공유한다.

원본 Python 배포물은 수정하지 않고 Rust PyO3/C-API adapter가 직접 호출한다. 새 Python wrapper·eval·module stub은 작성하지 않는다. DLL origin·private sys.path·원본 source Git blob·모델 strict load를 실제 검사하기 전에는 encoder 실행 가능 또는 parity 완료로 표시하지 않는다.

## 영향

추출에 더 많은 파일과 디스크가 필요하지만 사용자 host package와 기존 환경을 바꾸지 않는다. CPU 단일worker·admission을 유지한다. wheel 라이선스와 native 번들 조건은 모델 라이선스와 별도다. [실제 취득과 환경 구성](../../sots/verification/ai/forced-encoder-dependency-runtime.md)을 관측 정본으로 연결한다.
