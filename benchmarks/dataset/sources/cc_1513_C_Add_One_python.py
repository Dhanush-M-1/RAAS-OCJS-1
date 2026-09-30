from os import path
import sys,time
from math import ceil, floor,gcd,log,log2 ,factorial
from collections import defaultdict ,Counter , OrderedDict , deque
from heapq import heapify , heappush , heappop
from bisect import *
# from functools import reduce
from operator import mul
from itertools import permutations
maxx, mod  = float('inf') , int(1e9 + 7)
localsys ,start_time = 0 , time.time()
if (path.exists('input.txt')):
    localsys = 1;sys.stdin=open('input.txt','r');sys.stdout=open('output.txt','w');
#left shift --- num*(2**k) --(k - shift)
input = sys.stdin.readline
N = int(2e5 + 10)
dp =[1]*N
for i in range(10 , N):
    dp[i] = (dp[i-9] + dp[i-10])%mod
for _ in range(int(input())):
    n , m = map(int , input().split()) ; ans =0
    while n :
        i = n%10 ; n//=10
        ans = (ans + dp[i + m])%mod
    print(ans)



    






if localsys:
    print("\n\n\nTime Elased :",time.time() - start_time,"seconds")