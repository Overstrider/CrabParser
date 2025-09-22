# CrabParser

A high-performance text parsing library written in Rust with Python bindings.

## Features

- Fast semantic text chunking
- Respects paragraph and sentence boundaries
- Configurable chunk sizes
- Written in Rust for optimal performance
- Easy-to-use Python API

## Installation

```bash
pip install crabparser
```

## Usage

```python
from crabparser import TextParser

# Create parser with custom settings
parser = TextParser(
    chunk_size=500,
    respect_paragraphs=True,
    respect_sentences=True
)

# Parse text
text = "Your long text here..."
chunks = parser.parse(text)

# Parse file
chunks = parser.parse_file("document.txt")

# Save chunks to files
parser.save_chunks(chunks, "output_dir", "base_name")
```