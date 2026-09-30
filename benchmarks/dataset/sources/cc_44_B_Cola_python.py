def nik(rudy,x,y,z,cot):
    
    for i in range(z+1):
        for j in range(y+1):
            t = rudy - i*2 -j
            if t>=0 and x*0.5 >= t:
                cot+=1
    return cot
rudy, x, y, z = list(map(int,input().split()))
cot = 0
print(nik(rudy,x,y,z,cot))

