---
id: evidence:kinekino/20261029113649
title: 공식 Qwen encoder 모듈의 사설 import 감사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공식 Qwen encoder 모듈의 사설 import 감사

후속 [공식Torch·Rust encoder114공통stage 실제 대조](forced-encoder-stage-actual.md)를 완료했다. 이 문서의 import 단계·과거 실패 기록과 encoder 전용 forward·수치 대조 범위를 구분한다.

2026-10-18. [57archive 환경·원본 package](forced-encoder-dependency-runtime.md)의 전체 파일 SHA를 재검증한 뒤 Rust PyO3/C-API로 원본 module import를 시도했다. encoder 생성·weight load·forward는 아직0회다. 공식 module 실행 가능성, encoder 수치 일치, 정렬 품질은 별도 판정이다.

## 실행과 초기 감사 오류

기존 참조 실행기에 encoder-import 분기를 추가했다. 이 분기만 사설 Job deadline300초를 사용하며 기존 JER/mel/분절 분기는60초다. Job 할당 후 stdin admission·BelowNormal·수치 thread4·1MiB stdout/stderr 한도를 유지한다. HF offline 환경을 전달하고 host interpreter/site startup·새 Python wrapper·eval·module stub은 사용하지 않는다. PATH 변경은 실행한 자식 shell에만 적용한다.

| 실행 | 종료·시간 | 표본 peak private·working B | 관측 |
|---|---|---|---|
| r1 | exit1·98.0122691초·timeoutfalse | 790876160·616071168 | module import 후 torch.ops의 가상 __file__를 실제 파일로 해석한 감사 오류 |
| r2 | exit1·122.2900692초·timeoutfalse | 790675456·615911424 | Torch 가상 객체 처리 후 CPython frozen importlib의 가상 소스 경로에서 감사 오류 |
| r3 | exit1·90.7930556초·timeoutfalse | 790454272·615669760 | private stdlib zip 일반경로와 Windows canonical 확장경로 문자열을 비교한 감사 오류 |
| r4 | exit0·104.5994288초·timeoutfalse | 791552000·616747008 | 원본 encoder 클래스 import·출처 감사 통과 |

세 실패 stream과 build를 보존했다. package/source bytes를 고치거나 존재하지 않는 파일을 만들어 통과시키지 않았다. nagisa0.2.11 원본의 escape SyntaxWarning은 숨기거나 원본을 수정하지 않았다. 시간과 메모리는 전체 SHA 검사·import·감사·출력을 포함한20ms root-child 표본이며 모델 속도/단독 메모리가 아니다.

## 출처 감사 정책

실제 파일 모듈은 사설 runtime 또는 고정 원본 source 아래의 canonical path여야 하며 SHA를 기록한다. 표준 라이브러리 zip은 검증된 private python313.zip 경로 안의 entry만 허용한다. Torch 가상 객체는 정확한 namespace·객체 class module/name·선언 파일명이 원본과 일치할 때만 고정 torch/_ops.py 또는 torch/_classes.py 구현 파일 SHA로 귀속한다. 다른 가상 경로나 이름은 일반 허용하지 않는다.

CPython frozen module은 spec.origin=frozen뿐 아니라 _imp.is_frozen(spec.name)과 sys.modules의 실제 객체 identity도 확인한다. 해당 구현을 이미 검증한 CPython DLL로 기록한다. Python DLL SHA는69fd86ea29370697c203f7e12830084f920f490766a8e3045af52c036a9ad529다. 이러한 원본 런타임의 동적/내장 모듈을 fake stub으로 취급하거나 임의 누락 파일 전부를 허용하지 않는다.

## 증거와 후속

- [r1 관측](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-original-import-r1/observation.json)·[실패 stream](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-original-import-r1/worker.stderr)
- [r2 관측](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-original-import-r2/observation.json)·[실패 stream](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-original-import-r2/worker.stderr)
- [r1 build](../../../evidence/player/jer-function-reference-builds/e3c4f4d077e8e48b6cd3421c6b2c6ebcbb8b8b3b527ec1664b656d954af262d4/build-manifest.json)
- [r2 build](../../../evidence/player/jer-function-reference-builds/f833fabe94b2762854d9a4169915c7d4c490eebdf6a9c698a63c1de93841c9a4/build-manifest.json)
- [수정 후 r3 build](../../../evidence/player/jer-function-reference-builds/b285c269992c7014753bd2ec6ede92da93840df2d5d53d3a1384fd23e84adb16/build-manifest.json): compiled inputs58+EXE1
- [r3 관측](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-original-import-r3/observation.json)·[실패 stream](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-original-import-r3/worker.stderr)
- [수정 후 r4 build](../../../evidence/player/jer-function-reference-builds/11a95dbce39fc177e627a3dac94abb4eaa4b6681b10ba8e44f26eab23eb27849/build-manifest.json): compiled inputs58+EXE1

참조 도구4tests·fmt·clippy·locked offline build가 통과했다. root 모델 도구56tests·fmt·clippy·build도 통과했다. 다음은 strict CPU F32 encoder weight load·eval/no_grad·동일 full mel 입력과 단계별 tensor 관측이다. import 결과를 모델 품질 승인이나 제품 SDK 지원으로 승격하지 않는다.

## r4 실제 성공 범위

[실제 import 결과](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-original-import-r4/encoder-import-result.json), SHA `f5562985cc2fcd31908cc3f21c4ae603264fb2a613f114c98e111ee584207bc1`와 [관측](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-original-import-r4/observation.json)을 보존했다. NumPy2.3.2·Torch2.9.1+cpu·Transformers4.57.6·Accelerate1.12.0·librosa0.11.0·soundfile0.13.1 및 원본 modeling module을 실제 import했다. 원본 Qwen3ASRAudioEncoder 클래스가 존재했다. 모델은 생성하지 않았다.

출처가 있는 모듈3020개 중 실제 사설파일2792개, private stdlib zip222개, 검증된 CPython frozen4개, 검토한 Torch 가상 namespace2개를 기록했다. 전체 sys.modules 수나 모든 OS native DLL 감사 완료라는 뜻은 아니다. 실제파일 SHA·private sys.path·Python DLL의 실제 로드 경로를 확인했다. Torch thread4·interop1·default dtype torch.float32, CUDA build=null이었다. eval/no_grad·strict weight load·encoder forward는 이번 import 검사 범위가 아니다.

시작CPU15.831%·빈RAM32676372480B였고 숫자 기반 admission을 통과했다. 일반 preflight의 ready=false와 기존 player.exe 관측도 원본에 유지했다. 해당 사용자/다른 세션 앱을 종료하지 않았다. 5158회 메모리 표본·query failure0, stdout850752B·stderr948B로 stream 한도를 넘지 않았다. 원본 nagisa SyntaxWarning948B는 성공 시에도 보존했다. 추론0회·모델 품질 승인false를 유지한다.

## 기존 기본 JER 경로 회귀

같은 최종 EXE로 [기본 JER6개 실제 회귀](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-import-jer-regression-r1/observation.json)를 실행했다. exit0·17.3053209초·timeoutfalse, private259264512B·working81866752B였다. [전체 stdout](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/encoder-import-jer-regression-r1/worker.stdout)의 SHA `8b516b1ce73187162497b011789a0ee9e4b05b814b06485e7dc766e541b80fba`가 과거 segmentation-jer-regression-r1과 완전히 같았다. 원본 함수의 corrected JER·행렬·scored frames 결과를 유지했으며 전체 dscore CLI parity·모델 품질 승인은false다. 원본 Python DLL을 각 분기의 사설 환경에서 로드했고 host Python fallback은 쓰지 않았다.

## 보존 감사와 다음 실행

[감사 보고서](../../../evidence/verification/encoder-reference-integrity-2026-10-18-r1/build-evidence.json)의 SHA는 `af0afe754a96ac6b994f25b2e72a08c1b849b295dcb92948650c4f08139c4cf5`다. 참조 build10개·보존 컴파일입력523개·EXE10개와 기존 immutable105개를 검사했다. [링크 결과](../../../evidence/verification/encoder-reference-integrity-2026-10-18-r1/links.json)는332문서 정본 오류0·보존 참조3건이다. 기존 upstream 대상 부재2건과 원본 README의 취득범위 밖 참조1건을 status로 구분한다. checker8tests·fmt·clippy·release build도 통과했다. 함께 보존한 checker EXE SHA는484800fed4948a6454d5292a5654c863feac9f92fccd0997f83bd58fa458fd29다. checker-source 사본은 감사 당시 소스이며 compiler-input attestation으로 표현하지 않는다.

다음 Rust 쪽 관측은 [SDK 세션 준비 정본](../application/forced-encoder-observation-preparation-2026-10-18.md)의 r2-final 고정 EXE를 사용한다. 해당 세션은 비추론 검사만 수행했으며 source/입력/EXE를 선보존했다. root의 정확한 worker/인자/고정 manifest admission을 연결한 뒤 동일 mel 입력으로 실행하고 원본 Torch encoder와 단계별 tensor를 비교한다. 준비 도구를 제품 SDK capability나 이미 실행한 encoder로 합산하지 않는다.
