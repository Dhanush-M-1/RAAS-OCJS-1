import sys
input=sys.stdin.readline
f=lambda :list(map(int, input().strip('\n').split()))

n, k=f()
_11=[]
_01=[]
_10=[]
for _ in range(n):
	t, a, b=f()
	if a and b:
		_11.append(t)
	elif a:
		_10.append(t)
	elif b:
		_01.append(t)
_01.sort(); _10.sort(); _11.sort()
for i in range(1, len(_01)):
	_01[i]+=_01[i-1]
for i in range(1, len(_10)):
	_10[i]+=_10[i-1]
for i in range(1, len(_11)):
	_11[i]+=_11[i-1]
ans=3*1e9
if len(_01)>=k and len(_10)>=k:
	ans=min(ans, _01[k-1]+_10[k-1])
for i in range(len(_11)):
	if i+1<k and (len(_01)>=k-i-1) and (len(_10)>=k-i-1):
		ans=min(ans, _11[i]+_01[k-i-2]+_10[k-i-2])
	else:
		if len(_11)>=k:
			ans=min(ans, _11[k-1])
			break
print(-1 if ans==3*1e9 else ans)