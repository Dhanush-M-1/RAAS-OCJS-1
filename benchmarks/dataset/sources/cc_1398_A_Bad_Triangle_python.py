t=int(input())
for z in range(t):
    n=int(input())
    arr=list(map(int,input().split()))
    min= arr[0] + arr[1]
    pos=True
    for i in range(2,n):
        if(arr[i]>=min):
            pos=False
            posf=i
            break
    if(pos):
        print("-1")
    else:
        print("1 2", posf+1)
