---
id: evidence:kinekino/20261029113547
title: Sprite 기존 PCM clip 계보의 실제 byte 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Sprite 기존 PCM clip 계보의 실제 byte 검증

2026-10-26 중앙PM 단일슬롯의읽기전용배정. 새decode/추론/다운로드/GUI0. 기존full PCM20,156,486B와latinclip272,044B의guarded hash·WAV형식·선택sample구간을대조했다. 전체PCMhash는이한쌍의기존증거검증으로명시배정했으며다른PCM/미디어를재귀스캔하거나복사하지않았다. [앞선video PTS관측](sprite-stream-frame-pts-datalab-20261026.md)과분리한다.

## 관측 결과

runtime/verification/source375-datalab-b0-r1-logs/153-sprite-pcm-lineage.*와runtime/verification/sprite-pcm-lineage-datalab-r1/{INPUT,RETURN}.txt를보존한다. exit0/124ms,CPU2/BelowNormal·C tmp,source/tool별도worker나외부decoder기동0.

| 입력 | 새hash/형식·위치 |
|---|---|
|runtime/ai-spike/assets/sprite-clips-r1/full-mono-16k.wav|SHA872987BC1C6296B33241E833F5A315E8BD656B2D3ED05D14034231CAD4AE51A4,PCM data offset78,samples10078204|
|같은폴더latin-17.wav|SHAD12EC627660E5B3DE88579296D45B6F3CF368C7FE6E2419EA3ACBEB643C33270,PCM data offset44,samples136000|

두WAV는RIFF/PCM integer/mono/16000Hz/16bit/blockalign2/byte_rate32000으로검증됐다. fullPCM의start_sample272000에서136000sample,즉[272000,408000)의**모든PCM byte가clip data와일치**했다. header offset을44로가정하지않고chunk table·범위/padding을검증했다. 원fileID는full AE916536-0003000000139FD2,clip AE916536-0003000000139FD3이며guard전후bytes/creation/mtime·identity동일이다. RIFF파일해시는기존manifest와각각일치했다.

## 역사적 생성 근거와 현재 실측의 구분

현재읽은tools/model-spike/src/infrastructure/animation_audio.rs는-map0:a:0/-vn/-ac1/-ar16000/-c:a pcm_s16le의whole-file decode를호출한다. application/animation_clips.rs는hound로전체sample배열을읽어`selected = samples.get(start..end)`를새WAV로쓴다. 새리샘플링으로각clip을독립생성하는코드는아니다. 이번byte대조는이sample-slice관계를실제파일로확인했다. 현재소스가과거실행binary와동일했다는새주장은하지않는다.

원ffmpeg-version.txt는**FFmpeg6.1-essentials_build**를기록한다. 기존clip manifest의ffmpeg_sha714147…와옛절대경로는그대로유지한다. 새호스트에서그decode를다시실행한것은아니며앞선FFprobe8.1.1관측으로이를승계하지않는다.

기존sprite-asr-music-summary-v1.json의latin-17행은동일clipSHA를qwen-onnx-sprite-latin-int4-r1와yamnet-sprite-latin-r1에참조한다. 이는역사요약의모델실행provenance이며새추론/성능측정이아니다. 원문오류·human_verified_reference=false/quality_approved=false를유지한다. Sprite는actual9의Sintel텍스트cue원본으로붙이지않는다.

## 도구와 검증

std-onlyRust custody도구에fixed sprite-pcm-lineage를추가했다. knownfile길이/hash가다르면거절하고,ordinaryfile/directory guard동안만읽는다. WAV≤64MiB/chunk≤64,잘못된RIFF길이/fmt/data·중복·잘린chunk·sampleoffset overflow/범위밖·byte불일치를거절한다. 임의registry/영상PTS생성API는없다.

도구15시험/fmt/strict/build통과. 새2시험은서로다른WAV header/JUNK chunk에서도sample슬라이스가맞는지,잘못된offset·overflow·8kHz/잘린chunk를거절하는지확인했다. 기존guard해제/재획득시험도유지했다. product정상bin이나전체190의새검증을이도구시험으로대체하지않는다.

-sprite_pcm.rs SHA1ECF6A3EED1EE8C91323A70F798E8D927A4931188818CAFE754C2366E6972D75.
-도구exe SHABCC186EA3C0996DF4A79C60532F6A416F0F7C1C3A6B4E1AA10DD56C549425900.
-읽은animation_audio.rs SHA3EAE7342E73D27684F17C1A7B6A76F365F82381A5198314EBB2B7C847455E51E,animation_clips.rs SHAF6C4790CB9CBE0930A055FC2B4B141BE1A974A9B6D3543F7BE0ACAA8C85FC93C.

modoo.today test-fixture-and-mock-build의실제byte/guard/재획득원칙과code-change-review-verification의역사적claim/현재관측구분을이어적용했다. 정확본문/출처는 [case](../application/review-source-case-custody-20261026.md),[산술](../application/media-timeline-arithmetic-20261026.md)을따른다. 원자료/WIP/두lock/Media seal·index0·보류유지.

## 남은 연결

확인한것은**기존full PCM→기존clip**이다. source영상→resampledPCM의원점/priming/edit-list/연속성·새decoder재현은미검증이다. sample272000/16000을영상17초로승격하지않는다. RETURN은video_pts_mapping_verified=false/actual9_binding=false/quality_approved=false다. 다음은동일source에서decoder의timestamp/sample연속성과현재PCMdata재현을검증할별도제한gate및공통SDKseek/capture계약이다. Verified전이·AI사용자품질·GUI인수는계속미완료다.
