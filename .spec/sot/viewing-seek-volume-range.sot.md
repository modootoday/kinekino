---
id: sot:kinekino/viewing-seek-volume-range
title: Viewing seek·volume range 소비 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Viewing seek·volume range 소비 계약

2026-10-25. [일상 재생 제어](everyday-transport-contract.md)의 후속 소스 설계다. UX16 원본과 별도 overlay에 작성했으며 컴파일·기능·GUI 검증 전이다. 최신 product-viewing-range-r2/effective17 SHA `0B31EF741AA52E6D8182C6BEF00C72746630B66A4429F015C05B0A65FEDFF16E`, Meari nested r4 effective37 SHA `CDAB446C5496305EBD71EF154DC88FE53FAA600BA20D904FF15791E6E7141283`가 입력 핀이다. 기존 retirement/SDK 검증을 대체하거나 현재 중앙 gate에 자동 합성하지 않는다.

원 source17 `B7FC18A15BBF080EFF5014019FDDA49B85D5428407DFF7225C4AE337F1F58CB8`에서 Meari 소스 검토로 draft 함수의 미정의 props 참조를 발견했다. 원본을 보존하고 r2 최소 후계에서 disabled 게시를 props 생성부로 옮기고 원 시험에 disabled 게시3개 단언을 추가했다. 이는 소스 차단 수정이며 실제 컴파일 성공 관측이 아니다.

공용 Range 선언은 값·label·disabled·preview·draft Text·terminal Text를 받는다. draft는 로컬 표시만 바꾸며 SDK ticket과 자동 refresh 재시작을 만들지 않는다. 원 라이브러리의 실제 종료 이벤트만 terminal을 발생시키고 취소·원 export·pending queue·Completion의 소유권을 유지한다. 제품 terminal 어댑터는 원 draft와 현재 engine/artifact/duration 및 원 admission이 일치해야 SDK control을 인코딩한다. seek 좌표0..1,000,000은 Rust u128 산술로 μs에, volume0..1000은0..100%에 변환한다. SDK domain 검증을 재사용하며 관측값은 명령 접수만으로 변경하지 않는다. 기존 수동 시각:분:초 입력과 range 초안을 분리한다.

드래그 피드백은 full product root props의 엄격한 차이 검사를 통과한 값/표시 갱신에만 기존 export를 유지한다. 다른 prop·실 duration·engine·artifact·disabled·수동 버튼 captured 값 변경은 원 fresh export를 처음부터 선택한다. 실제 expanded root에서 알려진 여섯 span의 기존 단일 Text(String) 경로만 명시하고, 원 document의 workspace lease와 Meari의 구조/이벤트 guard를 사용한다. retained 갱신 실패는 원 첫 오류를 보존하며 같은 turn에서 fresh 경로로 우회하지 않는다.

작성된 자동 시험은 순수 좌표/typed payload·stale context·원 local queue/한 terminal ticket과 실제 제품 factory의 중첩 값/텍스트 및 callback 정합성을 다룬다. individual rustfmt13만 통과했고 등록·실행0이다. OS pointer/key·focus·접근성·실 미디어·전 화면 시각 일치 및 사용자 품질 승인은 미검증이다. 중앙 SDK/CLI와 Meari nested 검증 뒤 별도 제품 통합 및 정상 창 인수를 배정한다. 정확한 파일별 전임과 검증 경계는 private `runtime/development/product-viewing-range-r1/HANDOFF.md`를 따른다.
