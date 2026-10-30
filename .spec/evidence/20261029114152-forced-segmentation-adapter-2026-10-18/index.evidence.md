---
id: evidence:kinekino/20261029114152
title: 고정 외부 분절 단일 요인 A/B 어댑터 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 고정 외부 분절 단일 요인 A/B 어댑터 준비

2026-10-18. 원본 [분절 관측](../ai/forced-segmentation-actual.md)을 고정 입력으로 받는 별도 Rust 진단 경로를 구현했다. **추론·가중치 로드는0회이며 실제 default↔control 결과 동일성과 tokenizer A/B의 시간 품질은 아직 root 검증 항목이다.** 분절 개수/목록·prompt·token 대조를 시간 정답이나 품질 승인으로 승격하지 않는다.

## 범위와 재현 가능한 분리

- 새 adapter: `tools/forced-aligner-segmentation-spike`0.2.0. 기존 observation adapter를 Rust bootstrap으로 복사하고 독립 package 이름으로 분리했다. domain은 외부 단위/원문 계약, application은 기존 정렬 유스케이스, infrastructure는 파일·tokenizer·engine 연결, presentation은 CLI를 소유한다.
- 새 engine: `runtime/ai-spike/source/lumos-segmentation-7984967`. 원본 observation fork에서 `.git`/target을 제외하고 복사했으며, 심볼릭 링크와 기존 목적지 overwrite를 거부한다. `tools/segmentation-bootstrap.rs`에 복사·이름 치환을 재현할 Rust 코드가 있다. bootstrap은 기존 경로에서 재실행하여 덮어쓰는 도구가 아니다.
- 원본 `lumos-qwen3`, `lumos-aligner-observation-7984967`, 기존 tools/forced-aligner-* 소스·EXE·결과는 변경하지 않았다. 새 fork에서 원본과 다른 engine 파일은 package Cargo.toml, processor.rs, observation.rs, model.rs의4개뿐이다. 전체 트리 양쪽 해시와 `git diff --no-index --binary` 패치를 보존했다. parent Git index/설정은 변경하지 않았다.
- processor는 optional fixed words를 보관하고 원문 일치를 재확인한 뒤 기존 timestamp prompt 조립에 같은 단위 목록을 전달한다. 기본값은 그대로다. model.rs 차이는 기존 관측 분기에서 실제 model input IDs를 복사하는1줄이다. 관측 구조에 해당 벡터를 추가했다.
- 회귀가 줄바꿈 정규화 후 model 수치 body의 동일성과 fix_timestamp 이하 코드 동일성을 검사한다. 나머지 engine src 파일은 실제 bytes가 원본과 같은지 모두 대조한다. tensor/mel/forward/argmax/head 선택/class→ms/repair/round3/zero clamp 정책은 바꾸지 않았다. 실제 timestamp 위치의 전체 classifier row를 계속 보존하며 비timestamp 위치 전체 tensor를 저장한다는 뜻은 아니다.

## 입력과 관측 계약

외부 JSON은 deny_unknown_fields인 schema1이다:

```json
{
  "schema_version": 1,
  "transcript_sha256": "exact UTF8 transcript SHA256, 64 lowercase hex",
  "language": "Japanese",
  "source": {
    "name": "provenance label",
    "evidence_sha256": "64 lowercase hex",
    "case_id": "ja-1634"
  },
  "words": ["already cleaned fixed units"]
}
```

원문 punctuation/newline까지 포함한 SHA와 **전체** cleaned concatenation을 모두 검사한다. Unicode category L/N 및 ASCII apostrophe만 남기는 clean이며 NFKC·trim·재분절·숫자 합치기를 외부 단위에 추가하지 않는다. 빈 단위, 공백/제어/마커 주입, 언어/원문/hash 불일치,512단위 초과,단위256bytes 초과,파일256KiB 초과를 거부한다. provenance label도 비어 있거나256bytes/제어 문자가 있으면 거부한다. `--words-file`과 Japanese merge-term override는 동시 사용할 수 없다.

일반 words file의 evidence_sha256은 형식만 검사하는 **출처 선언**이다. 임의 외부 원본 자체의 진위를 자동 검증했다고 주장하지 않는다. 별도 export/verify helper는 root의 실제 분절 결과 bytes SHA `b4b09dde514f23903698ec9341b1aefcc6c21e1c779095e95ab447356fbd6089`를 확인하고 원본 record의 text SHA·목록·prompt와 대조한다. root cohort의 declared source SHA/case ID도 확인했다.

metadata에는 원본 JSON UTF8·파일 bytes/SHA·전체 source/words, 기본/선택 단위·원시 prompt·unexpanded BPE IDs와 equality를 담는다. 실제 align 이후에는 별도로 audio-pad 확장·padding 이후 **실제** model input IDs, timestamp positions, selected class IDs, raw f32 ms/bits, fixed ms, final f64/bits, 전체 timestamp logits/row hash를 보존한다. 원시 prompt IDs와 모델 입력 IDs는 서로 다른 관측이다. 실제 결과에서 timestamp positions가 model IDs의 timestamp token151705를 가리키는지도 확인한다.

report는 실제 adapter EXE SHA와 compiled input SHA를 기록한다. inherited `processor_configuration`/`segmentation_control.configuration`은 기본 processor 준비의 내역이며 외부 words 선택 여부는 `segmentation_control.external`, selected words/prompt를 읽어야 한다. limitations도 기본 Lindera/IPADIC 경로와 명시적 외부 override를 구분한다. inherited diagnostic_patch_sha256은 기존 observation patch의 출처이며, 추가 변경의 정체성은 새 compiled manifest/4개 difference patch가 제공한다.

## 고정 identity와 빌드 보존

[최종 snapshot](../../../evidence/player/forced-aligner-segmentation-builds/r2-final/build-manifest.json)은 compiler input128개를 현재 파일과 비교하고 양쪽 원본/new fork 소스·lock을 별도 경로로 보존했다. 저장 EXE의 `--build-info` compiled SHA와 preserve helper의 embedded SHA도 일치했다. tree hash manifest와 원본/차이 패치가 같은 디렉터리에 있다. 모델 weights는 snapshot에 복사하지 않았다.

| 항목 | 값 |
|---|---|
| upstream engine commit | `7984967e8701cc90bb17d06453901036e36ca578` |
| Qwen3-ForcedAligner-0.6B revision | `c7cbfc2048c462b0d63a45797104fc9db3ad62b7` |
| 고정 safetensors | 1835544544bytes / `47831d0e82f96b20e9034dba01a075ee06436654719f6a68289e49f1b65ce0e7` |
| 실행 precision/device | 기존 LoadOptions CPU/F32, flash-attn false; root A/B threads4 |
| 모델 라이선스 표기 | 고정 local README front matter의 apache-2.0. 배포/보조 사전 고지 승인과 별개 |
| 최종 EXE | `b069f93674fdd38ce74e7b343681b16aa501c1775f3dbeb289501a78ab3e6bb2` |
| build-manifest SHA | `7aef099d50cb5574ff6ab71b611501c6a1a26866d88b12b7c85ec6d0a0bcbb3b` |
| embedded compiled-source SHA | `a890c4f732158ed97be32b9d2192f00aed73d32f5ddd49188796edb7cf2b9637` |
| adapter Cargo.lock SHA | `d4829ecb43e8e1d6b80f3c98a3af6ad884ccfa4a094a925d5d37a61b8fd242f3` |
| saved compare-runs.exe | `260d177884deaed24c5bb1959b6497695e2936e3fdae0eed979a67f0bac4178c` |
| saved verify-segmentation.exe | `a0c473a0419c277415f10efa04bb0c19b82d03dc44ca93b0c18934e172241777` |

앞선 r1-6cf0e50a snapshot도 보존했으며 최종 실행에는 사용하지 않는다. r2는 limitations의 기본/외부 분절 구분을 수정한 이후 빌드다. Rust1.99.0/cargo1.99.0, 기존 `lumos-observation` release cache, -j2, process-local 기존 LINDERA_DICTIONARIES_PATH를 사용했다. 초기 debug profile의 불필요한 재컴파일은 자체 cargo/child만 종료하고 release cache로 전환했다. 사용자 앱은 종료하지 않았다.

## 비추론 검증 결과

root가 제공한 `segmentation-cohort-r1`의 실제6개 파일을 수정하지 않고 사용했다. cohort result SHA는 `0b83adb94cac29abca3c6b1fbbca71b919c57c16664bc42811572626e3052e18`이며 원래3개 transcript를 그대로 읽었다.

| 원문 | default / baseline-control | original-tokenizer | 실제 prompt/token 비교 |
|---|---:|---:|---|
| KO1883 | 19 / 19 | 19 | 세 경로 단위·prompt·BPE IDs 모두 동일 |
| JA1516 | 28 / 28 | 28 | control은 동일, original 목록·prompt·IDs는 다름 |
| JA1634 | 31 / 31 | 32 | control은 동일, original 목록·prompt·IDs는 다름 |

공통 prepare 및 실제 CLI `inspect-processor`6회가 이 결과를 확인했다. local vocab151643개/added63개/timestamp151705 identity 검사도 수행했다. [최종 r2 비추론 summary](../../../../runtime/ai-spike/acquisitions/sdk-segmentation-inspect-r2/summary.json) SHA는 `bdc7fa6eb0028bc62339a232122f291a24cd9072c91ec9dc30df8a28e6e8da13`이다. 별도 r1 summary도 같은 bytes hash이며 overwrite하지 않았다. 어떤 경로도 가중치·forward를 호출하지 않았다.

Rust21tests(include-ignored의 기존 local tokenizer-only1 포함), tool fmt, 수정 fork3파일 rustfmt, bootstrap clippy-driver -Dwarnings, adapter clippy --all-targets -Dwarnings, dependency -p qwen3_asr_segmentation --lib -Dwarnings, locked/offline release --bins build가 통과했다. 새 검사에는 원문/hash/전체 clean/주입/상한 거부, 동일 단위 prompt와 repair 동등성, 수치 소스 불변, comparator의 동일 입력 허용·raw 차이와 다른 audio 거부를 포함했다. 메모리/synthetic 비교를 실제 neural 동일성으로 표시하지 않았다.

초기 새 CLI가 모델 경로 없는 기존 inspect-processor 사용을 거부한 회귀1개를 발견해 복구했고 최종21개가 통과했다. 기존 MSVC LNK4098 LIBCMT warning은 이전 [동일 engine 계열 기록](../../../../runtime/coordination/forced-aligner-raw-observation.md)에도 있던 native 링크 경고로 남는다. 이를 억제하거나 Rust clippy 경고와 혼동하지 않는다. 최초 dependency feature 지정 검사 명령은 workspace 밖 package에 features를 지정할 수 없어 거절됐고, 올바른 `-p ... --lib` 검사로 통과했다.

## root 전용 실제 A/B 명령

아래 PowerShell 변수는 경로만 지정한다. 모든 arm은 saved 같은 EXE·model·원본 WAV/text·CPU4/F32를 사용하며 서로 다른 NEW output을 지정한다. root의 기존 resource supervision 안에서 실행한다. 이 세션은 아래 align을 실행하지 않았다.

```powershell
$segExe = 'D:/workspace/@myso-kr/com.kinekino.player/.spec/evidence/player/forced-aligner-segmentation-builds/r2-final/forced-aligner-segmentation-spike.exe'
$segModel = 'D:/workspace/@myso-kr/com.kinekino.player/runtime/ai-spike/models/Qwen--Qwen3-ForcedAligner-0.6B/c7cbfc2048c462b0d63a45797104fc9db3ad62b7'
$segText = 'D:/workspace/@myso-kr/com.kinekino.player/runtime/ai-spike/assets/alignment/ja-1634.txt'
$segAudio = 'D:/workspace/@myso-kr/com.kinekino.player/runtime/ai-spike/assets/speech-ja-1634.wav'
$segCohort = 'D:/workspace/@myso-kr/com.kinekino.player/runtime/ai-spike/acquisitions/segmentation-cohort-r1'
$segOut = 'D:/workspace/@myso-kr/com.kinekino.player/runtime/ai-spike/results'
& $segExe align --model-dir $segModel --audio $segAudio --text-file $segText --language Japanese --threads 4 --output "$segOut/segmentation-ab-ja1634-default-r1.json"
& $segExe align --model-dir $segModel --audio $segAudio --text-file $segText --language Japanese --threads 4 --words-file "$segCohort/ja-1634-baseline-control.json" --output "$segOut/segmentation-ab-ja1634-control-r1.json"
& $segExe align --model-dir $segModel --audio $segAudio --text-file $segText --language Japanese --threads 4 --words-file "$segCohort/ja-1634-original-tokenizer.json" --output "$segOut/segmentation-ab-ja1634-original-r1.json"
```

JA1516은 `speech-ja-1516.wav`/`alignment/ja-1516.txt`/Japanese 및 같은 ID의 cohort를, KO1883은 `speech-ko.wav`/`alignment/ko-1883.txt`/Korean 및 ko-1883 cohort를 그대로 사용한다. 각 audio SHA와 transcript SHA는 root cohort result에 고정되어 있고 report에도 남는다. root가 source/model identity를 바꾼 경우 동일 요인 A/B로 비교하면 안 된다.

```powershell
$segCompare = 'D:/workspace/@myso-kr/com.kinekino.player/.spec/evidence/player/forced-aligner-segmentation-builds/r2-final/compare-runs.exe'
& $segCompare "$segOut/segmentation-ab-ja1634-default-r1.json" "$segOut/segmentation-ab-ja1634-control-r1.json" same-words "$segOut/segmentation-ab-ja1634-control-comparison-r1.json"
& $segCompare "$segOut/segmentation-ab-ja1634-default-r1.json" "$segOut/segmentation-ab-ja1634-original-r1.json" segmentation "$segOut/segmentation-ab-ja1634-original-comparison-r1.json"
```

compare-runs는 실제 report files를 받는다(stdout envelope가 아님). text/audio/model/device/CPU4/F32/engine/adapter/compiled input/EXE identity 일치를 요구한다. same-words에서는 source/file provenance·load/inference timing 차이를 제외하고 selected words/prompt/tokenizer, 실제 model IDs와 전체 raw_head/logits/fixed/final을 정확 비교한다. 실패면0이 아닌 exit로 거부하며 원본 결과를 수정하지 않는다. segmentation 모드는 차이를 그대로 기록하고 quality=false를 유지한다. 구조 오류로 align이 exit3이어도 raw report가 새 파일로 보존된 경우 root가 원문을 확인해야 하며 zero clamp로 성공 처리하지 않는다.

root의 실제 동일 단위 결과 대조, 일본어 분절 A/B의 raw equal/repair collapse 영향, 사람 word 경계 품질 검토·공식 encoder 수치 대조는 아직 남아 있다. API/GUI 제품 기능으로 연결하지 않았고 모델/패키지 다운로드·새 추론·장시간재생·Pages·VM·DPI·사용자 앱 종료는 하지 않았다.
