---
id: sot:kinekino/jobs-native-batch-journal
title: 동일 Jobs 루트에 묶인 native batch journal
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 동일 Jobs 루트에 묶인 native batch journal

R131 입력:2200fmt검사서식차이 실패/2201fmt종료0 및2202snapshot종료0,2379파일/inventory A561FB506FB2064C46854A1A7A475EF3A147FA795593FC81A470E049041A7C95,SOURCE.md149D5C7063ACFC149186C8E6F9B012C2CDEC1FAD92B5DC83C42FE7C76997DBBA.2203 기본 no-run 진행 중이다. 실제 파일시험·strict/190/bins/custody는 미완료이며 기존 첫실패·원 증거·최신완료기준선R129 유지.

R130 첫컴파일2199 종료실패: 기존content_digest::sha256(reader,maximum)의 API를 bytes→String 함수로 잘못 호출해3호출에서6개형식오류가 발생했다. 원로그/input-r130 및 preimage-r130-batch-hash-call을 보존하고 R131에서 sha2의바이트SHA256을 native journal 모듈 책임으로 계산하도록 수정했다. 실제파일시험은 미실행이며2200fmt/새snapshot/재컴파일을 진행한다.

고정 입력:2198snapshot 종료0,2379파일/inventory4E845BEEBF7FCF8AFBDFF086B3A950A5284D03BB06790A1FC0459C3C026B6F87,SOURCE.md337E1DCF29757C5D27ABD481E09A83F1B7CD5070ADC371BE6D40E703EF032FFA.2199 기본 lib/Jobs/CLI no-run 진행 중이다. 실제 파일시험·strict/190/bins/custody는 아직 미완료이며 최신 완료기준선 R129를 유지한다.

R130 구현 중이다. BatchFiles::open은 host가 지정한 원 Jobs 루트의 고정 ancestor/root 핸들과 batch 하위 디렉터리 핸들을 보유한다. scope는 versioned namespace와 두 디렉터리의 실제 volume/FileID 관측을 해시한 값이다. 요청이 scope나 root를 선택하지 않으며 같은 경로의 다른 디렉터리/다른 호스트 복사본을 같은 origin으로 승인하지 않는다. host가 원 Jobs API와 같은 root로 조립하는 연결은 아직 미구현이다.

원 Jobs 루트의 batch-writer.lock을 기존 native share_mode0 lock으로 독점한다. create/lease load/publish/원Jobs 호출 중 같은 writer lease를 유지한다. 최초 기록은 validate_initial을, 후속 기록은 원 validate_successor와 revision CAS를 검사한다. 디렉터리 생성 실패/불완전한 최초생성은 원 증거를 보존하며 자동 정리·재개하지 않는다.

최초 이벤트에만 고정 plan/request/document를 저장하고 후속 이벤트에는 변경한 항목 index/progress와 parent SHA를 저장한다. 이벤트 envelope는 schema/scope/batch ID/revision/parent를 포함하며 canonical bytes와 파일명 SHA·연속 revision·부모 해시·원 상태 전이 정책을 재열기에서 검증한다. 전체24MiB 문서를 매 revision 파일로 복제하지 않는다. 이벤트1MiB/디렉터리 항목65536 상한을 둔다. 조회는 전체 이력을 검증하므로 긴 이력의 성능 검증은 남는다.

발행은 create_new 원 파일 핸들로 complete bytes를 write/sync한 뒤 ReplaceIfExists=false의 FILE_RENAME_INFO로 새 최종이름을 만든다. writer/원 directory/file handle을 유지하고 기존 이벤트를 덮어쓰지 않는다. 실패한 .pending-* 조각은 보존하며 조회에서 최신 이벤트로 사용하지 않는다. 잘못된 최종 파일·gap·중복·부모/해시/scope 불일치는 NeedsInput으로 거절하며 부분 최신 문서를 반환하지 않는다. 장치 전원 손실 내구성이나 하드웨어 원자성까지 검증한 것은 아니다.

preimage-r129-batch-native-store에 infrastructure mod 원본을 보존했다.2196/2197fmt 종료0,2198snapshot 진행 중. 실제 native 시험은 원 자막capture로 만든 문서의 create/reopen·CAS·writer 독점·delta 기록·다른 root 복사거절·미발행조각보존·부분최종파일거절·원host shutdown을 검사한다. 아직 컴파일/통과로 계상하지 않는다. worker부분실패/SDK·CLI/GUI·원AI승인0/9·기존소유권/WIP·보류 유지.

기존 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용해 실제 OS파일검증과 synthetic포트·미구현 host 범위를 구분한다.
