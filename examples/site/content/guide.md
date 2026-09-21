---
title: A short guide
date: 2026-09-18
tags: guide
---

# A short guide

Lume reads like Python and runs like Rust. Start with `def main:`.

## Values

Everything is a value: `1`, `"text"`, `[1, 2]`, `{"a": 1}`, `{1, 2}`.
A value that may be absent has type `T?`, and one that may fail is
`T or Error`.

## Blocks

    xs.map { |x| x * 2 }
    xs.each do |x|
      puts x

That is the whole of it. See the [front page](index.html).
