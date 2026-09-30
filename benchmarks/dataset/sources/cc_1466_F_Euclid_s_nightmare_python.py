import sys
input = sys.stdin.buffer.readline

def _find(s, u):
    p = []
    while s[u] != u:
        p.append(u)
        u = s[u]
    for v in p: s[v] = u
    return u

def _union(s, u, v):
    su, sv = _find(s, u), _find(s, v)
    if su != sv: s[su] = sv
    return su != sv

n, m = map(int, input().split())
s, solo = list(range(m+1)), [0]*(m+1)
res, pos = [], set()
for i in range(n):
    p = list(map(int, input().split()))
    if p[0] == 1:
        pos.add(p[1])
        p1 = _find(s, p[1])
        if not solo[p1]:
            res.append(i+1)
            solo[p1] = 1
    else:
        pos.add(p[1])
        pos.add(p[2])
        p1, p2 = _find(s, p[1]), _find(s, p[2])
        if not (p1 == p2 or (solo[p1] and solo[p2])):
            _union(s, p1, p2)
            res.append(i+1)
            if solo[p1] or solo[p2]:
                solo[_find(s, p1)] = 1

cc = 0
for u in pos:
    su = _find(s, u)
    cc += 1
    if not solo[su] and su == u:
        cc -= 1

print(pow(2, cc, 10**9+7), len(res))
print(*res)
        


