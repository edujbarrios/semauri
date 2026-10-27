numbers = range(1, 10001)
selected = sum(1 for n in numbers if n % 3 == 0 or n % 5 == 0)
print(selected)
