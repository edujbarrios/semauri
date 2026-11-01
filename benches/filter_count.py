values = range(1,10000)
selected = [n for n in values if n%3==0 or n%5==0]
print(sum(1 for n in selected if True))
