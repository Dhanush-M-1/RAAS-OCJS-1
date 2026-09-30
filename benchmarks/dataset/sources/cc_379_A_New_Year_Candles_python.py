a, b = map(int, input().split())
c, s = a, 0
while a >= b:
    s += a // b
    a = (a // b) + (a % b)
print(s + c)
