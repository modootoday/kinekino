---
id: sot:kinekino/vad-license-native-windows
title: VAD 공식 license·native 고지·Windows 근거
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# VAD 공식 license·native 고지·Windows 근거

후속 소형 HTTP 수집에서 HF e71 LICENSE1075B/SHA2E63E9A38B6E8FC0C7BC37CE174CACA1862870856C6DAF5697CFB785E925520B와 ORT 고정 source LICENSE/TPN 원 bytes를 확보했다. [원 bytes 정본](vad-release-document-bytes-2026-10-23.md)을 우선하며 아래 ‘아직 bytes 미취득’은 최초 웹 도구 조사 시점의 이력이다. DirectML HTML은 license로 채택하지 않았다.

2026-10-23 PM의 공식 1차 웹 읽기 배정으로 조사했다. 모델/runtime/package 다운로드, 추론, Cargo, GUI/REA/WSL, 시스템 변경, 제품 record 변경은0이다. 브라우저 렌더 대신 웹 도구의 공식 문서/commit diff를 읽었다. 아래 본문 확인과 원 파일 bytes 취득·SHA256 보존은 구별한다. 후자는 아직 수행하지 않았다.

## 바로 보완한 근거

고정 HF revision 자체가 **LICENSE를 추가한 commit**이다. [e71cae966052b992a7eca6b17738916ce0eca4ec](https://huggingface.co/onnx-community/silero-vad/commit/e71cae966052b992a7eca6b17738916ce0eca4ec)의 21줄 diff에서 Silero Team MIT 본문을 확인했다. 따라서 기존 ‘로컬에는 README의 MIT 선언만 확보’라는 사실을 ‘고정 모델에 대응하는 공식 license 본문을 찾지 못함’으로 확대하면 안 된다. 동일 revision의 공식 LICENSE 원문 출처가 이제 확인됐다. MIT 본문은 사용·수정·배포 등을 허용하고 copyright/permission notice 포함을 조건으로 둔다. 별도 모델 품질 승인·등록 절차는 이 본문에 없다.

[고정 model.onnx 페이지](https://huggingface.co/onnx-community/silero-vad/blob/e71cae966052b992a7eca6b17738916ce0eca4ec/onnx/model.onnx)의 SHA256은 캐시의 `a4a068cd6cf1ea8355b84327595838ca748ec29a25bc91fc82e6c299ccdc5808`와 일치한다. 원 [upload commit730bca06348210595fb8cc15f28538707e58abbb](https://huggingface.co/onnx-community/silero-vad/commit/730bca06348210595fb8cc15f28538707e58abbb)는 2243022bytes의 같은 LFS oid와 별도 FP16 파일을 추가했다. 이는 uploader/repository 내 파일 계보이며 upstream checkpoint→export script/version/flags→ONNX의 재현 recipe를 제공하는 것은 아니다. 기존 weights를 다른 모델로 바꾸거나 다시 받을 이유로 삼지 않는다.

## 버전별 source와 적용 범위

| 책임 | 정확한 공식 출처·버전 | 문서에서 확인 / 문서로 채워지지 않는 것 |
|---|---|---|
| 고정 모델 license | 위 HF e71 commit의 LICENSE diff; [동일 revision README](https://huggingface.co/onnx-community/silero-vad/blob/e71cae966052b992a7eca6b17738916ce0eca4ec/README.md); [Silero upstream LICENSE](https://github.com/snakers4/silero-vad/blob/master/LICENSE) | 같은 고정 HF tree의 MIT/Silero copyright 본문을 확인했다. master upstream은 보조 대조이고 e71을 대신하는 고정 revision이 아니다. 원 text 파일의 bytes/SHA, export recipe·graph 전체 precision inventory는 남아 있다. |
| 모델 변환 provenance | 위 upload730bca0 및 [HF 공식 commit history](https://huggingface.co/onnx-community/silero-vad/commits/main) | Xenova uploader/commit·동일 weights oid·FP16 sibling 및 후속 quantized 파일 추가를 구별한다. 업로드 이력을 export 재현성·모든 tensor FP32 검증으로 표시하지 않는다. 조사한 공식 README/commit에는 exact upstream checkpoint/export recipe가 문서화되지 않았다. |
| ORT 상위 license | [공식1.31.0 release](https://github.com/microsoft/onnxruntime/releases/tag/v1.31.0)가 가리키는 commit `402979ac1bef89dcd55add41eaa7277c407599eb`; [그 commit LICENSE](https://raw.githubusercontent.com/microsoft/onnxruntime/402979ac1bef89dcd55add41eaa7277c407599eb/LICENSE) | MIT 본문과 고지 포함 조건을 확인했다. 기존 조사에서 미확인했던 release tag→immutable source commit 연결은 보완했다. 이것만으로 Pyke cache lib의 실제 build commit/config를 그 commit이라고 확정하지 않는다. |
| ORT third-party 고지 | [동일402979a ThirdPartyNotices.txt](https://raw.githubusercontent.com/microsoft/onnxruntime/402979ac1bef89dcd55add41eaa7277c407599eb/ThirdPartyNotices.txt) | 구성요소별 고지·조건을 별도로 제공한다. 상위 MIT 한 장으로 대체하지 않는다. 광범위 source notice 목록의 모든 component가 현재 static lib에 포함됐다고 단정하거나 임의로 고지를 삭제하지 않는다. 실제 배포에 동반된 고지/빌드 설정과 대응해야 한다. |
| Pyke 공급 경로 | [고정 ort commit1ee1f842e8a381153ede7a506f01734e545854e0의 dist.tsv](https://raw.githubusercontent.com/pykeio/ort/1ee1f842e8a381153ede7a506f01734e545854e0/ort-sys/build/download/dist.tsv) | x86_64-pc-windows-msvc/directml 행은 ORT1.31.0 archive와 기대 SHA `f99aaef23f12986abb05924c944d8433521263ad756b462df762a5a982c98405`를 명시한다. archive 기대 hash와 실제 lib62264…의 build/source/notice 연결은 별개다. archive를 새로 받지 않았다. |
| DirectML binary terms | [Microsoft.AI.DirectML1.15.4 공식 NuGet](https://www.nuget.org/packages/Microsoft.AI.DirectML/1.15.4)의 License Info가 연결하는 [버전별 license 본문](https://www.nuget.org/packages/Microsoft.AI.DirectML/1.15.4/License) §1(a),1(c),3(c),3(e) | Windows/Xbox에서 설치·사용하는 범위와 해당 platform의 ML apps/services/games 내 배포 범위, 동반 third-party terms 및 고지 보존/standalone 배포 제한을 확인했다. 기술적 사용과 외부 배포 조건을 구별한다. GitHub DirectML 예제 source의 MIT를 이 binary terms로 대체하지 않는다. package192.92MB는 다운로드하지 않았으며 포함 TPN 파일의 정확 원이름/bytes와 DLL9C9E…까지의 package hash chain은 남아 있다. |
| DirectML 버전·OS | [Microsoft 공식 release history](https://github.com/microsoft/DirectML/blob/master/Releases.md), [Microsoft Learn version history](https://learn.microsoft.com/en-us/windows/ai/directml/dml-version-history) | 1.15.4는 feature level6.4/redistributable-only이고 해당 버전의 inbox OS release는 없다. DirectML1.0의 Windows10 1903/build18362 도입 사실을 별도1.15.4 bundle이나 제품 minimum_windows_build 인수로 승격하지 않는다. 이번 조사로 엔진/배포 방식을 교체하지 않는다. |
| VC runtime 배포·서비스 | [Redistribute Visual C++ files](https://learn.microsoft.com/en-us/cpp/windows/redistributing-visual-cpp-files?view=msvc-170), [VS2022 REDIST 목록](https://learn.microsoft.com/en-us/visualstudio/releases/2022/redistribution), [latest v14 supported 문서](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170) | 배포는 해당 VS license terms와 REDIST 목록의 조건을 따른다. package/runtime은 실제 MSVC build tools 이상 버전·target architecture가 필요하다. 문서는 서비스 측면에서 중앙 redistributable 배포를 권장하며 debug_nonredist는 배포 목록에서 제외한다. 현재 System32 VC4를 그대로 복사하거나 사용자의 license 보유 상태를 추론하지 않는다. 기존 설치 runtime을 쓰는 기술 시험과 새 DLL/설치 파일 재배포를 분리한다. |
| ORT 실행 prerequisite | [공식 install requirements](https://onnxruntime.ai/docs/install/), [공식 compatibility](https://onnxruntime.ai/docs/reference/compatibility.html) | Windows builds의 VC2019 runtime 요구를 확인했다. 일반 compatibility 페이지는 가능한 모든 조합을 시험한 목록이 아니라고 명시한다. 오래된 플랫폼 예시·일반 WinML 설명을 현재 static ORT/Pyke 전용 worker의 최소 OS 인증으로 쓰지 않는다. |
| 현재 Windows 서비스 범위 | [Windows11 release information](https://learn.microsoft.com/en-us/windows/release-health/windows11-release-information), [Windows10 Home/Pro lifecycle](https://learn.microsoft.com/en-us/lifecycle/products/windows-10-home-and-pro) | 2026-10-23 읽기 시 Windows11 26H2/26H1/25H2 등과 edition별 종료 시점이 표에 구분돼 있다. 24H2 Home/Pro는 종료 상태지만 Enterprise/Education 등은 다른 종료 시점이다. Windows10 Home/Pro의 일반 지원은2025-10-14 종료이며 LTSC는 별도다. OS lifecycle, runtime 문서의 호환 범위, 실제 제품의 tested minimum build는 서로 다르다. 새 target/support policy를 자동 선정하지 않는다. |

## 기술 검증과 외부 배포의 경계

기존 사용자의 모델 실측 허용은 유지한다. 확인한 Silero/ORT MIT의 사용 허용 및 DirectML의 Windows 기술 사용 범위를 ‘독립 배포 담당의 새 승인 없이는 시험 불가’라는 새 체계로 바꾸지 않는다. 누락된 export recipe나 사용자 품질 검토는 그 자체로 MIT 사용 금지를 뜻하지 않는다. 자원·worker build/load·추론의 중앙 단일 배정과 실제 native guard 조건은 기존 프로젝트 규칙으로 계속 적용한다.

제품 record는 의미가 다르다. 정확 license bytes·worker/manifest/module/notice pins가 결속돼야 하므로 웹 화면이나 문서 제목으로 ProductReleasePins를 채우지 않는다. 외부 배포에서는 동반 고지·실제 payload provenance·VS/DirectML 배포 조건과 runtime servicing 방식을 반영해야 한다. 이는 새 사용자 법률 승인 절차가 아니라 조회한 문서의 조건과 기존 코드 계약을 정확 파일로 충족시키는 작업이다. PM이 해당 작업을 직접 배정할 수 있다.

## 책임별 파일 준비안 — 아직 생성한 bundle 아님

| 준비할 파일/근거 | 입력·작성 방법 | PM이 바로 배정할 후속 |
|---|---|---|
| `model-license.txt` | 고정 e71 tree의 LICENSE 원 bytes를 text로 취득·개행/인코딩 보존·SHA/bytes 기록. commit diff에서 내용을 재작성한 파일에 원파일 SHA를 붙이지 않는다. | 작은 공식 text 취득/핀 보존은 모델 재다운로드와 분리 가능. body 출처는 확정됐으며 ‘본문을 못 찾음’ 상태를 반복하지 않는다. |
| `model-origin.json` | e71, uploader730bca0, 같은 model oid/size와 실제 weights pin, export recipe=unverified를 명시한다. | 변환 recipe를 모르는 사실을 남기고 기존 FP32 선택을 유지한다. 새로운 script/모델/quantization을 도입하지 않는다. |
| `ort-license.txt`, `ort-third-party-notices.txt` | 위 immutable402979a의 원 text + 실제 Pyke 배포 동반 notice/build inventory 대응. 파일명은 제안이며 제품 role은 Notice로 분리한다. | source documents 취득과 archive/build 대응을 별도 산출물로 배정. 상위 MIT만으로 native closure가 끝났다고 표시하지 않는다. |
| `directml-license.txt`, `directml-third-party-notices.txt` | 공식 NuGet1.15.4가 제공하는 실제 license/TPN bytes와 package→DLL hash chain. 임의 GitHub MIT/추정 TPN 파일명으로 채우지 않는다. | 이번에는 공식 license 본문까지 확인했다. 대형 package 취득 없이 확보 가능한 공식 text/source metadata를 우선 수집하고 실제 archive 취득은 별도 다운로드 배정으로 남긴다. |
| `vc-prerequisites.json` 또는 정확 redist inventory | build toolchain/arch와 설치 runtime 조건, 배포 시 해당 VS terms/REDIST 대응·servicing 선택을 기록한다. | 기존 설치 VC4의 기술 pin을 유지하며 시스템 파일 복사를 배정하지 않는다. 앱 prerequisite 사용과 설치 파일 동봉을 구별해 PM이 제품 방식을 정한다. |
| `native-build-origin.json`, 최종 `release.manifest.json` | 실제 전용 native worker의 source/build/ABI/BIN pin과 module/notice/phase·실제 tested OS 근거를 기존 schema에 결속한다. | 현재09F 실험 도구/default6AF/child22를 제품 worker로 전용하지 않는다. 새 기술 gate가 배정된 후 관측값으로 작성한다. source tag402979a를 cache build commit의 확인값으로 복사하지 않는다. |

이번 웹 도구는 HF pinned LICENSE의 blob/raw/resolve와 일부 API/nuspec 직접 열기에 실패했으나 공식 commit diff 및 NuGet 페이지의 License Info 경유 읽기는 성공했다. 실패를 원문 부재로 해석하지 않았다. 웹 렌더 텍스트를 원 bytes/hash로 표시하지 않으며 현재 파일 준비안의 pin은 미발급이다. [기존 실제 입력 준비표](vad-positive-input-readiness-2026-10-23.md)의 weights/audio2 경로·기술 pin과 이 조사 결과를 함께 사용한다.

## 스킬 적용

modoo.today 검색에서 cross-runtime 위임/데이터학습·성능 실험 등은 현재 문서/고지 조사와 달라 제외했다. 실제 적용은 `modootoday/operations-cs/version-pinned-procedure-from-vendor-docs` SKILL.md, SHA256 `eb3f771f822a329bc1794cdb71aa3f49bff1f63bd82eb7ee0a36dd0c171ebb23`다. 본문을 읽었고 추가 필수 참조는 없다. 제품/정확 버전·문서 적용 범위·documented/not documented·read-only/state change를 위 표와 파일 준비안에 적용했다. 공급된 공식 페이지가 있으므로 임의 승인 질문이나 다른 runtime 실행을 추가하지 않았다. 기존 QA verdict의 실제 관측/승인 분리도 유지한다. 어떤 준비 파일도 설치/게시/제품 record에 적용하지 않았다.
