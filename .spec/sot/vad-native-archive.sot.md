---
id: sot:kinekino/vad-native-archive
title: VAD native archive·고지의 제한 캐시 탐색
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# VAD native archive·고지의 제한 캐시 탐색

2026-10-23 중앙 배정의 bounded read-only 조사다. 결과는 `runtime/coordination/vad-native-cache-inventory-r1`의 scopes/dfbin-direct/legacy-lib-direct/nuget-cache-direct/cargo-matches/ai-source-matches/summary JSON과 HANDOFF.md에 있다. 원 binary 재해시·변경, 네트워크/재다운로드·추론·Cargo·WSL·archive 추출0이며 전체 drive/home 검색0이다. 아래 한정 범위에서 탐색을 종료했고 다른 PC 경로의 전역 부재는 주장하지 않는다.

| 확인한 책임 경로 | 실제 관측 | 제품 입력 판정 |
|---|---|---|
| 기본 `.nuget/packages/microsoft.ai.directml/1.15.4`, package parent, 전체 기본 packages, `%LOCALAPPDATA%/NuGet/v3-cache` | 해당 경로 없음, NUGET_PACKAGES override 없음 | 이 기본 cache로는 exact package/LICENSE.txt/TPN를 공급할 수 없다. 다른 모든 cache의 부재를 뜻하지 않는다. |
| `%USERPROFILE%/AppData/Local/NuGet/Cache` 직접 목록 | Chocolatey/VC/Python 관련 nupkg21, DirectML1.15.4·현재 ORT distribution 없음 | 기존 archive는 있지만 대상과 무관하다. 추출·새 검증이나 공식 native provenance로 전용하지 않았다. |
| Cargo registry cache의 index별 직접 파일 | ort/ort-sys rc12/rc14 source crate4 | Rust wrapper source archive이며 native runtime/package가 아니다. source license를 DirectML binary 고지로 대체하지 않는다. |
| `%USERPROFILE%/AppData/Local/ort.pyke.io/dfbin/x86_64-pc-windows-msvc/f99aaef23f12986abb05924c944d8433521263ad756b462df762a5a982c98405` 직접 목록 | DirectML.dll18527776B + onnxruntime.lib359948106B만 존재 | 원 native archive·LICENSE/TPN·별도 build/hash manifest 없음. cache 폴더 이름은 실제 archive 현재 검증이 아니다. |
| 같은 x64 parent의 다른 관측 hash2 | b685bfc… lib305821070B/DLL18527776B;540D19…/onnxruntime/lib에 lib303265000B/DLL18527776B | 다른 lib를 현재359948106B 배포본에 합치지 않는다. DLL 크기가 같다는 이유로 같은 SHA/package라고 하지 않는다. 관측 directory 깊이까지만 목록을 읽었다. |
| `runtime/ai-spike` root/source의 대상 이름·archive 패턴 직접 검색 | DirectML/ONNXRuntime/ort/해당 nupkg/lzma2 공급 파일 없음; downloads 경로 없음 | 기존 모델/미디어/source 전체를 재귀 검색하거나 미디어를 열지 않았다. native package 매핑이 추가로 생기지 않았다. |

local `ort-sys-2.0.0-rc.14/build/main.rs:119-145`는 cache/dfbin/target/dist.hash를 구성하고 HTTP reader→VerifyReader→extract_tgz로 임시 디렉터리에 직접 추출한다. 조사한 이 call path는 compressed archive를 별도 cache 파일로 보존하지 않는다. 이는 archive가 남지 않을 수 있는 소스 설명이며 과거 실제 archive 내용·hash 성공의 새 관측은 아니다. 해당 build 코드를 실행하지 않았다.

## SDK 기술 gate에 제공하는 정확 입력

같은 audit의 `technical-input-evidence-map.json`에 실제 weights 경로/원 pin·새 model license1075B/SHA2E63…·ORT build-only lib62264…/DirectML9C9E…/SDK import lib874E…의 기존 pin 출처·loaded5·audio2 원 manifest·미완료 SDK binding을 연결했다. 이것은 evidence-only 자료이며 application 설정/issuer/admission record가 아니다. 새 native bytes의 동일성을 확인한 pin으로 표시하지 않는다.

현재 대응 가능성은 다음과 같다. model license는 [실제 원 bytes](vad-release-document-bytes-2026-10-23.md)를 공급할 수 있다. DirectML은 공식 nuspec1.15.4와 DLL의 과거 FileVersion이 대응하지만 **package bytes→DLL SHA→LICENSE.txt/TPN의 원 chain은 성립하지 않았다**. Pyke dist.tsv의 기대 f99…는 있지만 archive 자체를 찾지 못해 목록/소형 고지를 추출할 수 없다. upstream ORT402979a의 source 고지를 cache의 실제 build commit/포함 component로 승인하지 않는다.

탐색은 여기서 종료한다. 이미 확보한 공식 nuspec/약관 HTML은 버전·조건의 증거로만 사용한다. 문서화된 작은 공식 raw license/TPN 출처가 추가 제공되면 제한 취득하거나, PM이 exact package/archive 취득을 별도 배정해야 원 package mapping을 진전시킬 수 있다. 추정 URL을 반복하거나 HTML을 package-original license로 변환하지 않는다. 대형 package 다운로드는 여전히0이다.

기존 사용자의 기술 실측 허용을 취소하거나 새 법률 승인 체계를 만들지 않는다. SDK는 위 원 파일·문서 핀과 명시 누락으로 전용 native CLI 기술 gate를 계획할 수 있으나 새 worker/BIN/ABI·실제 native notice/OS baseline·audio2 submit/watch/get/취소/재시도/원 owner 인수는 별도 중앙 배정이 필요하다. cacheunknown2도 그대로이며 안전 재사용을 주장하지 않는다. PRODUCT_RELEASE·코드·index 변경0이다. 적용한 vendor version-pinned skill의 정확 본문 SHA와 사용 판단은 audit HANDOFF에 기록했다.
