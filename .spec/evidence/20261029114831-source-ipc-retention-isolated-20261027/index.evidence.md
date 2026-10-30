---
id: evidence:kinekino/20261029114831
title: Source IPC B 격리 rebase·비 OS gate — 2026-10-27
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Source IPC B 격리 rebase·비 OS gate — 2026-10-27

실제 OS 후속은 [고정 후보 OS10 종료](source-ipc-os-completion-20261027.md)에 기록했다. 이 문서의 OS 미실행/보존옵션 OS미검증은 당시 비OS gate 상태이며 이후 지정OS10을 수행했다. root 채택·HTTP/replay/제품 다운로드 지원은 여전히 미완료다.

다운로드 생산 연결 선행 후보를 최신 owned-process WIP 위에 **격리 복사본**으로 rebase했다. root 제품/lock/index·원 B r4 raw13·역사D manifest는 변경하지 않았다. C HTTP source와 D browser replay issuer는 여전히 미구현이며 다운로드 지원·물리 정리·OS IPC soundness를 주장하지 않는다.

## 입력·실패 보존

격리 root는 `runtime/development/source-ipc-b-retention-build-r1`이다. 최신 root owned-process의 전체 src/tests를 복사하고 r4의 exact7 변경은 공통 preimage와3-way merge, new6은 원 candidate/보존 leaf에서 공급했다. prepared_job WIP 때문에 windows/mod.rs와 launch.rs가 각2곳 충돌했다. 원 충돌 파일은 `runtime/verification/source-ipc-b-retention-rebase-r1/{windows-mod-conflicts,launch-conflicts}.rs`에 보존했다. spawn_prepared/PreparedFamilyJob·원 budget검사를 보존하고 IPC source 인자를 별도로 전달했다. 기존 부모13 전체를 새 WIP에 덮어쓰지 않았다.

독립 Cargo.toml에는 workspace 경계만 추가하고 원 dependency graph를 offline generate-lockfile로 생성했다(802). root lock과 전역 Cargo/Rustup은 변경하지 않았다. 격리 lock SHA256=D9B51D989D58E64CFE5CC55DABB6A80260BD3F37D440BA86EC7BFC8FD0FA90D3. target은 `runtime/build-cache/native/source-ipc-retention-r1`, TMP/TEMP는 공통 C tmp, resource-runner conservative2/BelowNormal/단일 heavy다.

| 입력/원 실패 | 관측 및 최소 수정 |
|---|---|
| R1 input-r1 | 803fmt/805all-targets check 통과.806strict는 source_pipe.rs collapsible_if 실패 |
| R2 input-r2 | 동등 let-chain으로 정리.808strict는 test child while_let_loop 실패 |
| R3 input-r3 | 동일 control poll 조건을 while-let로 정리.810strict는 Operation의 test module 뒤 Drop 배치 오류 |
| R4 input-r4 | 같은 Drop 구현을 test module 앞으로 이동.812all-targets strict 통과,818fmt통과 |

각 입력 snapshot과 원 실패 stdout/stderr/receipt를 보존했다. 수정 전3leaf는 rebase 증거 폴더의 *before-strict.rs에 있다. 경고 허용·정리 guard 완화·기한 갱신·새 owner로의 교체는 하지 않았다. 형식 수정의 실제 OS 동등성은 미검증이다.

최종 입력은 격리 root의 `runtime/verification/input-r4/SOURCE.md` SHA256=150FF75D5BAAAEC3A9B737E1DA723E47AB4C0250FEF5C78865C0AD5AAD9750DA, inventory=C6C1C11CDF0664F09AEFB99DDD19E8E4D501DF2ABA90B5E36A1EBC8425FA3FB9,32개다. 이는 합성/rebase한 전체 package 문맥이며 부모13 또는 제품source375의 canonical 적용이 아니다.

## 실제 gate

실제 로그는 `runtime/verification/source375-datalab-b0-r1-logs/802~819-source-ipc-*`다.

- 812 R4 `clippy --locked --offline -j1 --features test-support --all-targets -- -D warnings` exit0.
- 813 lib/source_ipc no-run exit0. 실제 Cargo JSON으로 libtest와 source_ipc 및 test child producer를 구분했다.
- 814 source_ipc --list=14,815 --list --ignored=10. 이를 모두 실행했다고 표시하지 않는다.
- 816 source_ipc default=4pass/10ignored: segment 상한·tick 예산 만료/overflow·절대 cutoff 단축·소수 budget 내림. 프로세스/IPC를 시작하지 않는 논리 시험이다.
- 817 library exact checked_count=1pass: Connect의 미정 count를 바이트/EOF로 읽지 않고 Read/Write count 상한을 검사했다. OS GetOverlappedResult를 실제 호출한 시험은 아니다.
- 818fmt exit0,819 최종32입력 content/identity 동일 exit0.

원 producer 경로는 `runtime/build-cache/native/source-ipc-retention-r1/debug/` 아래다. source_ipc test(deps/source_ipc-4a775237ca45156f.exe) SHA1745B48B46B1B96D964993B0B24EE4FE75098EC03CD95EB3435FE0746A728B67, library test(deps/owned_process-4eccd8178d736487.exe) SHA71C1FEBBD4A28F4512D90F7CF0D9E6E398BF2BFACE3CADD24E4901DD1D6195B4, child(owned-process-test-child.exe) SHAC6D67C3E4744BAA5F79F2AAB3174A811217C818A0FF0B5E5F3F1B68643CBF134. child는 빌드됐고 실행하지 않았다. 제품 player/CLI alias를 교체하지 않았다.

## 잔여와 보류

OS10은 원 baseline-owned outer timeout driver·원 source owner/기한·inner/outer raw 증거/정리의 별도 gate 후에만 실행한다. resource-runner에는 timeout이 없어 대체할 수 없다. 새 SOURCE_IPC_EVIDENCE_ROOT의 정상 raw 보존 역시 타입/형식만 통과했고 실제 OS 시험에서는 아직 미실행이다. root 적용·전체190/정상bins 재검증이 이번 격리 scope의 성공으로 생기지 않는다. root의 최신 통합은 SOURCEDFB0EB90이고 별도로 유지한다. WARP·추론·다운로드·수동GUI·push 보류/소유권/WIP 유지.

기존 읽은 modoo.today test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100)와 code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)의 원 실패·고정 입력·실제/합성·미실행 분리를 적용했다.
