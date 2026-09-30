# class SegmentTree(): # adapted from https://www.geeksforgeeks.org/segment-tree-efficient-implementation/
#     def __init__(self,arr,func,initialRes=0):
#         self.f=func
#         self.N=len(arr) 
#         self.tree=[0 for _ in range(2*self.N)]
#         self.initialRes=initialRes
#         for i in range(self.N):
#             self.tree[self.N+i]=arr[i]
#         for i in range(self.N-1,0,-1):
#             self.tree[i]=self.f(self.tree[i<<1],self.tree[i<<1|1])
#     def updateTreeNode(self,idx,value): #update value at arr[idx]
#         self.tree[idx+self.N]=value
#         idx+=self.N
#         i=idx
#         while i>1:
#             self.tree[i>>1]=self.f(self.tree[i],self.tree[i^1])
#             i>>=1
#     def query(self,l,r): #get sum (or whatever function) on interval [l,r] inclusive
#         r+=1
#         res=self.initialRes
#         l+=self.N
#         r+=self.N
#         while l<r:
#             if l&1:
#                 res=self.f(res,self.tree[l])
#                 l+=1
#             if r&1:
#                 r-=1
#                 res=self.f(res,self.tree[r])
#             l>>=1
#             r>>=1
#         return res
# def getMaxSegTree(arr):
#     return SegmentTree(arr,lambda a,b:max(a,b),initialRes=-float('inf'))
# def getMinSegTree(arr):
#     return SegmentTree(arr,lambda a,b:min(a,b),initialRes=float('inf'))
# def getSumSegTree(arr):
#     return SegmentTree(arr,lambda a,b:a+b,initialRes=0)
from collections import Counter
def main():
    
    # mlogn solution
    n=int(input())
    a=readIntArr()
    
    b=sorted(a,reverse=True)
    
    m=int(input())
    allans=[]
    for _ in range(m):
        k,pos=readIntArr()
        cnt=Counter(b[:k])
        totalCnts=0
        for x in a:
            if cnt[x]>0:
                cnt[x]-=1
                totalCnts+=1
                if totalCnts==pos:
                    allans.append(x)
                    break
    multiLineArrayPrint(allans)
    
    return

import sys
input=sys.stdin.buffer.readline #FOR READING PURE INTEGER INPUTS (space separation ok)
# input=lambda: sys.stdin.readline().rstrip("\r\n") #FOR READING STRING/TEXT INPUTS.
 
def oneLineArrayPrint(arr):
    print(' '.join([str(x) for x in arr]))
def multiLineArrayPrint(arr):
    print('\n'.join([str(x) for x in arr]))
def multiLineArrayOfArraysPrint(arr):
    print('\n'.join([' '.join([str(x) for x in y]) for y in arr]))
 
def readIntArr():
    return [int(x) for x in input().split()]
# def readFloatArr():
#     return [float(x) for x in input().split()]

def makeArr(defaultValFactory,dimensionArr): # eg. makeArr(lambda:0,[n,m])
    dv=defaultValFactory;da=dimensionArr
    if len(da)==1:return [dv() for _ in range(da[0])]
    else:return [makeArr(dv,da[1:]) for _ in range(da[0])]
 
def queryInteractive(i,j):
    print('? {} {}'.format(i,j))
    sys.stdout.flush()
    return int(input())
 
def answerInteractive(ans):
    print('! {}'.format(' '.join([str(x) for x in ans])))
    sys.stdout.flush()
 
inf=float('inf')
MOD=10**9+7
# MOD=998244353
 
 
for _abc in range(1):
    main()