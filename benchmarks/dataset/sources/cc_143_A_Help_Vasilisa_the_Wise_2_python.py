inn = list(map(int, input().split(" ")))
r1 = inn[0]
r2 = inn[1]
inn = list(map(int, input().split(" ")))
c1 = inn[0]
c2 = inn[1]
inn = list(map(int, input().split(" ")))
d1 = inn[0]
d2 = inn[1]

x = int((d1+c1-r2)/2)
y = int(((2*r1)-d1-c1+r2)/2)
a = int(((2*c1)-d1-c1+r2)/2)
b = int((r2-(2*c1)+d1+c1)/2)
if x == y or x == a or x == b or y == a or y == b or a == b or x < 1 or x > 9 or y < 1 or y > 9 or a < 1 or a > 9 or b < 1 or b > 9:
    print(-1)
else:
    print(x, y)
    print(a, b)