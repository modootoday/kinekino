---
id: evidence:kinekino/20261029114149
title: MKV 실제 producer 실패 보존·source demuxer 수리
status: active
created: 2026-10-29
updated: 2026-10-29
---
# MKV 실제 producer 실패 보존·source demuxer 수리

최종1650 default strict exit0/19778ms,1651 fmt exit0/2816ms,1652 custody exit0/41889ms·2321입력 content/identity 동일 확인. 아래 진행 중 표기는이실제완료로보완한다. 실제timeline성공으로바꾸는것은아니다. 원 R74/R75 actual 실패는그대로유지한다.

2026-10-28 중앙 PM. R73 입력 분류 후 준비된 실제Sintel로 fresh timeline producer/SDK association 시험을 추가했다. 첫 실패와 후속 실패를 모두 보존했으며 **timeline/Verified association/재생 성공은 아직 없다.** 원9 range/품질·원source375/private 경계·소유권/WIP/보류는 유지한다.

## 입력과 고정 도구

- 원R74 SOURCE8FCF712C62E8C42B94FFAA1C034B7AB7E30C3E0680E922B50E35D708F81553F2, inventory27FE8F442615D586DF4A2A349C961E4F2565368E0163B28D3A082976C72A3143/2321파일.
- 수리후R75 SOURCE78FBCD5FCD3886CEAE4913005BB96EEBF097DBE2382ACC47E23D14EB7EC5D5C0, inventory644A246D8BE51F7198A7A6423A60BF40CDDCE8B1435F53139740F3D93A46922C/2321파일. mirror runtime/verification/input-r74와input-r75에별도snapshot보존.
- 실제 prepared case B1E2BE1056E044CD3A8F9E41D16D768B6292DE81C2467890D79EEE844268E390/gemma270-highlights, 미디어F12C070E295B38CFC94EBD61AC3357C3BAC82D1015985F1E8DA93A4C6496C46D/681285280bytes.
- probe8.1.1 A6618E99BB58869DED3C6F37B53AA1A8D701C3591DBB7B5B317D47369C112BE2, audio6.1 71414777922D05F2639A5CF4A0F9A405F169809CF459000A6F976C9ADD61F060, capture8.1.1 09948D4CDD0650DA6FF5A87577469F2A218DC2615AE379F8F734D24C49DE0F73를로컬파일에서재확인했다. PATH탐색/설치/다운로드0.

실제 시험은 source≤1GiB/output≤64MiB/publication≤2GiB, 원producer deadline120초/cleanup2초, WindowsJob memory1GiB/affinity1CPU를명시했다. video query107~134초, PCM PTS1716000~2140800(1/16000), capture107.25~133.8초는 명시한검토선택이며 원 모델의검증된시간으로승격하지 않는다. incomplete cleanup은 원pending을보유하는기존assembly경로와비복귀진단callback을사용한다. timeout재설정/자동retry가아니다.

## 첫 R74 실패

원 로그 root `runtime/verification/source375-datalab-b0-r1-logs/1642-source-sdk-r74-actual-timeline.*`: test exit101/1304ms/0pass. 실제 probe INPUT의 source classification은 matroska_header지만 argv는 -f mov/enable_drefs0였다. 원 stderr의 moov atom not found/Invalid data를 보존한다. publication/SDK association에 도달하지 않았다. 이 첫실행의별도 PROCESS.json은없으므로원probe exit/family의외부JSON영수증을추정하지않는다. parenttest종료를그증거로대체하지않는다.

원 시험·runtime raw는 `runtime/verification/actual9-matroska-timeline-r74-r1`; 원exe/PDB는 fixed-r74와 source-sdk-r74-fixed-artifacts.json이다. 삭제/덮어쓰기/같은root 재시도0이다.

## R75 수리와 후속 관측

원6파일을 mirror `runtime/verification/preimage-r74-source-demuxer`에보존했다. 공통live_contract가원retained source class에서mov또는matroska를선택하며video/audio/capture와publication reader의expected argv가같은계약을소비한다. ISO의mov/enable_drefs0·file protocol제한·기존raw PCM replay s16le는유지했다. reader의ftyp전용검사도bounded raw 분류와원token의class/MIME대조로바꿨다. archive v1의 MOV 전용 original_execution은원계약에보존했다.

|로그|범위|실제결과|
|---|---|---|
|1644|수리후 Native all-targets strict|exit0/23877ms|
|1646|Native lib/새actual integration no-run|exit0/175383ms|
|1647|R75 snapshot|원terminal receipt보존|
|1648|source class→video/audio/capture/reader argv 재구성|1pass·MOV외부참조제한/Matroska선택/file protocol확인|
|1649|후계 실제producer|exit101/3372ms·0pass·다음 PTS검증거절|

1649의probe는이번에 child_exit0/family_empty=true/forced=false로완료했다. 원 PROCESS.json은 actual9-matroska-timeline-r75-r1/runtime/video-*/PROCESS.json에보존한다. 이어 strict video parser가 Video PTS ambiguity or discontinuity로거절했다. 원time_base=1/1000, 첫PTS98958/99000/99042/99083/99125/99167, duration은모두41이다. 41틱duration과41/42틱PTS간격이일치하지않아현재정확연속성규칙이거절한다. requested107초보다앞선keyframe preroll 관측도그대로보존한다.

이 단계는demuxer수리가원probe실행까지이어졌음을입증한다. **새 timeline bundle/publication/SDK Verified association은여전히미완료**다. duration을다음PTS로덮거나누락프레임을만들거나연속성검사를제거하지않았다. 후속에서는원컨테이너시간해상도와프레임cadence의검증된입력/별도계약을정의하고실제관측과누락·중복·gap음성시험을보존해야한다. 기존정확연속형MP4/archived근거를느슨한새규칙으로소급승인하지않는다.

수리후source와실행파일은fixed-r75/source-sdk-r75-fixed-artifacts.json에보존한다. actual9-matroska-timeline-r75-r1은새root이며원R74실패를대체하지않는다. 작성시 default strict/fmt/최종custody는진행중이다. 지정190/정상bins는원R67, CLI100/103/24는원R72,분류/입력보유는원R73의범위다.

새추론/다운로드/수동GUI/엔진재생/WARP/push0,사용자품질0/9. 기존modootoday/software-qa/test-fixture-and-mock-build(A393F15FAF68FEE96C187509D0F22ED72987D8D1929AFAF782D43BCA80F3F100),code-change-review-verification(9F73865F96EC0CEFCF3A4023B294400258473640DF3B1D2ADD03DCCA8192D64B)을적용해원실패·정상프로세스종료·관측검증실패·제품인수를분리했다. C-only heavy1/CPU2/BelowNormal/locked/offline/-j1 유지, snapshot만읽기전용으로compile과겹쳤다.
