---
id: sot:kinekino/manifest
title: 자산 묶음 출력과 manifest
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 자산 묶음 출력과 manifest

[프로젝트 입력](asset-projects.md)과 공통 raster를 소비하는 application 출력 계약이다. 원 Jobs/SDKCLI 렌더 지원과 디스크 publication 성공을 별도 인수한다.

## 원 보유자와 순서

CapturedAssetProject는 하나의 원 lease/plan 보유자를 Arc로 공유한다. wrapper clone은 같은 핸들 identity를 공유하며 JSON 역직렬화나 독립 capture로 대체하지 않는다. NativeAssetProjectLease 자체는 Clone/Deserialize가 아니다. Rendered/Encoded/PreparedAssetBundleOutput은 borrow 수명에 묶이지 않은 소유 값이며 원 프로젝트와 source lease를 최종 결과가 해제될 때까지 유지한다. 실제 concrete 출력이 Send+'static인지 컴파일 검사한다. 별도 capture에서 만든 결과와 계획 순서가 다른 unit은 builder가 거절한다.

plan/2는 선택 MissingAssetFramePolicy를 명시적으로 보존한다. 과거 plan/1 증거를 바꾸지 않는다. 원 frame 위치·gap 이유·source clock/PTS·visible layer 순서를 유지하며 숨긴 unit이나 누락 unit을 채운 것으로 표시하지 않는다. finish는 모든 계획 unit이 순서대로 있어야 가능하다.

## 파일과 alpha

unit마다 원 raster RGBA8, straight RGBA PNG, 변환된 Gray8 alpha/coverage PNG를 제공한다. 파일명은 frame position/unit index의 고정 숫자로 만들며 frame/layer ID를 경로로 사용하지 않는다. 원 input mask 파일과 출력 alpha/coverage 파일은 다르다. mask PNG에는 sRGB/gAMA를 붙이지 않는다.

raw RGBA8은 요청된 straight/premultiplied 버퍼를 그대로 보존한다. PNG는 straight sRGB8이며 premultiplied sRGB8을 unassociate할 때 저 alpha 양자화가 있음을 png_conversion에 명시한다. alpha0은 RGB0이다. encoder는 실제 PNG decode round-trip을 확인한다. 이 결과를 실제 측색/색 승인으로 확대하지 않는다.

project.json은 원 publication의 canonical bytes이며 원 project pin SHA와 같아야 한다. manifest.json은 원 plan·입력 image/mask/review pin·geometry·UV·pivot·raw/PNG alpha·변환·색 선언·unreviewed warning과 각 파일의 실제 bytes/SHA/media type을 기록한다. quality_approved/colorimetry_verified는 false다. manifest 자신의 SHA는 외부 파일 descriptor로 제공해 순환 hash를 만들지 않는다. 원 input PNG/mask는 이 단계에서 복사하지 않으므로 project.json만으로 self-contained 재편집을 약속하지 않는다.

현재 출력 파일 합계512MiB와 계상한 source/raster/encoded buffer512MiB 한도를 적용한다. metadata/allocator/encoder 임시 메모리를 포함한 RSS 보증이 아니며 경계 실측은 별도다. 허용 unit을 조용히 생략하지 않고 ResourceExhausted로 거절한다. PNG codec 자체 안에는 취소 hook이 없어 전후 checkpoint로 검사한다.

## 다음 연결

native staging은 이 opaque output의 파일을 create-only로 작성하고 실제 readback/hash/원 directory identity를 확인한 뒤 원 Jobs의 단일 publication CAS와 연결해야 한다. source blob/attempt/host pin·발행 후 조회/ACK 불확실성·취소/release·중단 복구·source bundle/self-contained 재편집·sequence/투영·SDKCLI/GUI 인수는 계속 남는다. manifest DTO나 파일 생성 성공을 Jobs 완료 proof로 쓰지 않는다.

기존 modoo.today QA fixture/변경 검증 및 image-postprocess의 원본 불변/출력 규격/재검증 판단을 적용한다. 정확한 출처·SHA는 [원 입력 검증](../../../verification/application/asset-input-admission-20261029.md)과 공통 자산 계약을 따른다. 새 정책·테스트·codec은 Rust로 작성한다.
