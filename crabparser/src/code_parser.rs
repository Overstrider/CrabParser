use pyo3::prelude::*;
use regex::Regex;
// Removed unused import: rayon::prelude

/// Supported programming languages
#[derive(Debug, Clone)]
pub enum Language {
    Python,
    JavaScript,
    TypeScript,
    Rust,
    Go,
    Java,
    CSharp,
    Cpp,
    Ruby,
    Php,
    Swift,
    Kotlin,
    Unknown,
}

impl Language {
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "py" | "pyw" => Language::Python,
            "js" | "mjs" | "cjs" => Language::JavaScript,
            "ts" | "tsx" => Language::TypeScript,
            "rs" => Language::Rust,
            "go" => Language::Go,
            "java" => Language::Java,
            "cs" => Language::CSharp,
            "cpp" | "cc" | "cxx" | "hpp" | "h" => Language::Cpp,
            "rb" => Language::Ruby,
            "php" => Language::Php,
            "swift" => Language::Swift,
            "kt" | "kts" => Language::Kotlin,
            _ => Language::Unknown,
        }
    }
}

/// Code structure for semantic chunking
#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub block_type: BlockType,
    pub name: String,
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    Class,
    Function,
    Method,  // Keep for future use
    Interface,
    Struct,
    Enum,
    Module,
    Namespace,
    Import,
    Global,  // Keep for future use
    Comment, // Keep for future use
}

/// Parse code into semantic chunks based on language structure
pub fn parse_code(code: &str, language: Language, chunk_size: usize) -> PyResult<Vec<String>> {
    let blocks = match language {
        Language::Python => parse_python(code),
        Language::JavaScript | Language::TypeScript => parse_javascript(code),
        Language::Rust => parse_rust(code),
        Language::Go => parse_go(code),
        Language::Java | Language::CSharp => parse_java_like(code),
        Language::Cpp => parse_cpp(code),
        _ => parse_generic(code),
    };

    // Convert blocks to chunks respecting size limits
    let chunks = blocks_to_chunks(blocks, chunk_size);
    Ok(chunks)
}

/// Parse Python code
fn parse_python(code: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = code.lines().collect();

    // Regex patterns for Python
    let class_regex = Regex::new(r"^class\s+(\w+)").unwrap();
    let func_regex = Regex::new(r"^def\s+(\w+)").unwrap();
    let _method_regex = Regex::new(r"^\s+def\s+(\w+)").unwrap();
    let import_regex = Regex::new(r"^(from\s+\S+\s+)?import\s+").unwrap();

    let mut current_block = None;
    let mut current_indent = 0;
    let mut block_content = String::new();
    let mut block_start = 0;

    for (i, line) in lines.iter().enumerate() {
        let indent = line.len() - line.trim_start().len();

        // Check for class definition
        if let Some(cap) = class_regex.captures(line) {
            // Save previous block if exists
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

            current_block = Some(CodeBlock {
                block_type: BlockType::Class,
                name: cap[1].to_string(),
                content: String::new(),
                start_line: i,
                end_line: i,
            });
            current_indent = indent;
            block_content = line.to_string();
            block_start = i;
        }
        // Check for function/method definition
        else if let Some(cap) = func_regex.captures(line) {
            if current_block.is_some() && indent > 0 {
                // This is a method inside a class
                block_content.push('\n');
                block_content.push_str(line);
            } else {
                // This is a top-level function
                if let Some(block) = current_block.take() {
                    blocks.push(block);
                }

                current_block = Some(CodeBlock {
                    block_type: BlockType::Function,
                    name: cap[1].to_string(),
                    content: String::new(),
                    start_line: i,
                    end_line: i,
                });
                current_indent = indent;
                block_content = line.to_string();
                block_start = i;
            }
        }
        // Check for imports
        else if import_regex.is_match(line) && current_block.is_none() {
            blocks.push(CodeBlock {
                block_type: BlockType::Import,
                name: "imports".to_string(),
                content: line.to_string(),
                start_line: i,
                end_line: i,
            });
        }
        // Continue current block
        else if let Some(ref mut block) = current_block {
            if !line.trim().is_empty() && indent <= current_indent && i > block_start + 1 {
                // End of current block
                block.content = block_content.clone();
                block.end_line = i - 1;
                blocks.push(block.clone());

                current_block = None;
                block_content.clear();
            } else {
                block_content.push('\n');
                block_content.push_str(line);
                block.end_line = i;
            }
        }
    }

    // Save last block
    if let Some(mut block) = current_block {
        block.content = block_content;
        blocks.push(block);
    }

    blocks
}

/// Parse JavaScript/TypeScript code
fn parse_javascript(code: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = code.lines().collect();

    // Regex patterns for JavaScript/TypeScript
    let class_regex = Regex::new(r"^(export\s+)?(abstract\s+)?class\s+(\w+)").unwrap();
    let func_regex = Regex::new(r"^(export\s+)?(async\s+)?function\s+(\w+)").unwrap();
    let arrow_func_regex = Regex::new(r"^(export\s+)?const\s+(\w+)\s*=\s*(\(|async)").unwrap();
    let interface_regex = Regex::new(r"^(export\s+)?interface\s+(\w+)").unwrap();
    let import_regex = Regex::new(r"^import\s+").unwrap();

    let mut brace_count = 0;
    let mut current_block = None;
    let mut block_content = String::new();

    for (i, line) in lines.iter().enumerate() {
        // Count braces
        for ch in line.chars() {
            match ch {
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                _ => {}
            }
        }

        // Check for class
        if let Some(cap) = class_regex.captures(line) {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

            let name = cap.get(3).map_or("", |m| m.as_str());
            current_block = Some(CodeBlock {
                block_type: BlockType::Class,
                name: name.to_string(),
                content: String::new(),
                start_line: i,
                end_line: i,
            });
            block_content = line.to_string();
        }
        // Check for function
        else if let Some(cap) = func_regex.captures(line) {
            if current_block.is_none() {
                let name = cap.get(3).map_or("", |m| m.as_str());
                current_block = Some(CodeBlock {
                    block_type: BlockType::Function,
                    name: name.to_string(),
                    content: String::new(),
                    start_line: i,
                    end_line: i,
                });
                block_content = line.to_string();
            }
        }
        // Check for arrow function
        else if let Some(cap) = arrow_func_regex.captures(line) {
            if current_block.is_none() {
                let name = cap.get(2).map_or("", |m| m.as_str());
                current_block = Some(CodeBlock {
                    block_type: BlockType::Function,
                    name: name.to_string(),
                    content: String::new(),
                    start_line: i,
                    end_line: i,
                });
                block_content = line.to_string();
            }
        }
        // Check for interface (TypeScript)
        else if let Some(cap) = interface_regex.captures(line) {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

            let name = cap.get(2).map_or("", |m| m.as_str());
            current_block = Some(CodeBlock {
                block_type: BlockType::Interface,
                name: name.to_string(),
                content: String::new(),
                start_line: i,
                end_line: i,
            });
            block_content = line.to_string();
        }
        // Check for imports
        else if import_regex.is_match(line) && current_block.is_none() {
            blocks.push(CodeBlock {
                block_type: BlockType::Import,
                name: "imports".to_string(),
                content: line.to_string(),
                start_line: i,
                end_line: i,
            });
        }
        // Continue current block
        else if let Some(ref mut block) = current_block {
            block_content.push('\n');
            block_content.push_str(line);
            block.end_line = i;

            // Check if block is complete (brace count back to level)
            if brace_count == 0 && line.contains('}') {
                block.content = block_content.clone();
                blocks.push(block.clone());
                current_block = None;
                block_content.clear();
            }
        }
    }

    // Save last block
    if let Some(mut block) = current_block {
        block.content = block_content;
        blocks.push(block);
    }

    blocks
}

/// Parse Rust code
fn parse_rust(code: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = code.lines().collect();

    // Regex patterns for Rust
    let struct_regex = Regex::new(r"^(pub\s+)?struct\s+(\w+)").unwrap();
    let enum_regex = Regex::new(r"^(pub\s+)?enum\s+(\w+)").unwrap();
    let impl_regex = Regex::new(r"^impl(?:\s+<[^>]+>)?\s+(\w+)").unwrap();
    let func_regex = Regex::new(r"^(pub\s+)?(async\s+)?fn\s+(\w+)").unwrap();
    let mod_regex = Regex::new(r"^(pub\s+)?mod\s+(\w+)").unwrap();
    let use_regex = Regex::new(r"^use\s+").unwrap();

    let mut brace_count = 0;
    let mut current_block = None;
    let mut block_content = String::new();

    for (i, line) in lines.iter().enumerate() {
        // Count braces
        for ch in line.chars() {
            match ch {
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                _ => {}
            }
        }

        // Check for struct
        if let Some(cap) = struct_regex.captures(line) {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

            let name = cap.get(2).map_or("", |m| m.as_str());
            current_block = Some(CodeBlock {
                block_type: BlockType::Struct,
                name: name.to_string(),
                content: String::new(),
                start_line: i,
                end_line: i,
            });
            block_content = line.to_string();
        }
        // Check for enum
        else if let Some(cap) = enum_regex.captures(line) {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

            let name = cap.get(2).map_or("", |m| m.as_str());
            current_block = Some(CodeBlock {
                block_type: BlockType::Enum,
                name: name.to_string(),
                content: String::new(),
                start_line: i,
                end_line: i,
            });
            block_content = line.to_string();
        }
        // Check for impl block
        else if let Some(cap) = impl_regex.captures(line) {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

            let name = cap.get(1).map_or("impl", |m| m.as_str());
            current_block = Some(CodeBlock {
                block_type: BlockType::Class, // Treat impl as class-like
                name: format!("impl {}", name),
                content: String::new(),
                start_line: i,
                end_line: i,
            });
            block_content = line.to_string();
        }
        // Check for function
        else if let Some(cap) = func_regex.captures(line) {
            if current_block.is_none() || (current_block.as_ref().map_or(false, |b| b.block_type != BlockType::Class)) {
                if let Some(block) = current_block.take() {
                    blocks.push(block);
                }

                let name = cap.get(3).map_or("", |m| m.as_str());
                current_block = Some(CodeBlock {
                    block_type: BlockType::Function,
                    name: name.to_string(),
                    content: String::new(),
                    start_line: i,
                    end_line: i,
                });
                block_content = line.to_string();
            }
        }
        // Check for module
        else if let Some(cap) = mod_regex.captures(line) {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

            let name = cap.get(2).map_or("", |m| m.as_str());
            current_block = Some(CodeBlock {
                block_type: BlockType::Module,
                name: name.to_string(),
                content: String::new(),
                start_line: i,
                end_line: i,
            });
            block_content = line.to_string();
        }
        // Check for use statements
        else if use_regex.is_match(line) && current_block.is_none() {
            blocks.push(CodeBlock {
                block_type: BlockType::Import,
                name: "imports".to_string(),
                content: line.to_string(),
                start_line: i,
                end_line: i,
            });
        }
        // Continue current block
        else if let Some(ref mut block) = current_block {
            block_content.push('\n');
            block_content.push_str(line);
            block.end_line = i;

            // Check if block is complete
            if brace_count == 0 && line.contains('}') {
                block.content = block_content.clone();
                blocks.push(block.clone());
                current_block = None;
                block_content.clear();
            }
        }
    }

    // Save last block
    if let Some(mut block) = current_block {
        block.content = block_content;
        blocks.push(block);
    }

    blocks
}

/// Parse Go code
fn parse_go(code: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = code.lines().collect();

    // Regex patterns for Go
    let func_regex = Regex::new(r"^func\s+(\(.*?\))?\s*(\w+)").unwrap();
    let struct_regex = Regex::new(r"^type\s+(\w+)\s+struct").unwrap();
    let interface_regex = Regex::new(r"^type\s+(\w+)\s+interface").unwrap();
    let import_regex = Regex::new(r"^import\s+").unwrap();
    let package_regex = Regex::new(r"^package\s+(\w+)").unwrap();

    let mut current_block = None;
    let mut block_content = String::new();
    let mut brace_count = 0;

    for (i, line) in lines.iter().enumerate() {
        // Count braces
        for ch in line.chars() {
            match ch {
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                _ => {}
            }
        }

        // Check for package declaration
        if let Some(cap) = package_regex.captures(line) {
            blocks.push(CodeBlock {
                block_type: BlockType::Module,
                name: format!("package {}", &cap[1]),
                content: line.to_string(),
                start_line: i,
                end_line: i,
            });
        }
        // Check for function
        else if let Some(cap) = func_regex.captures(line) {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

            let name = cap.get(2).map_or("", |m| m.as_str());
            current_block = Some(CodeBlock {
                block_type: BlockType::Function,
                name: name.to_string(),
                content: String::new(),
                start_line: i,
                end_line: i,
            });
            block_content = line.to_string();
        }
        // Check for struct
        else if let Some(cap) = struct_regex.captures(line) {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

            current_block = Some(CodeBlock {
                block_type: BlockType::Struct,
                name: cap[1].to_string(),
                content: String::new(),
                start_line: i,
                end_line: i,
            });
            block_content = line.to_string();
        }
        // Check for interface
        else if let Some(cap) = interface_regex.captures(line) {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

            current_block = Some(CodeBlock {
                block_type: BlockType::Interface,
                name: cap[1].to_string(),
                content: String::new(),
                start_line: i,
                end_line: i,
            });
            block_content = line.to_string();
        }
        // Check for imports
        else if import_regex.is_match(line) && current_block.is_none() {
            blocks.push(CodeBlock {
                block_type: BlockType::Import,
                name: "imports".to_string(),
                content: line.to_string(),
                start_line: i,
                end_line: i,
            });
        }
        // Continue current block
        else if let Some(ref mut block) = current_block {
            block_content.push('\n');
            block_content.push_str(line);
            block.end_line = i;

            // Check if block is complete
            if brace_count == 0 && line.contains('}') {
                block.content = block_content.clone();
                blocks.push(block.clone());
                current_block = None;
                block_content.clear();
            }
        }
    }

    // Save last block
    if let Some(mut block) = current_block {
        block.content = block_content;
        blocks.push(block);
    }

    blocks
}

/// Parse Java/C#-like code
fn parse_java_like(code: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = code.lines().collect();

    // Regex patterns for Java/C#
    let class_regex = Regex::new(r"^(public\s+|private\s+|protected\s+)?(abstract\s+|static\s+)?class\s+(\w+)").unwrap();
    let interface_regex = Regex::new(r"^(public\s+|private\s+|protected\s+)?interface\s+(\w+)").unwrap();
    let _method_regex = Regex::new(r"^(public\s+|private\s+|protected\s+)?(static\s+)?(\w+\s+)?(\w+)\s*\(").unwrap();
    let import_regex = Regex::new(r"^(import|using)\s+").unwrap();

    let mut current_block = None;
    let mut block_content = String::new();
    let mut brace_count = 0;

    for (i, line) in lines.iter().enumerate() {
        // Count braces
        for ch in line.chars() {
            match ch {
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                _ => {}
            }
        }

        // Check for class
        if let Some(cap) = class_regex.captures(line) {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

            let name = cap.get(3).map_or("", |m| m.as_str());
            current_block = Some(CodeBlock {
                block_type: BlockType::Class,
                name: name.to_string(),
                content: String::new(),
                start_line: i,
                end_line: i,
            });
            block_content = line.to_string();
        }
        // Check for interface
        else if let Some(cap) = interface_regex.captures(line) {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

            let name = cap.get(2).map_or("", |m| m.as_str());
            current_block = Some(CodeBlock {
                block_type: BlockType::Interface,
                name: name.to_string(),
                content: String::new(),
                start_line: i,
                end_line: i,
            });
            block_content = line.to_string();
        }
        // Check for imports
        else if import_regex.is_match(line) && current_block.is_none() {
            blocks.push(CodeBlock {
                block_type: BlockType::Import,
                name: "imports".to_string(),
                content: line.to_string(),
                start_line: i,
                end_line: i,
            });
        }
        // Continue current block
        else if let Some(ref mut block) = current_block {
            block_content.push('\n');
            block_content.push_str(line);
            block.end_line = i;

            // Check if block is complete
            if brace_count == 0 && line.contains('}') {
                block.content = block_content.clone();
                blocks.push(block.clone());
                current_block = None;
                block_content.clear();
            }
        }
    }

    // Save last block
    if let Some(mut block) = current_block {
        block.content = block_content;
        blocks.push(block);
    }

    blocks
}

/// Parse C++ code
fn parse_cpp(code: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = code.lines().collect();

    // Regex patterns for C++
    let class_regex = Regex::new(r"^(class|struct)\s+(\w+)").unwrap();
    let _func_regex = Regex::new(r"^(\w+\s+)*(\w+)\s*\([^)]*\)\s*(const)?\s*\{?").unwrap();
    let namespace_regex = Regex::new(r"^namespace\s+(\w+)").unwrap();
    let include_regex = Regex::new(r"^#include\s+").unwrap();

    let mut current_block = None;
    let mut block_content = String::new();
    let mut brace_count = 0;

    for (i, line) in lines.iter().enumerate() {
        // Count braces
        for ch in line.chars() {
            match ch {
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                _ => {}
            }
        }

        // Check for class/struct
        if let Some(cap) = class_regex.captures(line) {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

            current_block = Some(CodeBlock {
                block_type: BlockType::Class,
                name: cap[2].to_string(),
                content: String::new(),
                start_line: i,
                end_line: i,
            });
            block_content = line.to_string();
        }
        // Check for namespace
        else if let Some(cap) = namespace_regex.captures(line) {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

            current_block = Some(CodeBlock {
                block_type: BlockType::Namespace,
                name: cap[1].to_string(),
                content: String::new(),
                start_line: i,
                end_line: i,
            });
            block_content = line.to_string();
        }
        // Check for includes
        else if include_regex.is_match(line) && current_block.is_none() {
            blocks.push(CodeBlock {
                block_type: BlockType::Import,
                name: "includes".to_string(),
                content: line.to_string(),
                start_line: i,
                end_line: i,
            });
        }
        // Continue current block
        else if let Some(ref mut block) = current_block {
            block_content.push('\n');
            block_content.push_str(line);
            block.end_line = i;

            // Check if block is complete
            if brace_count == 0 && line.contains('}') {
                block.content = block_content.clone();
                blocks.push(block.clone());
                current_block = None;
                block_content.clear();
            }
        }
    }

    // Save last block
    if let Some(mut block) = current_block {
        block.content = block_content;
        blocks.push(block);
    }

    blocks
}

/// Generic code parser for unknown languages
fn parse_generic(code: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = code.lines().collect();

    // Simple heuristics for generic parsing
    let func_keywords = ["function", "def", "fn", "func", "sub", "procedure"];
    let class_keywords = ["class", "struct", "interface", "type", "trait"];

    let mut current_chunk = String::new();
    let mut chunk_start = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim().to_lowercase();

        // Check if line starts a new semantic block
        let starts_block = func_keywords.iter().any(|&k| trimmed.starts_with(k))
            || class_keywords.iter().any(|&k| trimmed.starts_with(k));

        if starts_block && !current_chunk.is_empty() {
            // Save current chunk
            blocks.push(CodeBlock {
                block_type: BlockType::Function,
                name: format!("block_{}", blocks.len()),
                content: current_chunk.clone(),
                start_line: chunk_start,
                end_line: i - 1,
            });

            current_chunk = line.to_string();
            chunk_start = i;
        } else {
            if !current_chunk.is_empty() {
                current_chunk.push('\n');
            }
            current_chunk.push_str(line);
        }
    }

    // Save last chunk
    if !current_chunk.is_empty() {
        blocks.push(CodeBlock {
            block_type: BlockType::Function,
            name: format!("block_{}", blocks.len()),
            content: current_chunk,
            start_line: chunk_start,
            end_line: lines.len() - 1,
        });
    }

    blocks
}

/// Convert code blocks to chunks respecting size limits
fn blocks_to_chunks(blocks: Vec<CodeBlock>, chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for block in blocks {
        let block_str = format_block(&block);

        // If single block is too large, split it
        if block_str.len() > chunk_size {
            // Save current chunk if not empty
            if !current_chunk.is_empty() {
                chunks.push(current_chunk);
                current_chunk = String::new();
            }

            // Split large block
            let parts = split_large_block(&block_str, chunk_size);
            chunks.extend(parts);
        }
        // If adding block would exceed limit
        else if !current_chunk.is_empty() && current_chunk.len() + block_str.len() + 2 > chunk_size {
            chunks.push(current_chunk);
            current_chunk = block_str;
        }
        // Add to current chunk
        else {
            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(&block_str);
        }
    }

    // Save last chunk
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

/// Format a code block as string
fn format_block(block: &CodeBlock) -> String {
    format!(
        "# {} {} (lines {}-{})\n{}",
        match block.block_type {
            BlockType::Class => "Class",
            BlockType::Function => "Function",
            BlockType::Method => "Method",
            BlockType::Interface => "Interface",
            BlockType::Struct => "Struct",
            BlockType::Enum => "Enum",
            BlockType::Module => "Module",
            BlockType::Namespace => "Namespace",
            BlockType::Import => "Import",
            BlockType::Global => "Global",
            BlockType::Comment => "Comment",
        },
        block.name,
        block.start_line + 1,
        block.end_line + 1,
        block.content
    )
}

/// Split a large block into smaller chunks
fn split_large_block(block_str: &str, chunk_size: usize) -> Vec<String> {
    let mut parts = Vec::new();
    let lines: Vec<&str> = block_str.lines().collect();
    let header = lines[0]; // Keep the header with each part

    let mut current_part = header.to_string();

    for line in lines.iter().skip(1) {
        if current_part.len() + line.len() + 1 > chunk_size {
            parts.push(current_part);
            current_part = format!("{} (continued)\n{}", header, line);
        } else {
            current_part.push('\n');
            current_part.push_str(line);
        }
    }

    if !current_part.is_empty() && current_part != header {
        parts.push(current_part);
    }

    parts
}