---
id: evidence:kinekino/20261029113522
title: 세션 자동 유지보수와 작업 재개
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 세션 자동 유지보수와 작업 재개

최종 R252 결과: maintenance fixture6개 통과.3190 Rust project 회귀는13 passed/0 failed/2 ignored/원 runner 종료0이었으나 후속 Python cleanup이260자 초과 경로에서 WinError3으로 실패했다. 이를 Rust 실패나 전체 성공으로 바꾸지 않았다. 첫 오류는 session-maintenance-20261029-r1/first-cleanup-failure.json과 원3190receipt에 보존했다. 확장 길이 경로를 사용하고, 일부 marker가 삭제된 성공 scratch는 원 durable maintenance receipt의 정확한 경로/exit0/idle 기록이 있을 때만 복구하도록 수정했다. 실패·unknown receipt는 복구 대상이 아니다. 새 테스트가 실제260자 초과 fixture와 부분 marker 상태를 재현해 통과했다.

3191 실제 fmt check wrapper 종료0: removed_before가 원3190 scratch(run-94c69dc13103453190f252cb536bdc45)였고 새 scratch도 후속 정리됐다. `.cleanup.json`은 실제 완료 정리를 원 runner 결과와 별도로 기록한다.3192 custody+cleanup 종료0: source2513개 content/identity unchanged. 최초 실패 때문에 Rust13시험을 중복 실행하지 않았다.3189 실제 원본 archive/readmission/rerender+import plan1시험도 종료0이다. 최종 Python 코드·Cargo config pins와 tests-long-path-fix.txt는 같은 evidence root에 있다. 최종 C 여유43,926,994,944bytes. Legacy 재귀 삭제 거절을 우회하지 않고 새 managed 성공 scratch만 자동 정리했다.

실제 재사용 빌드 경로는 Python wrapper → 기존 resource-runner → Cargo/선택한 runtime exe이며 모든 이후 이 세션의 Cargo/원 시험/custody 호출을 이 경로로 배정한다. 호출 예: `python -B runtime/tools/session-maintenance/session_build.py --log-stem 3193-example --program cargo -- fmt --all --check`. 전역 Cargo/Rustup이나 독립 Meari workspace 설정은 변경하지 않았다.

2026-10-29 사용자 명시 지시로 Python 유지보수 도구를 추가했다. 제품 정책/새 제품 시험의 Rust 규칙은 유지하며 예외를 maintenance wrapper/그 격리 시험에만 한정한다. 기존 설치 Python3.12 `%USERPROFILE%/AppData/Local/Programs/Python/Python312/python.exe`, 실행 파일 SHA `4D6F5F81A4BCA11191C4C7C6B43632694D0A4CE74E068619D8FDC161D469859A`를 사용했다. 신규 의존성/다운로드/예약 작업/전역 설정은 없다.

runtime/tools/session-maintenance/session_build.py가 기존 resource-runner 앞뒤를 감싼다. wrapper 파일 잠금·실제 process inventory로 heavy slot을 확인하고20GiB 여유를 검사한다. 제품 mirror/native target과 jobs1·incremental0·stable 도구체인을 고정하고 매 실행마다 build-cache/tmp/session-managed 아래에 새로운 owner-marked TMP/TEMP를 만든다. resource-runner 원 stdout/stderr/receipt는 기존 위치 그대로이고 별도 maintenance receipt에 executable SHA/argv/환경 경로/종료 상태를 기록한다. 원 runner 종료와 프로세스 idle 확인 후 성공 marker가 있는 자기 scratch만 지운다. 실패/미완료/외부 owner/원본 검증/고정 빌드/PDB/전역·Meari/legacy tmp는 삭제하지 않는다. 성공한 이번 호출이 새로 만든 loose rcgu.o/tmp/partial만 개별 정리하며 실행 전 존재한 잔여 파일은 보존한다. 이전 정책 거절된 legacy 재귀 삭제의 대체 실행 경로가 아니다.

root와 SDK mirror Cargo config에 incremental=false를 명시했다. 제품 기능/프로세스 CPU 제한/원 runner 계약은 바꾸지 않는다. 앞으로 모든 성공 중간 회차마다 exe/PDB 복사본을 추가하지 않고 원 producer 로그·입력 pin과 선택한 검토 이정표/첫 실패를 보존한다. 이전 고정본/검증 증거는 자동 삭제 대상이 아니다.

실제 maintenance fixture5개가 통과했다: independent workspace에서 완료된 owner만 정리/미완료와 fixed proof 보존, 잘못된 completion 이름 거절, 알려진 Python PID process 관측, build slot 중복 거절, 이번 호출 새 loose object만 정리/기존 파일 보존. 테스트 root는 프로젝트 build-cache/tmp/maintenance-tests이며 completed fixture는 자동 해제된다. Windows reparse/다른 OS의 모든 race를 인증한 시험은 아니다. 본문·pins·출력은 runtime/verification/session-maintenance-20261029-r1에 있다.

3183/3184 실제 fmt·3185snapshot·3186native strict·3187default strict 종료0. 각 실제 wrapper 실행에서 원 runner 종료 후 자기 TMP 정리를 관측했다.3188lib no-run 진행 중으로 아직 실제 제품 시험 성공이 아니다. R251 재가져오기 계획 WIP를 R252 입력에서 검증 재개했다. 최초 C여유46,604,357,632bytes. 현재 user goal 범위로 작업을 재개했으며 기존 목표/소유권/WIP/보류를 유지한다.

적용 skill: modoo.today `modootoday/build-performance/optimization-portability-review` 본문을 읽고 경로는 명시 local policy, 원 runner 제한은 유지, 독립 fixture/known-present process/빈 선택과 정상 정리/실패 보존을 구분했다. CI·타 OS·성능 개선 전체를 완료로 표시하지 않는다.
