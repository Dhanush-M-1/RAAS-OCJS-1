num,wiz,per = map(int,input().split())
k = 0
while (k+wiz)/num*100 < per:
    k += 1
print(k)