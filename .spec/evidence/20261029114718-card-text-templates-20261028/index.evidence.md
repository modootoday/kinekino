---
id: evidence:kinekino/20261029114718
title: 텍스트 전용 카드의 실제 CLI — R118/R119
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 텍스트 전용 카드의 실제 CLI — R118/R119

최종 R120:2099사후custody 종료0,current source content/identity unchanged files=2362 확인. 텍스트3종 실제CLI/과거descriptor2개bytes동일/양strict/fmt/지정190/정상2bins/입력동일 완료. 아래 진행 중/미완료는 당시 기록이다. 역할별/페이지번호·부분retry/cleanup/ACK·GUI/사용자품질은 계속 남는다.

R120 통합gate:2090card all-target strict/2091fmt/2092기본 strict/2093기본 no-run 종료0. 외부 blitz-paint의 기존 경고1개는 남는다.2094Product145/2095Core18/2096Native26/2097OS1 모두 종료0,네 group-return의 execution_result/producer_integrity Ok/동일 R120 SOURCE.md 확인. Meari는622/624 manifest의 실제시험을 이번에 재실행했으며 새 컴파일이라고 표현하지 않는다.2098정상bins build 종료0,fixed-r120/native-bins exe/PDB 보존. GUI SHA9803E8D3AB3DC8F8A1B5FE46FA9E1BBD21169460D284D8138E56D6DAB6AE4A3F,CLI SHA100957F40F20AEB159695D534854A6CDC03E62E6D6A1216974274FE10EECC4C3.2099사후custody 진행 중이다.

R120 실제확장 통과:input-r120 inventory779C49A29232ACD8A69D0CB08A22744A8E046D6C8C7D6E7A082FAEA05CD5F9BF/2362파일,SOURCE.mdAE52632DAB4AF5A03DF3F2189833561A3D66735B53E8C799943C5E3FF04A44D2.2086snapshot·2087card no-run 종료0.2088실제CLI1시험 통과18.35초/67filtered,13CLI요청·텍스트전용3+기존frame2+자막1=완료6 및 원EOF 확인. 각 텍스트PNG 전체decode/크기·frame provenance null/source_frames=[]/원본frame파일부재·manifest모든entry hash/bytes 확인.2089원R115blob2개의 SHA검증·새DTO deserialize/serialize bytes 완전동일1시험통과. 실제 원본 파일은 읽기전용으로 소비했다.

fixed-r120 CLI SHA49E5D0C04003763CCE3DC872EEBA9A744CEF0EA91C7E6DEE1B5C93DDE2A30D13(실제측정 R118과동일),Jobs시험2FC30DCD71057EDC1585FCA59B593767D1E097C193EF77F3E15E5DE0A740FCE5,lib시험F482C14F431CDD110EEA77DFAD8F8B3040FD3FD42DB6E1CBAD88362D0ED5A056/PDB보존. 실제증거 runtime/verification/source-sdk-r120-text-cards-r1.2090card strict 진행 중이며 전체gate는 아직 미완료다.

R119 첫 실패:2084card no-run의 Jobs 통합시험에서 E0603 private inspect_static_png 호출로 실패했다. lib 시험 산출물은 생성됐으나 전체 no-run 성공으로 계상하지 않는다. 원 input-r119 inventory008A8F6F1D0697B9768FFB6450E98E0413817FE6083FCDBA415BAC84D349E5AB/2362파일,SOURCE.md41C6459C50378A0F37797DCE3C267D55AFAB65F7A00D6DDE8801B60C003893AB와 preimage-r119-private-png-test 보존. R120에서는 제품 함수 공개범위를 유지하고 시험의 생성PNG를 png decoder로 전체 decode/크기 확인하도록 수정했다. 제품 내부 strict parser 검증은 실제 executor에서 유지한다.2085fmt종료0,2086새 snapshot 진행 중.

R118 input-r118 inventoryD3E3C9C7DB2AE519E6A83A4D9C7E95A48EDC8A6666CF84F90CCA6AD9F201C7F3/2362파일,SOURCE.mdE0CC8841F104C906AC18DAE49201A66051012430CD7273913AA833AD691A04F4.2075fmt·2076snapshot·2077card lib/Jobs/CLI no-run 종료0. AGENTS40C7F5A6D382E0A61DAFCD7E651971E377704D38B59B7935B8D6C99B7E26768C 재확인. fixed-r118 CLI SHA49E5D0C04003763CCE3DC872EEBA9A744CEF0EA91C7E6DEE1B5C93DDE2A30D13,Jobs시험 B9FBDE181362FC44DB6FE5F522741A914F3EF7D860993D943771667606F699C9,lib시험2940F9083C7498F91789AAB497F5DA729EFCAE933033B11D4768827FCFD9DBE2/PDB 보존.

2078실제CLI1시험 통과14.22초/67filtered. 11CLI요청(기존10+텍스트전용정사각submit1),typed카탈로그/저장/제출/poll은별도. 기존프레임카드2+텍스트전용1+자막1의6이아닌4작업 완료를 원EOF/receipt에서 확인했다. 정사각 텍스트전용의 source_frames=[]/원본frame파일부재 및 manifest의 모든entry SHA/bytes 검증. catalog7개/공통폰트3개/typed동등성/품질미승인을 확인했다.2079기존bundle4시험통과(새canvas시험ignored1),2080준비4시험통과. 증거 runtime/verification/source-sdk-r118-text-cards-r1.

R118 실제CLI로 검증한 텍스트전용은 정사각1종이다. R119는3종 전체CLI발행과 actualPNG크기/프레임provenance null 확인으로 확장하고 원R115 prepared source blob2개의 SHA/재직렬화bytes 동일 검사를 추가했다. 원R118 시험2파일은 preimage-r118-text-cards-coverage에 보존했다.2081/2082fmt종료0,2083snapshot 진행 중이며 새 확장 시험은 아직 미실행이다. 새 여정은13CLI요청/완료6작업(기존프레임2+텍스트3+자막1)을 기대한다.

R118/R119 strict/전체190/정상bins/사후custody는 미완료이며 최신 제품통합gate R115/renderer기반 R117이다. 원R115blob은 읽기전용 비교 대상으로 파일을 변환·재저장하지 않는다. 역할별/페이지번호·부분실패·GUI/사용자품질은 남으며 원AI9 NeedsRepair/승인0/9·소유권/WIP/private overlay·보류 유지.
