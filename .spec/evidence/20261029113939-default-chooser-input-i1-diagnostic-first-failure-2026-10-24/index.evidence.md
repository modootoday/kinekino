---
id: evidence:kinekino/20261029113939
title: I1 진단 actual 최초 실패와 관측 경계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# I1 진단 actual 최초 실패와 관측 경계

sourceCB574CC7/fixed764810AD/carry147을 그대로 사용해 기존 Card1만 실행했다. 원 resource-runner/checked-window protocol과 total20/reserve2/run18·모든 quota를 유지했고 Cargo/overlay0이다. 결과는 parent0pass/1fail/18.05초, runner terminal101/18075ms다. child48720/creation134372912584239147는 원 run18 cutoff에서124/forced/Empty, original_owner_retained=false였다. SDK ACK/join·WorkersQuiescent(Closed)는 미확인이다. 진단 출력이 있어도 시험 성공으로 표시하지 않는다.

근거는 runtime/verification/default-chooser-input-i1-diagnostic-actual-r1 및 S/runtime/quality-review/ui-viewer-no-media-process/61088-1792817658420601900의 원6 파일이다. seq14 initial 표시 뒤 checkpoint27~35가 다음을 관측했다.

| marker | 실제 범위 |
| --- | --- |
| first-text-before / first-text-ok | 첫 set_text 호출이 Ok로 반환됨 |
| probe return notify=true/close=false | probe의 반환 의도이며 실제 wake 수신과 구별 |
| delivery live=true/consumer=true/closing=false | 같은 observer 처리의 원 반환 경계 |
| 원 wake Ok/delivered=true | 원 wake 호출 결과; 이후 수신/처리와 별도 |
| HostTurn Completion/rev14/accepted1/rejected0/remaining0 | 원 다음 host callback 처리 수치 |
| bridge Ok/redraw=true, 최종 redraw=true | 원 bridge 반환·reflection 뒤 redraw 요청 |
| post-Text frame rev15/retained=true | 후속 notice의 실제 frame 관측 |

global checkpoint는35줄/failed=false, generic notice8 quota는 이미 소진돼 있었다. enqueue 내 정지나 최초 wake/HostTurn 미도착은 이번 관측과 맞지 않는다. 단, changed field·controlled/editor 각각의 값, current projection 정합, probe dirty/pending·원 post-card guard 값은 이 로그에 없다. changed event·restore·Save·VIEW·정상 cleanup은 미관측이며 absence를 전부 미실행으로 바꾸지 않는다.

읽기에서 live_fixture_turn이 card.after_dispatch 전 guard를 probe 캐시에 전달하고, 이후 실제 guard를 담은 SaveSnapshot은 Saving wake 관측만 처리하는 것을 확인했다. I4 Changed gate는 현재 값 및 캐시 dirty/pending을 기다린다. PM/독립 연구가 이 관측 캐시 freshness 결함을 P2로 확인해 최소 private 수리를 승인했다. 이는 실제 after guard/field 값을 모두 관측했다거나 수리로 전체 실패가 해결됐다는 뜻이 아니다. 제품 dirty 계산·Changed 단언·Save1·전이·wake·기한을 바꾸지 않고 같은 owner의 기존 post-card snapshot으로 입력용 캐시만 갱신하는 후계다. 실제 gate는 별도다.

최초 F169 실패6과 새 진단 실패6은 각각 보존했다. raw4 exclusive-open/EOF, 기록 runner/parent/inner PID86632/61088/48720 부재·writer0, protection199 stat/FI·source SHA 변화0이다. fixed2(F169/7648)와 원 실패6은 불변이다. 추가3/대표2/retry/Close/Abort/kill/기한 갱신0이다. 원 물리회수와 SDK cleanup 미확인을 구별한 실패 반환이며 전체 S/T/cache 보호로 확대하지 않는다.

modoo.today frontend-first-journey의 단계별 실제 관측과 미입증 구분, test-fixture-and-mock-build의 원 cutoff/raw/typed owner와 실패 보존 원칙을 적용했다. 새 gate 제어 도구는 만들지 않았다. 명시 반환은 같은 gate의 slot-return.json이다.
