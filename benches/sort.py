numbers = range(1, 10001)
backwards = list(reversed(list(numbers)))
ordered = sorted(backwards)
print((ordered[4999] + ordered[5000]) / 2)
