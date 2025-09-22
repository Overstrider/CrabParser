use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;
use csv::ReaderBuilder;
use crate::encoding::read_file_auto_detect;

/// Parse CSV file content into structured text
pub fn parse_csv(file_path: &str) -> PyResult<String> {
    // Read file with auto-detected encoding
    let content = read_file_auto_detect(file_path)?;

    let mut reader = ReaderBuilder::new()
        .flexible(true)  // Allow variable number of fields
        .from_reader(content.as_bytes());

    let mut text_content = Vec::new();

    // Get headers if they exist
    let headers = reader.headers()
        .map_err(|e| PyIOError::new_err(format!("Failed to read CSV headers: {}", e)))?
        .clone();

    // Format headers as a title row
    if !headers.is_empty() {
        text_content.push(format!("Headers: {}", headers.iter().collect::<Vec<_>>().join(" | ")));
        text_content.push("-".repeat(50));
    }

    // Process each record
    for (row_num, result) in reader.records().enumerate() {
        let record = result
            .map_err(|e| PyIOError::new_err(format!("Failed to read CSV row {}: {}", row_num + 1, e)))?;

        // Format record as text
        let row_text = if headers.is_empty() {
            // No headers, just join fields
            format!("Row {}: {}",
                row_num + 1,
                record.iter().collect::<Vec<_>>().join(" | "))
        } else {
            // With headers, create key-value pairs
            let mut pairs = Vec::new();
            for (i, field) in record.iter().enumerate() {
                if i < headers.len() {
                    pairs.push(format!("{}: {}", headers.get(i).unwrap_or(""), field));
                } else {
                    pairs.push(format!("Field {}: {}", i + 1, field));
                }
            }
            format!("Row {}:\n  {}", row_num + 1, pairs.join("\n  "))
        };

        text_content.push(row_text);
    }

    Ok(text_content.join("\n\n"))
}

