---
id: evidence:kinekino/20261029114444
title: R42 프레임 의존성 탐색 오류 보존
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R42 프레임 의존성 탐색 오류 보존

2026-10-28 중앙 PM. R41 review shard2에서 frame 지원이 달라진 사실을 유지한다. 이전 실패 원인이 Timeout이었다고 소급 단정하지 않는다. 현재 AGENTS SHA40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C를 확인했다.

## 실제 선택 후보와 핀

R42 후속 verify-current는 동일 SOURCE 핀으로 terminal0·2298 content/identity unchanged를 반환했다(직접 handle35737).1318에서 snapshot과 compile만 읽기 중첩했고 compile/strict 무거운 작업은 직렬이었다. 별도 custody runner receipt가 있는 것으로 주장하지 않는다.

후속1320 Native all-target strict terminal0/23227ms,1321 전체fmt terminal0/3720ms.1319 default strict terminal0/17909ms. 이들은 R42 입력의 실제 실행이며 전체190/bins 성공을 뜻하지 않는다.

현재 PATH의 실행파일은 `%USERPROFILE%/AppData/Local/Microsoft/WinGet/Packages/Gyan.FFmpeg_Microsoft.Winget.Source_8wekyb3d8bbwe/ffmpeg-8.1.1-full_build/bin` 아래 ffmpeg.exe SHA09948D4CDD0650DA6FF5A87577469F2A218DC2615AE379F8F734D24C49DE0F73 및 ffprobe.exe SHAA6618E99BB58869DED3C6F37B53AA1A8D701C3591DBB7B5B317D47369C112BE2다. executable 인접 debug/ffmpeg.exe 및 Chocolatey 후보는 조회에서 발견되지 않았다. 이는 현재 후보 관측이며 R41 실행 당시의 파일 식별을 소급 증명하지 않는다. 설치/다운로드/전역PATH 변경 없음. 보존된 ffprobe6.1 핀과 별개다.

## 변경과 근거

mirror `src/infrastructure/frame_capture/mod.rs`의 원본은 `runtime/verification/preimage-r41-frame-discovery/mod.rs`, SHA26780451858A281619966FE63257C73789112C8BDC0389AD163986D167752490. 새 책임 모듈 `discovery.rs`는 stage·ApiError code만 보존하고 백엔드 원문/상세를 공개하지 않는다. version의 invalid UTF-8/empty response도 구분한다. 기존2초·도구 선택 순서·지원 판정·실제 캡처/저장 정책은 유지한다. features 두 검사 오류를 모두 수집한다.

변경 mod SHA6D85E758A1B1894C0AD02A173DCF0FB86FAB79153EFCBBE1EC828F5601B0933E, discovery SHAD6DF48DC6D863B02C2C2DA873109823A0D6A64D1CCEC85B9161CFF72079C0721.

R42 SOURCE E4D59F694483673524CA3155C45945C56F67A202C870B49C3F41794420E576F9, inventoryAF3CF8C6048F7AC54EF592163FB9C2DA8D620A9F11E87AE68EAF39506F1E2CC3/2298. snapshot은 no-run을 겸한1318 compile 진행 중 읽기 전용으로 생성했으며 compile 시작 후 소스 변경 없음. private mirror이며 canonical 채택 아님.

1318 default lib의 새 diagnostic2시험 통과/934filtered, runner terminal exit0/60565ms. Timeout의 stage/code 보존·비밀 원문 비노출 및 invalid UTF-8와 Timeout 구분을 검증했다. 실제 FFmpeg 실패 재현·정상 capture·새CLI 실행 근거는 아니다. 1319 default all-target strict는 실제 terminal0이다. Native strict/fmt·후속 custody 및 전체190/정상bins는 실제 결과 확인 후 추가한다. 원R40의190/bins 성공을 R42로 승계하지 않는다.

다음은 실제 탐색 오류 관측, 명시 경로/해시 강제 및 사용 전 재검증 계약이다. 현재 제품은 여전히 자동 경로 탐색을 사용하므로 이 변경만으로 도구체인 고정 완료가 아니다. 전체SDK/CLI·GUI/AI 품질 인수는 미완료다.

modootoday/software-qa/test-fixture-and-mock-build SHA A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100의 통제 오류/실제 도구 구분과 code-change-review-verification SHA9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B의 원 실패·scope 보존을 적용했다. root/WIP·담당 소유권·추론/다운로드/수동GUI/WARP/push 보류 유지.
