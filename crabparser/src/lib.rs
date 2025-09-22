use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;
use regex::Regex;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// Import the new file type modules
mod encoding;
mod docx;
mod csv;
mod pdf;
mod code_parser;

use encoding::{read_file_auto_detect, clean_text};
use docx::parse_docx;
use csv::parse_csv;
use pdf::parse_pdf;
use code_parser::{parse_code, Language};

/// A memory-efficient container for text chunks stored in Rust memory
#[pyclass]
struct ChunkedText {
    chunks: Arc<Vec<String>>,
    metadata: ChunkMetadata,
}

#[derive(Clone)]
struct ChunkMetadata {
    total_size: usize,
    source_file: Option<String>,
}

#[pymethods]
impl ChunkedText {
    /// Get the total number of chunks
    #[getter]
    fn len(&self) -> usize {
        self.chunks.len()
    }

    /// Check if empty
    fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// Get a specific chunk by index (0-based)
    fn get(&self, index: usize) -> PyResult<Option<String>> {
        Ok(self.chunks.get(index).cloned())
    }

    /// Get a slice of chunks (returns a new Python list)
    fn get_slice(&self, start: usize, end: usize) -> PyResult<Vec<String>> {
        if start > end || end > self.chunks.len() {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                format!("Invalid slice [{}, {}] for {} chunks", start, end, self.chunks.len())
            ));
        }
        Ok(self.chunks[start..end].to_vec())
    }

    /// Get all chunks (use with caution for large datasets)
    fn to_list(&self) -> Vec<String> {
        self.chunks.to_vec()
    }

    /// Python iterator support
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<ChunkIterator> {
        Ok(ChunkIterator {
            chunks: slf.chunks.clone(),
            index: 0,
        })
    }

    /// Python indexing support (allows chunk_text[0] syntax)
    fn __getitem__(&self, index: isize) -> PyResult<String> {
        let len = self.chunks.len() as isize;
        let actual_index = if index < 0 {
            (len + index) as usize
        } else {
            index as usize
        };

        self.chunks.get(actual_index)
            .cloned()
            .ok_or_else(|| pyo3::exceptions::PyIndexError::new_err(
                format!("Index {} out of range for {} chunks", index, len)
            ))
    }

    /// Python len() support
    fn __len__(&self) -> usize {
        self.chunks.len()
    }

    /// Get metadata about the chunks
    #[getter]
    fn total_size(&self) -> usize {
        self.metadata.total_size
    }

    #[getter]
    fn source_file(&self) -> Option<String> {
        self.metadata.source_file.clone()
    }

    /// Save all chunks to files (batch operation)
    fn save_all(&self, output_dir: &str, base_name: &str) -> PyResult<usize> {
        let output_path = PathBuf::from(output_dir);
        fs::create_dir_all(&output_path)
            .map_err(|e| PyIOError::new_err(format!("Failed to create directory: {}", e)))?;

        let output_path = Arc::new(output_path);
        let base_name = Arc::new(base_name.to_string());

        // Save chunks in parallel
        let results: Vec<Result<(), String>> = self.chunks
            .par_iter()
            .enumerate()
            .map(|(i, chunk)| {
                let file_name = format!("{}_chunk_{:03}.txt", base_name, i + 1);
                let file_path = output_path.join(file_name);
                fs::write(&file_path, chunk)
                    .map_err(|e| format!("Failed to write chunk: {}", e))
            })
            .collect();

        for result in results {
            result.map_err(|e| PyIOError::new_err(e))?;
        }

        Ok(self.chunks.len())
    }
}

/// Iterator for ChunkedText to support Python iteration
#[pyclass]
struct ChunkIterator {
    chunks: Arc<Vec<String>>,
    index: usize,
}

#[pymethods]
impl ChunkIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<String> {
        if slf.index < slf.chunks.len() {
            let chunk = slf.chunks[slf.index].clone();
            slf.index += 1;
            Some(chunk)
        } else {
            None
        }
    }
}

/// A text parser that intelligently splits text into semantic chunks.
#[pyclass]
struct TextParser {
    chunk_size: usize,
    respect_paragraphs: bool,
    respect_sentences: bool,
    sentence_regex: Regex,
    paragraph_regex: Regex,
}

#[pymethods]
impl TextParser {
    #[new]
    #[pyo3(signature = (chunk_size=1000, respect_paragraphs=true, respect_sentences=true))]
    fn new(
        chunk_size: Option<usize>,
        respect_paragraphs: Option<bool>,
        respect_sentences: Option<bool>,
    ) -> PyResult<Self> {
        // Using simpler regex patterns that don't require lookaround
        // We'll handle sentence splitting differently
        let sentence_regex = Regex::new(r"[.!?]\s+")
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        let paragraph_regex = Regex::new(r"\n\n+")
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        Ok(TextParser {
            chunk_size: chunk_size.unwrap_or(1000),
            respect_paragraphs: respect_paragraphs.unwrap_or(true),
            respect_sentences: respect_sentences.unwrap_or(true),
            sentence_regex,
            paragraph_regex,
        })
    }

    /// Parse text and return ChunkedText object (memory-efficient)
    fn parse_chunked(&self, text: &str) -> PyResult<ChunkedText> {
        let chunks = self.parse(text)?;
        let total_size: usize = chunks.iter().map(|c| c.len()).sum();

        Ok(ChunkedText {
            chunks: Arc::new(chunks),
            metadata: ChunkMetadata {
                total_size,
                source_file: None,
            },
        })
    }

    /// Parse text into semantic chunks with parallel processing for large paragraphs
    fn parse(&self, text: &str) -> PyResult<Vec<String>> {
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Clean text before processing
        let text = clean_text(text);

        // Normalize line endings
        let text = text.replace("\r\n", "\n");

        // Split by paragraphs first if respect_paragraphs is true
        let paragraphs: Vec<String> = if self.respect_paragraphs {
            self.paragraph_regex.split(&text)
                .map(|s| s.to_string())
                .collect()
        } else {
            vec![text.clone()]
        };

        // Process large paragraphs in parallel
        let chunk_size = self.chunk_size;
        let processed_paragraphs: Vec<Vec<String>> = paragraphs
            .into_par_iter()
            .map(|paragraph| {
                let paragraph = paragraph.trim();
                if paragraph.is_empty() {
                    vec![]
                } else if paragraph.len() > chunk_size {
                    // Split large paragraphs in parallel
                    self.split_large_paragraph_parallel(paragraph).unwrap_or_else(|_| vec![paragraph.to_string()])
                } else {
                    vec![paragraph.to_string()]
                }
            })
            .collect();

        // Flatten and combine chunks sequentially to maintain order
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for paragraph_chunks in processed_paragraphs {
            for paragraph in paragraph_chunks {
                if paragraph.is_empty() {
                    continue;
                }

                if !current_chunk.is_empty() && current_chunk.len() + paragraph.len() > self.chunk_size {
                    if !current_chunk.trim().is_empty() {
                        chunks.push(current_chunk.trim().to_string());
                    }
                    current_chunk = paragraph;
                } else {
                    if !current_chunk.is_empty() {
                        current_chunk.push_str("\n\n");
                        current_chunk.push_str(&paragraph);
                    } else {
                        current_chunk = paragraph;
                    }
                }
            }
        }

        // Add remaining chunk
        if !current_chunk.trim().is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        Ok(chunks)
    }

    /// Parse a file and return ChunkedText object (memory-efficient)
    fn parse_file_chunked(&self, file_path: &str) -> PyResult<ChunkedText> {
        let chunks = self.parse_file(file_path)?;
        let total_size: usize = chunks.iter().map(|c| c.len()).sum();

        Ok(ChunkedText {
            chunks: Arc::new(chunks),
            metadata: ChunkMetadata {
                total_size,
                source_file: Some(file_path.to_string()),
            },
        })
    }

    /// Parse a file into chunks (supports txt, docx, csv, pdf, and code files)
    fn parse_file(&self, file_path: &str) -> PyResult<Vec<String>> {
        // Determine file type by extension for special processing
        let path = Path::new(file_path);
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Check if it's a programming language file
        let language = Language::from_extension(&extension);

        match language {
            Language::Unknown => {
                // Use specialized parsers for documents
                let text = match extension.as_str() {
                    "pdf" => parse_pdf(file_path).unwrap_or_else(|_| String::new()),
                    "docx" | "doc" => parse_docx(file_path).unwrap_or_else(|_| String::new()),
                    "csv" => parse_csv(file_path).unwrap_or_else(|_| String::new()),
                    _ => read_file_auto_detect(file_path).unwrap_or_else(|_| String::new()),
                };

                if text.is_empty() {
                    return Ok(vec![]);
                }

                // Parse extracted text into chunks
                let chunks = self.parse(&text).unwrap_or_else(|_| vec![text.clone()]);
                Ok(chunks)
            }
            _ => {
                // It's a code file - use code-aware parsing
                let code = read_file_auto_detect(file_path).unwrap_or_else(|_| String::new());

                if code.is_empty() {
                    return Ok(vec![]);
                }

                // Parse code with semantic understanding
                parse_code(&code, language, self.chunk_size)
            }
        }
    }

    /// Save chunks to separate files with parallel I/O
    fn save_chunks(&self, chunks: Vec<String>, output_dir: &str, base_name: &str) -> PyResult<usize> {
        let output_path = PathBuf::from(output_dir);

        // Create output directory if it doesn't exist
        fs::create_dir_all(&output_path)
            .map_err(|e| PyIOError::new_err(format!("Failed to create output directory: {}", e)))?;

        let chunk_count = chunks.len();
        let output_path = Arc::new(output_path);
        let base_name = Arc::new(base_name.to_string());

        // Save chunks in parallel
        let results: Vec<Result<(), String>> = chunks
            .into_par_iter()
            .enumerate()
            .map(|(i, chunk)| {
                let file_name = format!("{}_chunk_{:03}.txt", base_name, i + 1);
                let file_path = output_path.join(file_name);

                fs::write(&file_path, chunk)
                    .map_err(|e| format!("Failed to write chunk file: {}", e))
            })
            .collect();

        // Check for any errors
        for result in results {
            if let Err(e) = result {
                return Err(PyIOError::new_err(e));
            }
        }

        Ok(chunk_count)
    }

    // Getter methods for Python
    #[getter]
    fn get_chunk_size(&self) -> usize {
        self.chunk_size
    }

    #[getter]
    fn get_respect_paragraphs(&self) -> bool {
        self.respect_paragraphs
    }

    #[getter]
    fn get_respect_sentences(&self) -> bool {
        self.respect_sentences
    }

    // Setter methods for Python
    #[setter]
    fn set_chunk_size(&mut self, value: usize) {
        self.chunk_size = value;
    }

    #[setter]
    fn set_respect_paragraphs(&mut self, value: bool) {
        self.respect_paragraphs = value;
    }

    #[setter]
    fn set_respect_sentences(&mut self, value: bool) {
        self.respect_sentences = value;
    }

}

impl TextParser {
    /// Split a large paragraph by sentences (parallel version)
    fn split_large_paragraph_parallel(&self, paragraph: &str) -> PyResult<Vec<String>> {
        if !self.respect_sentences {
            // Parallel split by chunk size for very large text
            let chunk_size = self.chunk_size;
            let chunks: Vec<String> = (0..paragraph.len())
                .into_par_iter()
                .step_by(chunk_size)
                .map(|i| {
                    let end = std::cmp::min(i + chunk_size, paragraph.len());
                    paragraph[i..end].to_string()
                })
                .collect();
            return Ok(chunks);
        }

        // For sentence-based splitting, use the original sequential method
        // as sentence boundaries need to be preserved in order
        self.split_large_paragraph(paragraph)
    }

    /// Split a large paragraph by sentences
    fn split_large_paragraph(&self, paragraph: &str) -> PyResult<Vec<String>> {
        if !self.respect_sentences {
            // Simple split by chunk size
            let mut result = Vec::new();
            let mut start = 0;
            while start < paragraph.len() {
                let end = std::cmp::min(start + self.chunk_size, paragraph.len());
                result.push(paragraph[start..end].to_string());
                start = end;
            }
            return Ok(result);
        }

        // Split by sentences - handle the regex match differently
        let mut sentences = Vec::new();
        let mut last = 0;

        for mat in self.sentence_regex.find_iter(paragraph) {
            let end = mat.start() + 1; // Include the punctuation
            sentences.push(&paragraph[last..end]);
            last = mat.end();
        }

        // Add the remaining text if any
        if last < paragraph.len() {
            sentences.push(&paragraph[last..]);
        }

        // If no sentences were found, treat the whole paragraph as one sentence
        if sentences.is_empty() {
            sentences.push(paragraph);
        }

        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for sentence in sentences {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }

            if !current_chunk.is_empty() && current_chunk.len() + sentence.len() + 1 > self.chunk_size {
                if !current_chunk.trim().is_empty() {
                    chunks.push(current_chunk.trim().to_string());
                }

                // If single sentence is too large, split it by chunk size
                if sentence.len() > self.chunk_size {
                    let mut remaining = sentence.to_string();
                    while remaining.len() > self.chunk_size {
                        chunks.push(remaining[..self.chunk_size].to_string());
                        remaining = remaining[self.chunk_size..].to_string();
                    }
                    current_chunk = remaining;
                } else {
                    current_chunk = sentence.to_string();
                }
            } else {
                if !current_chunk.is_empty() {
                    current_chunk.push(' ');
                    current_chunk.push_str(sentence);
                } else {
                    current_chunk = sentence.to_string();
                }
            }
        }

        if !current_chunk.trim().is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        Ok(chunks)
    }
}

/// The crabparser_rust module exposed to Python
#[pymodule]
fn crabparser_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<TextParser>()?;
    m.add_class::<ChunkedText>()?;
    m.add_class::<ChunkIterator>()?;
    m.add("__version__", "0.2.0")?;
    Ok(())
}