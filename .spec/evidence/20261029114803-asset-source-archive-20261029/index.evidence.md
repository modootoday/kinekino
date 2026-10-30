---
id: evidence:kinekino/20261029114803
title: 원본 자산 source archive 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 자산 source archive 검증

R252 공간 차단 해소 후 검증 재개: 사용자 명시 Python 유지보수 요청의 범위를 [유지보수 정본](../environment/session-maintenance-20261029.md)에 기록했다.3185snapshot·3186native strict·3187default strict·3188lib no-run·3189실제 원본 archive/readmission/rerender/import 값 계획1시험 종료0.3190 project 회귀는13 passed/0 failed/2 ignored/원 runner 종료0이지만 후속 cleanup 오류는 별도 실패로 보존했다.3191fmt/부분 cleanup 복구·3192사후 custody/cleanup 종료0. 실제 source2513개 content/identity unchanged, inventory SHA `A0B7388DAA709173141D4CF1DD842D63DA0A773D2655D1CDF164157FA0BC5A63`, SOURCE.md SHA `C37664547585942C38FB592B4A520864F5921309D71430F347F0822B2E392845`.

R252 실제 fresh evidence는 runtime/verification/source-sdk-r252-import-plan-r1이다. 원 R243 archive에서 준비한 import draft가 새 document_id/revision1과 원 source/layer pin을 보존했고 quality/review=false였다. 명시 unreviewed mask 확인 누락, 원 project ID 재사용, 위조 manifest quality flag는 거절됐다. 이 계획은 값이며 Native project 저장/commit receipt·Jobs archive source 선택/CLI import 인수는 아직 후속이다. 전체190/정상 bins의 최신 통합은 R250이며 R252 성공으로 재계상하지 않는다. 중간 component exe/PDB 복사본은 추가하지 않았다.

R250 최종:3179 사후 custody 종료0, 실제2512개 source content/identity unchanged. PDB 보존 중 CLI PDB 복사가 공간 부족으로 두 차례 실패했으며 해당 단계는 당시 성공이 아니었다. native 제품 deps/incremental의 NTFS 압축을 진행했고 두 압축 명령이 겹쳐 시작된 운영 실수를 확인·보고했다. 추가 작업 없이 원 핸들 종료를 확인한 뒤 여유 공간이 CLI PDB 길이+256MiB를 넘는지 검사하고 복사를 완료했다. 원본/고정 CLI PDB SHA `F51BADB9236D8905CF9D529AB383E3177CFFAEE9F502758A5DC030F7F6E25000`, GUI PDB SHA `AC766008F27E1D00394576AF239DB1E9D12794A3FCE04ECF80E25F86533BB5B7`가 각각 일치했다. 고정 exe SHA도 위 값과 다시 같았다. 원본·실패 증거·캐시를 삭제하지 않았다. 압축 최종 deps4961files/34,476,678,190 logical/13,720,835,287 stored, incremental8779files/26,110,544,496 logical/15,600,405,377 stored, 각 종료0. 일부 초기 압축은 공간 부족 오류로 종료1이었다. 마지막 C 여유265,363,456bytes로 다음 heavy build 전 공간 확보가 필요하다. 캐시 전체 content/FileID 전수 검증으로 과장하지 않는다.

R250 실제 통합 결과:3171combined lib/Jobs/unitCLI no-run·3172실제 CLI/typed source archive+원Jobs 시나리오·3173Product145·3174Core18·3175Native26·3176OS1·3177정상2bins·3178fmt check 종료0. 지정190 모두 execution_result/producer_integrity=Ok이고 동일 R250 SOURCE.md SHA다. Core/Native/OS는 원622/624 compiler 산출물의 실제 재실행이며 새 컴파일이 아니다.3179사후 custody 진행 중이다.

실제 root runtime/verification/source-sdk-r250-cli-r1/asset-build. CLI source schema/export/inspect와 typed inspect 동등성·typed 별도 export·기존 ID RevisionConflict·경로 주입/foreign SHA 거절, all-hidden publication3 source export가 통과했다. 원 CLI/typed Jobs2개는 publication1을 유지해 완료했고 산출물 typed/CLI inspector도 실행했다. 원 host EOF/exit0/not forced/family Empty·pipe/sdk joined/completed_total2를 시험 assert로 확인했다. CLI job `job-c4664c0e3d0540b784530c82eee96486`, typed job `job-1764db249b23420dbc590678398e017c`, render marker `8c656069d1e20b7715e4a0cd1d9595782c299887a312c54ec41112da7f2c62eb`, config SHA `210b00ba873de78385a6fa033566b636e8eede0a0b0a06143f68e9e0b26adf9c`. source-archive-summary.json의 원본 묶음 marker는 `c8334a9b5bbd200ea7af63d416a460fc29b90418ac4bdcdc7a69cb3dbc9882eb`,1531bytes이며 publication1 SHA `a42a57f2fbd3ca6b1651c10ec226aa30243ed0f7b1e45005222ce60ccc4a6656`을 보존했다. 독립 source 저장이며 원Jobs CAS 완료로 계상하지 않는다.

고정 exe/PDB는 fixed-r250/native-bins: CLI SHA `48C8CE135C9C28E8F45EE62A68BF5040F7E1E3526DCA69FDE7650DEB479F5773`, GUI `39E83B0DA6A9043EABB7807BEBBFE1FA5B9464C47CCA24E71B5A1E8F9B24EA75`. native-core lib `ED424BFF4D7F51F024BDB674EBC0B68CD3458DCCE434BD3302D55C053AF76969`, Jobs `6F95F229D0FDE329220E4260B5E9473939E1C2C7413A06A2E2CC5B2919D431F8`, unitCLI `F5CEC7E8A6114E2FA67F358E1F00EC3A92EAAF7AD13EC2C38CCD8BCC28789545`를 보존했다. GUI 수동실행·AI/색 품질·원본의 새 프로젝트 재가져오기·원Jobs archive 통합·비정상 crash/모든 fault/batch·전체 제품 인수는 남는다. 명령118개를 완성률로 쓰지 않으며 source 명령 experimental·제작0/5·AI품질0/9·private overlay/보류를 유지한다.

R250 endpoint 인수 진행: source export/inspect의 strict 요청·nested 응답 schema를 기존 project/input 계약에서 재사용해 연결했다. source 서비스가 원 host config/ancestor guard도 직접 보유한다. 실제 CLI 시나리오에 export/inspect·typed 동등성·별도 typed export·기존 ID overwrite 거절·경로 주입/foreign SHA 거절·revision3의 all-hidden source export를 추가했다. 기존 원 Jobs2개는 계속 publication1을 선택한다.3166/3167fmt·3168snapshot·3169native strict·3170default strict 종료0.3171combined no-run 진행 중이며 실제 실행 성공으로 계상하지 않는다.

R250 inventory2512 SHA `9940A566B985769BFCDE4B3F20BC1B919702E79BADA5C401F73ADA23EBCCC7DC`, SOURCE.md SHA `FFFD1385156AC32625B439DC6EFABDC54158492A85E8331CB21BD4412D2D3BB8`. 변경 전 API/host/service/실제시험은 preimage-r249-source-cli에 보존했다. 현재 archive 명령은 experimental이며 원 Jobs CAS 완료·review/품질 승인·전체 기능 인수와 구분한다. 원본·소유권/WIP/보류·private overlay를 유지한다.

3165 최종 사후 custody 종료0: 실제2511개 source content/identity unchanged. R249 저장 capture 분리 검증 범위를 확정했다.

R249 source 저장 capture 분리:3156/3157fmt·3158snapshot·3159native strict·3160default strict·3161lib no-run·3162실제 숨김 source capture/기존 렌더/발행·취소1·3163project 회귀13 passed/0 failed/2 ignored·3164fmt check 종료0.3165사후 custody 진행 중. 새 실제 root runtime/verification/source-sdk-r249-source-storage-r1. 원 publication1을 보유한 상태에서 새 revision2의 모든 layer를 hidden으로 저장했다. 그 publication의 render capture는 NeedsInput으로 거절하고 별도 source capture는 PNG2개/project/manifest4파일을 준비했다. 기존 publication1 렌더·source 발행/충돌/취소 전후 재조회와 최종 원본 접근 복구도 같은 시험에서 통과했다. 새 타입은 렌더 builder로 전달할 수 없으며 기존 렌더 carrier는 동일 원 lease를 Arc로 공유한다.

고정 lib fixed-r249/native-core SHA `EF9DC6B1EC2DDACA8B3C7E9C656D7E6E159C5E1C473513225DCC60680C6C10A4`. inventory2511 SHA `D7891484820A44E745AEFEB20AFBD03486A8578E61BC7FF4C4DD5959B667FB5B`, SOURCE.md SHA `17D6B24210697EC896CDA6CD072E3411FD01EA678D57079B5969BF5E413AD84A`. preimage-r248-source-capture에 변경 전 carrier/service를 보존했다. endpoint schema·실제 CLI/typed source export·원Jobs 통합·전체190/정상 bins는 아직 후속이며 최신 통합 R236을 유지한다.

R248 3154default strict·3155사후 custody 종료0: 실제2511개 source content/identity unchanged. 현재는 SDK/CLI dispatch의 컴파일 경계만 확인한 단계다. schema/실제 endpoint 시험 전에 지원 인수를 확정하지 않는다.

R248 SDK/CLI 연결 구현 중: application ExportAssetSources/InspectAssetSources/AssetSourcesObservation/AssetSourceArchivePort를 추가하고 Native service가 원 project capture→원 PNG carrier→create-only 저장/strict inspect를 호출한다. typed SDK·고정 asset host·CLI dispatch를 연결했다. path 입력은 없고 timeout/idempotency/expected_revision은 거절한다. 상태는 experimental이며 schema/실제 CLI·typed 시험·reimport/원Jobs archive 통합은 아직 후속이다. 모든 hidden layer로 render plan이 없는 프로젝트의 source-only export는 현재 capture 제약으로 미지원이며 별도 분리가 필요하다. 품질/review/Jobs 완료 권한을 만들지 않는다.

3150/3151fmt·3152snapshot·3153native strict 종료0,3154default strict 실행 중이다. inventory2511 SHA `FF340AAC5FD790925DC293AF5FC1FB9D4A4465700A7C08FFB165634D4164D929`, SOURCE.md SHA `B1B4F0105D620BB67FCBD611C1007EC58FBD60632780D00C59E1B0FD54460230`. preimage-r247-source-sdk에 변경 전 SDK/host를 보존했다. 새 실제 시험·사후 custody·현재소스190/정상 bins를 아직 성공으로 계상하지 않는다.

3149 최종 사후 custody 종료0: 실제2508개 source content/identity unchanged. R247 독립 발행 검증 범위를 확정했다.

R247 native create-only 발행:3136/3137fmt·3138R245 snapshot·3139fmt/3140R246 snapshot·3141fmt/3142R247 snapshot·3143native strict·3144default strict·3145lib no-run·3146실제 source 발행1·3147읽기/재렌더1·3148fmt check 종료0. R246~247은 writer→reader FileID 대조 보강이며 실패를 숨기거나 과거 입력을 새 성공으로 합산하지 않는다.3149사후 custody 실행 중이다.

실제 root runtime/verification/source-sdk-r247-source-publication-r1. `published-sources/sources-complete`에4파일을 원 no-replace writer로 발행·원 핸들 해시/길이/실제 FileID와 목록을 확인하고 strict reader로 다시 열었다. marker SHA `9012a3af08dc992a7bf39a319af958ba3d1661aaa8361c5af16822fbacab0b74`,1533bytes. 같은 ID 충돌은 RevisionConflict, 보유 결과의 PNG 쓰기는 거절됐다. checkpoint가 실제 project 파일 생성 뒤 취소를 반환한 경우 sources-cancelled/project.json은 남고 marker는 없었다. marker 생성 직후 취소는 원 Cancelled 오류와 marker_committed=true/partial_files_removed=false를 보존했고 명시 SHA 재조회가 성공했다. source-publication.json에 원 오류와 pin을 기록했다. recovery의 created/committed는 이번 호출의 동작이며 충돌한 기존 디렉터리 상태를 추정하지 않는다. 자동 재시도·삭제·Jobs CAS는 하지 않았다.3137 기존 원본/출력은 보존됐다.

읽기 회귀 evidence는 runtime/verification/source-sdk-r247-source-readback-r1이다. 고정 lib fixed-r247/native-core SHA `ADF4A7A84B94CFA1037BFB08F4DF9D44D5D5EBE7C46881EA32DFAF7CB152D19A`. inventory2508 SHA `99A3A2CA68F3FBAC85B842FED73C16BCB1CC52828CF0C15DCAE3C7CF053FF786`, SOURCE.md SHA `3A2423F6A55C05E3D40554CB82F136D544CA4C5CF1939115CD5B86E5F346803F`. 비정상 crash/전원 유실/directory durability·모든 fault·Jobs/SDKCLI archive 기능은 미완료다. 최신 통합190/정상 bins R236·제작0/5·AI품질0/9를 유지한다.

3135 최종 사후 custody 종료0: 실제2507개 source content/identity unchanged. R244 읽기/admission/재렌더 시험 경계를 확정했다.

R244 읽기/재렌더:3125/3126/3127fmt·3128snapshot·3129native strict·3130default strict·3131lib no-run·3132읽기/admission/재렌더1·3133source capture 실제1·3134fmt check 모두 종료0.3135사후 custody 진행 중. exact marker `9012a3af08dc992a7bf39a319af958ba3d1661aaa8361c5af16822fbacab0b74`인 R243 원본 묶음의 fresh copy를 `runtime/verification/source-sdk-r244-source-readback-r1/sources-existing`에서 열었다. 원 image/mask admission 뒤 재렌더 raw SHA가 고정 R243 실제 출력 `573096b33ff4f61a98ca0444399dc3eb3a79f7aca05eafd308a8a24ffbabda9a`와 같았다. reader/catalog 해제 뒤 prepared 입력의 원 파일 쓰기 거절, 최종 drop 후 접근 복구와 wrong pin·foreign binding·탈출 이름·review=true·extra entry·실제 변조 거절을 확인했다. 원 R243 파일은 변경하지 않았다.3133은 새 validator가 포함된 capture로 `source-sdk-r244-source-capture-r1`에 실제 별도 실행했다.

Native reader는 일반 root/directory/file 핸들과 FileID witness·bounded byte hash·strict source/project 의미 대응을 보유한다. 프레임 catalog 생성은 입력 admission 자체가 아니며 실제 open에서 PNG를 다시 검사한다. no-overwrite 발행/원 Jobs/SDKCLI archive endpoint는 미완료다. 외부 catalog 경로를 요청 JSON에서 받지 않는다.

R244 inventory2507 SHA `38A83B91708F39771DE9395AE13A928E337146C8AE2A7686FE5248949A30D43C`, SOURCE.md SHA `7760D1F4EB784A492914CE2BFC7F07BE5B8CC613066C378B56493E205888FFCC`. 고정 lib fixed-r244/native-core SHA `D0B8ADF871613D82E44D4D73853C12B91BCB4354786A46A45FE05BA1504A9860`. preimage-r243-source-readback에 변경 전 carrier를 보존했다. 전체190/정상 bins는 이번 범위가 아니며 최신 통합 R236을 유지한다.

3124 최종 사후 custody 종료0: 실제2504개 source content/identity unchanged. R243 carrier 검증 범위를 확정했다.

[공통 carrier 계약](../../player/application/production/asset-source-archive.md)의 R243 private overlay다. 원 Native image/mask lease가 해시 검사한 PNG bytes를 보유하고 공통 source archive가 원 project/입력 lease를 계속 소유한다. 재인코딩하지 않으며 exact original bytes가 없는 adapter는 Unsupported다. 현재 no-overwrite 발행/reader/reimport/Jobs/SDKCLI archive 기능은 미완료다.

preimage-r241-source-png에 변경 전 입력/application을 보존했다.3113/3114fmt·3115R242 snapshot·3116fmt·3117R243 snapshot·3118native strict·3119default strict·3120lib no-run·3121실제PNG/source archive1·3122project 회귀13 passed/0 failed/2 ignored·3123fmt check 모두 종료0. R242는 carrier 구현, R243은 실제 기존 시험 확장 입력이다.3124사후 custody는 실행 중이다.

실제 root runtime/verification/source-sdk-r243-source-archive-r1. original-sources 아래 project.json·sources.json·원 PNG2개를 시험 코드가 새 경로에 썼고 각 원본 SHA 일치를 확인했다. source PNG `0a787bb7b0d47b8ac2f1b46932a7c6f048084334d0b6719150d8e0b2513aa220`, mask `77793436acdb3ef415642ee84e0b1ff82e298c22fa1e4fcaee41c32aa368502c`. project publication SHA `d0f9c1b3c68347324698af7360ed2d1d821693452a9ddef914081a987a9e655a`. cancel checkpoint 거절과 렌더/carrier 해제 후 source archive 보유 중 원 image/mask 쓰기 거절, 최종 archive drop 후 접근 복구를 확인했다. 실제 원 Jobs 발행이 아니라 application carrier/시험 파일 출력이다. 정적321×137/수동 mask이며 AI/영상 품질 인수가 아니다.

고정 lib fixed-r243/native-core SHA `9569F1EAB791C2A9020CC7B240CDAC927A01B137A96786643996C5C75FF8239F`. inventory2504 SHA `7A67829FAB57A182D05868205FC92C5AEBA22A76D96ABDD246B7BD733993B7A0`, SOURCE.md SHA `9E0CA1FA4FDE03EABFEE1DEACF957A15626287C7F5D28E0DD08665F02C4FBFE7`. 최신 통합190/정상 bins는 R236이고 새 input의 성공으로 재계상하지 않는다. 전체 제작0/5·AI사용자품질0/9·기존 소유권/WIP·원 증거와 보류를 유지한다.
