---
id: evidence:kinekino/20261029114626
title: Asset project SDK/CLI — 2026-10-29
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Asset project SDK/CLI — 2026-10-29

private source375 후속 mirror이며 canonical/commit/push 없음. [저장 검증](asset-project-storage-20261029.md)과 [계약](../../player/application/production/asset-projects.md)의 공통 유스케이스를 typed ApplicationClient·PlayerSdk·로컬 CLI에 연결한다. source/review admission, render/Jobs, quality 승인과 별도다.

## 입력과 진행 증거

AGENTS SHA `40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C` 재확인. preimage-r218-sdk 및 preimage-r219-command-registry에 기존 파일 보존. R219 snapshot2483개/2931 native strict 종료0. R220은 등록 명령3개를 반영한 실제 registry 기대115 갱신으로 별도 고정했다. 기존111 기대도 최신 registry와 일치시켰으며 명령 수를 기능 완성률로 사용하지 않는다.

R220 inventory SHA `23935B875EC3E8EDC1F4B8C837BB2EF57E2C475E261303C1CC78E85D8B72A049`, SOURCE.md SHA `C1E2B16552DD79F3CA493625564BC6EFC6A6D169B17F08B3600D94ECB5860EAC`. 2932fmt·2933snapshot·2935native strict·2936default strict 종료0. 2937 lib/asset_project_cli no-run은 실제 rlib archive 생성 중 디스크 부족(os error112)으로 실패했다. 원 stdout JSON 진단과 stderr를 유지하며 성공으로 계상하지 않는다.

2934는 미압축 product rlib2개,2938은 큰 미압축 product cache PDB15개만 NTFS 압축했다. 삭제·다른 프로젝트/전역/Meari cache 변경 없음. 전후 해시 기록은 `runtime/verification/source-sdk-r220-cache-compress-r1` 및 r2에 보존하며 각 비교는 차이 없음. r2의 최초 해시 읽기 완료 전 압축 runner가 시작됐으므로 엄격한 직렬 사전 해시라고 주장하지 않는다. 압축은 논리 내용을 변경하지 않으며 종료 후 동일 파일들의 해시를 다시 확인했다. 2938 종료0: 논리10380382208바이트→물리3779883008바이트, 당시 C여유6950756352바이트. 2939는 같은 R220 소스의 no-run 재실행이다.

## 미완료 범위

2960 최종 사후 custody 종료0:2483개 source content/identity unchanged. 지정190 실행과 정상 bins/실제 자산 저장 SDKCLI를 R221 입력 경계로 확정한다. 전체 제품 완료나 제작5명령 인수 종료는 아니다.

통합2960 전 실행:2953 combined lib/Jobs/unitCLI no-run 종료0. 2954 exact 첫 시도는 결과 상위 디렉터리 미준비(os error3)로 시작 전 종료1이며 시험 통과로 계상하지 않는다. 상위 디렉터리를 준비하고 새 r2 결과에2956Product145·2957Core18·2958Native26·2959OS1을 직렬 실행했다. 각 group-return.txt의 execution_result/producer_integrity 모두 Ok이며 동일 R221 SOURCE.md SHA를 확인했다. Core/Native/OS는 기존622/624 컴파일 producer를 실제 재실행한 결과로 새 컴파일이라고 표시하지 않는다. 결과 root `runtime/verification/source-sdk-r221-exact190-r2`. 정상 bins는2950과 동일하다. 새로운 Jobs exe SHA `691B5C8B00D796986134088391A0A34CA41A83DD71D4405DEFCCE3BA33945A88`, unitCLI exe SHA `AA1558774961943B650130700A59EB6677096989EAEBE561FB538A0C1AAD823D`를 fixed-r221/native-core에 보존했다. 원 producer cache PDB는 유지하며 추가 중복 PDB 복사는 하지 않았다.

2951 사후 custody 종료0: R221 실제2483개 source content/identity unchanged. 2950 정상 bins의 위 해시도 일치했다. 이 단계 마지막 C여유1611567104바이트로, 다음 무거운 통합 producer 전에 원본 보존 방식의 추가 공간 확보가 필요하다. 현재 실행 중 producer는 없다.

2939 재실행 종료0: lib/asset_project_cli no-run과 실제 정상 player/player-cli 산출을 확인했다. fixed-r220/native-core의 lib SHA `EA1F13706CE824950108877F7866D899FCED925175BD2461A4DA73924747ACE1`. 2941 저장/typed SDK10통과·기존PNG1ignored. R220 CLI 첫 실제 시험2942는 미설정 Unsupported 종료코드의 잘못된 기대4(actual3)로 실패했다. 실제 응답은 call-001/stdout.bin과 owned 프로세스 증거에 보존했다. 제품을 바꾸지 않고 시험의 기대만3으로 수정한 R221을 별도 snapshot으로 고정했다.

R221 inventory2483개 SHA `1C9F2CBED6AAB7A9653E483EC056AACFC134FDE63F299B0B6594D2643D3AA504`, SOURCE.md SHA `E5D9EC525B13459E0949D925FF957B54C896416C7FEE15AFA4E779DE2F54E7DA`. 2943fmt check·2944snapshot·2946asset CLI no-run·2947실제CLI1시험·2948native strict·2949default strict·2950정상 bins 종료0. 2947은19개 실제 정상 CLI 프로세스 호출과 typed SDK의 create/save/inspect 동등성을 확인했다. 각 프로세스가 저장소를 다시 열었고 stale CAS·과거 publication/원본 보존·설정 없는 Unsupported·root 주입·timeout/idempotency·endpoint/serve 거절을 확인했다. 결과는 `runtime/verification/source-sdk-r221-asset-cli-r1/verification.json`. 값은 빈 draft와 수동 편집이므로 원 미디어/모델/사용자 승인 증거가 아니다.

CLI 시험 exe SHA `62C5C6D4E85DE8FA9AAE95C85775F2887750499C365CED5B4ADFFC3063A296F0`는 fixed-r221/native-core에, 실제 CLI SHA `9F097D972E14831546CFA4BD8A780C8746B372AFDF35C6D7F2462AEB098B7075`와 GUI SHA `1D63CCEF05E80299AEFEC5AFCEDC3CD29EA74E29C13CC80FF5D758016219689F`는 fixed-r220/native-bins에 보존했다. R221은 시험 기대값만 변경했고2950에서도 같은 정상 bin 해시를 재확인한다. GUI 실행/시각 인수는 하지 않았다.

최초 frozen 복사에서 player_cli.pdb 디스크 부족 오류가 발생했으며 부분 파일은 실제로 남지 않았다. 동일 원 cache PDB를 단일 복사로 보존해 전후 SHA `99628EF172CCB5CC44981FDDAC10DF3A01C4D309264D646C2BCA01F6B39D1BC6` 일치를 확인했다. 2940은 기존 frozen 파일의 NTFS 압축 상태를 확인한 종료0이다. 2945는 추가 product cache PDB10개를 직렬 사전 해시→압축→사후 해시로 보존했고 비교 차이 없음을 확인했다. 해당 경로/해시는 source-sdk-r221-cache-compress-r1에 보존했다. 삭제/덮어쓰기 청소는 하지 않았다.

이번 입력은 지정 전체190 회귀와 자산 프로젝트 저장 SDK/CLI를 검증했다. R210의 숏폼 실제 미디어 실행을 이번 입력에서 재실행한 것으로 계상하지 않는다. 저장 명령은 timeout/idempotency/in-flight cancel 미지원이며 request 경로 주입·endpoint/serve 조합을 거절한다. 원본/후보 비교 GUI·AI 사용자 품질·실제 asset video admission·asset Jobs/sequence/projection 공백과 기존 보류/소유권을 유지한다. 적용 QA 및 image-postprocess skill 출처/판단은 기존 입력 검증과 공통 계약을 따른다.
