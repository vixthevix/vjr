import math

print("{")

a = 8
b = int(4096 / 8)
k = 0

for i in range(0, b):
    print("    ", end="")
    for j in range(0, a):
        angle = (2 * math.pi * k) / 4096
        x = int(math.sin(angle) * 4096)
        print(f"{x},".ljust(8, " "), end="")
        k += 1
    print("")
        

#for i in range(0, 4096):
#    angle = (2 * math.pi * i) / 4096
#    x = int(math.sin(angle) * 4096)
#    print(f"    {x},")

print("}")
