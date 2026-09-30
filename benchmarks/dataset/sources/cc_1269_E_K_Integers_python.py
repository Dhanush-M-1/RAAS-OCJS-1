n = int(input())
a = [0] + list(map(int, input().split()))

pos, pb, ps = [[0] * (n + 1) for x in range(3)]


def add(bit, i, val):
    while i <= n:
        bit[i] += val
        i += i & -i


def sum(bit, i):
    res = 0
    while i > 0:
        res += bit[i]
        i -= i & -i
    return res


def find(bit, sum):
    i, t = 0, 0
    if sum == 0:
        return 0
    for k in range(17, -1, -1):
        i += 1 << k
        if i <= n and t + bit[i] < sum:
            t += bit[i]
        else:
            i -= 1 << k
    return i + 1


for i in range(1, n + 1):
    pos[a[i]] = i

invSum = 0
totalSum = 0
for i in range(1, n + 1):
    totalSum += pos[i]
    invSum += i - sum(pb, pos[i]) - 1
    add(pb, pos[i], 1)
    add(ps, pos[i], pos[i])
    mid = find(pb, i // 2)
    if i % 2 == 1:
        mid2 = find(pb, i // 2 + 1)
        seqSum = (i + 1) * (i // 2) // 2
    else:
        mid2 = mid
        seqSum = i * (i // 2) // 2
    leftSum = sum(ps, mid)
    rightSum = totalSum - sum(ps, mid2)
    print(rightSum - leftSum - seqSum + invSum, end=" ")
