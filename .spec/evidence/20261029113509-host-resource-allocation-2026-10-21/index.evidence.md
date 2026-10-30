---
id: evidence:kinekino/20261029113509
title: AI 재개 전 호스트 순간 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# AI 재개 전 호스트 순간 관측

2026-10-21 08:17:29 KST. 읽기 전용 Win32_Processor/OperatingSystem/PerfOS_Processor·Get-Volume/Get-Process 및 nvidia-smi 질의다. 순간 관측이며 지속 여유·모델 peak·GPU 추론 호환성을 입증하지 않는다. 사용자 앱·서비스·WSL·VM 설정을 변경하지 않았다.

| 항목 | 관측 |
|---|---|
| CPU | Intel Core i9-9900KF, 8물리/16논리, 전체사용률10% |
| RAM | 총63.93GiB, 가용27.27GiB |
| GPU | NVIDIA RTX2070 SUPER, 총8192MiB, 사용2445MiB, 가용5747MiB |
| GPU 순간사용률 | GPU25%, memory utilization18% |
| 저장소 | C:가용17.65GiB, D:가용190.60GiB |

상위 working set 관측에는 Memory Compression8.15GiB, vmmemWSL4.07GiB, codex2.40GiB, ChatGPT1.58/0.79GiB 등이 있었다. working set·압축 메모리와 앱별 private bytes를 혼동하지 않는다. CPU/GPU 여유가 UI 반응성을 보장하지 않으며 이번 관측으로 과거 지연 원인을 재판정하지 않는다.

실행 배분은 [AI 자원 계획](../../../plans/player/ai/resource-allocation-2026-10-21.md)을 따른다. 이전 AI 실측은 [모델 상태](../ai/model-status.md)의 CPU 경로이며 GPU 품질/성능으로 승계하지 않는다.

## SDK 정상 media 게이트 직전 재관측

`runtime/verification/native-export-actual-media-r3/resources-before.json`에는 가용 RAM22.638GiB·GPU5139/8192MiB·C188.172GiB가 기록됐다. SDK 보고의 직전 RAM21.53GiB/VRAM5119MiB와 측정 시점이 다르다. PM은 C 여유 증가를 재확인했고 `Get-Volume -DriveLetter C,D`에서 C204036599808B·D204509958144B를 관측했다. 정리 원인은 확인하지 않았으며 PM이 삭제·캐시 이동을 수행한 결과가 아니다. 08:17의 C17.65GiB는 과거 관측으로 보존한다.

같은 게이트 `build-cli.receipt.json`은 conservative2/affinity3/BelowNormal/원 child8116·launcher49848의 terminal=true/exit0/160490ms를 기록한다. 정상 CLI 빌드 완료 근거이며 실제 재생 성공·자식 전체 회수 근거로 확대하지 않는다.
