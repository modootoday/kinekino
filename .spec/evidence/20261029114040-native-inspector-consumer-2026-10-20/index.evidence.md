---
id: evidence:kinekino/20261029114040
title: 공개 P1b Inspector 제품 소비 소스
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공개 P1b Inspector 제품 소비 소스

2026-10-20 PM 배정으로 실제 구현 소스를 작성했다. live Meari가 main/clean/HEAD2c0883262a9eaf2d0120732f4f5cbd6603bd7178임을 확인하고0589b58a9e8b40be1ff10b8ad27b3dfd27f68d61의 ancestor임을 검사한 뒤 merge --ff-only로 전환했다. tracked status clean이다. clone/파일복사/reset/push/상위gitlink stage/commit은 없다. SDK의 외부old2c 검증 snapshot은 변경하지 않았다.

제품 source9 inventory는 `runtime/quality-review/native-inspector-consumer-r1/source-inventory.json`, SHA `088937C80F57DCC19BB4B47CAA0E5D9468442F8B09D3D2F4476C1DFB6F29E147`이며 정확 사본과 변경 전6파일 핀을 보존했다. 그 밖의 기존 원본은 UI11/P1a 기존 evidence와 커밋으로 보존한다. 이 소스는 새 revision/미컴파일·미검증이며 이전 gate 통과를 소급하지 않는다. Cargo/lock·원frame.tsx/inspector-dock.tsx/공통style/P2 Nav/application·OBJ 초안은 변경하지 않았다.

## 실제 소스 연결

FrameComposition.with_inspector(title,state)를 opt-in으로 추가했다. 같은 원 StudioFrame에 원 InspectorDock graph를 선언하고 실제 SDK editor를 한 번만 발급해 Dock children에 전달한다. P1b 조합의 workspace/nav/queue는 명시 Empty다. 이는 전체 제작 화면의 완성이 아니라 원 Dock과 기존 실제 편집기의 소비 경계다. 기존 P1a editor workspace 조합은 Inspector 미선택 때 유지한다. 기존 initial_panel/product_window 위임 인자는 이미 FrameComposition을 전달하므로 추가 변경이 필요하지 않았다.

새 InspectorIntents는16개 bounded bool/layout 요청을 FIFO로 보유한다. QueueBinding::InspectorOpen은 기존 NodeLease/queue owner guard를 확인하고 같은 registry 등록·Boolean signature/schema로 요청만 enqueue한다. ReviewAction/TextInput/새 SDK/registry/worker로 변환하지 않는다. mailbox가 가득 차고 미처리 suffix가 남아도 host가 상태 사본을 유지하며 큐를 비워 capacity를 회복하도록 했다. suffix가 남는 동안 document/export를 교체하지 않는다.

after_dispatch는 committed FrameProjection 사본에 요청을 적용하고 기존 actual SDK queue drain/completion을 보존한다. prepared binding에 Inspector callback을 추가하고 같은 try_reproject→session.reproject_with→update_module_props_with_callbacks의 원자적 교체를 사용한다. renderer commit 성공 뒤에만 frame state를 확정한다. renderer/export 실패 시 원 state/DOM/export를 유지하고 처리한 요청을 자동 재전송하지 않는다. explicit 다음 admission과 옛 export의 revoke는 기존 session 수명 경로를 따른다. Native의 승인 후 focus 이동을 소비하고 제품 focus OS 명령을 중복 실행하지 않는다.

Inspector state/intent의 source는 studio 디렉터리가 소유하되 선언 module은 proposals에서 한 번 등록하고 studio는 재export한다. 기존 feature-off standalone source-include consumer가 부모 studio module이 없는 문맥에서도 같은 소스/타입을 일관되게 사용할 수 있도록 한 구조다. 두 개의 상태 모듈을 복제 컴파일해 타입을 갈라놓지 않는다.

## 실제 첫 gate와 보존된 실패

PM 후속 배정에 따라 실패 입력을 동결하고 heavy와 snapshot 소유를 명시 반환했다. 최종 live/snapshot9 차이0·보호 기준선 차이0·owned[]를 다시 확인했다. source9r5 SHA62ECB687D7A0D99323215FF716050EF830251A04EF90B8852C1502B80C2A1CC4, 미인수 overlay49 SHA D7CB58E9D2DA6C65AE771F38927AC09B71EE73CE53D47BADBD6D662C3A7B20B8이다. runtime handoff.md와 final 차이 JSON이 반환 근거다. 아래 당시의 heavy UI 유지 상태는 이 반환 기록으로 대체한다. 신규4 실패/미완료 strict gate를 성공으로 표시하지 않는다. 이후 SDK 독립 검증이 같은 디렉터리를 재사용해도 이 UI 실패 근거는 불변이다.

같은 snapshot/기존 C native·D consumer cache에서 고정 runner/mask3/BelowNormal/descendant Job/offline locked j1/test thread1로 실행했다. 아래 결과는 전체 root test를 대신하지 않는다.

| 회차 | 결과·범위 | receipt ms |
|---|---|---:|
| check | 제품 feature-on lib 타입검사 종료0 |39524|
| unit | Inspector state2+mailbox1의3pass |123587|
| consumer 원본 | 기존 frame5pass·신규 Inspector4fail. 직접 mount의 graph collision과 prepare5초 실패 |415732/101|
| consumer-r2 | 잘못된 live cwd로 시작해 명시 중단. 빌드/시험 결과에 합산하지 않음 |별도 입력 오류|
| consumer-r3 | 올바른 snapshot의 수리 후 Inspector4 모두 host 갱신 오류 |159249/101|
| consumer-r4 | Card exact1에서 typed Renderer mixed keyed/positional 오류 확인 |21967/101|
| consumer-r5 | 같은 Card exact1에 실제 expanded AST parent 진단 추가, 동일 오류 |24979/101|

첫 조립 오류는 parse_module_graph가 전달한 inventory 전체 모듈을 포함하는데 Dock parse에 원 frame/chrome도 같이 전달한 소비자 잘못이었다. Dock 원 source1개만 parse한 뒤 원 frame/provider graph에 합치는 최소 수리를 했다. 원 Frame/Dock·Meari는 수정하지 않았다. prepare helper는 startup.mount_failure/bridge_failure/initial_failure의 원 cause를 즉시 출력하게 보강했다. 실패owner를 take/close하거나 retry 정책·5초 deadline·기존 assertion을 바꾸지 않았다. 원 소스9와 로그/receipt는 보존한다. 수리 후 source9r2 inventory SHA6149B0667B0BF14927D640F9FCFC920C64EAC2AF2E6777C6C15F5B47DFD3CC07, overlay49r2 SHA74A3007BA8D413951FC6D0A1D6EC39F059878080E4F1FD47122488C2298EB0FC다.

수리 후 재검사 r2의 workdir를 live root로 잘못 지정했다. stderr의 D:/workspace 경로를 발견해 command line으로 자체 runner29424를 유일 식별하고 그 후손104444/117124/63852만 종료했다. 원 exec10058 종료1과 가족 부재를 확인했다. consumer-r2-wrong-cwd-owned-runner.json/consumer-r2-interrupted.json에 입력 오류를 별도 보존하며 제품 실패/통과나 snapshot 기능 검증으로 간주하지 않는다. 이후 복사/포맷과 실행 호출을 분리하고 실제 cwd guard·절대 snapshot manifest를 적용했다. live SDK 후속 소스를 snapshot으로 복사하지 않았다.

r4 typed host cause는 `Reprojection(Renderer("mixed keyed and positional children require a stable slot contract"))`다. r5는 실제 projection에서 아래 parent를 관측했다.

- `0/1`, div, 원 frame span418..551, keyed children=[false,true]
- `0/1/1/1`, DialogContent, 원 Dock span684..1157, keyed children=[false,true]

소비자 경계 대조상 initial은 같은 ModuleProgram factory이고 갱신은 현재 InteractiveDocument.node_slot→현재 module_program.node_slots에서 lease를 발급한다. Native update_module_props_with_callbacks는 clone program.update_props의 owner/active/generation 검증과 CallbackSession::new의 scope/bindings 검사를 통과한 뒤 retained_shape에서 이 오류를 반환한다. Core slot identity는 현재 instance/local/key와 ComponentReference로 계산되며 caller는 같은 editor reference/key와 proposal-inspector key를 유지한다. 이는 외부/retired lease나 Boolean schema 오류와 구별된다. Native0589 state_program.rs209/217의 any-key→all-key 검사와 원 고정 slot/positional sibling 조합의 경계를 Meari 담당에게 readonly 반례로 인계했다. 원 시안 TSX에 key를 임의 추가하거나 flatten/별도 Dock을 만들어 우회하지 않는다. 생산자 판단은 회신과 대조 후 확정한다.

현재 source9r5 inventory SHA62ECB687D7A0D99323215FF716050EF830251A04EF90B8852C1502B80C2A1CC4이며 diagnostic helper 변경을 포함한다. 088937 원 핀을 현재 최종 핀으로 재사용하지 않는다. current-owned.json=[]이고 모든 원 handle 종료를 확인했다. UI는 PM 지시로 heavy 소유를 유지하되 새 진단 Cargo를 반복하지 않고 생산자 대조를 기다린다. on/off strict/fmt 최종 gate와 신규4 통과는 아직 미완료이며 P1b 제품 인수로 표시하지 않는다.

PM 배정 후 저부하 입력 준비를 완료했다. 기존 외부 snapshot에 SDK3a8/UI11/oldMeari344 일치를 확인한 뒤 Meari40 변경과 UI9만 적용해 ui-native-inspector-r1로 동결했다. 정확 삭제 목록은0개이며 Meari/Core/Native/paint·manifest/lock의 일관성과 보호 입력 차이0을 기록했다. 변경 전후 파일만 보존했고 전체 clone/archive를 만들지 않았다. [집중 명령과 입력 경계](../../../plans/player/presentation/native-inspector-consumer-gate.md)를 따른다. Cargo/GUI는 여전히0이며 입력 준비를 기능 인수로 표시하지 않는다.

기존 studio_frame_sdk에4개 실제 소비 시험을 추가했다. Card/Highlight/Correction 각각 원 Dock trigger→승인→dirty 입력→닫기→wide/drawer→다시 열기에서 같은 editor NodeId/값·SDK mutation0을 확인하고 같은 SDK의 save/accept와 typed join까지 검사한다. 별도 시험은 실제 같은 SDK/registry에서 raw inspector slot을 commit seam에 주입해 원자적 실패, committed open/dirty/focus 유지, 이전 callback의 명시 재admission 성공→다음 성공 뒤 stale 거부·mutation0·worker join을 확인하도록 작성했다. 기존 assertion은 삭제하지 않았다. mailbox에는 capacity/FIFO/상태 사본·거부 후 자동 replay 없음의 단위시험1개를 추가했다.

소스 작성 당시 직접 rustfmt/check9파일 종료0만 확인했으며 이후 실제 타입검사·unit/consumer 실행은 위 표가 소유한다. 제품 Inspector4는 아직 통과하지 못했고 on/off strict·최종fmt gate도 미완료다. 공개 포트 생산자의 단독 인수와 제품 소비 검증을 구분한다. 이후 실제 viewport/focus/hit/a11y·OS 공백은 중앙 별도 배정에 따른다. P2/full13/native 영상과 default launcher 전환은 별도다.

## mixed 수리 인수 후 별도 소비 revision

이 절은 원 r5 실패를 소급 변경하지 않는 후속 입력이다. Meari 검증1df44812c11e61b7e8c000324e35ce461b0a53b0의 정확13(A4/M9/D0)을 before/final bytes·핀으로 보존한 뒤 기존 snapshot에 적용했다. SDK NativeChild r3 exact4·SDK3a8·UI11·UI9 r5 및 manifest/assets는 별도 보호했다. 새 P2/predicate나 live SDK domain7/app4는 복사하지 않았다. 기록은 `runtime/quality-review/native-inspector-mixed-r1/`에 있다.

live Meari는 승인된0589 clean→1df ff-only로 전환했다. 생산자 source-r7 inventory SHA8719DE3F27EDB77D5176F227A15A8A20E8BBA4E5C655B0F6A3158E84124B2AB0와 live 대조 시 문서2개 raw만 달랐다. 이 둘은 양쪽 모두 tracked이고 Git index/commit·필터 blob은 같았다. 처음 ignored로 잘못 해석한 인계는 즉시 정정했다. live는 mixed 줄 끝/producer는LF이며 문서 정규화·추가 복사0이다. 실행 snapshot381 raw는 전부 생산자 핀과 일치하므로 live379raw+문서2blob 일치와 구분한다. 최초 차이/필터/eol 관측을 보존했다.

| 회차 | 실제 결과 | receipt ms |
|---|---|---:|
| mixed consumer | 동일 r5 source9에서 기존5+Inspector4 모두9pass |258681|
| root fmt / bridge fmt |각 종료0|1336 / 803|
| root on all-target strict |종료0|20628|
| bridge on all-target strict |종료0|52010|
| off direct-include4 strict |PreparedPanel 타입 불일치 E0308, 종료101|15191|
| off strict-r2 |SRP 인자 경계 수리 후4target 통과|5940|

off 실패는 source-include한 local panel_model::PreparedPanel과 외부 FrameProjection helper가 요구한 canonical PreparedPanel이 구분된 데서 발생했다. local PreparedPanel의 bindings는 이미 같은 canonical ReviewBindings다. 따라서 helper를 필요한 ReviewBindings만 받도록 좁히고 document_factory/window_bridge는 prepared.bindings를 전달하는3파일 수리를 했다. generic/Any/새 SDK나 harness 검증 우회가 아니다. queue/권한/rollback 정책·원Frame/Dock는 그대로다.

수리 전 source3와 off 실패는 보존했다. 최종 source9r2 inventory SHA0DAE1CC30D2F0D94332E20245944C4B770C2736F6CA196D1E0A06F3C02EC517B, overlay22r2 SHA0B3C26038A2D344803C804155BC6C20B6C18A372293E02B21BBEDD374D697EB8다. 다음 표는 이 최종 소스를 다시 실행한 결과이며 위 첫9 성공을 소급하지 않는다.

| 최종 r2 검사 | 결과 | receipt ms |
|---|---|---:|
| consumer studio_frame_sdk |기존5+Inspector4=9pass/0fail|77246|
| off direct-include4 strict |종료0|5940|
| root on all-target strict |종료0|18202|
| bridge on all-target strict |종료0|15762|
| root fmt / bridge fmt |각 종료0|1424 / 634|

최종 gate 기능 인수: 12개 원 receipt는 모두 terminal이며 보존된 첫 off101과 후속 r2 성공을 구분한다. 최종6receipt는 전부exit0, 모든 원exec 종료/24launcher-child PID 부재·final-owned.json=[]를 확인했다. source9 live/snapshot, overlay22, Meari381, 보호 SDK/manifest 기준선과 assets83의 최종 차이는 모두0이다. 기록은 final-r2-* 차이 JSON/final-receipts.json에 있으며 heavy와 snapshot 소유를 PM에 명시 반환했다. source9+root gitlink의 정확2문맥 commit 준비만 인계했고 stage/commit0이다.

이 결과는 실제 SDK/editor의 controlled Inspector·dirty/identity·명시 Save/Accept·실패 rollback·stale callback/typed join 소비 범위다. 전체root시험을 새로 실행한 것이 아니며 live SDK 후속14를 포함한 전체 checkout의 성공으로 확대하지 않는다. GUI·OS·실제 Web 렌더·Native/Web 시각 비교·전체13/native 영상/default launcher·사람 품질 인수는 미실행/미완료로 유지한다. 이전 UI11/P3와 원0589 실패 입력은 불변 보존한다.

PM의 exact2문맥 index 배정 후 root gitlink 단독 `6757712b13846c91ae7db6637172bb192b4b41f9`와 UI9 단독 `f878e2e0054b4a1233d556c98421f9d6f0561a88`를 커밋했다. source9 핀 차이0·각 commit 정확 경로·최종 staged 잔여0을 기록하고 index를 PM에 명시 반환했다. SDK14/OBJ/다른 경로는 stage하지 않았고 push0·새 Cargo/GUI0·snapshot 수정0이다. 이 후속 커밋 기록이 위 당시의 stage/commit0을 대체한다.
