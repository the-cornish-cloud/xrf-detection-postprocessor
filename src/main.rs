use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use std::io::Write;


fn main() {
    // Get the current directory where the executable is running
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    
    // Walk through all files in current directory and subdirectories
    for entry in WalkDir::new(current_dir)
        .into_iter()
        .filter_map(|e| e.ok()) {
            
        let path = entry.path();
        
        // Check if the file has .qan or .txt extension
        if let Some(extension) = path.extension() {
            if extension == "qan" || extension == "txt" {
                process_file(path);
            }
        }
    }
}

fn process_file(path: &Path) {
    println!("Processing file: {}", path.display());
    
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            println!("Error reading file {}: {}", path.display(), e);
            return;
        }
    };

    match process_and_write_file(&content, path) {
        Ok(_) => println!("Successfully processed and updated file: {}", path.display()),
        Err(e) => println!("Error processing file {}: {}", path.display(), e),
    }
}

fn process_and_write_file(content: &str, output_path: &Path) -> Result<(), String> {
    // Detect original line ending style (Windows CRLF or Unix LF)
    let line_ending = if content.contains("\r\n") { "\r\n" } else { "\n" };
    
    let mut output_lines = Vec::new();
    // Split by both CRLF and LF to handle both cases
    let lines: Vec<&str> = content.split(&['\r', '\n'][..]).filter(|s| !s.is_empty()).collect();
    
    for line in lines {
        if line.is_empty() {
            output_lines.push(String::new());
            continue;
        }

        // Check if line contains a measurement
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && !line.starts_with("Sample") && 
           !line.starts_with("Application") && !line.starts_with("Measurement") &&
           !line.starts_with("Initial") && !line.starts_with("Final") {
            
            if let Ok(value) = parts[1].parse::<f64>() {
                let element = parts[0];
                let unit = parts[2];
                let formatted_value = format_value(value, unit);
                
                // Create new line with fixed column positions
                let mut new_line = String::new();
                new_line.push_str(element);
                
                // Pad to column 18 for the value
                while new_line.len() < 17 {
                    new_line.push(' ');
                }
                new_line.push_str(&formatted_value);
                
                // Pad to column 34 for the unit
                while new_line.len() < 33 {
                    new_line.push(' ');
                }
                new_line.push_str(unit);
                
                output_lines.push(new_line);
                continue;
            }
        }
        
        // If not a measurement line or parsing failed, keep original line
        output_lines.push(line.to_string());
    }

    // Write to file with original line endings
    let mut file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    for (i, line) in output_lines.iter().enumerate() {
        if i < output_lines.len() - 1 {
            write!(file, "{}{}", line, line_ending)
                .map_err(|e| format!("Failed to write to file: {}", e))?;
        } else {
            // Don't add line ending after the last line if original didn't have one
            if content.ends_with(line_ending) {
                write!(file, "{}{}", line, line_ending)
                    .map_err(|e| format!("Failed to write to file: {}", e))?;
            } else {
                write!(file, "{}", line)
                    .map_err(|e| format!("Failed to write to file: {}", e))?;
            }
        }
    }

    Ok(())
}

fn format_value(value: f64, unit: &str) -> String {
    match unit {
        "%" => {
            if value < 0.01 {
                "<0.01".to_string()
            } else {
                format!("{:.2}", value)
            }
        },
        "ppm" => {
            if value < 2.0 {
                "<2".to_string()
            } else {
                format!("{:.1}", value)
            }
        },
        _ => format!("{}", value)
    }
}