use encoding_rs::{Encoding, WINDOWS_1252};
use std::fs::File;
use std::io::Read;
use pyo3::prelude::*;
use rayon::prelude::*;

/// Read file with automatic encoding detection - NEVER FAILS
pub fn read_file_auto_detect(file_path: &str) -> PyResult<String> {
    // Read file as bytes first
    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => {
            // If we can't open the file, return empty string
            return Ok(String::new());
        }
    };

    let mut buffer = Vec::new();

    // Read the file bytes - if this fails, just return empty
    if file.read_to_end(&mut buffer).is_err() {
        return Ok(String::new());
    }

    // If empty file, return empty string
    if buffer.is_empty() {
        return Ok(String::new());
    }

    // Convert bytes to string with maximum tolerance
    let text = bytes_to_string_lossy(&buffer);

    // Clean the text
    let cleaned = clean_text(&text);

    Ok(cleaned)
}

/// Convert bytes to string, replacing any invalid characters
fn bytes_to_string_lossy(data: &[u8]) -> String {
    // First, try UTF-8
    if let Ok(text) = std::str::from_utf8(data) {
        return text.to_string();
    }

    // Try common encodings
    let encodings: Vec<&'static Encoding> = vec![
        WINDOWS_1252,
        encoding_rs::ISO_8859_2,
        encoding_rs::GBK,
        encoding_rs::SHIFT_JIS,
        encoding_rs::BIG5,
    ];

    for encoding in encodings {
        let (text, _, had_errors) = encoding.decode(data);
        if !had_errors {
            return text.into_owned();
        }
    }

    // For large buffers, use parallel processing
    if data.len() > 10000 {
        // Process in chunks for better parallelization
        let chunk_size = 1000;
        let chars: Vec<String> = data.par_chunks(chunk_size)
            .map(|chunk| {
                let mut chunk_result = String::with_capacity(chunk.len());
                for &byte in chunk {
                    if byte.is_ascii() {
                        chunk_result.push(byte as char);
                    } else if byte >= 0xA0 {
                        chunk_result.push(char::from_u32(byte as u32).unwrap_or('�'));
                    } else if byte >= 0x20 {
                        chunk_result.push(char::from_u32(byte as u32).unwrap_or(' '));
                    }
                }
                chunk_result
            })
            .collect();

        return chars.join("");
    }

    // For smaller buffers, use sequential processing
    let mut result = String::with_capacity(data.len());
    for &byte in data {
        if byte.is_ascii() {
            result.push(byte as char);
        } else if byte >= 0xA0 {
            result.push(char::from_u32(byte as u32).unwrap_or('�'));
        } else if byte >= 0x20 {
            result.push(char::from_u32(byte as u32).unwrap_or(' '));
        }
    }

    result
}

/// Clean text by removing or replacing invalid characters
pub fn clean_text(text: &str) -> String {
    let mut cleaned = String::with_capacity(text.len());

    for ch in text.chars() {
        match ch {
            '\0' => {},  // Skip null characters
            '\x0B' | '\x0C' => cleaned.push('\n'),  // Convert form feeds to newlines
            '\x01'..='\x08' | '\x0E'..='\x1F' => {
                // Skip other control chars except \n, \r, \t
                if ch != '\n' && ch != '\r' && ch != '\t' {
                    cleaned.push(' ');  // Replace with space
                } else {
                    cleaned.push(ch);
                }
            }
            '�' => cleaned.push(' '),  // Replace replacement character with space
            _ => cleaned.push(ch),
        }
    }

    // Normalize line endings
    cleaned = cleaned.replace("\r\n", "\n");
    cleaned = cleaned.replace('\r', "\n");

    // Remove excessive whitespace
    let mut final_text = String::with_capacity(cleaned.len());
    let mut _last_was_newline = false;
    let mut newline_count = 0;

    for ch in cleaned.chars() {
        if ch == '\n' {
            newline_count += 1;
            if newline_count <= 2 {  // Max 2 newlines in a row
                final_text.push(ch);
            }
            _last_was_newline = true;
        } else {
            newline_count = 0;
            _last_was_newline = false;
            final_text.push(ch);
        }
    }

    final_text
}

