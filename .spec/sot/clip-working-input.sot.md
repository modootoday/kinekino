---
id: sot:kinekino/clip-working-input
title: 숏폼 검증·자막의 작업 입력 소유권
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 숏폼 검증·자막의 작업 입력 소유권

이 계약은 private overlay의 새 검증용 MP4 및 자막 PNG sequence에 적용한다. 원 미디어, 기존 임시 파일, 게시 산출물, 원 프로세스 영수증은 삭제 대상이 아니다.

`ClipWorkingFile`은 고정 C 캐시 tmp 아래 CREATE_NEW로 만든 파일의 원핸들을 소유한다. 파일 식별과 바이트 해시를 관측하고, 쓰기는 최대 한 번 시도한다. 부분 취소 이후 같은 파일에 재시도하지 않는다. 봉인 뒤 API는 바이트 수정을 제공하지 않으며 외부 쓰기·이름 변경을 차단한다.

읽기 핸들은 원 파일과 동일한 실제 식별인지 확인하고 독립 커서를 사용한다. 원 process launcher는 해당 regular file의 읽기 전용 핸들만 자식 stdin으로 전달한다. MP4는 mov demuxer와 fd protocol로 probe 및 전체 decode를 수행한다. PNG는 총 64MiB 이하의 연결된 sequence를 image2pipe로 읽고 split/select로 각 run 이미지를 선택하며 원 source frame의 반열린 표시 구간을 유지한다. 경로별 PNG 재열기는 사용하지 않는다.

원 작업 파일에는 DELETE_ON_CLOSE가 설정된다. 원 process supervisor는 자식 종료·family empty·영수증 기록이 정리되기 전에 실행 완료를 반환하지 않는다. 입력 소유자와 마지막 reader가 해제되면 해당 새 파일이 삭제된다. 부분 spawn 실패에서도 이미 생성된 원 프로세스 소유권을 정리 경로에 유지한다. 영수증 실패를 성공이나 취소 완료로 바꾸지 않는다.

실제 입력 파일 수명과 정리 근거는 [R185/R186 검증](../../../verification/application/clip-working-input-20261029.md)으로 추적한다. 이 계약만으로 전체 Jobs 취소·ACK 유실·재시작/재시도·게시 중단 파일의 정리가 완료되지는 않는다. `partial_cleanup_supported:false`, 기존 보류와 사용자 AI 품질 승인 경계를 유지한다.
