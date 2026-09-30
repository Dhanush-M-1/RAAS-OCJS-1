from sys import stdin
def main():
    n = int(stdin.readline())
    a = list(map(int, stdin.readline().split()))
    inf = [0] * (n + 1)
    curr = 0
    d = 0
    for i in range(n):
        curr += abs(i + 1 - a[i])
        if a[i] > i + 1:
            d += 1
            inf[a[i] - i - 1] += 1
        elif a[i] <= i + 1:
            d -= 1
            if a[i] == i + 1:
                inf[0] += 1
            else:
                inf[a[i] + n - i - 1] += 1
    best = curr
    num = 0
    for i in range(n):
        curr -= d
        curr -= 1
        curr = curr - abs(a[n - i - 1] - n) + abs(a[n - i - 1] - 1)
        d += 2
        d -= inf[i + 1] * 2
        if curr < best:
            best = curr
            num = i + 1
    print(best, num)
    
main()