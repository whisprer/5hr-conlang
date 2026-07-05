def analyse(values):
    total = 0
    for value in values:
        if value:
            total += 1
    return total / max(len(values), 1)
