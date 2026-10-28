def square(n):
    return n*n
values = range(1,10001)
squares = [square(n) for n in values]
total = 0
for n in squares:
    total += n
print(total)
