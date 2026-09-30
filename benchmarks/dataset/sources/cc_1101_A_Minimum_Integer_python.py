n = int(input())
A = []
for i in range(n):
    A = A+[input().split()]
for a in A:
    if int(a[2]) < int(a[0]) or int(a[2]) > int(a[1]):
        print(a[2])
    else:
        print(int(a[2])*(int(a[1])//int(a[2])+1))
