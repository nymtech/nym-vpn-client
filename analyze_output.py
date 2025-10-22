import re
from collections import Counter

# Path to your log file
log_file = "testoutput.txt"

# Pattern for both formats:
# 1) average_delay = 42000000.0000
# 2) average_delay 42ns
pattern = re.compile(r'average_delay(?:\s*=\s*([\d\.]+)|\s+(\d+)ns)')

delays = []

with open(log_file, 'r') as f:
    for line in f:
        match = pattern.search(line)
        if match:
            # Use the numeric value if '=' version, otherwise the ns one
            delay = match.group(1) or match.group(2)
            delays.append(delay)

# Count occurrences
counts = Counter(delays)

# Print results sorted by delay value
for delay, count in sorted(counts.items(), key=lambda x: float(x[0])):
    print(f"{delay} appeared {count} times")

