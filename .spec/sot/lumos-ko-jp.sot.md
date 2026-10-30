---
id: sot:kinekino/lumos-ko-jp
title: "lumos KO/JP 로컬 정렬: 사전 반입·권리·빌드 준비 보충"
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# lumos KO/JP 로컬 정렬: 사전 반입·권리·빌드 준비 보충

조사일2026-10-18. lumos SHA `7984967e8701cc90bb17d06453901036e36ca578`에 고정한다. 앞선 [정렬 runtime 조사](rust-forced-aligner-runtime.md)를 보충한다. 이번에는 source/metadata/작은 권리 문서 GET와 archive **HEAD만** 수행했다. archive·사전 본체·가중치 다운로드, source clone, Cargo fetch/build, 추론·설치는 하지 않았다. root 단일 추론 worker에 간섭하지 않았다.

## 결론과 로컬 검증의 조건

현재 source에서 `forced-aligner` feature는 JP/KO를 함께 활성화하고 IPADIC를 내장하도록 설정한다. **입력을 한국어로만 정해도 IPADIC build fetch는 생길 수 있다.** 따라서 실행 담당은 약12.9MB archive와 registry/source dependency를 별도 반입으로 기록한다. 이 연구 세션의 다운로드·빌드·추론 금지는 담당 분리와 root 단일 worker 정책이며, root의 기존 포괄 승인 범위를 금지하거나 추가 사용자 승인을 요구하는 조건이 아니다. 아래 명령/환경은 이 연구 세션에서 실행하지 않았다.

Lindera 코드 MIT, IPADIC COPYING의 사용·복제 허용 문구, lumos Cargo의 MIT OR Apache-2.0 선언은 로컬 비배포 검증의 권리 근거다. 이 자료에서 연구 전용/비상업 사용만 허용한다는 제한이나 로컬 검증 전 별도 저작자 승인을 요구하는 조항은 찾지 못했다. **제품 배포용 고지 패키징이 미완성이라는 이유만으로 로컬 실험에도 동일한 배포 승인 절차를 요구하지 않는다.** 비배포 검증에서도 출처/저작권 문구를 보존하며 IPADIC 원본/변형 사본에 지정 고지를 포함한다. 실행은 root 통합 담당이 기존 승인 범위에서 수행한다.

현행 실험에 실제 필요한 것은 사용권 근거와 고지를 보존한 고정 source·사전 준비, 승인된 반입 범위의 구분, archive 무결성/캐시 격리, source가 자동으로 실행할 다운로드 파악이다. GUI에 attribution 화면을 만들거나 최종 설치 패키지 NOTICE를 완성하는 작업은 로컬 실험의 선행조건이 아니다. 제품 번들/고객 배포 전에는 사전 포함 바이너리와 LICENSE/COPYING·수정 고지 범위를 다시 확정한다.

## 1. 고정 의존성과 source 크기

고정 [lumos Cargo.lock](https://github.com/lumosimmo/qwen3-asr-rs/blob/7984967e8701cc90bb17d06453901036e36ca578/Cargo.lock)의 registry identity:

| package | 고정 버전 | registry checksum(SHA256) |
|---|---|---|
| lindera | 2.2.0 | `d1ada0aedb9696306a29fde03ec9c57fc49c0666bd79252f243f14221462f89f` |
| lindera-ipadic | 2.2.0 | `230940a008229feec880d542c337f47e7ecf10aa5823c8ddcfcaac856c575fb8` |
| lindera-dictionary | 2.2.0 | `5573447b93c7f63dc95b02e7e52825e02d188f4f3c5055d5f76f8886d8ae22c5` |

IPADIC crate의 [.cargo_vcs_info.json](https://docs.rs/crate/lindera-ipadic/2.2.0/source/.cargo_vcs_info.json)는 Lindera Git SHA `d40ff76b085f5fd671040ed3c8c36d6a7a0fa6dc`, path `lindera-ipadic`를 명시한다. 최신6.x 문서 대신 이2.2.0 source를 검토했다. registry API의 `lindera-ipadic` crate_size는21,377 bytes다. **이 작은 crate tarball에 실제 사전 archive가 전부 들어 있다는 뜻은 아니다.**

Git tree API(모두 truncated=false) 메타데이터:

| source/자산 | byte 또는 개수 | 해석 |
|---|---:|---|
| lumos 고정 tree 전체 | 259 blob, 합계17,840,356 bytes | Git blob의 논리 byte 합. clone 전송량/`.git` 크기/빌드 산출물 크기가 아님. fixtures 등 비코드 자산도 포함 |
| lumos Cargo.lock | 139,736 | dependency snapshot |
| lumos Korean dictionary | 275,834 | source tree에 포함된 텍스트 자산, 별도 HF 모델 fetch 아님 |
| lumos Japanese merge list | 354 | source에 포함된6개 사용자 정의 merge term |
| IPADIC upstream tag tree | 합계64,967,372 bytes | 아래 tag의 Git blob 합. HTTP tar.gz의 압축해제 전체 byte와 동일하다고 미검증 |
| IPADIC archive HEAD | 12,907,073 | 아래 URL의 압축 전송 Content-Length. 본체를 받지 않음 |

registry의 Rust package들·Git source·IPADIC HTTP archive·Qwen 모델 파일은 각각 다른 전송이다. Cargo.lock에 ko-dic/unidic/neologd 등의 package 이름이 있어도 해당 feature의 실제 활성화/빌드 fetch를 증명하지 않는다. 이 조사에서 확정한 추가 사전 fetch는 `embed-ipadic` 경로다. 전체 transitive dependency download 총량·IPADIC 컴파일 후 binary 증가는 미실측이다.

## 2. 실제 build fetch URL·버전·검증값

고정 [lindera-ipadic/build.rs](https://github.com/lindera/lindera/blob/d40ff76b085f5fd671040ed3c8c36d6a7a0fa6dc/lindera-ipadic/build.rs)와 [registry source](https://docs.rs/crate/lindera-ipadic/2.2.0/source/build.rs)는 동일한 FetchParams를 선언한다:

| 항목 | source/HEAD에서 읽은 값 |
|---|---|
| download URL | `https://Lindera.dev/mecab-ipadic-2.7.0-20250920.tar.gz` |
| archive basename | `mecab-ipadic-2.7.0-20250920.tar.gz` |
| input directory | `mecab-ipadic-2.7.0-20250920` |
| output directory | `lindera-ipadic` |
| build script expected MD5 | `a95c409f12f1023fce8ef91f991ef042` |
| HEAD ETag | `a95c409f12f1023fce8ef91f991ef042`(따옴표 제외) |
| HEAD Content-Length | 12,907,073 bytes |
| HEAD Last-Modified | Fri, 19 Sep 2025 15:12:53 GMT |
| archive SHA256 | **미수집·미검증**. MD5나 ETag를 SHA256이라고 쓰지 않음 |

HEAD는 서버가 광고한 메타데이터만 확인한다. archive body의 MD5/내용/해제 결과를 실제 검증한 것이 아니다. 다음 반입 담당은 기존 Rust download/hash 도구로 body를 확보한 뒤 SHA256·길이·MD5를 별도로 manifest화해야 한다. 그 전 다운로드 완료 상태로 바꾸지 않는다.

[공식 mecab-ipadic tag `2.7.0-20250920`](https://github.com/lindera/mecab-ipadic/tree/79f5146f8615277ca2c1817f04af9b932c9c6423)의 commit은 `79f5146f8615277ca2c1817f04af9b932c9c6423`다. README의 옛 `lindera-morphology/mecab-ipadic` 주소는 현재 `lindera/mecab-ipadic`으로 연결된다. archive 버전명과 tag가 대응하지만 archive 본체를 안 받았으므로 tag tree와 tar 내용 전체의 동등성을 검증했다고 쓰지 않는다. 이전20070801판 다운로드 URL을 현행2.2.0 build URL로 사용하지 않는다.

## 3. 자동 네트워크·캐시·Windows 동작

고정 [assets.rs](https://github.com/lindera/lindera/blob/d40ff76b085f5fd671040ed3c8c36d6a7a0fa6dc/lindera-dictionary/src/assets.rs)를 확인했다.

- `embed-ipadic`이 켜져 있으면 사전 build가 실행된다. feature가 꺼져 있어도 `LINDERA_DICTIONARIES_PATH`/`LINDERA_CACHE`가 설정되면 build가 시작될 수 있다. 환경 변수를 단순히 무해한 metadata로 취급하지 않는다.
- `LINDERA_DICTIONARIES_PATH` 또는 deprecated `LINDERA_CACHE` 아래 version 디렉토리, 아니면 Cargo `OUT_DIR`를 사용하고 `LINDERA_WORKDIR`를 dependents에 전달한다. 상대 경로는 workspace 맥락에서 해석될 수 있으므로 후속 시험에서는 **작업 전용 절대 경로**를 사용한다. 기존 사용자/공유 캐시로 지정하지 않는다.
- 소스 archive가 존재하면 MD5를 읽어 검사하고 실패한 파일을 지운 뒤 재다운로드한다. 미완성·해제/출력 디렉토리에도 정리/교체 코드가 있다. 따라서 기존 증거 폴더를 이 캐시로 재사용하지 않는다.
- reqwest client의 GET·재시도→body bytes→MD5 확인→temporary 파일→tar.gz 해제→DictionaryBuilder 순서다. Rust package 취득만 끝났다고 network-free build가 되는 것은 아니다. `cargo --offline`은 Cargo resolver/download 제어이지 이 build script의 reqwest 호출을 막는 근거가 아니다.
- Windows 전용 directory copy 후 remove fallback은 구현되어 있다. 이는 Windows 고려 증거이며 실제 이 호스트의 빌드 성공을 뜻하지 않는다. source는 사전 archive·압축해제 데이터를 메모리에 읽는 구간이 있으므로 streaming download만의 작은 peak를 보장하지 않는다.
- `DOCS_RS`가 설정되면 테스트용 dummy 사전을 만드는 경로가 있다. 네트워크를 피하려고 이 값을 켜서 실제 일본어 검증을 통과시키면 안 된다.

현재 lumos는 `forced-aligner` feature에서 Lindera와 IPADIC embed를 함께 요구한다. **KO-only 입력으로 runtime 분기를 피하는 것과 dependency build를 끄는 것은 다르다.** KO-only feature split·외부 dictionary loading 변경은 가능한 후속 설계지만 아직 구현하지 않았고 기존 build 명령이 자동으로 그렇게 된다고 하지 않는다. 연구 세션은 source/metadata만 맡고 추가 fetch/build를 수행하지 않는다. root 통합 담당은 기존 승인 범위에서 사전 반입·빌드를 순차 실행하고 manifest·전용 캐시로 기록한다. 별도 사용자 승인 대기를 새로 만들지 않는다.

## 4. IPADIC·Lindera 권리 원문과 로컬/배포 구분

Lindera 소프트웨어의 고정 [LICENSE](https://github.com/lindera/lindera/blob/d40ff76b085f5fd671040ed3c8c36d6a7a0fa6dc/LICENSE)는 MIT, Git blob `9b50cef40b03e44df754cea1540cba6aa9e94cdb`,1,080 bytes다. code crate의 MIT와 dictionary 자체의 COPYING은 별개다.

tag 고정 [IPADIC COPYING](https://github.com/lindera/mecab-ipadic/blob/79f5146f8615277ca2c1817f04af9b932c9c6423/COPYING)은3,795 bytes, Git blob `dc7db628f00808db67545ba3686032ef2126e356`이다. 원문을 읽었으며 NAIST2000–2003 저작권·사용/복제/배포 허용·원본/변형 사본의 지정 고지 포함·ICOT 유래 부분의 무보증 고지·법규 준수 조건을 확인했다. 이를 MIT나 Apache로 바꿔 표시하지 않는다. `Program was produced ... research`라는 배경 문장을 연구 전용 라이선스 제한으로 해석하지 않는다.

| 구분 | 지금 확인한 근거로 필요한 사항 | 로컬 시험과 혼동하지 않을 사항 |
|---|---|---|
| 로컬 비배포 준비/검증 | 고정 source와 COPY/저작권 문구 보존, root 담당의 사전 반입/해시·전용 캐시, 단일 worker 정책 | 최종 설치 패키지 attribution UI·고객 배포 승인까지 먼저 만들 필요는 없음 |
| 사전 포함 binary/제품 배포 | Lindera MIT와 IPADIC COPYING·ICOT 고지 적용 범위 보존, 파생 사전/수정 기록·패키지 inventory 확정 | 로컬 사용 허용 확인이 모든 배포 고지 작업을 끝내는 것은 아님 |
| 남은 확인 | 실제 archive에 들어 있는 COPYING이 tag 원문과 같은지, build 출력에 고지가 어떻게 남는지, SHA256·전체 파일 목록 | source/tag 읽기만으로 archive 본체와 output package를 검증했다고 하지 않음 |

## 5. 한국어 dictionary: 원본 hash와 고지 연결

다음 두 고정 Git tree의 파일은 모두275,834 bytes이며 Git blob ID가 **`42b570689e065ed2803d0f43b00ca38255813d2f`로 같다.** 이 ID는 SHA1 기반 Git blob hash(파일 내용+Git blob header)이지 raw SHA256이 아니다. 사전 본문을 추가 다운로드·hash 계산하지 않고 tree metadata로 직접 원본 연결을 확인했다.

- [lumos assets/korean_dict_jieba.dict](https://github.com/lumosimmo/qwen3-asr-rs/blob/7984967e8701cc90bb17d06453901036e36ca578/qwen3_asr/src/forced_aligner/assets/korean_dict_jieba.dict)
- [공식 Qwen 원본](https://github.com/QwenLM/Qwen3-ASR/blob/7c6daf77a2421100f5fb066495372c00129d39ff/qwen_asr/inference/assets/korean_dict_jieba.dict)

lumos processor 주석도 이 Qwen 경로를 Apache-2.0으로 지목한다. 공식 Qwen [LICENSE](https://github.com/QwenLM/Qwen3-ASR/blob/7c6daf77a2421100f5fb066495372c00129d39ff/LICENSE)는 Apache-2.0이며 Git blob `c347eb5ec781d2702623e8e2b5923fa1a766a64d`,11,343 bytes다. 조사한 공식 tree에서 Korean 자산에 대한 별도 NOTICE/COPYING은 발견하지 못했다. 파일명의 `jieba`만으로 jieba/soynlp 소프트웨어나 다른 사전의 라이선스를 자동 대입하지 않는다. Qwen보다 이전의 사전 제작자·수집 데이터·제3자 권리는 이 연결만으로 모두 확인되지 않는다.

로컬 검증은 Qwen source의 Apache 선언·정확한 사본 identity를 근거로 원본 경로/고지를 보존해 진행할 준비가 된 상태다. 이것을 한국어 사전 자체가 gated 모델이라고 표현하지 않는다. 제품 배포 전에는 Apache 원문·upstream attribution·복사/변형 여부와 더 이전 데이터 provenance의 불명 범위를 inventory에 남긴다. raw SHA256은 현재 미수집이며 후속 Rust 자산 manifest 생성 때 기록한다. Qwen 전체 모델 가중치와 Korean dictionary는 별도 파일이다.

## 6. 일본어 기본 merge 목록의 유래

고정 [japanese_merge_terms_default.txt](https://github.com/lumosimmo/qwen3-asr-rs/blob/7984967e8701cc90bb17d06453901036e36ca578/qwen3_asr/src/forced_aligner/assets/japanese_merge_terms_default.txt)는354 bytes, Git blob `5b50388aff695b58450eeb318d70e2fd76f09828`이다. 헤더는 Lindera 경계를 공식 nagisa에 가깝게 하려는 보수적 merge 목록이라고 설명한다.6개 term은 OpenAI, testexamplecom, 트랜스포머/파인튜닝/멀티모달/리얼타임의 일본어 표기다. 대규모 외부 사전이나 nagisa model weights가 이 파일에 내장된 것은 아니다.

이 경로의 공개 commit 기록은 현재 고정 commit의 `publish` 한 건이고 해당 commit parent는 없다. 따라서 이전 작성 이력·term 선정 실험의 연혁은 공개 Git history로 추적하지 못했다. 파일에 별도 외부 출처·작성자·라이선스 override 표기는 없다. workspace Cargo는 MIT OR Apache-2.0 선언이지만 root LICENSE/NOTICE 원문이 없는 점은 이전 조사와 같다. **문서화된 source 내부 사용자 정의 목록**으로 기록하되 제3자 복제가 없다는 독립 증명이나 nagisa 공식 사전이라고 주장하지 않는다.

로컬 실험에서는 고정 목록을 입력 조건으로 남기고 사용자 사전 추가 없이 baseline을 시작하면 된다. 원문 분절과의 차이를 평가하며 목록이 품질을 개선했다는 결론은 미실측이다. 제품 배포 때에는 workspace 라이선스 선언과 해당 파일/merge 규칙의 고지를 포함하고, 제3자 유래 정보가 새로 발견되면 별도 조건을 반영한다.

## 다음 실행 담당에게

1. 이 연구 세션은 **source/metadata 검토만 담당**하며 다운로드/build/inference는 root 통합 담당의 기존 승인 범위다. 담당 분리 지시를 사용자 전체 금지나 새 승인 요구로 확대하지 않는다.
2. root는 IPADIC2.7.0-20250920 archive12,907,073 bytes와 checksum을 별도 Rust manifest로 기록하고 COPYING·압축해제 경로·전용 cache를 확인한다. Cargo registry package와 HTTP archive 취득을 다른 로그/상태로 기록한다. 인계 시점 root 보고에 따르면 고정 source clone을 완료하고 CPU library build(session88533)를 시작했으며 cache는 `runtime/ai-spike/dictionaries/lumos-7984967`이다. Qwen model weights 추가 다운로드는 아직 없으며 빌드 검증 뒤 root가 판단한다. 이 진행 보고는 연구 세션의 독립 실행/검증 결과가 아니다.
3. KO-only 검증에도 현행 feature에서 IPADIC build가 필요할 수 있음을 예산·네트워크 조건에 포함한다. IPADIC 없이 KO만 가는 경로가 필요하면 작은 feature split을 별도 Rust 변경으로 계획하고 기존 golden/JP 기능을 우연히 끄지 않는다. 이번에는 코드를 수정하지 않았다.
4. lumos local align API에 기존 원음/정답을 넣는 adapter는 아직 구현 제안이다. raw/fixed 시간·분절·원본 duration·사전 identity를 함께 보존하고 일본어 nagisa 분절 차이를 표시한다. 이 문서는 사전 readiness이며 정렬 품질 통과 증거가 아니다.
