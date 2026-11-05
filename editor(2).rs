//! Éditeur de texte — buffer, curseur, syntaxe, I/O fichiers
//!
//! Performance brute : opérations O(1) sur lignes, gap buffer conceptuel.
//! Pas de reflow complet, pas de recalcul global.

use std::fs;
use std::path::PathBuf;

// ============================================================================
// BUFFER DE TEXTE
// ============================================================================

#[derive(Debug, Clone)]
pub struct Buffer {
    pub lines: Vec<String>,
    pub path: Option<PathBuf>,
    pub modified: bool,
    pub language: Language,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    Rust,
    Python,
    Prolog,
    Text,
    Markdown,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            lines: vec![String::new()],
            path: None,
            modified: false,
            language: Language::Text,
        }
    }

    pub fn from_text(text: &str, path: Option<PathBuf>) -> Self {
        let lines: Vec<String> = if text.is_empty() {
            vec![String::new()]
        } else {
            text.split('\n').map(|s| s.to_string()).collect()
        };
        let language = path.as_ref().map(|p| detect_language(p)).unwrap_or(Language::Text);
        Buffer {
            lines,
            path,
            modified: false,
            language,
        }
    }

    pub fn load(path: &PathBuf) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("Lecture impossible : {}", e))?;
        Ok(Buffer::from_text(&content, Some(path.clone())))
    }

    pub fn save(&mut self) -> Result<(), String> {
        if let Some(path) = &self.path {
            let content = self.lines.join("\n");
            fs::write(path, &content).map_err(|e| format!("Écriture impossible : {}", e))?;
            self.modified = false;
            Ok(())
        } else {
            Err("Aucun chemin associé au buffer".to_string())
        }
    }

    pub fn save_as(&mut self, path: PathBuf) -> Result<(), String> {
        let content = self.lines.join("\n");
        fs::write(&path, &content).map_err(|e| format!("Écriture impossible : {}", e))?;
        self.path = Some(path.clone());
        self.language = detect_language(&path);
        self.modified = false;
        Ok(())
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn line(&self, idx: usize) -> &str {
        self.lines.get(idx).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn insert_char(&mut self, row: usize, col: usize, c: char) {
        if row >= self.lines.len() {
            return;
        }
        let line = &mut self.lines[row];
        if col <= line.len() {
            line.insert(col, c);
            self.modified = true;
        }
    }

    pub fn insert_str(&mut self, row: usize, col: usize, s: &str) {
        if row >= self.lines.len() {
            return;
        }
        let line = &mut self.lines[row];
        if col <= line.len() {
            line.insert_str(col, s);
            self.modified = true;
        }
    }

    pub fn delete_char(&mut self, row: usize, col: usize) {
        if row >= self.lines.len() {
            return;
        }
        let line = &mut self.lines[row];
        if col < line.len() {
            // Suppression dans la ligne
            let mut new_line = line.clone();
            new_line.remove(col);
            self.lines[row] = new_line;
            self.modified = true;
        } else if row + 1 < self.lines.len() {
            // Fusion avec ligne suivante
            let next = self.lines[row + 1].clone();
            self.lines[row].push_str(&next);
            self.lines.remove(row + 1);
            self.modified = true;
        }
    }

    pub fn insert_newline(&mut self, row: usize, col: usize) {
        if row >= self.lines.len() {
            return;
        }
        let line = self.lines[row].clone();
        let (before, after) = line.split_at(col.min(line.len()));
        self.lines[row] = before.to_string();
        self.lines.insert(row + 1, after.to_string());
        self.modified = true;
    }

    pub fn delete_line(&mut self, row: usize) {
        if row < self.lines.len() {
            if self.lines.len() == 1 {
                self.lines[0] = String::new();
            } else {
                self.lines.remove(row);
            }
            self.modified = true;
        }
    }

    pub fn full_text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn filename(&self) -> String {
        self.path.as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "[sans nom]".to_string())
    }
}

fn detect_language(path: &PathBuf) -> Language {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => Language::Rust,
        Some("py") => Language::Python,
        Some("pl" | "prolog") => Language::Prolog,
        Some("md") => Language::Markdown,
        _ => Language::Text,
    }
}

// ============================================================================
// CURSEUR
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Cursor { row: 0, col: 0 }
    }

    pub fn move_left(&mut self, buffer: &Buffer) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.row > 0 {
            self.row -= 1;
            self.col = buffer.line(self.row).len();
        }
    }

    pub fn move_right(&mut self, buffer: &Buffer) {
        let current_len = buffer.line(self.row).len();
        if self.col < current_len {
            self.col += 1;
        } else if self.row + 1 < buffer.line_count() {
            self.row += 1;
            self.col = 0;
        }
    }

    pub fn move_up(&mut self, buffer: &Buffer) {
        if self.row > 0 {
            self.row -= 1;
            let new_len = buffer.line(self.row).len();
            if self.col > new_len {
                self.col = new_len;
            }
        }
    }

    pub fn move_down(&mut self, buffer: &Buffer) {
        if self.row + 1 < buffer.line_count() {
            self.row += 1;
            let new_len = buffer.line(self.row).len();
            if self.col > new_len {
                self.col = new_len;
            }
        }
    }

    pub fn move_line_start(&mut self) {
        self.col = 0;
    }

    pub fn move_line_end(&mut self, buffer: &Buffer) {
        self.col = buffer.line(self.row).len();
    }

    pub fn page_up(&mut self, buffer: &Buffer, page_size: usize) {
        self.row = self.row.saturating_sub(page_size);
        let new_len = buffer.line(self.row).len();
        if self.col > new_len {
            self.col = new_len;
        }
    }

    pub fn page_down(&mut self, buffer: &Buffer, page_size: usize) {
        self.row = (self.row + page_size).min(buffer.line_count().saturating_sub(1));
        let new_len = buffer.line(self.row).len();
        if self.col > new_len {
            self.col = new_len;
        }
    }
}

// ============================================================================
// COLORATION SYNTAXIQUE (simplifiée, par regex manuelle)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Keyword,
    String,
    Comment,
    Number,
    Function,
    Type,
    Operator,
    Punctuation,
    Text,
}

#[derive(Debug, Clone)]
pub struct HighlightToken {
    pub kind: TokenKind,
    pub text: String,
}

pub fn tokenize_line(line: &str, language: Language) -> Vec<HighlightToken> {
    match language {
        Language::Rust => tokenize_rust(line),
        Language::Python => tokenize_python(line),
        Language::Prolog => tokenize_prolog(line),
        _ => vec![HighlightToken { kind: TokenKind::Text, text: line.to_string() }],
    }
}

fn keywords_token(line: &str, keywords: &[&str], type_words: &[&str]) -> Vec<HighlightToken> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Commentaire
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            tokens.push(HighlightToken { kind: TokenKind::Comment, text: chars[i..].iter().collect() });
            break;
        }

        // Chaîne
        if c == '"' {
            let mut s = String::new();
            s.push(c);
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                s.push(chars[i]);
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 1;
                    s.push(chars[i]);
                }
                i += 1;
            }
            if i < chars.len() {
                s.push(chars[i]);
                i += 1;
            }
            tokens.push(HighlightToken { kind: TokenKind::String, text: s });
            continue;
        }

        // Nombre
        if c.is_ascii_digit() {
            let mut s = String::new();
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                s.push(chars[i]);
                i += 1;
            }
            tokens.push(HighlightToken { kind: TokenKind::Number, text: s });
            continue;
        }

        // Identificateur / mot-clé
        if c.is_alphabetic() || c == '_' {
            let mut s = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                s.push(chars[i]);
                i += 1;
            }
            if keywords.contains(&s.as_str()) {
                tokens.push(HighlightToken { kind: TokenKind::Keyword, text: s });
            } else if type_words.contains(&s.as_str()) {
                tokens.push(HighlightToken { kind: TokenKind::Type, text: s });
            } else if i < chars.len() && chars[i] == '(' {
                tokens.push(HighlightToken { kind: TokenKind::Function, text: s });
            } else {
                tokens.push(HighlightToken { kind: TokenKind::Text, text: s });
            }
            continue;
        }

        // Opérateur / ponctuation
        if "+-*/%=<>!&|^~".contains(c) {
            tokens.push(HighlightToken { kind: TokenKind::Operator, text: c.to_string() });
            i += 1;
            continue;
        }
        if "()[]{},;:.".contains(c) {
            tokens.push(HighlightToken { kind: TokenKind::Punctuation, text: c.to_string() });
            i += 1;
            continue;
        }

        // Texte par défaut
        tokens.push(HighlightToken { kind: TokenKind::Text, text: c.to_string() });
        i += 1;
    }

    tokens
}

fn tokenize_rust(line: &str) -> Vec<HighlightToken> {
    keywords_token(line, &RUST_KEYWORDS, &RUST_TYPES)
}

fn tokenize_python(line: &str) -> Vec<HighlightToken> {
    // Commentaires Python : #
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        return vec![HighlightToken { kind: TokenKind::Comment, text: line.to_string() }];
    }
    keywords_token(line, &PYTHON_KEYWORDS, &[])
}

fn tokenize_prolog(line: &str) -> Vec<HighlightToken> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('%') {
        return vec![HighlightToken { kind: TokenKind::Comment, text: line.to_string() }];
    }
    keywords_token(line, &PROLOG_KEYWORDS, &[])
}

const RUST_KEYWORDS: &[&str] = &[
    "fn", "let", "mut", "if", "else", "match", "while", "for", "loop", "return",
    "struct", "enum", "impl", "trait", "pub", "use", "mod", "self", "Self", "crate",
    "as", "where", "ref", "move", "static", "const", "unsafe", "async", "await", "dyn",
    "break", "continue", "in",
];

const RUST_TYPES: &[&str] = &[
    "String", "Vec", "Option", "Result", "Box", "Rc", "Arc", "i32", "i64", "u32", "u64",
    "f32", "f64", "bool", "char", "str", "usize", "isize",
];

const PYTHON_KEYWORDS: &[&str] = &[
    "def", "class", "if", "elif", "else", "for", "while", "return", "import",
    "from", "as", "try", "except", "finally", "with", "lambda", "yield", "async",
    "await", "pass", "break", "continue", "in", "not", "and", "or", "is", "None",
    "True", "False", "self",
];

const PROLOG_KEYWORDS: &[&str] = &[
    ":-", "?-", "not", "true", "fail", "findall", "assertz", "retract",
];

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_insert_delete() {
        let mut buf = Buffer::new();
        buf.insert_char(0, 0, 'h');
        buf.insert_char(0, 1, 'i');
        assert_eq!(buf.line(0), "hi");
        buf.delete_char(0, 1);
        assert_eq!(buf.line(0), "h");
    }

    #[test]
    fn test_cursor_movement() {
        let buf = Buffer::from_text("hello\nworld", None);
        let mut cur = Cursor::new();
        cur.move_right(&buf);
        assert_eq!(cur.col, 1);
        cur.move_down(&buf);
        assert_eq!(cur.row, 1);
        assert_eq!(cur.col, 1);
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language(&PathBuf::from("foo.rs")), Language::Rust);
        assert_eq!(detect_language(&PathBuf::from("foo.py")), Language::Python);
        assert_eq!(detect_language(&PathBuf::from("foo.pl")), Language::Prolog);
    }
}
