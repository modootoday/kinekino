---
id: sot:kinekino/frame-tools
title: 프레임 캡처의 명시 도구 구성
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 프레임 캡처의 명시 도구 구성

주의할 현재 구현 공백: [R47 후손 파이프 시험](../../verification/application/frame-descendant-deadline-failure-20261028.md)에서2초 기한 호출이30.04초 뒤 반환했다. 도구 핀 일치는 descendant containment나 시간 내 물리 정리 보장이 아니다. 이 helper의 실행 중 취소·family/receipt 보유는 수정 전이며, 기존 정상 frame/EOF 결과로 그 범위를 인수하지 않는다.

R46 후속: [jobs host 실제 연결](../../verification/application/job-host-pinned-frame-20261028.md)에 따라 네 옵션을 `--serve-jobs`에서도 받는다. jobs 시작/ready 전에 핀을 검증하며, 옵션이 없으면 기존 host의 frame 미지원 상태를 유지한다. direct SDK 명령을 추가하는 것이며 worker operation 목록은 바꾸지 않는다. ready의 frame_capture_configured는 설정 여부다. 다른 serve host·GUI와 실행 중 frame 취소/EOF fault는 미검증이다. 아래의 이전 'serve host 미적용' 범위는 이 jobs host 한정으로 갱신한다.

후속 [R44 CLI help/명시구성review discovery](../../verification/application/cli-review-pinned-discovery-20261028.md)에서help보완과97schema정합을확인했다.기본/serve/GUI host소비와기능전수인수는계속남는다.

2026-10-28. 공통 Rust 어댑터 `FfmpegFrameCapture::pinned(ffmpeg_path, ffmpeg_sha256, ffprobe_path, ffprobe_sha256)`를 제공한다. 신뢰된 host 설정만 이 생성자를 호출하며 media payload·브라우저 메시지에서 실행파일을 선택하지 않는다. 기존 `FrameCapturePort`의 캡처/오류/저장 정책을 그대로 사용한다.

Windows에서 절대 로컬 일반 파일·상위 디렉터리 체인을 기존 immutable_storage_paths로 검증한다. 각 파일 SHA256은 최대512MiB 범위에서 검증한다. 불일치는 SourceExpired이며 PATH/Chocolatey/다른 버전으로 fallback하지 않는다. 검증한 파일과 상위 디렉터리 guard를 어댑터가 보유해 해당 수명 동안 교체·쓰기·삭제를 제한한다. 임의 새 검증 registry·별도 캡처 정책을 만들지 않는다. 실행파일 핀은 종속DLL/codec 전체에 대한 hermetic 보장이 아니다.

로컬 standalone CLI의 옵션은 다음 네 개를 함께 제공한다.

```
--frame-ffmpeg ABSOLUTE_EXE
--frame-ffmpeg-sha256 SHA256
--frame-ffprobe ABSOLUTE_EXE
--frame-ffprobe-sha256 SHA256
```

옵션 누락·잘못된 경로/해시 형식·중복은 InvalidArgument. endpoint와 로컬 frame 설정을 함께 지정하지 않는다. JSON request payload는 변경하지 않으며 trusted CLI 설정과 분리한다. 네 값이 없으면 기존 자동 탐색을 유지한다. 명시 pinned 구성의 다른 target은 Unsupported이며 원래 자동 탐색 target 지원을 확대하지 않는다. 현재 serve host/GUI는 이 새 CLI 설정 소비 범위가 아니다.

파일 핀을 통과해도 원2초 version/encoder/filter 검사 및 PNG/select/showinfo 조건이 필요하다. 검사 실패는 R42 stage/code 진단과 available=false로 남으며 핀 일치만으로 기능 지원을 강제하지 않는다. pure PNG export는 기존대로 decoder 탐색/실행 없이 처리한다.

[R43 구현·실제 CLI 근거](../../verification/application/frame-pinned-tools-20261028.md). 명시 구성의 실제 캡처·시간초과/취소/저장 인수 및 안정 discovery 전수는 별도다. 자동 탐색을 프로젝트 기본 명시 구성으로 전환·GUI host 연결·CLI help 보완과 영향190/bins 회귀는 후속 작업이다.
