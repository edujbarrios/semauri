def square(n):
    return n * n
numbers = range(1, 10001)
squares = list(map(square, numbers))
total = 0
for n in squares:
    total += n
print(total)
