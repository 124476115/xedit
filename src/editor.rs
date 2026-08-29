use egui::{Color32, FontId, RichText, Ui};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style as SyntectStyle};
use syntect::util::LinesWithEndings;

use super::file_utils::{detect_file_type, read_file, write_file};

pub type TabId = usize;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FileType {
    Markdown,
    Xml,
    Json,
    Env,
    Properties,
    Text,
}

impl FileType {
    pub fn icon(&self) -> &'static str {
        match self {
            FileType::Markdown => "📝",
            FileType::Xml => "📄",
            FileType::Json => "📋",
            FileType::Env => "🔐",
            FileType::Properties => "⚙️",
            FileType::Text => "📄",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            FileType::Markdown => "md",
            FileType::Xml => "xml",
            FileType::Json => "json",
            FileType::Env => "env",
            FileType::Properties => "properties",
            FileType::Text => "txt",
        }
    }

    pub fn syntax_name(&self) -> &'static str {
        match self {
            FileType::Markdown => "Markdown",
            FileType::Xml => "XML",
            FileType::Json => "JSON",
            FileType::Env => "Shell-Unix-Generic",
            FileType::Properties => "Properties",
            FileType::Text => "Plain Text",
        }
    }
}

pub struct EditorTab {
    pub id: TabId,
    pub file_path: Option<PathBuf>,
    pub file_type: FileType,
    pub content: String,
    pub modified: bool,
    pub font_size: f32,
    pub cursor_line: usize,
    pub cursor_col: usize,
    highlighter: Arc<Mutex<Highlighter>>,
}

struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_syntax: Option<syntect::parsing::SyntaxReference>,
    current_theme: String,
}

impl Highlighter {
    fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        Self {
            syntax_set,
            theme_set,
            current_syntax: None,
            current_theme: "base16-ocean.dark".to_string(),
        }
    }

    fn set_syntax(&mut self, syntax_name: &str) {
        self.current_syntax = self.syntax_set.find_syntax_by_name(syntax_name);
    }

    fn highlight(&mut self, text: &str) -> Vec<(String, Color32)> {
        let syntax = self.current_syntax.as_ref()
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let theme = &self.theme_set.themes[&self.current_theme];
        let mut highlighter = HighlightLines::new(syntax, theme);

        let mut result = Vec::new();
        for line in LinesWithEndings::from(text) {
            let ranges = highlighter.highlight_line(line, &self.syntax_set).unwrap_or_default();
            for (style, text) in ranges {
                let color = syntect_style_to_egui(style);
                result.push((text.to_string(), color));
            }
        }
        result
    }
}

fn syntect_style_to_egui(style: SyntectStyle) -> Color32 {
    Color32::from_rgb(style.foreground.r, style.foreground.g, style.foreground.b)
}

impl EditorTab {
    pub fn new(id: TabId, font_size: f32) -> Self {
        Self {
            id,
            file_path: None,
            file_type: FileType::Text,
            content: String::new(),
            modified: false,
            font_size,
            cursor_line: 1,
            cursor_col: 1,
            highlighter: Arc::new(Mutex::new(Highlighter::new())),
        }
    }

    pub fn display_name(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("未命名")
            .to_string()
    }

    pub fn load_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let (content, _encoding) = read_file(path)?;
        self.file_path = Some(path.to_path_buf());
        self.file_type = detect_file_type(path);
        self.content = content;
        self.modified = false;
        self.update_highlighter();
        Ok(())
    }

    fn update_highlighter(&mut self) {
        let mut hl = self.highlighter.lock().unwrap();
        hl.set_syntax(self.file_type.syntax_name());
    }

    pub fn save_file(&mut self) -> anyhow::Result<()> {
        if let Some(path) = &self.file_path {
            write_file(path, &self.content)?;
            self.modified = false;
            Ok(())
        } else {
            self.save_file_as()
        }
    }

    pub fn save_file_as(&mut self) -> anyhow::Result<()> {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(self.display_name())
            .add_filter("All Files", &["*"])
            .save_file()
        {
            self.file_path = Some(path.clone());
            self.file_type = detect_file_type(&path);
            self.update_highlighter();
            write_file(&path, &self.content)?;
            self.modified = false;
        }
        Ok(())
    }

    pub fn undo(&mut self) {
    }

    pub fn redo(&mut self) {
    }

    fn line_col_from_index(text: &str, idx: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for (i, c) in text.char_indices() {
            if i >= idx {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    pub fn show(&mut self, ui: &mut Ui, font_size: &mut f32) {
        self.font_size = *font_size;

        let font_id = FontId::monospace(self.font_size);
        let mut layouter = |ui: &Ui, text: &str, _wrap_width: f32| {
            let mut hl = self.highlighter.lock().unwrap();
            let highlighted = hl.highlight(text);
            let mut job = egui::text::LayoutJob::default();
            for (txt, color) in highlighted {
                let format = egui::TextFormat::simple(egui::FontId::monospace(self.font_size), color);
                job.append(&txt, 0.0, format);
            }
            ui.fonts(|f| f.layout_job(job))
        };

        let output = egui::TextEdit::multiline(&mut self.content)
            .font(font_id)
            .layouter(&mut layouter)
            .desired_rows(40)
            .desired_width(f32::INFINITY)
            .lock_focus(true)
            .show(ui);

        if output.response.changed() {
            self.modified = true;
        }

        if let Some(cursor_range) = output.cursor_range {
            let (line, col) = line_col_from_index(&self.content, cursor_range.primary.ccursor.index);
            self.cursor_line = line;
            self.cursor_col = col;
        }

        *font_size = self.font_size;
    }
}