---
id: plan:kinekino/20261029113655
title: SDK/CLI source261 mount 후계 채택 보충
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SDK/CLI source261 mount 후계 채택 보충

기존 sdk-cli-source260-adoption-plan-2026-10-25.md의책임·순서·공개경계를유지한다. 최종선택 source60DFB2E3B8926C0EEC5FE6B03FD5C91D4B8302DDD9CDF31D03CC6BEFADDCD2B9. 동일66경로만 sdk-cli-source261-adoption-candidates.csv에후계payload/원Dpreimage SHA·bytes·mtime·FI/신규여부를추가했다. 전체공급261을새로소유권승인한목록이아니다.

66후보중선택과동일5/다름61(신규42). src/application/mod.rs는기존WIP SHA022B19D71163F31F45DEDA2EEFFC3CC51344E60DEE6EA5D2015E0D3850ADCA14 대선택919DBC7E75DE7AA38636A4E951B97AB7500A92D1A96D6B918FFCDF6CD9D037BC로상이하므로 preserve_WIP=true. 파일전체덮어쓰기/전체stage금지, PM별도원WIP보존병합필요. 나머지60개차이도담당·연결부검토후에만명시채택한다.

원source260대비이번SDK후계는visible_render_only/{mod.rs,observation.rs,tests.rs}3개. UI의live_entry2/mount_proposal1은별도UI소유로이66CSV의SDK단독소유권으로추가하지않았다. Meari21은담당독립소유. rootmodule/Cargo/sharedtestsupport등원66밖의연결부는PM전체조합채택계획에연결해야한다.

공개검토경계: CSV는Rust공통SDK/외부OS·mpv어댑터/테스트소스만대상. 모델/바이너리/캐시/미디어/비밀/runtime/.spec/.agents는채택대상아님. 신규42개는공개책임디렉터리·기존ignore기본거부경계·재배포권한/비밀노출을PM채택시검토할명시경로이며, 여기서전부공개승인으로표시하지않는다. 광범위gitignore해제/파일별예외나열0.

검증완료범위: 기존source1448 actual4 및CPU71, 다음source2CBED494 focus13/traceprivacy/관련Root회귀25, 최종source60DF의jointfmt/type/strict/Rootjobsno-run+liveentry원ignored6/rawpositive·tamper2/Rootfactory·start2/ManualRepairgraph1=11pass. 결과사슬을혼합하여최종전체기능완료로표시하지않는다. 전체unfilteredtest·최종GUIroute·수동시각/입력/AI사용자품질은별도. canonical부분조합은새조합이므로기존검증재사용단정0.

현재는준비만이며D제품소스/index/submodule/stage/commit/push변경0. PM이Meari소유커밋→Root기능별정확경로채택을직렬배정한후진행한다. UI단독GUI슬롯중root변경금지.
