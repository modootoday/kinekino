---
id: sot:kinekino/plugin-signature-verification
title: 플러그인 minisign 서명 검증 설계
status: draft
created: 2026-10-30T06:42:36Z
updated: 2026-10-30T06:42:36Z
rel:
  governs: [crates/kinekino-source-host]
---

# 플러그인 minisign 서명 검증 설계

## Rule

## Why

## Verification
## 범위

[ADR 0011](../adr/20261030200000-adr-0011-media-source-plugin.adr.md) 결정 5·6(1단계 격리 = 서명 검증 + 사용자 설치·권한 승인)의 서명 검증을 설계한다. 키 생성·공개키 등록·시크릿 저장은 원 님 승인 항목이라 이 문서는 절차만 적고 실행하지 않는다. 코드 변경도 없다.

## 형식

- 도구: [minisign](https://jedisct1.github.io/minisign/) (Ed25519). 검증은 Rust 크레이트 `minisign-verify`(의존성 없음, MIT)를 후보로 한다. 도입 시 버전·해시를 고정하고 THIRD_PARTY.md에 기록한다.
- 서명 대상: 배포 아카이브(`<plugin>-<target>.tar.gz|zip`)와 그 안의 `plugin.json`. 아카이브 서명 파일은 `<archive>.minisig`, 설치 후 검증용으로 `plugin.json.minisig`를 플러그인 폴더에 함께 둔다.
- 신뢰 주석(trusted comment)에 `id`, `version`, `target`, 아카이브 SHA-256을 넣어 서명이 다른 플러그인·버전에 재사용되지 않게 한다.
- 키 ID: minisign 공개키의 8바이트 key id를 매니페스트 `signature.key_id`로 선언하고, 호스트는 설정에 등록된 공개키 중 같은 key id만 시도한다.

## 검증 흐름(호스트)

1. 설치(`kinekino plugins install <archive>`): `.minisig` 검증 → trusted comment의 id·version·SHA-256이 아카이브·매니페스트와 일치하는지 확인 → 원 님에게 권한(`media_hosts`, `browser`) 승인 요청 → 설치 폴더(`OsPolicy` 경로)에 풀기.
2. 시작(`discover`): 각 플러그인의 `plugin.json.minisig`를 다시 검증하고, 실행 파일 SHA-256이 서명된 매니페스트의 `entry_sha256`과 같은지 확인한다. 실패하면 비활성화하고 이유를 `kinekino plugins list`와 설정 페이지에 표시한다.
3. 검증 실패는 항상 차단이다. "서명 없음 허용"은 개발 모드 설정 `plugins.allow_unsigned_dev`(기본 꺼짐, 로컬 경로 플러그인에만 적용)로만 두고, 이 설정은 설정 페이지에 경고와 함께 노출한다.

## 키·시크릿 절차(승인 후에만 실행)

1. 오프라인 머신에서 `minisign -G -p kinekino-plugins.pub -s kinekino-plugins.key` (비밀번호 설정).
2. 비밀키는 원 님 보관(패스워드 매니저·오프라인 백업). 저장소에 커밋 금지.
3. CI 서명이 필요하면 비공개 플러그인 저장소의 승인된 environment에만 `MINISIGN_SECRET_KEY`, `MINISIGN_PASSWORD` 시크릿을 둔다(ADR 0010 규칙). 공개 kinekino 저장소에는 시크릿을 두지 않는다.
4. 공개키는 공개 문서와 기본 설정에 넣고, 사용자 설정 `plugins.trusted_keys`에 추가 키를 등록할 수 있게 한다.
5. 키 교체: 새 키를 trusted_keys에 추가한 릴리스를 먼저 배포 → 새 키로 서명 → 옛 키 제거. 유출 시 즉시 옛 키를 제거한 코어 릴리스를 낸다.

## 구현 단계(제안)

| 단계 | 내용 | 완료 기준 |
|---|---|---|
| G1 | 매니페스트에 `signature{key_id}`, `entry_sha256` 필드(SDK S2와 함께) | 골든 fixture 갱신 |
| G2 | `minisign-verify` 어댑터를 source-host infrastructure에 추가, 테스트 전용 키쌍은 테스트 fixture로 생성해 커밋(제품 키 아님) | 위조·변조·다른 키·다른 버전 거부 테스트 |
| G3 | 설치·시작 검증, 실패 표시 | `kinekino plugins list` 실제 실행 |
| G4 | 비공개 저장소 CI 서명(시크릿 승인 후) | 서명된 릴리스를 코어가 설치 |

## 위협·한계

- 서명은 출처 확인이지 샌드박스가 아니다. 서명된 플러그인도 선언한 권한 안에서 네트워크를 쓴다(OS 샌드박스는 ADR 0011 후속).
- 비밀키 유출은 공개키 제거 릴리스로만 대응된다(폐기 목록 없음).
