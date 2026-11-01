values = ["semauri"]*10000
upper = [value.upper() for value in values]
found = sum(1 for value in upper if "SEMAURI" in value)
print(found)
