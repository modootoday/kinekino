---
id: evidence:kinekino/20261029113844
title: 제품 Meari CSS 전달 경로 — 정적 조사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 Meari CSS 전달 경로 — 정적 조사

관측 기준은 root3ebb4498655547aaa4f065fe0c34880e03e75c3a / Meari4331ff3ea5cad71a7159bdb9e3f86fbfccb16220다. 사용자 보고 증상은 흰 배경에 UI가 흩어져 보이는 것이다. 이번에는 source-only로 연결을 추적했으며 GUI·스크린샷·computed style·새 paint/렌더를 실행하지 않았다. 증상의 재현 여부·재현율·원인은 미확정이다. systematic-debugging의 증상/전달 단계 분리 원칙을 적용했다.

## 확인한 연결

| 단계 | 현재 소스에서 확인한 사실 |
| --- | --- |
| 제품 진입 | main의 NativeViewer 분기는 product_viewer_launch::run_product_viewer를 호출한다. 해당 caller는 dark:true/1180×760 DocumentOptions와 prepare_product_viewer_launch를 사용한다. 과거 egui나 별도 기술 시험창은 이 경로의 증거가 아니다. |
| 고정 graph | viewer_native_launch::product_frame이 Viewing graph와 StudioFrame/Card provider를 ProductFrameBlueprint의 닫힌 source inventory로 조립한다. ProductFrame/Viewing/Studio TSX의 aw-* class와 별도 스타일 전달을 사용한다. ModuleProgram 자체가 CSS를 내장한다는 전제는 없다. |
| 스타일 조립 | player_shell_document는 ProductFrame일 때 prepare_product_frame_styles, standalone Viewing일 때 prepare_viewing_styles를 선택한다. 전자는 tokens/components/viewing/studio/subtitles CSS를 고정 순서로 합친다. |
| 토큰/폰트 | tokens.css의 :root에 dark color-scheme·공통 표면/텍스트/여백/폰트 토큰이 있다. components.css는 html/body 배경·색·Pretendard family와 공통 입력/버튼을 이 토큰으로 정의한다. Pretendard Regular/SemiBold/JP 세 TTF와 라이선스·해시를 기존 asset manifest로 전달한다. |
| URL/테마 | prepare_styles는 기존 StylesheetProject(mode:Css, bridge_theme:false)를 asset rewrite에 사용하고 반환된 각 파일을 실제 css 문자열로 합친다. bridge_theme:false는 제품 :root 토큰 누락을 뜻하지 않는다. 별도 새 theme이나 임의 CSS를 추가하지 않았다. |
| 문서 생성 | mount_prepared_product_shell_with_callbacks는 InteractiveDocument::from_module_program_with_callbacks에 실제 &styles.css와 Some(styles.assets)를 넘긴다. Native document는 html dark class와 __bridge_stylesheet 노드를 만들고 CSS text 삽입·upsert_stylesheet_for_node·layout을 수행한다. |
| 실제 창 소유 | ShellHostController는 같은 NativeDocumentHandle로 NativeHostRuntime을 구성한다. Native window driver는 access::facade(handle.clone())를 창에 넣고 SharedDocument.inner/inner_mut는 그 handle의 동일 renderer.document를 borrow한다. 이 경로에서 CSS 없는 새 문서로 바꾸는 코드는 확인되지 않았다. |
| 프레임/패치 | checked_window는 같은 문서의 resolve_assets와 inner_mut를 사용한다. keyed patch는 renderer.document의 기존 source root 아래를 수정한다. 준비용 template HtmlDocument는 subtree patch를 만들기 위한 것이며 live head stylesheet를 대신하는 새 창 문서로 설치되지 않는다. 제품 caller에서 replace_stylesheet(빈 CSS)를 호출하는 연결도 확인되지 않았다. |
| standalone Card | product_native_launch의 prepare_product_styles 결과가 ProductWindowVisuals(css/assets)→product_window→document_factory의 동일 Native 생성자로 전달된다. 공통 text input 클래스의 표면/폰트 토큰 정의도 존재한다. |

확정된 stylesheet 전달 누락은 없다. 따라서 production CSS·테마·렌더 코드의 수리 후보를 억지로 만들지 않았다. 이는 증상이 없다는 증거나 실제 계산 스타일·레이아웃·시각 일치의 완료 판정이 아니다. 기존 product_styles의 폰트/표면 paint 시험은 작은 Probe에 대한 별도 이력이며 현재 제품 전체 화면의 GUI 재현으로 승계하지 않는다.

## 다음 실제 GUI 인수에서 필요한 동일성

원래 실행 파일의 absolute path/SHA/mtime와 launch args·실제 선택된 Native/standalone profile, graph와 styles bundle/source hashes, font/asset pins, NativeDocumentHandle/HostStamp와 window·FrameTicket을 함께 고정한다. 그 동일 실행/문서/프레임의 computed style 또는 paint/스크린샷을 증상과 대조한다. 다른 시점의 egui 시안·기술 Probe·다른 실행 파일을 대신 보지 않는다.

최근 UI gate는 Native libtest를 새로 빌드했고 normal player.exe는 보호된 기존 artifact로 유지했다. 따라서 현재 소스가 연결돼 있다는 사실만으로 사용자가 본 실행 파일이 이 소스를 포함했다고 단정하지 않는다. 이는 stale binary가 원인이라는 확정도 아니다. 기존 actor/자산·스타일을 기록해 대조하는 미완료 항목이며 현재 새 GUI 실행 권한은 없다. 사용자에게 과거 artifact 추가 제출을 지금 요청하지 않는다.

관측 파일 목록/해시는 `runtime/coordination/ui-style-delivery-trace-r1/observed-sources.json`에 기록한다. 코드 변경·새 컴포넌트·테마·대규모 리디자인·Meari49 gate 합성은0이다. 전체13화면군·하위탭·caption/입력/레이아웃/색상/폰트의 실제 확대 시각 인수는 계속 필요하다.
