s=input()
n=len(s)
l=list("0987654321")
cnt={}
for i in range(n-9):
  t=s[i:i+10]
  if t[0] in l and t[1] in l and t[2]=="-" and t[3] in l and t[4] in l and t[5]=="-" and t[6] in l and t[7] in l and t[8] in l and t[9] in l:
    if 2013<=int(t[6:11])<=2015  and 1<=int(t[3:5])<=12:
      if int(t[3:5]) in [1,3,5,7,8,10,12] and 1<=int(t[0:2])<=31:
        if not t in cnt:
          cnt[t]=1
        else:
          cnt[t]+=1
      elif int(t[3:5]) in [4,6,9,11] and 1<=int(t[0:2])<=30:
        if not t in cnt:
          cnt[t]=1
        else:
          cnt[t]+=1
      elif int(t[3:5])==2 and 1<=int(t[0:2])<=28:
        if not t in cnt:
          cnt[t]=1
        else:
          cnt[t]+=1
print(max(cnt,key=cnt.get))