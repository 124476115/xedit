use std::path::{Path, PathBuf};
use std::fs;
use crate::editor::FileType;

pub fn detect_file_type(path: &Path) -> FileType {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();

    match ext.as_str() {
        "md" | "markdown" | "mdown" | "mkd" => FileType::Markdown,
        "xml" | "xsd" | "xsl" | "xslt" | "svg" | "plist" | "config" => FileType::Xml,
        "json" | "jsonc" | "json5" => FileType::Json,
        "env" => FileType::Env,
        "properties" | "props" | "ini" | "cfg" | "conf" => FileType::Properties,
        _ => {
            if name == ".env" || name.starts_with(".env.") {
                FileType::Env
            } else {
                detect_by_content(path).unwrap_or(FileType::Text)
            }
        }
    }
}

fn detect_by_content(path: &Path) -> Option<FileType> {
    let content = fs::read_to_string(path).ok()?;
    let content = content.trim();

    if content.is_empty() {
        return Some(FileType::Text);
    }

    if content.starts_with("<?xml") || content.starts_with('<') {
        return Some(FileType::Xml);
    }

    if content.starts_with('{') || content.starts_with('[') {
        return Some(FileType::Json);
    }

    let lines: Vec<&str> = content.lines().collect();
    let env_count = lines.iter().filter(|l| {
        let l = l.trim();
        !l.is_empty() && !l.starts_with('#') && l.contains('=')
    }).count();

    if env_count > 0 && (env_count as f32 / lines.len() as f32) > 0.5 {
        return Some(FileType::Env);
    }

    let prop_count = lines.iter().filter(|l| {
        let l = l.trim();
        !l.is_empty() && !l.starts_with('#') && !l.starts_with('!') && (l.contains('=') || l.contains(':'))
    }).count();

    if prop_count > 0 && (prop_count as f32 / lines.len() as f32) > 0.5 {
        return Some(FileType::Properties);
    }

    let md_indicators = ["# ", "## ", "### ", "- ", "* ", "`", "|", "["];
    if md_indicators.iter().any(|ind| content.starts_with(ind)) {
        return Some(FileType::Markdown);
    }

    Some(FileType::Text)
}

pub fn read_file(path: &Path) -> anyhow::Result<(String, String)> {
    let content = fs::read_to_string(path)?;
    let encoding = detect_encoding(path)?;
    Ok((content, encoding))
}

fn detect_encoding(path: &Path) -> anyhow::Result<String> {
    let bytes = fs::read(path)?;
    let encoding = encoding_rs::Encoding::for_bom(&bytes).map(|e| e.name().to_string());
    if encoding.is_some() {
        return Ok(encoding.unwrap());
    }

    let encodings = ["utf-8", "gbk", "gb2312", "latin-1", "cp1252"];
    for enc in encodings {
        if encoding_rs::Encoding::for_label(enc.as_bytes()).is_some() {
            if encoding_rs::decode(&bytes, encoding_rs::Encoding::for_label(enc.as_bytes()).unwrap()).0.is_some() {
                return Ok(enc.to_string());
            }
        }
    }

    Ok("utf-8".to_string())
}

pub fn write_file(path: &Path, content: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}