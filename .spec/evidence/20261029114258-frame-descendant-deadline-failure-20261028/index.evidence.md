---
id: evidence:kinekino/20261029114258
title: "R47: 프레임 프로세스의 후손 파이프 기한 실패"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R47: 프레임 프로세스의 후손 파이프 기한 실패

후속 [R50 원 family 구현](frame-owned-family-20261028.md)에서 같은 후손 사례가2.02초에 통과했다. 아래30.04초 실패는 원입력/producer/raw와함께보존하며 R50의실행중취소·불완전정리전수인수로확대하지않는다.

2026-10-28 중앙 PM. 제품 helper의 기한 초과를 실제 Rust 프로세스 시험으로 재현했다. 시험은 실패이며, 외부 실행기가 정리를 마쳤다는 사실로 제품의 기한 보장 성공을 주장하지 않는다.

## 입력·원본 보존

제품 동작은 수정하지 않고 `src/infrastructure/frame_capture/process.rs`의 시험 모듈에 후손 fixture와 명시 ignored 재현 사례를 추가했다. 기존 파일은 `runtime/verification/preimage-r46-frame-descendant-deadline/process.rs`, SHA `96F83BA0C57DE3EB6BB9E0E37CCEEDA4DED04265A2170F9A358964C259978ACE`에 보존했다.

R47 SOURCE `7C488C64C0B944BCC30B4DE1A24ACEEA08585F1277E3924DDC0F09206C05331E`, inventory `7DB89DA43FFFFECFC2BF131D76EB185719DD7D05BD3C5AA2FF593316F049A91C`/2302. snapshot은 compile 중 읽기 전용으로 생성했으며 이후 소스 변경이 없다. AGENTS `40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C`를 확인했다.

1380 Native lib no-run terminal0/69879ms. fixed-r47/frame-process-tests.exe SHA `BCD13101EF75B8BCFBD463BFE5EE6E23BD1E91094DA235E59F58B2BDDCB83118`/191375872 및 대응 PDB를 보존했다. 두 후손도 같은 고정 Rust libtest를 실행한다. 실제 FFmpeg나 미디어 처리 시험으로 표시하지 않는다. operator index `source-sdk-r47-frame-deadline-index.json`은 proof=false다.

## 실제 실패

1381은 `infrastructure::frame_capture::process::tests::deadline_with_inherited_pipe_descendant_remains_bounded`를 정확히 선택하고 ignored=true로 실행했다. helper의 원 기한은2초, 허용 반환 단언은5초 미만이다. fixture 부모가 파이프를 상속하는 후손을 실행하고 후손은 파일을 독점 보유한 채30초 동안 살아 있다.

실제 호출은 **30.0431441초** 뒤 반환했다. Timeout 오류 자체는 반환했으나 기한 단언에서 실패했다. 결과는 **0pass/1fail/1553filtered**, runner terminal1/46208ms다. `should_panic`으로 바꾸거나 기대한 실패라는 이유로 통과 수에 넣지 않았다.

원 raw와 C2 outcome은 `runtime/verification/source-sdk-r47-frame-descendant-deadline-os1/outer`, runner 로그는 `source375-datalab-b0-r1-logs/1381-source-sdk-r47-frame-descendant-deadline.*`에 있다. fixture lock 파일에는 `startedunexpected completion`이 남아 후손의 실행이 끝났음을 확인했다. C2 first_error는 selected libtest exited nonzero, selected_one=false, child_exit101, family=Empty, forced=false, ownership=complete, physical_closed=true, stdout/stderr EOF=true다. 이것은 외부 시험 집합의 종료 관측이며 제품 helper가2초 안에 후손을 정리했다는 증거가 아니다.

## 검사와 배정 오류

1382 Native strict와1383 default strict를 앞 handle 종료 확인 전에 배정했다. 두 실행은 실제0으로 끝났지만1383에 Cargo build lock 대기가 기록됐다. PM의 직렬 배정 위반이며 원 로그를 보존했다. 두 handle의 terminal을 확인한 뒤1385 Native(738ms),1386 default(588ms)를 실제 직렬로 재실행하여 모두0을 확인했다.1384 fmt도 actual0/8201ms다. 이 성공으로1381 실패를 바꾸지 않는다.

## 의미와 다음 작업

최종1387 custody는 동일 R47 SOURCE 핀에서 terminal0/41202ms·2302 content/identity unchanged를 확인했다.

현재 helper는 `std::process::Child`의 부모를 종료하고 stdout/stderr reader를 join한다. 후손이 상속한 파이프가 reader 종료를 늦추는 경로를 확인했다. 제품은 취소 인수를 받지 않는 동기 FrameCapturePort를 사용하며, 이 경로의 실행 중 취소/원 family 정리·receipt는 아직 구현되지 않았다. 정상 FFmpeg 캡처·정상 EOF의 기존 근거와 별개다.

다음은 [원 family 소유권 연결 계획](../../../plans/player/frame-process-family-custody-20261028.md)이다. 기존 owned-process의 suspended/assigned 원 owner, raw/receipt, FinishOutcome::Incomplete를 재사용하고 API의 보유 경계를 먼저 고정한다. 파이프 reader를 detach하거나 제한 시간을 늘리는 변경으로 이 실패를 없애지 않는다.

R46 지정190/정상bins는 해당 원 입력의 실제 성공으로 보존한다. R47에 전체190/정상bins 성공을 승계하지 않으며, 전체 기능 인수와 goal은 미완료다. source375 private overlay·WIP/소유권·원 실패·AI0/9·추론/다운로드/수동GUI/WARP/push 보류 유지. 설치·네트워크·전역설정 변경 없음.

modootoday/software-qa/test-fixture-and-mock-build SHA `A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100` 및 review-verification SHA `9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B`의 실제 fault fixture·원 실패/외부 정리와 제품 완료 구분을 적용했다.
