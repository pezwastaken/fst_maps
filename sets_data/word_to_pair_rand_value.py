#!/bin/python3

import random
import string

# Open the input text file and read each line
with open('3000.txt', 'r') as file:
    lines = file.readlines()

# Open the output file to write the modified content
with open('output.txt', 'w') as file:
    for line in lines:
        word = line.strip()
        if word:  # Only process non-empty lines
            random_str = ''.join(random.choices(
                string.ascii_lowercase, k=10))

            # Format the line as desired
            modified_line = f'/{word}/{random_str}\n'
            file.write(modified_line)

print("File processing complete. Check 'output.txt' for results.")
