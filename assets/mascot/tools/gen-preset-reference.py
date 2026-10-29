"""Regenerate the body of .spec/guide/mascot-preset-reference.guide.md (front matter kept) from parts.json / layers.json / state-machine.json (JSON = single source of truth)."""
import json
R='src/presets/'
P=json.load(open(R+'parts.json'))['clips']; L=json.load(open(R+'layers.json')); S=json.load(open(R+'state-machine.json'))
o=['## Overview','','자동 생성: `cd assets/mascot && python3 tools/gen-preset-reference.py`. 원본은 JSON입니다. 이 문서를 고치지 말고 JSON을 고친 뒤 다시 생성하세요.','','## Steps','']
o+=['### 파트 클립','','| 클립 | 레이어 | 길이(s) | 루프 | holdAt | 배치 | 메모 |','|---|---|---|---|---|---|---|']
for n,c in P.items():
    m=c.get('mirror'); src=P[m] if m else c
    o.append(f"| `{n}` | {src['layer'].replace('_L','_R') if m else src['layer']} | {src['duration']} | {'예' if src.get('loop') else ''} | {src.get('holdAt') or ''} | {src.get('batch','v6')} | {('미러: `'+m+'`') if m else (src.get('doc') or '')}{(' · restExempt='+str(src['restExempt'])) if src.get('restExempt') else ''} |")
o+=['','### 조합 프리셋','','| 조합 | 그룹 | 의미 | 길이(s) | 루프 | reveal | blend | 트랙 (레이어 클립 @시작) |','|---|---|---|---|---|---|---|---|']
for n,c in L['combos'].items():
    tr='<br>'.join(f"{t[0]} `{t[1]}` @{t[2]}"+(f" {json.dumps(t[3],ensure_ascii=False)}" if len(t)>3 else '') for t in c['tracks'])
    o.append(f"| `{n}` | {c.get('group','v6')} | {c.get('meaning','')}{(' — '+c['note']) if c.get('note') else ''} | {c['duration']} | {'예' if c.get('loop') else ''} | {c.get('reveal','full')} | {c.get('blend','')} | {tr} |")
o+=['','### 시선(look) 프리셋','','| 이름 | 모드 | 옵션 |','|---|---|---|']
for n,c in L['looks'].items():
    if n.startswith('_'): continue
    o.append(f"| `{n}` | {c['mode']} | {json.dumps({k:v for k,v in c.items() if k not in ('mode','spec','batch')},ensure_ascii=False)} |")
o+=['','### 앱 이벤트 → 프리셋 (state-machine.json)','','| 이벤트 | 조합 | 단계 | 규칙 |','|---|---|---|---|']
for n,e in S['events'].items():
    o.append(f"| `{n}` | {e.get('combo','')} | {e.get('tier','')} | {json.dumps({k:v for k,v in e.items() if k not in ('combo','tier')},ensure_ascii=False)} |")
o+=['',f"idle 가중치: {json.dumps(S['idle']['weights'],ensure_ascii=False)} · 쿨다운 {json.dumps(S['idle']['cooldown'],ensure_ascii=False)} · 직전 {S['idle']['noRepeatLast']}개 제외 · 시작 {S['idle']['startAfter']}s · 깊은 idle {S['idle']['deepAfter']}s",'']
D='../../.spec/guide/mascot-preset-reference.guide.md'; h=open(D).read(); h=h[:h.index('\n## ')+1]
open(D,'w').write(h+'\n'.join(o))
print(len(o),'lines')
