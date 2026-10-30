# Rust 검증 도구

Windows 전용이다. 최상위 AGENTS.md의 Rust 통일·단일책임·모듈화·계층 구조를 따른다. domain/playback은 연속 재생 정책, application은 preflight/soak/benchmark 유스케이스, infrastructure는 Win32 프로세스/파일/고정 엔진 명령을 담당한다. main은 조립/실행만 한다.

저장소 최상위에서 실행한다.

```text
cargo build --manifest-path tools/validation/Cargo.toml
tools/validation/target/debug/source-validation.exe preflight
tools/validation/target/debug/source-validation.exe soak runtime/verification/soak-native-host.jsonl native 3600 live
tools/validation/target/debug/source-validation.exe soak runtime/verification/soak-web-host.jsonl web 1800
tools/validation/target/debug/source-validation.exe bench mistral
tools/validation/target/debug/source-validation.exe bench llama
tools/validation/target/debug/source-validation.exe dpi-report runtime/verification/dpi-app-100.jsonl
tools/validation/target/debug/source-validation.exe cleanup-vm plan
```

bench는 runtime/ai-benchmark의 고정 Q4 모델/모델 해시·tokenizer snapshot과 portable mistral.rs v0.9.4 또는 llama.cpp b11716을 사용한다. 파일 준비는 모델/공식 릴리스 데이터 확보 작업이며 제품 AI 설치 기능은 아직 아니다. 모델 snapshot 경로는 manifest.json, Q4 경로/revision/hash는 q4-model.json에 기록된다. 임의 웹 입력·셸·remote repo code를 실행하지 않는다.

소유 재생 검증 앱이 종료되고 CPU/RAM preflight가 통과해야 추론을 시작한다. 보호 앱 PID와 전후 자원을 기록하고 ChatGPT/Terminal·다른 앱·WSL·서비스를 종료하지 않는다. BelowNormal, Rayon/OMP/MKL4, llama 명시 thread4, offline, 엔진 순차 실행, 종료 시 모델 해제를 적용한다. 메모리는 소유 worker tree를 100ms 간격으로 관측하며 private/working set을 모두 기록한다. OS 캐시를 비우지 않는다. 최초 프로세스 시작과 cold disk를 혼동하지 않는다.

양 엔진의 synthetic benchmark 프로토콜은 동일하지 않다. prompt128/gen64/반복3을 맞추더라도 직접 속도 비율을 일반화하지 않는다. 정확도·ASR/OCR/비전·GPU·재생 경쟁 시험과 별개다. 최대 10분을 초과하면 해당 소유 worker만 중단한다.

검증:

```text
cargo fmt --manifest-path tools/validation/Cargo.toml --check
cargo test --manifest-path tools/validation/Cargo.toml
cargo clippy --manifest-path tools/validation/Cargo.toml --all-targets -- -D warnings
```

기존 scripts/validation의 Python/PowerShell은 새 개발 경로가 아니며 Rust 이관 대상이다. DPI/VM 관리 코드를 무단 변경하거나 기존 운영 스크립트를 대체 검증 없이 삭제하지 않는다.

사용자가 명시적으로 요청한 scripts/cleanup-validation-environment.ps1은 이 Rust 도구의 실행 진입점이다. 기본 plan은 삭제하지 않는다. cleanup-vm apply의 고정 대상은 runtime/validation-vm뿐이며 허용 목록·실제 경로·reparse·VM 미등록·devcon/출처 보존을 검사하고 파일별 감사 기록을 남긴다. 정책 차단을 우회하기 위해 에이전트가 apply를 실행하지 않는다. 실제 삭제 분기는 미검증이며 사용자 직접 실행 후 결과를 확인해야 한다.

DPI 시험 앱은 `kinekino-player.exe --record-validation --validation-position=3600,100`처럼 검증 기록을 명시한 경우에만 초기 창 위치를 지정할 수 있다. 좌표는 로컬 CLI 입력이며 브라우저 메시지로 설정하지 않는다. 먼저 실제 모니터 영역을 조회하고 해당 가상 화면 안의 좌표를 사용한다. window-metrics는 시험 HWND의 GetDpiForWindow·창/클라이언트/모니터 영역과 egui 배율을 기록한다. dpi-report는 실제 앱 DPI/배율/비주 모니터가 일치하는 표본만 집계하며 화면 조작·스크린샷의 사람 검토를 대체하지 않는다.
