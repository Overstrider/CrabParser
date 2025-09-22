# CrabParser 🦀

> High-performance text parsing library written in Rust with Python bindings

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3670A0?style=for-the-badge&logo=python&logoColor=ffdd54)](https://www.python.org/)

## 🎯 What is CrabParser?

CrabParser is a blazingly fast text parsing library that splits documents and code files into semantic chunks. Built with Rust for maximum performance and memory efficiency, it provides Python bindings for easy integration into your projects.

### Key Features

- 🚀 **Pure Rust Performance** - 10x faster than pure Python implementations
- 📄 **Multi-Format Support** - Handles TXT, PDF, DOCX, CSV, and 12+ programming languages
- 🛡️ **Bulletproof Encoding** - Never fails on any file, handles all encodings gracefully
- 💾 **Memory Efficient** - ChunkedText keeps data in Rust memory with Python access
- ⚡ **Parallel Processing** - Leverages Rayon for concurrent operations
- 🧩 **Semantic Chunking** - Respects document structure (paragraphs, sentences, code blocks)

## 📦 Installation

### From PyPI (Coming Soon)

```bash
pip install crabparser
```

### From Source

```bash
# Clone the repository
git clone https://github.com/Overstrider/crabparser.git
cd crabparser

# Create virtual environment
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install maturin (build tool)
pip install maturin

# Build and install
cd crabparser
maturin develop --release
cd ..
```

## 🚀 Quick Start

```python
from crabparser import TextParser, ChunkedText

# Create a parser instance
parser = TextParser(
    chunk_size=1000,          # Maximum characters per chunk
    respect_paragraphs=True,  # Keep paragraphs together
    respect_sentences=True    # Split at sentence boundaries
)

# Parse text
text = "Your long document text here..."
chunks = parser.parse(text)
print(f"Split into {len(chunks)} chunks")

# Parse with memory-efficient ChunkedText
chunked = parser.parse_chunked(text)
print(f"First chunk: {chunked[0]}")
print(f"Total size: {chunked.total_size} bytes")

# Parse files directly
chunks = parser.parse_file("document.pdf")

# Save chunks to files
parser.save_chunks(chunks, "output_dir", "document")
```

## 📚 Supported Formats

### Documents
- **PDF** - Extracts and preserves text content
- **DOCX** - Full support for Word documents
- **CSV** - Intelligent handling of structured data
- **TXT** - Universal text files with encoding detection

### Programming Languages
Semantic code parsing that respects function and class boundaries:

- Python, JavaScript, TypeScript, Rust
- Go, Java, C#, C++
- Ruby, PHP, Swift, Kotlin

## 🔧 API Reference

### TextParser

The main parser class for processing text and files.

```python
parser = TextParser(
    chunk_size=1000,        # Maximum size of each chunk
    respect_paragraphs=True,  # Maintain paragraph boundaries
    respect_sentences=True    # Maintain sentence boundaries
)
```

**Methods:**
- `parse(text: str) -> List[str]` - Parse text into chunks
- `parse_chunked(text: str) -> ChunkedText` - Memory-efficient parsing
- `parse_file(path: str) -> List[str]` - Parse any supported file
- `parse_file_chunked(path: str) -> ChunkedText` - Memory-efficient file parsing
- `save_chunks(chunks, output_dir, base_name) -> int` - Save chunks to files

### ChunkedText

Memory-efficient container that keeps chunks in Rust memory.

```python
# Access chunks without loading all into Python memory
chunked[0]            # First chunk
chunked[-1]           # Last chunk
len(chunked)          # Number of chunks
chunked.total_size    # Total size in bytes
chunked.source_file   # Source file path (if applicable)

# Iteration
for chunk in chunked:
    process(chunk)

# Get slice of chunks
batch = chunked.get_slice(0, 10)  # Get first 10 chunks
```

## 🤝 Contributing

We love contributions! CrabParser is an open-source project and welcomes contributors of all skill levels.

### Ways to Contribute

1. **Report Bugs** - Found something broken? [Open an issue](https://github.com/Overstrider/crabparser/issues)
2. **Suggest Features** - Have an idea? We'd love to hear it!
3. **Submit Pull Requests** - Ready to code? Check out our [good first issues](https://github.com/Overstrider/crabparser/labels/good%20first%20issue)
4. **Improve Documentation** - Help others by enhancing our docs
5. **Add Language Support** - Know a programming language we don't support yet?

### Development Setup

```bash
# Fork and clone the repository
git clone https://github.com/Overstrider/crabparser.git
cd crabparser

# Create a new branch
git checkout -b feature/your-feature-name

# Set up development environment
python3 -m venv venv
source venv/bin/activate
pip install maturin

# Build in development mode
cd crabparser
maturin develop
cd ..

# Run examples
python main.py
```

### Pull Request Process

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to your branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

### Code Style

- **Rust**: Follow standard Rust conventions (use `cargo fmt` and `cargo clippy`)
- **Python**: Follow PEP 8 guidelines
- **Comments**: Write clear, concise comments for complex logic

### Adding New File Format Support

To add support for a new file format:

1. Create a new module in `crabparser/src/` (e.g., `markdown.rs`)
2. Implement the parsing logic
3. Add the module to `crabparser/src/lib.rs`
4. Update the file extension detection in `parse_file()`
5. Add examples

Example structure for a new parser:

```rust
// src/markdown.rs
use pyo3::prelude::*;

pub fn parse_markdown(file_path: &str) -> PyResult<String> {
    // Your parsing logic here
}
```

## 📊 Performance

CrabParser is designed for speed and efficiency:

- **10x faster** than pure Python text processing
- **Parallel chunk processing** using Rayon
- **Zero-copy operations** where possible
- **Memory-efficient** chunk streaming

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [PyO3](https://github.com/PyO3/pyo3) for Rust-Python bindings
- Uses [Rayon](https://github.com/rayon-rs/rayon) for parallelization
- PDF parsing powered by [pdf-extract](https://github.com/jrmuizel/pdf-extract)

## 🌟 Support

If you find CrabParser useful, please consider:
- ⭐ Starring the repository
- 🐛 Reporting bugs
- 💡 Suggesting new features
- 📖 Improving documentation
- 💻 Contributing code

## 📮 Contact

- **Issues**: [GitHub Issues](https://github.com/Overstrider/crabparser/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Overstrider/crabparser/discussions)

---

<p align="center">
  Made with 🦀 and ❤️ by the open-source community
</p>