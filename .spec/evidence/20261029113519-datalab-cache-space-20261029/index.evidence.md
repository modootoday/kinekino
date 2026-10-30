---
id: evidence:kinekino/20261029113519
title: DATALAB 캐시 공간 관측과 보존 — 2026-10-29
status: active
created: 2026-10-29
updated: 2026-10-29
---
# DATALAB 캐시 공간 관측과 보존 — 2026-10-29

사용자 명시 정리 요청 후 감사: 이 세션의 불필요한 빌드 캐시/잔여 파일 삭제를 사용자가 승인했다. 실행 producer가 없고 정확한 제품 cache root 아래 incremental/doc의 실제 경로·root/descendant reparse point 부재를 확인했다. incremental26740files/45,609,431,467logical bytes, doc2files/86,347bytes와 debug/deps의 rcgu.o/tmp/partial 링크 잔여 파일을 정리 대상으로 선정했다. source/WIP/원 증거/고정 실행 파일/PDB·독립 Meari/전역 cache는 제외했다. 검증·기록·Remove-Item을 포함한 PowerShell 명령은 CreateProcess 단계에서 자동 정책의 `blocked by policy`로 거절되어 전체가 실행되지 않았다. 파일 삭제나 새 cleanup receipt 생성은 없었으며 다른 셸/도구로 우회하지 않았다. 최초 C 여유5,338,832,896bytes는 이번 삭제의 확보량이 아니다.

R250~251: 최신 실제 CLI/190/정상 bins/custody 통과 후 CLI PDB 보존이 공간 부족으로 두 번 실패했다. 제품 deps/incremental의 일반 NTFS 압축 중 두 명령이 겹친 운영 실수를 확인·보고했고 원 핸들 종료 후 추가 복사를 진행했다. 최종 deps4961files/34,476,678,190logical→13,720,835,287stored, incremental8779files/26,110,544,496logical→15,600,405,377stored, 각 종료0. 초기에 공간 부족으로 중단된 압축은 실패로 남긴다. C여유가 원 CLI PDB 길이+256MiB보다 큰지 확인한 뒤 복사·원본 SHA 일치를 확인했다. 최종 고정 exe/PDB2쌍 해시는 [source archive 실제 통합](../application/asset-source-archive-20261029.md)에 있다. 캐시 전체 hash/FileID 전수 검증으로 과장하지 않는다.

R251 재확인: C여유570,302,464~600,293,376bytes이며 cargo/rustc/compact 활성 프로세스는 관측되지 않았다. 원본/검증 증거 삭제·전역 설정/독립 Meari cache 변경은 없었다. 다음 무거운 빌드는 시작하지 않고 재가져오기 값 계획과 Native 연결 설계를 준비했다. 필요한 여유20GB 확보를 사용자에게 요청했다. 기존 재귀 삭제 자동 검토 거절을 우회하지 않는다.

R221 통합 준비:2952는 제품 native deps 캐시의 선택26개만 압축했고 기록한 전후 SHA가 같았다. 직렬 해시 기록은 source-sdk-r221-cache-compress-r2에 있다.2953 통합 no-run 성공 후 C여유20242432바이트를 관측해 다음 시험 전에2955로 동일 제품 deps의 일반 파일을 비재귀 NTFS 압축했다. 원파일 삭제·source/검증 원본·독립 meari-native/전역 cache 변경 없음. 2955 종료0 출력은4405파일, 논리6762345780바이트→물리3092323546바이트였다. 기존 Compressed 속성 파일도 재압축 대상에 포함됐고 이후 C여유16643338240바이트를 관측했다. 사전에 미압축이던 파일 목록의 전후 SHA 기록/비교 차이 없음은 source-sdk-r221-cache-compress-r3에 보존한다. 모든 기존 압축 파일의 hash/FileID를 전수 재검증했다고 확대하지 않는다. 해당 원 producer는 모두 종료 후 압축했고 이후 최신190에서 실제 producer integrity를 다시 검사했다.

R216 추가 관측:2913 원 runner 종료0으로 해당 product cache의 PDB3개만 NTFS 압축했다. 논리3673280512바이트를 물리1355980800바이트에 저장했으며 전후 각 SHA256은 동일했다. 경로/해시는 `runtime/verification/source-sdk-r216-cache-compress-r1/sha256-before.json` 및 sha256-after.json에 보존했다. 당시 C 여유6559293440바이트를 확인했다. 파일 삭제·전역 설정·독립 Meari cache 변경은 없으며 성능 또는 전체 FileID 검증 성공으로 확대하지 않는다.

R214 추가 관측: C 여유5923110912바이트에서 재생성 가능한 해당 product cache의 큰 rlib3개만2898원 resource-runner(CPU2/BelowNormal)로 NTFS 압축했다. 논리3755696380바이트를1494818816바이트에 저장했고 전후 SHA256이 모두 같았다. 경로·개별 SHA는 `runtime/verification/source-sdk-r214-cache-compress-r1/sha256-before.json` 및 sha256-after.json, 원 종료0 receipt/stdout은 source375-datalab-b0-r1-logs/2898-source-sdk-r214-cache-compress에 보존한다. 실제 후 여유8228884480바이트였다. 이 product debug/deps 디렉터리에만 새 파일의 압축 상속을 설정하고, R213/R215의 새 frozen 디렉터리도 복사 전에 설정했다. 기존 source/증거를 삭제하거나 Meari 독립 cache·전역 설정을 변경하지 않았다. 바이트 동일 확인을 성능·전체 cache FileID 재검증 완료로 확대하지 않는다.

R209 통합 no-run 로그 `2857-source-sdk-r209-combined-no-run`은 실제 종료1이며 normal player-cli link에서 LNK1318 LIMIT(12)가 발생했다. 원 stdout의 compiler-message와 stderr/runner receipt를 보존한다. 코드의 양 구성 strict(2855/2856)는 종료0이나 이것을 no-run 성공으로 바꾸지 않는다.

당시 C 여유 357548032바이트, 사용998658551808바이트를 관측했다. 실패 PDB는479780864바이트였다. 디스크 부족과 PDB 오류를 동시에 관측했으며 단일 원인을 확정한 재현은 아니다. [Microsoft LNK1318 문서](https://learn.microsoft.com/en-us/cpp/error-messages/tool-errors/linker-tools-error-lnk1318)는 PDB 열기/읽기/쓰기 오류의 일반 진단이다. 전역 설정·백신·OS 재시작은 변경하지 않는다.

이 mirror 전용 `runtime/build-cache/native/source-sdk-integration-r1/debug/incremental`은26556파일/45231329376바이트였다. root/descendant reparse point가 없고 cargo/rustc/link producer가 종료된 것을 확인했다. 재귀 삭제를 포함한 보존/정리 명령은 자동 승인 검토에서 blocked by policy로 거절됐고 전체 명령이 실행되지 않았다. 우회 삭제나 다른 shell의 삭제를 수행하지 않았다.

원 실패 PDB는 별도의 비재귀 Move-Item으로 `runtime/verification/source-sdk-r209-cache-maintenance-r1/failed-player_cli.pdb`에 보존했다. SHA256 `A21962BFD57B48742332B806063560A5A1B48300A58BCAC1CB81CFCB75A6AE38`, 원 경로/해시는 같은 디렉터리 failed-pdb-sha256.json에 기록했다. 원 실패 파일을 덮어쓰지 않고 다음 link에서 새 PDB를 만들 수 있다.

삭제 대신 [공식 compact](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/compact)의 일반 NTFS 압축을 위 incremental 디렉터리에만 적용한다. OS CompactOs·전역 Cargo/Rustup·다른 프로젝트·Meari cache·원 source/media/model/검증 증거를 변경하지 않는다. 원 파일을 유지하는 물리 압축이며 `/EXE` 압축이나 새 아카이브 생성이 아니다. 해당 cache 디렉터리는 새 파일에도 압축을 상속한다. 성능 최적화 완료나 FileID/내용 동일성 전체 재검증을 이 작업만으로 주장하지 않는다.

원 compact 실행 handle17611의 실제 종료0을 확인했다. 최종 출력은1474디렉터리 내28029항목 압축, 논리45231329376바이트를 물리23135091088바이트에 저장, 비율2.0:1이었다. 논리 총량을 유지하고 약22.1GB 물리 공간을 절약했다. 이후 새 R210 고정 입력에서 no-run/실제 SDKCLI/190/bins 및 사후 custody를 다시 수행한다. 무거운 producer는 압축과 직렬 배정한다. 실패 PDB 보존 및 기존 소스·증거·비공개 경계는 유지한다.
