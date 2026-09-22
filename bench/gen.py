#!/usr/bin/env python3
"""Generates the benchmark inputs, so nothing large is checked in."""
import random, sys, os
random.seed(7)
here = os.path.dirname(os.path.abspath(__file__))
words = [f"w{i:04d}" for i in range(2000)]
with open(os.path.join(here, "words.txt"), "w") as f:
    for _ in range(1000000):
        f.write(random.choice(words))
        f.write("\n" if random.random() < 0.05 else " ")
print("words.txt", os.path.getsize(os.path.join(here, "words.txt")), "bytes")
