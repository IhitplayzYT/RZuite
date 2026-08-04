pub mod Helper{
 

use std::{fs, path::PathBuf};

use regex::Regex;

pub fn non_pub_decl(source: &str) -> Vec<usize>{
    let re = Regex::new(
        r"(?ms)^(?:\s*#\[[^\]]*]\s*\n)*(?!\s*pub\b)\s*(?:fn|struct)\b",
    )
    .unwrap();

    re.find_iter(source).map(|m| m.start()).collect()
}

pub fn find_src_dir() -> std::path::PathBuf {
    let mut current = std::env::current_dir().unwrap();
    
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            let src_dir = current.join("src");
            if src_dir.exists() {
                return src_dir;
            }
        }
        
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => panic!("Could not find Rust project src directory"),
        }
    }
}


#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub full_path: String,
    pub symbols: Vec<String>,
    pub is_used: bool,
}

#[derive(Debug)]
pub struct FileDeps {
    pub file_path: String,
    pub imports: Vec<ImportInfo>,
}

pub fn parse_imports(content: &str) -> Vec<ImportInfo> {
    let mut imports = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("use ") {
            let import_line = line.strip_prefix("use ").unwrap();
            let import_line = import_line.trim_end_matches(';');
            let symbols = extract_symbols_from_import(import_line);
            let full_path = import_line.to_string();
            imports.push(ImportInfo {full_path,symbols,is_used: false});
        }
    }
    imports
}

pub fn extract_symbols_from_import(import: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    // Handles braced imports
    let flat = flatten_nested_imports(import);

    for prt in flat.split(',') {
        let prt = prt.trim();
        if !prt.is_empty() {
            if let Some(lpos) = prt.rfind("::") {
                let imprt = prt[lpos + 2..].trim();
                if !imprt.is_empty() && imprt != "{" && imprt != "}" {
                    symbols.push(imprt.to_string());
                }
            } else {
                let imprt = prt.trim();
                if !imprt.is_empty() && imprt != "{" && imprt != "}" {
                    symbols.push(imprt.to_string());
                }
            }
        }
    }
    if symbols.is_empty() {
        if let Some(last_colon) = import.rfind("::") {
            symbols.push(import[last_colon + 2..].trim().to_string());
        } else {
            symbols.push(import.trim().to_string());
        }
    }
    symbols
}

pub fn flatten_nested_imports(import: &str) -> String {
    let mut result = String::new();
    let mut stack = Vec::new();
    let mut current = String::new();
    let mut chars = import.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                if !current.trim().is_empty() {
                    stack.push(current.clone());
                    current.clear();
                }
                result.push('{');
            }
            '}' => {
                if let Some(prefix) = stack.pop() {
                    if !current.trim().is_empty() {
                        if !result.is_empty() && result.ends_with('{') {
                        } else if result.ends_with(',') {
                        } else {
                            result.push(',');
                        }
                        result.push_str(&prefix);
                        if !prefix.ends_with("::") {
                            result.push_str("::");
                        }
                        result.push_str(&current.trim());
                    }
                    current.clear();
                }
                result.push('}');
            }
            ',' => {
                if !current.trim().is_empty() {
                    if !result.is_empty() && !result.ends_with('{') && !result.ends_with(',') {
                        result.push(',');
                    }
                    result.push_str(&current.trim());
                    current.clear();
                }
                result.push(',');
            }
            ':' => {
                if chars.peek() == Some(&':') {
                    chars.next();
                    current.push_str("::");
                } else {
                    current.push(ch);
                }
            }
            ' ' | '\t' => {
            }
            _ => {
                current.push(ch);
            }
        }
    }
    
    if !current.trim().is_empty() {
        if !result.is_empty() && !result.ends_with('{') && !result.ends_with(',') {
            result.push(',');
        }
        result.push_str(&current.trim());
    }
    
    result.replace('{', "").replace('}', "")
}

pub fn check_import_usage(imports: &mut Vec<ImportInfo>, content: &str) {
    for import in imports {
        import.is_used = import.symbols.iter().any(|symbol| {
            content.contains(symbol.as_str())
        });
    }
}


pub fn build_dep_tree(files: Vec<FileDeps>) -> String {
    let mut tree = String::new();
    for file_deps in &files {
        tree.push_str(&format!("{}\n", file_deps.file_path));
        
        for import in &file_deps.imports {
            if import.is_used {
                tree.push_str(&format!("  ├── {} (used)\n", import.full_path));
                for symbol in &import.symbols {
                    tree.push_str(&format!("  │   └── {}\n", symbol));
                }
            }
        }
        tree.push_str("\n");
    }
    tree
}

pub fn traverse(src: PathBuf) -> Vec<FileDeps> {
    let mut ret = Vec::new();
    fs::read_dir(&src).unwrap().for_each(| entries| {
        let ent = entries.unwrap();
        if let Ok(ft) = ent.file_type() {
            if ft.is_dir() {
                ret.extend(traverse(ent.path()));
            }else {
                if ent.file_name() != "mod.rs" && ent.path().extension().and_then(|s| s.to_str()) == Some("rs") {
                    if let Ok(body) = fs::read_to_string(ent.path()) {
                        let mut imports = parse_imports(&body);
                        check_import_usage(&mut imports, &body);
                        ret.push(FileDeps {file_path: ent.path().to_string_lossy().to_string(),imports});
                    }
                }
            }
        }
    });
    
    ret
}


}