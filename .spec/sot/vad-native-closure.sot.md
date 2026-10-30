---
id: sot:kinekino/vad-native-closure
title: 고정 VAD native 배포 closure의 추가 근거
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 고정 VAD native 배포 closure의 추가 근거

2026-10-21 중앙 source-only 후속 조사다. [이전 P1 조사](../../../../../plans/player/ai/cached-vad-ort-worker-closure-2026-10-21.md)의 read15/원파일pins는 재사용했다. 이번 추가는 고정 dist provenance와 공식 배포 조건/파일 역할의 대조다. 바이너리·모델 다운로드, 설치, 새엔진 교체, Cargo/추론/worker 실행0이다. 제품 배포 승인·AI 지원·품질 승인0이다.

## 새로 확인한 provenance

local ort-sys2.0.0-rc.14 .cargo_vcs_info.json은 pykeio/ort commit `1ee1f842e8a381153ede7a506f01734e545854e0`를 선언한다. 해당 commit의 [공식 dist.tsv](https://raw.githubusercontent.com/pykeio/ort/1ee1f842e8a381153ede7a506f01734e545854e0/ort-sys/build/download/dist.tsv)와 local source의 Windows x64/directml 행은 archive `https://cdn.pyke.io/0/pyke:ort-rs/ms@1.31.0/x86_64-pc-windows-msvc+directml.tar.lzma2`, SHA `f99aaef23f12986abb05924c944d8433521263ad756b462df762a5a982c98405`를 가리킨다. 이는 기존 dfbin 캐시 디렉터리 식별자와 일치한다. local dist.tsv SHA는 `C97E3C8C098D9C704EB7E98BC568254BEDD487FAD9A61A7ACB088D63E5DE9675`다.

이 SHA는 압축 archive의 기대 hash다. 개별 onnxruntime.lib hash나 native build source commit을 뜻하지 않는다. local VerifyReader는 압축 stream을 hash하고 남은 bytes까지 소비하며 extractor는 archive를 푼다. 이번에는 archive를 다시 받거나 그 동작을 실행하지 않았다. 캐시 디렉터리 이름만으로 현재 파일이 변조되지 않았다고 판정하지 않는다. 개별 파일은 이전 실제 gate의 pins/protection을 별도 근거로 사용한다.

현재 직접 캐시 파일을 hidden 포함 열거해 onnxruntime.lib359948106B와 DirectML.dll18527776B만 확인했다. DLL FileVersion은 `1.15.4+241025-1615.1.dml-1.15.fac7597`다. LICENSE/NOTICE/ThirdPartyNotices는 이 디렉터리에서 발견하지 못했다. exact inventory와 local 원소스5 hash는 `runtime/coordination/vad-native-distribution-closure-r1/`의 cache-direct-files.json/local-source5.json(SHA53E991CCAB88934E7011F7AEE60CB11D77926915F4151EE3207120BD201558C2)에 있다. 전체PC의고지부재를주장하지않는다.

## 공식 원문과 현재 대응 수준

| 구성요소 | 확인한 공식 원문·조건 | 현재 payload와의 대응/잔여 |
|---|---|---|
| Rust ort/ort-sys rc14 | local LICENSE-MIT/APACHE와 Cargo declaration은 read15에서 이미 확인 | wrapper 소스 고지다. native ORT/DirectML 권리로 전이하지 않음 |
| ONNX Runtime1.31.0 | [version tag LICENSE](https://raw.githubusercontent.com/microsoft/onnxruntime/v1.31.0/LICENSE)는 MIT, copyright/permission notice 유지 조건을 명시한다 | dist 버전 선언과 대응하는 공식 원문은 확보 가능한 상태다. 캐시 lib의 정확 upstream build commit/config·archive 포함고지 hash 대응은 아직 없음 |
| ORT bundled components | [같은 version의 ThirdPartyNotices](https://raw.githubusercontent.com/microsoft/onnxruntime/v1.31.0/ThirdPartyNotices.txt)에는 여러 개별 구성요소의 고지가 있다 | 상위 MIT 한 장만 넣으면 closure가 되는 것은 아니다. 이 문서 전체의 구성요소가 실제 lib에 모두 포함됐다는 뜻도 아니므로 archive/build-specific inventory와 대응해야 함 |
| DirectML1.15.4 | [Microsoft 공식 release 목록](https://github.com/microsoft/DirectML/blob/master/Releases.md)은 해당버전을 redistributable-only로 표시하고 [NuGet1.15.4](https://www.nuget.org/packages/Microsoft.AI.DirectML/1.15.4)를 연결한다 | 현재 FileVersion과 버전은 일치하지만 NuGet archive→추출DLL의 공식 hash chain은 이번에 확인하지 않았다. 제3자hash목록으로 대신하지 않음 |
| DirectML binary 조건 | [NuGet 버전별 License 원문](https://www.nuget.org/packages/Microsoft.AI.DirectML/1.15.4/License)의 §1은 Windows/Xbox 사용 및 지정된 앱/서비스/게임 배포 범위, 제3자 고지를 다룬다. §3은 고지 훼손과 허용범위 밖 단독배포 등에 제한을 둔다 | GitHub 예제코드/ort wrapper MIT와 별개다. 원문/동반 ThirdPartyNotices의 제품 배포 편입이 필요하며 현재캐시에는없다. package ZIP을받지않았으므로원TPN파일명/내용/전체package목록은미검증 |
| VC runtime4 | [Microsoft 재배포 문서](https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files?view=msvc-170)는 허가된 VS 사용자·해당terms/REDIST목록에 따른 배포를 설명한다 | 기존 actual loaded5 중 MSVCP140/MSVCP140_1/VCRUNTIME140/VCRUNTIME140_1은 기술적 의존 관측이다. 사용자라이선스보유/개별DLL재배포허가를추론하지않음 |

NuGet LicenseInfo 경로 조회는 실패했지만 NuGet 페이지의 공식 License 링크(`/License`)에서 버전별 원문을 읽었다. ORT tag ref→immutable commit을 조회한 GitHub API는 이번 도구로 읽지 못했다. version tag 원문을 읽은 사실과 native build commit/문서 원bytes hash를 보존한 상태는 다르며 후자는 잔여다.

VC의 [공식 VS2022 REDIST 목록](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution)도 이어서 확인했다. 해당terms를전제로VC/redist아래배포범위와debug_nonredist제외를설명한다. 따라서문서위치자체는확인됐으나현재loaded VC4가어느허가된toolset/redist payload와대응하는지,제품toolchain에맞는지,사용자license조건을충족하는지는미인수다. System32의현재파일을임의복사하는근거로쓰지않는다.

## static/dynamic의 실제 파일 차이

| 배포 항목 | 현재 static 후보 | dynamic 후보 |
|---|---|---|
| ORT native code | build 단계에서 기존 onnxruntime.lib를 worker에 링크한다. .lib359MB 자체를 제품 runtime 파일로 복사할 이유는 없음 | onnxruntime.dll 및 선택한 배포의 실제 companion/provider DLL이 필요. 현재 캐시에 그 DLL은 없으므로 준비됐다고 할 수 없음 |
| DirectML | 실제 r3는 CPU EP여도 DMLCreateDevice1 링크가 필요했고 DirectML.dll을 load했다. 이 현재lib를쓰면별도runtime closure가남음 | 선택한 실제 ORT 배포가 DML을요구하면동일한별도DirectML payload/조건이필요. dynamic이라는이유로없어진다고하지않음 |
| build-only input | 기존 Windows SDK DirectML.lib import library(1956B, 원pin874E30D9…)와 MSVC·Rust link 환경 | import/dynamic load 방식에따른별도검증. load-dynamic feature는캐시staticlib를DLL로변환하지않음 |
| VC dependency | 현재원worker의actual VC4를배포또는공식prerequisite정책으로해결해야함 | 새실제DLL/worker도target별의존성검사가필요하며기존VC4를전수목록으로사용하지않음 |
| 고지/원출처 | Rustwrapper LICENSE·nativeORT LICENSE/TPN·DirectML terms/TPN·해당VC redist terms/inventory와build provenance | 동일책임별고지에실제새payload의원package/hash/architecture를추가 |

DirectML.Debug.dll은 Microsoft가 설명하는 [선택적 개발 debug layer](https://learn.microsoft.com/en-us/windows/ai/directml/dml-debug-layer)다. 현재 캐시/loaded5에서는 확인하지 못했고 release payload에 기계적으로 추가하지 않는다. Microsoft의 [DLL 배포 결정 문서](https://learn.microsoft.com/en-us/cpp/windows/determining-which-dlls-to-redistribute?view=msvc-170)는 실제 의존성 및 해당 REDIST/도구버전과 맞춰 선택하도록 안내한다. 이 조사에서 System32 파일을 복사하거나 redist를 설치하지 않았다.

기존 private player-model-spike BIN09F는 여러실험dependency/source를담은증거실행파일이다. 그대로제품worker로배포하지않는다. 새작은Rustadapter/worker를만들때원현재staticlink근거를출발점으로삼을수있지만, 배포closure와별도실행인수전까지상용/공개배포가능으로표시하지않는다. CPU-only새engine/build로전환하는조사는이번범위밖이며아무것도교체하지않았다.

## 다음 제품 연결에 필요한 정확한 잔여

1. 기존 Pyke dist의 실제 upstream ORT commit/build flags·포함nativecomponent와 LICENSE/TPN의 연결. version1.31.0 표기만으로 개별lib 재현·권리를 완료 판정하지 않는다.
2. 공식 NuGet DirectML1.15.4 package의 license/TPN 파일목록과 DLL9C9E6D82…E92DA1까지의 원archive/추출 hash chain. 바이너리다운로드권한이없어이번에는취득하지않았다.
3. 제품의 Windows x64 runtime layout에 worker·DirectML·필요VC/OS prerequisite·고지 파일을분리한package manifest. build-only.lib/importlib를runtimebundle로혼동하지않는다. 설치/전역PATH/시스템DLL교체0이다.
4. 제품 build toolchain에맞는공식VC배포방식·license/REDIST목록·아키텍처/버전의 PM 인수. 현재PC에있다는사실은제품배포허가가아니다.
5. 원cachedstatic외payload를쓰려면새별도pin/명시자원배정·실제SDK/CLI load/취소/cleanup gate. 현재외부registry/worker0을유지한다.

따라서 **공식 원문 위치와 버전 대응은 보강됐지만 현재2파일 캐시만으로 제품배포 closure는 미완료**다. 미완료를새엔진/모델교체나기존추론실험반복으로우회하지않는다. 순수parser/P0·SDKsource작업은그대로진행가능하다.

## 방법과 불변

검색의 한국약관작성/pipa-review는 현재native license대조와달라채택하지않았다. 앞서본문/참조를읽은 modoo code-change-review-verification은 기존“cache원출처불명/고지부재”주장을actual dist row·direct inventory·원primarysource와대조하는표에적용했다. plan-authoring의관측/미검증분리와기존provenance-licenses의wrapper/weight/native권리분리를유지했다. primary문서만근거로썼으며제3자블로그/재배포목록의hash나허가판정을인수하지않았다.

코드·원P0/parser source/S35·jobs/registry/rootCargo/index변경0. 이번네트워크사용은공식텍스트문서읽기뿐이며binary/modeldownload0이다. 원실험·사람품질·전체goal잔여를축소하지않는다.
