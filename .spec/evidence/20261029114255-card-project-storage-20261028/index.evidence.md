---
id: evidence:kinekino/20261029114255
title: 카드 프로젝트 저장의 실제 파일 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 카드 프로젝트 저장의 실제 파일 검증

최종 1927 custody 종료0: current source content/identity unchanged files=2344. 고정 R101 입력과 실제 시험 이후 입력이 동일하다.

2026-10-28 R101 중앙 PM. [카드 저장 계약](../../player/application/production/card-project-storage.md), [카드 편집 문서](../../player/application/production/card-document.md), [구현 배정](../../../plans/player/production/remaining-executors-20261028.md). 공통 application CardProjectStore/create/save/inspect/prepare_publication과 Windows 파일 어댑터를 추가했다. 기존 ProjectFiles schema1/2·UI/미디어/Meari 담당 파일은 변경하지 않았다. application/mod.rs·infrastructure/mod.rs preimage는 mirror/runtime/verification/preimage-r100-card-storage에 보존했다.

R101 snapshot SOURCE.md SHA256 C9A5E94C40670B846D2173C34A2185613796BAD456E8B4DE7D47714A53AD9EAC, inventory 698F9B1FA0470E1E5D9C4D24A520C4E54E8C88C630BF052FE58FDDFDCA4D623F/2344. mirror/runtime/verification/input-r101에 고정했다. 도구의 일반 canonical snapshot 제목은 이 private 후보의 채택을 뜻하지 않는다. 기존 AGENTS/lock·C-only cache/tmp·CPU2/BelowNormal/heavy1/locked/offline/-j1 유지.

로그 루트 runtime/verification/source375-datalab-b0-r1-logs: 1921fmt0, 1922snapshot0, 1923 Native all-target strict0, 1924 실제 파일 저장 선택6시험 통과/1586filtered/0실패(시험0.05초, compile 포함91135ms), 1925 default all-target strict0, 1926 fmtcheck0. Native는 -p kinekino-player --features native-proposal-window이며 각각 live handle terminal을 확인한 뒤 다음 작업을 시작했다. 이번 단계의 첫 실패는 없으며 이전 R99 strict 실패는 별도 원 기록에 보존한다. transitive blitz-paint deprecation 경고는 별도다.

시험 exe/PDB는 runtime/build-cache/native/source-sdk-integration-r1/fixed-r101에 보존했다. kinekino_player-37bec848ae83f5da.exe SHA256 A764845EE34114458992CA7DEA71E09EEBE00F50553B311B6B82122ECF78EC56.

실제 증거 파일은 runtime/verification/source-sdk-r101-card-storage-r1의 card-project-{history,writers,rejections,corrupt,partial,boundaries}-22240에 보존했다. history는 발행1/편집1→발행2/편집4·부모 hash·새 store 인스턴스의 재열기·과거 조회·원 record bytes 불변을 검사했다. writers는 barrier로 경쟁하는 두 thread/두 store에서 성공1·RevisionConflict1을 검사했다. 별도 프로세스 경쟁까지 실행한 것은 아니다. rejections는 기존 ID create·stale head·같은 edit revision·source cue 변경의 거절과 새 revision 부재/원 record 불변을 검사했다. corrupt는 합성 최신 record를 잘라 원 bytes를 따로 보존하고 과거 조회조차 NeedsInput이며 손상 bytes가 유지됨을 확인했다. partial은 빈 r-2가 삭제/덮어쓰기 없이 조회·저장을 막는 것을 확인했다. boundaries는 상대 root/경로 이탈 ID·활성 writer lock 거절과 이후 정상 create를 확인했다.

불완전 발행은 복구 필요 오류이며 cleanup 성공이 아니다. 실제 power-loss·disk-full/권한 fault 주입·cross-process writer·서명/인증·지속 undo·멱등 ACK 복구·렌더러·Jobs·typed PlayerSdk/CLI는 미검증/미연결이다. production.export_cards는 계속 미지원이다. R101 전체190·정상2bins는 미실행이며 R92 지정 gate를 승계하지 않는다. 원 AI9는 NeedsRepair·사용자품질0/9다. root WIP/기존 소유권/원 증거/private overlay와 WARP/추론/다운로드/수동 GUI/push 보류를 유지한다.

기존에 읽은 modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100), code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을 적용했다. 실제 저장 파일과 실패 상태를 보존하고 unit/파일 검증을 제품 CLI·사용자 품질 인수로 확대하지 않았다.
