use pyo3::prelude::*;
use pyo3::exceptions::PyIOError;
use std::io::{Read, BufReader};
use zip::ZipArchive;
use quick_xml::Reader;
use quick_xml::events::Event;

/// Parse DOCX file content into text
pub fn parse_docx(file_path: &str) -> PyResult<String> {
    let file = std::fs::File::open(file_path)
        .map_err(|e| PyIOError::new_err(format!("Failed to open DOCX file: {}", e)))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| PyIOError::new_err(format!("Failed to read DOCX as ZIP: {}", e)))?;

    // DOCX files store document content in word/document.xml
    let document = archive.by_name("word/document.xml")
        .map_err(|e| PyIOError::new_err(format!("Failed to find document.xml in DOCX: {}", e)))?;

    extract_text_from_xml(document)
}

/// Extract text content from DOCX XML
fn extract_text_from_xml<R: Read>(reader: R) -> PyResult<String> {
    let buf_reader = BufReader::new(reader);
    let mut xml_reader = Reader::from_reader(buf_reader);

    let mut buf = Vec::new();
    let mut text_content = Vec::new();
    let mut in_paragraph = false;
    let mut current_paragraph = String::new();

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"w:p" => {
                        in_paragraph = true;
                        current_paragraph.clear();
                    }
                    b"w:br" => {
                        if in_paragraph {
                            current_paragraph.push('\n');
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"w:p" {
                    if !current_paragraph.trim().is_empty() {
                        text_content.push(current_paragraph.clone());
                    }
                    in_paragraph = false;
                    current_paragraph.clear();
                }
            }
            Ok(Event::Text(e)) => {
                if in_paragraph {
                    let text = e.unescape()
                        .map_err(|err| PyIOError::new_err(format!("XML decode error: {}", err)))?;
                    current_paragraph.push_str(&text);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(PyIOError::new_err(format!("Error parsing XML: {}", e)));
            }
            _ => {}
        }
        buf.clear();
    }

    // Join paragraphs with double newlines to maintain structure
    Ok(text_content.join("\n\n"))
}

