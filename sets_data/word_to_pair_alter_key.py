#!/bin/python3

import random
import string

# Open the input text file and read each line
with open('alphanumeric_words.txt', 'r') as file:
    lines = file.readlines()

# Open the output file to write the modified content
with open('output.txt', 'w') as file:
    for line in lines:
        word = line.strip()
        if word:  # Only process non-empty lines

            random_str = ''.join(random.choices(
                string.ascii_lowercase, k=4))

            value = ""
            if random.random() < 0.1 or len(word) < 4:
                value = word + random_str
            else:
                # Replace 3 adjacent characters at a random position
                if len(word) >= 4:
                    # Randomly pick a start index for replacement
                    start_index = random.randint(0, len(word) - 4)
                    value = word[:start_index] + \
                        random_str + word[start_index + 4:]

                    # Format the line as desired
                    modified_line = f'/{word}/{value}\n'

                    file.write(modified_line)

print("File processing complete. Check 'output.txt' for results.")
