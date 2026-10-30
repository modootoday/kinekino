---
id: evidence:kinekino/20261029113716
title: 공식 md-eval DER 대조용 Rust adapter 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공식 md-eval DER 대조용 Rust adapter 준비

2026-10-18. **별도 `tools/diarization-der-reference` 구현·합성 검증·source/lock/EXE 보존 완료. 실제 Perl/scorer 실행·원문 script/코퍼스 취득·모델 load/infer는 이 작업에서 0이다.** root가 원문 취득·기존 host Perl 확인·실제 DER 대조를 소유한다. 기존 `diarization-evaluation`/5202 동결본·실제 corpus 평가 JSON·제품/model-spike/UI·이전 문서를 수정하지 않았다.

이 도구는 검증용 외부 기술 adapter다. 새 Perl/Python 코드를 작성하거나 설치하지 않으며 제품 runtime 의존성을 추가하지 않는다. 원문 source는 웹에서 읽기만 했고 로컬 script 사본을 이 작업이 취득/저장하지 않았다. build bundle에도 scorer 원문이나 corpus는 없다.

## 검토한 원문과 고정 옵션

정본은 [dscore commit e02f949…의 수정 없는 md-eval-22.pl](https://github.com/nryant/dscore/blob/e02f949ac6592279300a2c33d03daf9e0c12fd27/scorelib/md-eval-22.pl)이다. Git blob SHA1 **`27b7bc9da5751eec8fb8de2f7fe570d301f966bd`**, **109574 bytes**를 컴파일된 gate에 고정했다. root가 취득한 local file은 이 길이·Git object hash와 request에 명시한 실제 SHA256이 모두 맞아야 한다. Git blob hash는 `blob {bytes}\0` header와 raw bytes를 합쳐 계산하므로 plain file SHA1과 구별한다.

[같은 commit의 LICENSE](https://github.com/nryant/dscore/blob/e02f949ac6592279300a2c33d03daf9e0c12fd27/LICENSE)는 BSD-2-Clause다. 1318 bytes/Git blob SHA1 `1ba4497266665de07bfd0818f020464a807125dc`와 root의 SHA256을 대조한다. adapter의 실제 실행 때 원문 script/header와 LICENSE를 새 결과 디렉터리에 byte 그대로 복사한다. LF/CRLF 변경·패치·header 재작성은 하지 않는다. 저장하는 source URI는 고정 raw GitHub 경로뿐이며 인증 token/임시 query URI를 요청하거나 보존하는 기능은 없다.

외부 실행은 다음 고정 argv만 쓴다. 괄호 내용은 adapter가 만든 절대 경로 하나이며 각 경로는 하나의 `Command` argument다.

```text
[root가 확인한 Perl.exe] -- [새 디렉터리/md-eval-22.pl]
    -r [reference.rttm] -s [hypothesis.rttm] -u [scoring.uem] -c 0
```

`-1`은 reference overlap 제외이므로 사용하지 않는다. `-o`는 source상 metadata(MD) overlap 옵션이며 DER overlap 포함에 필요한 스위치가 아니므로 사용하지 않는다. 기본 time-mediated/time-overlap mapping을 유지하기 위해 `-w/-W`도 쓰지 않는다. UEM을 명시해 silence FA가 사라지는 자동 UEM 추정을 피한다. SPEAKER/channel1/<NA>만 출력하므로 lexical/non-lexical no-score 확장 자료는 주지 않는다. conditional analysis 옵션을 주지 않아 ALL speaker analysis 하나를 기대한다.

### 시간·정답 의미

기존 Rust evaluator library의 ref/hyp/UEM 파서·crop·union·validation과 metric 계산을 **path dependency로 읽기 전용 재사용**한다. 원래 5202 보고서의 source offset/fixture/stdout/ref/UEM hashes를 모두 다시 검사하고 같은 입력에서 재계산한 Rust DER components/점수가 원래 보고서와 정확히 일치해야 다음 단계로 진행한다. reference 화자 수를 clustering에 주거나 hypothesis를 정답에 맞춰 고치지 않는다.

동일 UEM에 교차된 reference와 **non-null known speaker_id만의 hypothesis**를 local 반열림 구간으로 만든다. null short span과 unresolved overlap은 reference/UEM에서 제거하지 않는다. 각 timestamp/duration을 고정 소수 6자리 초로 내보내 1us 정보를 보존한다. 같은 화자 중복은 union하며 다른 화자의 overlap은 남긴다. UEM도 원래의 silence 포함 영역을 유지한다.

md-eval 원문은 onset/duration을 Perl numeric으로 더해 end를 만들며 DER 경계를 frame grid로 양자화하지 않는다. event 정렬과 짧은 구간 판정에는 `epsilon=1E-8`초가 있다. Rust의 정확한 정수 microsecond와 Perl binary64의 표현·누적 차이는 실제 대조에서 관측할 대상이며 gold를 shift하거나 tolerance를 사후 확대하지 않는다. NIST UEM reader가 basename/extension을 제거하므로 이 adapter는 점/콜론 없는 단순 file ID를 요구해 UEM 불일치와 자동 fallback 위험을 피한다.

## host Perl과 Windows↔Cygwin 경계

root가 전달한 실제 확인 값은 **`C:/Program Files/Git/usr/bin/perl.exe`**, Perl **v5.42.2**, **x86_64-cygwin-thread-multi**다. 이 worker는 해당 interpreter를 실행하거나 별도 버전 값을 추측하지 않았다. 경로는 위 명시 경로만 허용하고 executable SHA256은 root의 실제 hash를 request로 받아 다시 파일과 대조한다. root가 저장한 `--version` stdout 원본 hash와 정확한 `This is perl …` 한 줄도 필수다. 버전 family/architecture와 증거의 line 일치를 검사한다. exe·version 증거 검증은 host core module/DLL 전체 attestation을 대체하지 않는다.

공백 경로를 하나의 argv로 전달하고 Windows verbatim `\\?\` prefix를 제거해 `D:/…` 형태의 forward-slash 경로를 만든다. shell을 통하지 않으며 quoting된 문자열을 명령으로 재해석하지 않는다. 이 경로 변환/argv 형태는 합성 테스트만 통과했다. **Cygwin Perl이 실제 생성 경로와 UTF-8/파일을 열고 scorer를 완료하는지는 root 실제 실행에서 검증해야 한다.** 실패 시 원문/이름을 임의 patch하거나 넓은 tolerance로 성공 처리하지 않는다.

child에만 PERL5OPT/PERL5LIB/PERLLIB/PERL_LOCAL_LIB_ROOT/PERL_MB_OPT/PERL_MM_OPT를 제거하고 LC_ALL/LANG=C를 설정한다. global 환경은 변경하지 않는다. stdin은 null, window는 숨김, 프로그램은 고정 exe이고 `-e`, shell, 임의 extra argument, 웹에서 받은 프로그램 실행은 없다. spawn 직전에도 원문 blob/길이와 exe/script SHA를 재검사한다.

timeout은 **60초**, stdout/stderr는 **각 1,048,576 bytes**다. 두 pipe를 동시에 drain하고 cap 초과나 timeout 때 child를 kill/wait한다. 성공·오류 경로의 child 정리 guard가 있다. cap flag는 reader join 후 확인해 빨리 종료한 process의 초과 출력도 놓치지 않는다. 이는 고정된 검토 script 하나를 위한 제어이며 일반적인 임의 프로그램 sandbox/프로세스 트리 관리 기능이 아니다. 실제 spawn·timeout 동작은 이번 worker가 실행하지 않았다.

## root의 실행 요청

CLI는 `run ABS_REQUEST_JSON ABS_NEW_OUTPUT_DIRECTORY`다. 기존 디렉터리나 파일에 덮어쓰지 않는다. request는 unknown field를 거부하며 다음 11개 field를 요구한다. 모든 경로는 local absolute path, 모든 SHA256은 root가 실제 취득/확인한 lowercase64 hex여야 한다. `<…>`는 설명용 자리 표시자이며 실제 request로 실행하지 않는다.

```json
{
  "evaluation_report": "<기존5202의 실제 평가 JSON 절대 경로>",
  "evaluation_report_sha256": "<해당 JSON 실제 SHA256>",
  "perl_executable": "C:/Program Files/Git/usr/bin/perl.exe",
  "perl_sha256": "<root가 확인한 기존 Perl EXE SHA256>",
  "perl_version_evidence": "<root의 실제 --version stdout 절대 경로>",
  "perl_version_evidence_sha256": "<그 파일 SHA256>",
  "perl_version_line": "<stdout에서 복사한 정확한 This is perl … 한 줄>",
  "md_eval_script": "<root가 취득한 수정 없는 md-eval-22.pl 절대 경로>",
  "md_eval_sha256": "<원문 실제 SHA256>",
  "dscore_license": "<같은 commit LICENSE 절대 경로>",
  "dscore_license_sha256": "<LICENSE 실제 SHA256>"
}
```

원래 report의 `evaluator_executable_sha256`은 `5202d56c5d91afece9a344a9b6d2572868707ecf46904b425ae5fa73a10aca6b`, schema/protocol은 v0.1.0/collar0/overlap-included여야 한다. 해당 JSON에서 참조하는 실제 audio/stdout/ref/UEM 파일을 다시 해시하고 WAV format/frame count·source_offset_applied_us0·모든 span bounds를 기존 library로 검증한다. 원래 JSON은 수정하지 않는다. JSON provenance는 서명된 attestation이 아니므로 root의 원래 frozen 실행 증거와 함께 해석한다.

새 디렉터리에는 reference.rttm/hypothesis.rttm/scoring.uem, 수정 없는 md-eval-22.pl/LICENSE.dscore, 원래 Perl version stdout, execution-plan.json을 저장한다. plan은 각 입력 hash와 argv·허용오차·host 정보·adapter EXE hash를 실행 전에 고정한다. 이어 raw md-eval.stdout/md-eval.stderr와 comparison.json을 새로 저장한다. spawn/capture 자체 오류는 execution-error.json과 INCOMPLETE marker로 구별한다. finalized comparison 실패는 false/원인/원출력을 보존하며 exit error다. 실패를 NIST DER=0으로 채우지 않는다.

## 엄격 파서와 사전 고정 통과 기준

원문 `print_sd_scores`의 ALL header가 정확히 한 번 있어야 한다. scored/missed/false-alarm/confusion speaker-time 네 원문 label, six-decimal 초, 전체 DER label과 two-decimal percentage/ALL suffix를 정확히 읽는다. label 중복·누락·다른 조건·NaN/inf·지수·음수·잘못된 정밀도·0분모는 오류다. 실제 process exit0·timeout0·cap0·비공백 stderr 없음도 필수다. 100% 초과 DER은 정상 숫자로 유지한다.

**DER 통과 기준은 사전에 `abs(Rust_DER − NIST_%.2f) ≤ 0.0051 percentage point`로 고정**했다. 2자리 반올림의 최대0.005와 작은 부동소수 여유0.0001의 근거다. 추가로 `%f`(6자리 초) 네 components를 각각 비교해 **차이≤1us**여야 한다. 2자리 점수가 우연히 같아도 components가 다르면 통과시키지 않는다. parser 내부의 printed-components↔printed-DER 자기 일관성 검사는 `%f`/`%.2f`의 출력 반올림 전파 bound를 쓰지만, **실제 Rust↔NIST 통과 허용오차를 확대하지 않는다**.

성공해도 `external_der_parity=true`는 **이 exact known-hyp/UEM/collar0/overlap-included 쌍의 DER**에만 적용한다. `external_jer_parity=false`, `external_scorer_parity_verified=false`는 그대로다. 공식 Python JER을 실행하지 않았으므로 JER나 전체 scorer parity를 true로 만들지 않는다. DER parity는 모델 품질 승인과도 별개다.

## 합성 검증과 보존 결과

최종 합성 테스트 **9개 통과**: 엄격 ALL 파서/별도 parity scope, malformed·missing·duplicate·ambiguous·nonfinite/precision·0분모, 고정 tolerance 경계·component 불일치, DER300 유지, six-decimal RTTM roundtrip와 overlap·silence UEM, 고정 argv/공백/verbatim 경로, bounded pipe reader와 Git blob known answer, 실패 raw output 보존, 동일 synthetic known-hyp 파생/unknown MISS 보존/원래 report 불일치 검출이다. synthetic provenance test의 5202 hash field는 gate 검사용 합성 envelope이며 실제5202가 그 synthetic report를 만들었다는 증거로 쓰지 않는다. 실제 corpus나 외부 scorer output은 테스트에 포함하지 않았다.

독립 manifest와 별도 target에서 다음이 모두 exit0이었다.

```text
cargo fmt --manifest-path tools/diarization-der-reference/Cargo.toml --check
cargo test --manifest-path tools/diarization-der-reference/Cargo.toml --target-dir runtime/parallel-build/diarization-der-reference -j 2 --locked
cargo clippy --manifest-path tools/diarization-der-reference/Cargo.toml --target-dir runtime/parallel-build/diarization-der-reference -j 2 --locked --all-targets -- -D warnings
cargo build --manifest-path tools/diarization-der-reference/Cargo.toml --target-dir runtime/parallel-build/diarization-der-reference -j 2 --locked
```

build.rs가 새 crate와 읽기 전용 기존 evaluator dependency의 source/tests/manifests/locks **40개 입력**을 fingerprint한다. preserve-build 시 현재 byte hashes가 내장 목록과 일치해야 하고 새 content-addressed 디렉터리에만 쓴다. EXE·compiled-source.json을 합친 **42개 파일**을 보존한 뒤 저장된 frozen EXE의 verify-build로 재확인했다. 새 bundle에 script·corpus·Perl binary는 포함하지 않는다. Windows SDK/전역 Cargo config/host Perl core modules 전체를 동결하거나 bit-identical 재빌드를 입증한 것은 아니다.

- [새 build manifest](../../../evidence/ai/diarization-der-reference-builds/aed71a64a8524f68b34c6938f46d6942b59af3eaba8301dfc6162ed0896f2745/build-manifest.json)
- EXE SHA256 **`aed71a64a8524f68b34c6938f46d6942b59af3eaba8301dfc6162ed0896f2745`**, 1,867,776 bytes
- manifest SHA256 `d314172eaeef1e8921d4d7c25211308ae2ca7562062aeaf13db284e570942b49`
- 새 Cargo.lock SHA256 `7bef8c9e09d758debeeab14237ada10df14a88c6bbb005e7fbcddb55eb4f6d37`
- root가 사용할 frozen EXE: `.spec/evidence/ai/diarization-der-reference-builds/aed71a64a8524f68b34c6938f46d6942b59af3eaba8301dfc6162ed0896f2745/diarization-der-reference.exe`

기존 5202 frozen EXE의 verify-build도 다시 exit0이었으며 기존 bundle을 변경하지 않았다. 이 준비 작업의 끝 상태는 **adapter 준비·합성 검증 완료, actual NIST DER/JER parity 미검증**이다. 실제 실행은 root가 위 새 EXE와 검증한 host/source evidence·새 출력 디렉터리로 수행한다.

이전 준비 build 86aba1e2…는 변경하지 않고 보존했다. 최종 aed71a64…는 execution-plan의 argv를 읽기 쉬운 문자열 배열로 저장하는 변경을 포함하며 위 최종 검증을 다시 통과했다. 실제 Command argv와 기록용 문자열은 같은 고정 생성 함수를 사용한다.
