---
id: evidence:kinekino/20261029113545
title: A pure protocol 실제 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# A pure protocol 실제 검증

2026-10-24 KST PM 단독 heavy/shared 배정. rootaa88706e2ff2e1bf1e9f71ec681aa9de9dcb931a/libraryb532, canonical lockE469 기준으로 기존 제품 S/T와 runnerEEA9DEE6/conservative2/CPU2/BelowNormal/descendant Job/offline/j1/thread1을 사용했다. fresh RAM29.06GiB/GPU6270MiB/C103.53GiB/writer0 확인 후 진행했다. 근거는 runtime/verification/media-source-protocol-a-r1이다.

## 입력과 입장

동결 source6 F0496B800F8C7A34EEA5A306507BB3A93E629635821BA8A3BEA348108A9CC7AF/new6의 원 bytes와 root/S 부재를 확인했다. S와 root의 차이는 채택된 R1 baseline9뿐이었다. 원 S5341·T574·root1359/library493·fixed94/alias4/known2/index와 normal preimage copies를 보존했다. 승인 R1 baseline9만 동기화한 뒤 source6과 S workspace.members 한 항목을 추가했다. default-members/root dependencies는 불변이다.

승인된 runner01 `cargo metadata --offline --format-version=1`은 한 번만 실행했다. Cargo가 생성한 lock은8F2C1D56C8A2C22D36D9E57A285D727C7EE9C7D2D1160902B66CC6630B042C67이며, dependency 없는 local media-source-host0.1.0 entry를 제거하면 원E469 bytes와 정확히 같다. 기존 package/version/source/checksum/dependency graph 변경0이다. 실제 metadata target은 lib+codec integration, edition2024·dependencies0다. 이후 검증은 모두 locked였다. canonical lock/root Cargo/.gitignore/index는 수정하지 않았다.

## 실제 결과와 원 실패

fmt02는 실행 cwd를 canonical root로 잘못 지정해 package가 workspace member가 아니라며 exit1했다. 원 stdout/stderr/receipt를 보존하고 cwd만 승인 S로 수정했다. 소스 수리0, fmt03·package all-target strict04·Cargo JSON no-run05가 모두 exit0이다. 이 실패를 source 결함 또는 통과로 바꾸지 않는다.

Cargo JSON producer의 codec BIN을 고정했다: SHA256 AF860C4B4FB7DE3506104D9389E4C4884A7954D4F58555E258A0A6408C85A72D/843776B, features빈목록·test=true·fresh=false. --list 등록12가 source-derived12와 정확히 일치하고 ignored0이었다. 동일 고정 BIN을 한 번 실행해12pass/0failed/0ignored/0filtered를 확인했다. 전체 workspace 검증으로 확대하지 않는다. 원 receipt8개 모두 terminal,7exit0/원 cwd실패1이다.

## 보호·인계

root1359/library493의 bytes/mtime·index 차0이다. S 기존5341 중 승인 input9개만 변경됐다(R1 기존7+workspace/lock2). 신규8개는 R1 baseline2+source6이고, 비입력5332 bytes/UTC ticks 차0이다. R1 baseline은 현재 root 채택 bytes로 유지했으며 private R2/SDKlist/runtime13은 합성하지 않았다.

T574 기존 exe bytes/mtime 차0, 신규1 codec만 Cargo JSON path/SHA와 일치한다. fixed94와 normal alias4의 SHA/mtime/FileID·knowncache4709 두 bytes 모두 보존됐다. historical producer 미귀속2/all-cache=false는 계속 유지한다. 원 runner terminal receipt와 handle 회수로 마감하며 보조 process 관측만으로 cleanup을 대신하지 않는다. slot-return.json으로 명시 반환하며 추가 실행 권한은 없다.

후보 채택용 raw8은 runtime/development/media-source-protocol-a-verified-r1/source8.json SHA256 9F4051D2C10695DFB10035308E6531A768382587E272EC002AD2DE16C0CDBAF1, before8 D0F4FAFAD2BF460A346D2B9EC5C6D789A024F5A938A515B0A5BECD4E183D115E다. 원 source6은 불변이다. source8은 source6+workspace 등록1+Cargo 생성 lock1이며 root 적용/commit/public .gitignore 경계는 PM 소유다. push0이다.

## 실제 의미

순수 framing·byte/state/cutoff 계약12만 입증한다. 동기 Read는 nonblocking 부모 IPC가 아니고 wire/BodyEnd/EOF는 replay authority나 physical ACK가 아니다. OS clock 변환·worker·HTTP·DNS·browser issuer·B/C/D·SDK/CLI 생산 다운로드 지원은 미완료다. 이번 작업은 모델·네트워크·GUI·OS source host를 실행하지 않았다. 성능 benchmark도 아니다.

이미 읽은 modoo.today test-fixture-and-mock-build SKILL SHA a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100(추가 필수 참조 없음)의 정의 기반 fixture·synthetic/actual 분리를 적용했다. source/타입/등록/실제 및 첫 오류와 보호 한계도 구분했다.
