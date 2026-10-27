words = ["semauri"] * 10000
uppercase_words = [s.upper() for s in words]
matches = sum(1 for s in uppercase_words if "MAU" in s)
print(matches)
