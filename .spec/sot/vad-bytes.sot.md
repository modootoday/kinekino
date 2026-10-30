---
id: sot:kinekino/vad-bytes
title: VAD 고정 배포 문서 원 bytes
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# VAD 고정 배포 문서 원 bytes

2026-10-23 중앙의 소형 문서 수집 배정으로 `runtime/coordination/vad-release-documents-r1`에 원 HTTP body·headers·transfer/exit·책임별 사본을 보존했다. 목적 HTTP CLI curl의 direct --output을 사용했고, 응답1MiB·redirect3·connect10초/total30초·HTTPS만 허용했다. 새 PS/Python 정책 도구는 만들지 않았다. 원 bytes를 PowerShell text pipeline으로 저장하지 않았다.

`manifest11.json`은 요청/최종 URL·revision·bytes/SHA256·content type/status/exit·원본/사본 경로와 역할을 기록한다. 11응답 모두 HTTP200/curl0, 합계429253B다. 검토한 license/notice text3과 기타 provenance/vendor 문서·metadata 및 HTML 증거를 구분했다. HTML/오류를 license로 대체하지 않았다. 선택한10기존 model/runtime/audio 파일을 다시 다운로드하거나 재해시한 것이 아니다.

| 실제 확보한 원문 | bytes | SHA256 |
|---|---:|---|
| model/LICENSE — HF e71cae966052b992a7eca6b17738916ce0eca4ec |1075|2E63E9A38B6E8FC0C7BC37CE174CACA1862870856C6DAF5697CFB785E925520B|
| ort/LICENSE — upstream402979ac1bef89dcd55add41eaa7277c407599eb |1073|2F07C72751AED99790B8A4869CF2311DF85A860B22DED05FA22803587A48922C|
| ort/ThirdPartyNotices.txt — 같은 upstream commit |338088|143764B952FDB1A7C69CE653BFBA74A7744D6A8A573BFB73E235FBA356C83DE3|

HF resolve의 공식 redirect는 같은 revision과 Git blob ETag0bf5e90cac691b999d4a35044f97167d7bbbf0b9를 보존한다. 이제 license 파일 pin이 실제로 확보됐으므로 이전의 ‘정확 본문 bytes/pin 미확보’를 반복하지 않는다. 이 문서 수집으로 PRODUCT_RELEASE를 바꾸거나 품질·배포를 승인하지 않았다.

## 실제 runtime과의 대응 자료

| 기존 실제 입력 | 확보한 대응 문서 | 인수 가능한 사실 / 남는 경계 |
|---|---|---|
| model A4A068…/2243022B·고정 e71 | model/LICENSE 및 같은 revision API 원본 model/metadata.json | 고정 repository의 license 본문을 실제 pin으로 공급 가능. upstream export recipe·새 native worker load/결과·audio2 binding은 별도다. metadata의 repo counters는 revision endpoint에서도 변할 수 있다. |
| cached ORT lib62264C…/359948106B | ort/dist.tsv의 고정 Pyke commit1ee1f842…/기대 archive f99aae… + 위 upstream ORT license/TPN | archive 기대값·원 source 고지의 출처를 보존했다. cache lib의 실제 native build commit/config·포함 component를 upstream402979a와 동일하다고 확인한 것은 아니다. 모든 TPN entry를 현재 payload 구성으로 표시하지 않는다. |
| DirectML DLL9C9E6D…/18527776B/version1.15.4 | directml/package.nuspec803B, SHA2708C28252406D8EB04061ABD1A0CB06369255245F541170D8CD044A0E856F4E | 공식 nuspec는 내부 license 파일명을 LICENSE.txt로 명시한다. gallery license page의23916B HTML은 evidence에만 보존했다. 원 LICENSE.txt/동반TPN bytes, package→DLL hash chain은 미확보다. requireLicenseAcceptance=false를 약관 부재로 해석하지 않는다. |
| 실제 loaded System32 VC4 | vc/의 공식 source markdown3개를 cpp-docs commitf29e25fb8f05bb38cf0246476b5c1fc92bf33876에 고정 | redistribution/prerequisite/servicing 문서 원 bytes와 pins를 확보했다. main commit 조회 JSON은 mutable lookup으로 별도 보존하고 원문은 그 commit raw URL을 사용했다. 문서를 runtime notice나 사용자 license 보유의 증거로 표시하지 않는다. |

## 다음 PM 기술 gate에 공급할 것

기존 native package/archive의 추가 [제한 캐시 탐색](vad-native-cache-inventory-2026-10-23.md)은 종료했다. 알려진 NuGet/Cargo/ORT/ai-spike 책임 경로에서 exact DirectML1.15.4 package·현재 ORT 원 archive/고지를 찾지 못했으며, SDK 소비용 technical-input-evidence-map.json에 원 파일/문서 pin과 누락을 묶었다. archive 없는 상태를 추가 다운로드나 다른 버전 cache 합성으로 우회하지 않았다.

확보된 HF license1075B/2E63…는 향후 제품 record의 정확 license pin 입력 후보로 사용할 수 있다. weights/revision·실제 WAV2의 원 pin과 함께 공급하되, 별도 script나 새 사용자 법률 승인 체계가 필요한 상태로 되돌리지 않는다. 기존 사용자 기술 실측 허용은 유지하며 새 worker build/load·추론은 기존 중앙 자원 배정만 따른다.

실제 dedicated native worker source/build/BIN·ABI와 native module/notice mapping, 원 scope/control·audio/destination2·descriptor/CAS·일반 CLI submit/watch/get/결과·취소/재시도/cleanup 인수는 남아 있다. ORT source TPN 하나로 DirectML 고지나 검증된 Windows 최소 build를 채우지 않는다. 기존 cacheunknown2도 미해결로 남는다. 정확 native 공급/기술 gate 입력을 PM이 별도 배정하기 전에는 Cargo/추론/record 변경을 시작하지 않는다.

모델/대형 runtime/package/미디어 다운로드·Cargo·추론·GUI/WSL/REA·제품 record/공개 경계/index 변경0이다. canonical root에 소스 수정은 없다. modoo.today vendor version-pinned skill 본문 SHA `eb3f771f822a329bc1794cdb71aa3f49bff1f63bd82eb7ee0a36dd0c171ebb23`의 버전·원문·공백/역할 분리를 적용했다. [공식 source 조사](vad-official-license-sources-2026-10-23.md)와 [실제 입력 준비표](vad-positive-input-readiness-2026-10-23.md)를 연결한다.
