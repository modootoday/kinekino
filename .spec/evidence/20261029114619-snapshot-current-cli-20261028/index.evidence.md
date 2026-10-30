---
id: evidence:kinekino/20261029114619
title: R94 현재 CLI PNG snapshot 경로 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R94 현재 CLI PNG snapshot 경로 인수

2026-10-28 중앙 PM. 전체CLI근거대조에서production.export_snapshot의현재입력귀속공백을확인해기존Rust시험을실행했다. 제품/시험소스변경0이며원R93 SOURCE07D836401A481E065B8C8235EAA7B02C928572E7B05FA97FD2537DFF6EF07A78/2336을그대로사용한다. 원2026-10-20 성공을새호스트성공으로승계하지않았다.

원로그 runtime/verification/source375-datalab-b0-r1-logs:1874 native-feature snapshot_export_api_cli no-run exit0,1875 실제시험18pass/0fail/0ignored·0.56sec. 18개중11은증거/프로세스helper시험,7은snapshot API/SDK/CLI경로다. 기능18개나실제CLI18회로계상하지않는다.

1876 verify-current exit0·원R93입력2336파일 content/identity동일을확인했다.

fixed-r94/snapshot_export_api_cli-cd607afdd9bb6aa3.exe SHA E15993F6EFEEC9FEB4A5BE7A680068E59982D014A289DE7DFDC791C2F7FABE21/PDB보존. 정상CLI는현재5ECE053CD866500B21F3808A76503A43C421724F09B961D44B7565ED51621BAE와대조했다. 원JSON요청/stdout/stderr·source/output전후사본/metadata·원processreceipt는runtime/verification/source-sdk-r94-snapshot-cli-r1/run-15540-1793141992042455800아래에보존됐다.

7개경로는미구성SDK의capability/schema/Unsupported일치·typedclient builder에서port유지·명시port schema/context정합·정상fileCLI정확픽셀crop/덮어쓰기거절·schema/핀불일치/범위overflow/unknownfield의미발행·CRC손상/미지원timeout거절·endpoint미주입Unsupported/주입원counter및동일공통API소비다. fixture는Rust생성3×2그레이PNG이고crop은2×2다. 실제사용자미디어/GUI시각인수·독립golden/색관리동등성시험은아니다. 원파일/출력픽셀/no-overwrite·metadata미복사계약을검증하며새PTS/모델품질을만들지않는다.

helper11시험은합성transport/receipt/fault모델을포함하므로각주장범위가다르다. 현시험의실제CLIprocessreceipt를모든host의cleanup/fault전수로확대하지않는다. source변경이없어fmt/strict는원R93실제실행에유지했고이번같은입력검증은별도custody를따른다. 새190/정상GUI빌드는실행하지않았으며원R92근거를유지한다.

기존WIP/소유권/원실패/privateoverlay·WARP/추론/다운로드/수동GUI/push보류·품질0/9·goalactive유지. C-only/heavy1/CPU2/BelowNormal/offline/locked/-j1준수. modootoday/software-qa/test-fixture-and-mock-build A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100 및code-change-review-verification 9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B 적용:현재실제실행/과거성공·helper/기능·수치픽셀/시각품질을구분했다.
