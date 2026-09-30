# -*- coding:utf-8 -*-

"""

created by shuangquan.huang at 2020/7/1

"""

import collections
import time
import os
import sys
import bisect
import heapq
from typing import List



def solve(n, l, r):
    # 1, 2, 1, 3, ..., 1, n
    # 2, 3, 2, 4, ..., 2, n
    # ...
    # n-1, n
    # 1
    
    lo, hi = 1, n
    while lo <= hi:
        k = (lo + hi) // 2
        s = k * (2*n-1-k)
        if s < l:
            lo = k + 1
        else:
            hi = k - 1
    
    k = lo
    s = k * (2*n-1-k)
    b = k
    
    
    # [b, b+1, b, b+2, ..., b, n]
    row = []
    for i in range(b+1, n+1):
        row.append(b)
        row.append(i)
    ans = row[l-s-1:]
    d = r-l+1
    if len(ans) >= d:
        return ans[:d]
    
    while len(ans) < d:
        b += 1
        row = []
        for i in range(b + 1, n + 1):
            row.append(b)
            row.append(i)
        if not row:
            break
        ans += row
    
    ans.append(1)
    # print(ans[:d])
    return ans[:d]
    

if __name__ == '__main__':
    T = int(input())
    ans = []
    for ti in range(T):
        N, L, R = map(int, input().split())
        ans.append(solve(N, L, R))
    print('\n'.join([' '.join(map(str, v)) for v in ans]))