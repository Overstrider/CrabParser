#!/usr/bin/env python3
"""
CrabParser Examples - Demonstrating all features of the library
No external dependencies except crabparser itself
"""

from crabparser import TextParser, ChunkedText
import os
import time

print("✓ CrabParser - High-performance text parser powered by Rust")

def print_section(title):
    """Print a formatted section header"""
    print("\n" + "=" * 60)
    print(f" {title}")
    print("=" * 60)

def example_1_basic_text_parsing():
    """Example 1: Basic text parsing"""
    print_section("Example 1: Basic Text Parsing")

    # Create sample text
    text = """
    This is the first paragraph of our document. It contains some sample text
    that will be parsed into chunks. The parser respects paragraph boundaries
    by default, which means it tries to keep paragraphs together when possible.

    This is the second paragraph. It's a bit longer and contains more information.
    When the chunk size is exceeded, the parser will intelligently split the text
    while trying to maintain semantic boundaries. This ensures that the chunks
    remain meaningful and complete.

    The third paragraph demonstrates how the parser handles various text structures.
    It can respect sentences, paragraphs, and maintain readability. This is especially
    useful when processing documents for search engines or AI models that need
    coherent text segments.
    """

    # Create parser with different settings
    parser = TextParser(chunk_size=200, respect_paragraphs=True)

    # Parse text
    chunks = parser.parse(text)

    print(f"Text length: {len(text)} characters")
    print(f"Number of chunks: {len(chunks)}")
    print("\nChunks:")
    for i, chunk in enumerate(chunks, 1):
        print(f"\n--- Chunk {i} ({len(chunk)} chars) ---")
        print(chunk[:100] + "..." if len(chunk) > 100 else chunk)

def example_2_memory_efficient_parsing():
    """Example 2: Memory-efficient parsing with ChunkedText"""
    print_section("Example 2: Memory-Efficient Parsing (ChunkedText)")


    # Create a large text
    large_text = ("This is a sample paragraph. " * 50 + "\n\n") * 20

    parser = TextParser(chunk_size=500)

    # Parse into ChunkedText (memory stays in Rust)
    chunked = parser.parse_chunked(large_text)

    print(f"Total chunks: {len(chunked)}")
    print(f"Total size: {chunked.total_size} bytes")

    # Access individual chunks without loading all
    print("\n1. Accessing specific chunks:")
    print(f"   First chunk: {len(chunked[0])} chars")
    print(f"   Middle chunk [10]: {len(chunked[10])} chars")
    print(f"   Last chunk: {len(chunked[-1])} chars")

    # Iterate through chunks
    print("\n2. Iterating through chunks:")
    total_processed = 0
    for i, chunk in enumerate(chunked):
        total_processed += len(chunk)
        if i < 3:  # Show first 3
            print(f"   Chunk {i}: {len(chunk)} chars")
    print(f"   ... processed all {len(chunked)} chunks")
    print(f"   Total characters: {total_processed}")

    # Get a slice
    print("\n3. Getting a slice of chunks:")
    batch = chunked.get_slice(5, 10)
    print(f"   Got chunks 5-9: {len(batch)} chunks")
    print(f"   Total size of batch: {sum(len(c) for c in batch)} chars")

def example_3_file_parsing():
    """Example 3: Parse various file types"""
    print_section("Example 3: File Parsing")

    # Create test files
    test_files = []

    # Create a text file
    with open("example.txt", "w") as f:
        f.write("This is a sample text file.\n\n")
        f.write("It has multiple paragraphs.\n\n")
        f.write("Each paragraph will be respected during parsing.")
    test_files.append("example.txt")

    # Create a Python file
    with open("example.py", "w") as f:
        f.write("""
import os

class DataProcessor:
    def __init__(self):
        self.data = []

    def process(self):
        for item in self.data:
            print(item)

def main():
    processor = DataProcessor()
    processor.process()
""")
    test_files.append("example.py")

    # Create a JavaScript file
    with open("example.js", "w") as f:
        f.write("""
function calculateSum(a, b) {
    return a + b;
}

class Calculator {
    constructor() {
        this.result = 0;
    }

    add(value) {
        this.result += value;
        return this;
    }
}

const calc = new Calculator();
calc.add(5).add(10);
""")
    test_files.append("example.js")

    # Parse each file
    parser = TextParser(chunk_size=500)

    for file_path in test_files:
        print(f"\nParsing: {file_path}")
        chunks = parser.parse_file(file_path)
        print(f"  → {len(chunks)} chunks")

        # Show first chunk preview
        if chunks:
            preview = chunks[0][:80].replace('\n', ' ')
            print(f"  → Preview: {preview}...")

    # Clean up
    for file_path in test_files:
        os.remove(file_path)

def example_4_code_aware_parsing():
    """Example 4: Code-aware parsing respects functions and classes"""
    print_section("Example 4: Code-Aware Parsing")

    # Create a Python file with multiple functions
    code = """
def fibonacci(n):
    '''Calculate fibonacci number'''
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

def factorial(n):
    '''Calculate factorial'''
    if n <= 1:
        return 1
    return n * factorial(n-1)

class MathOperations:
    def __init__(self):
        self.history = []

    def add(self, a, b):
        result = a + b
        self.history.append(f"{a} + {b} = {result}")
        return result

    def multiply(self, a, b):
        result = a * b
        self.history.append(f"{a} * {b} = {result}")
        return result

def main():
    math_ops = MathOperations()
    print(math_ops.add(5, 3))
    print(math_ops.multiply(4, 7))
"""

    # Save to file
    with open("math_code.py", "w") as f:
        f.write(code)

    # Parse with small chunk size to show code-aware splitting
    parser = TextParser(chunk_size=200)
    chunks = parser.parse_file("math_code.py")

    print(f"Code file parsed into {len(chunks)} chunks")
    print("\nCode-aware chunks preserve function/class boundaries:")

    for i, chunk in enumerate(chunks, 1):
        lines = chunk.split('\n')
        # Find function or class definitions
        for line in lines[:5]:  # Check first 5 lines
            if line.startswith('def ') or line.startswith('class '):
                print(f"\nChunk {i}: Contains {line}")
                break
        else:
            if '# Function' in chunk or '# Class' in chunk:
                first_line = lines[0] if lines else ""
                print(f"\nChunk {i}: {first_line[:50]}...")

    os.remove("math_code.py")

def example_5_save_chunks_to_files():
    """Example 5: Save chunks to separate files"""
    print_section("Example 5: Saving Chunks to Files")

    text = """
    Chapter 1: Introduction

    This is the beginning of our story. It was a dark and stormy night,
    and the parser was working hard to split the text into meaningful chunks.

    Chapter 2: The Journey

    As we continue our journey through the text, we encounter various
    challenges and obstacles that must be overcome.

    Chapter 3: The Resolution

    Finally, we reach the end of our tale, where all the chunks
    come together in perfect harmony.
    """

    parser = TextParser(chunk_size=150)
    chunks = parser.parse(text)

    print(f"Parsed text into {len(chunks)} chunks")

    # Save chunks to files
    output_dir = "output_chunks"
    saved = parser.save_chunks(chunks, output_dir, "story")

    print(f"Saved {saved} chunks to '{output_dir}/' directory")

    # List created files
    if os.path.exists(output_dir):
        files = sorted(os.listdir(output_dir))
        print("\nCreated files:")
        for file in files[:3]:  # Show first 3
            print(f"  - {file}")
        if len(files) > 3:
            print(f"  ... and {len(files) - 3} more")

        # Read one chunk back
        with open(os.path.join(output_dir, files[0]), 'r') as f:
            content = f.read()
            print(f"\nContent of {files[0]}:")
            print(f"  {content[:100]}...")

        # Clean up
        import shutil
        shutil.rmtree(output_dir)
        print(f"\nCleaned up: removed '{output_dir}' directory")

def example_6_chunk_statistics():
    """Example 6: Analyze chunk statistics"""
    print_section("Example 6: Chunk Statistics")

    # Create text with varying paragraph lengths
    text = """
    Short paragraph.

    This is a medium-length paragraph with more content. It contains
    several sentences and provides more detailed information about the topic.

    This is a very long paragraph that contains a lot of information. It goes on and on,
    describing various aspects of the subject matter in great detail. The parser will need
    to handle this appropriately, potentially splitting it if it exceeds the chunk size limit.
    We continue to add more content to make this paragraph even longer, ensuring that it will
    definitely need to be handled specially by the chunking algorithm.

    Another short one.

    And finally, a medium paragraph to end our example. It provides a nice
    conclusion to our text and wraps things up nicely.
    """

    # Test with different chunk sizes
    chunk_sizes = [50, 100, 200, 500]

    print("Analyzing chunking behavior with different sizes:\n")
    print(f"Original text: {len(text)} characters")
    print(f"Paragraphs: {len([p for p in text.split('\\n\\n') if p.strip()])}")

    for size in chunk_sizes:
        parser = TextParser(chunk_size=size)
        chunks = parser.parse(text)

        # Calculate statistics
        chunk_lengths = [len(c) for c in chunks]
        avg_length = sum(chunk_lengths) / len(chunk_lengths) if chunks else 0

        print(f"\nChunk size {size}:")
        print(f"  Chunks created: {len(chunks)}")
        print(f"  Average length: {avg_length:.1f} chars")
        print(f"  Min/Max length: {min(chunk_lengths)}/{max(chunk_lengths)} chars")

def example_7_different_file_types():
    """Example 7: Parse different file types with ChunkedText"""
    print_section("Example 7: Different File Types")

    # Create various test files
    files_created = []

    # CSV-like data
    with open("data.csv", "w") as f:
        f.write("Name,Age,City\n")
        f.write("Alice,30,New York\n")
        f.write("Bob,25,Los Angeles\n")
        f.write("Charlie,35,Chicago\n")
    files_created.append("data.csv")

    # JSON-like structure (will be parsed as text)
    with open("config.json", "w") as f:
        f.write("""{
    "name": "CrabParser",
    "version": "1.0.0",
    "features": ["parsing", "chunking", "parallel"],
    "settings": {
        "chunk_size": 1000,
        "respect_boundaries": true
    }
}""")
    files_created.append("config.json")

    # Markdown text
    with open("readme.md", "w") as f:
        f.write("""# CrabParser

## Features
- Fast parsing
- Memory efficient
- Code aware

## Usage
Simply import and use!
""")
    files_created.append("readme.md")

    # Parse each file
    parser = TextParser(chunk_size=100)

    for file_path in files_created:
        print(f"\n{file_path}:")

        # Use memory-efficient ChunkedText
        chunked = parser.parse_file_chunked(file_path)

        print(f"  Source: {chunked.source_file}")
        print(f"  Chunks: {len(chunked)}")
        print(f"  Total size: {chunked.total_size} bytes")

        # Access first chunk
        if len(chunked) > 0:
            first_chunk = chunked[0]
            preview = first_chunk[:60].replace('\n', '\\n')
            print(f"  First chunk: '{preview}'...")

    # Clean up
    for file_path in files_created:
        os.remove(file_path)

def example_8_performance_comparison():
    """Example 8: Performance comparison"""
    print_section("Example 8: Performance Comparison")

    # Generate different sized texts
    sizes = [1000, 5000, 10000]

    print("Comparing parsing performance:\n")

    for size in sizes:
        # Generate text
        text = ("This is a test sentence. " * (size // 25))

        parser = TextParser(chunk_size=500)

        # Time regular parsing
        start = time.perf_counter()
        chunks = parser.parse(text)
        regular_time = time.perf_counter() - start

        # Time chunked parsing (memory-efficient)
        start = time.perf_counter()
        chunked = parser.parse_chunked(text)
        chunked_time = time.perf_counter() - start

        print(f"Text size: {len(text)} chars")
        print(f"  Regular parse: {regular_time*1000:.2f}ms → {len(chunks)} chunks")
        print(f"  Chunked parse: {chunked_time*1000:.2f}ms → {len(chunked)} chunks")
        print(f"  Speed ratio: {regular_time/chunked_time:.2f}x\n")

def main():
    """Run all examples"""
    print("\n" + "█" * 60)
    print(" " * 20 + "CRABPARSER EXAMPLES")
    print("█" * 60)
    print("\nDemonstrating all CrabParser features")

    examples = [
        ("Basic Text Parsing", example_1_basic_text_parsing),
        ("Memory-Efficient Parsing", example_2_memory_efficient_parsing),
        ("File Parsing", example_3_file_parsing),
        ("Code-Aware Parsing", example_4_code_aware_parsing),
        ("Save Chunks to Files", example_5_save_chunks_to_files),
        ("Chunk Statistics", example_6_chunk_statistics),
        ("Different File Types", example_7_different_file_types),
        ("Performance Comparison", example_8_performance_comparison),
    ]

    print("\nAvailable Examples:")
    for i, (name, _) in enumerate(examples, 1):
        print(f"  {i}. {name}")

    print("\nRunning all examples...")
    print("(Press Ctrl+C to stop at any time)")

    try:
        for name, func in examples:
            func()
            time.sleep(0.5)  # Small pause between examples
    except KeyboardInterrupt:
        print("\n\nExamples interrupted by user.")

    print("\n" + "█" * 60)
    print(" " * 15 + "EXAMPLES COMPLETED SUCCESSFULLY!")
    print("█" * 60)
    print("\nCrabParser is ready for your text parsing needs!")
    print("Import with: from crabparser import TextParser")

if __name__ == "__main__":
    main()