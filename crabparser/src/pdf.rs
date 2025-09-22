use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;
use pdf_extract::extract_text;
use std::path::Path;

/// Parse PDF file content into text
pub fn parse_pdf(file_path: &str) -> PyResult<String> {
    let path = Path::new(file_path);

    // Extract text from PDF
    let text = extract_text(path)
        .map_err(|e| PyIOError::new_err(format!("Failed to extract text from PDF: {}", e)))?;

    // Clean up the extracted text
    let cleaned_text = clean_pdf_text(&text);

    Ok(cleaned_text)
}

/// Clean and normalize PDF extracted text
fn clean_pdf_text(text: &str) -> String {
    let mut lines = Vec::new();
    let mut current_paragraph = String::new();
    let mut last_was_empty = false;

    for line in text.lines() {
        let line = line.trim();

        if line.is_empty() {
            // Double empty lines indicate paragraph break
            if !current_paragraph.is_empty() {
                lines.push(current_paragraph.clone());
                current_paragraph.clear();
            }
            last_was_empty = true;
        } else {
            // Check if this line is likely a continuation of the previous line
            if !last_was_empty && !current_paragraph.is_empty() {
                // Check if previous line ended with punctuation
                let last_char = current_paragraph.chars().last().unwrap_or(' ');
                if last_char == '.' || last_char == '!' || last_char == '?' || last_char == ':' {
                    // Start new paragraph
                    lines.push(current_paragraph.clone());
                    current_paragraph = line.to_string();
                } else if line.chars().next().unwrap_or(' ').is_uppercase() && last_char != ',' {
                    // Likely a new sentence
                    current_paragraph.push(' ');
                    current_paragraph.push_str(line);
                } else {
                    // Continuation of the same line
                    current_paragraph.push(' ');
                    current_paragraph.push_str(line);
                }
            } else {
                // Start new paragraph
                if !current_paragraph.is_empty() {
                    lines.push(current_paragraph.clone());
                }
                current_paragraph = line.to_string();
            }
            last_was_empty = false;
        }
    }

    // Add any remaining content
    if !current_paragraph.is_empty() {
        lines.push(current_paragraph);
    }

    // Join paragraphs with double newlines
    lines.join("\n\n")
}

