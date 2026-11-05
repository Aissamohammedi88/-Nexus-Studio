//! État applicatif Nexus Studio
//!
//! Orchestre les 4 couches :
//!   1. Couche UI (ratatui) — affichage
//!   2. Couche Runtime (éditeur) — manipulation de texte
//!   3. Couche Modèle (MockLLM) — génération
//!   4. Couche Souveraine (ChaimaLayer) — commandement
//!
//! Toute action utilisateur passe par la Couche Souveraine AVANT exécution.
//! Règle 2 : si la règle n'autorise pas, l'action n'existe pas (Zéro-Path).

use crate::chaima::ChaimaLayer;
use crate::editor::{Buffer, Cursor};
use crate::llm::MockLLM;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub action: String,
    pub verdict_authorized: bool,
    pub reason: String,
}

pub struct App {
    pub buffer: Buffer,
    pub cursor: Cursor,
    pub mode: Mode,
    pub chaima: ChaimaLayer,
    pub llm: MockLLM,
    pub command_input: String,
    pub log: Vec<LogEntry>,
    pub should_quit: bool,
    pub scroll_offset: usize,
    pub status_message: String,
    pub last_suggestion: Option<String>,
    /// Largeur du panneau Chaima (en caractères)
    pub chaima_panel_width: u16,
}

impl App {
    pub fn new(initial_path: Option<&std::path::Path>) -> Self {
        let buffer = if let Some(path) = initial_path {
            Buffer::load(&path.to_path_buf()).unwrap_or_else(|e| {
                let mut b = Buffer::new();
                b.lines[0] = format!("// Erreur chargement {} : {}", path.display(), e);
                b
            })
        } else {
            let mut b = Buffer::new();
            b.lines = vec![
                "// Nexus Studio v1.0 — Protocole Chaima Architect".to_string(),
                "// IDE souverain — Découplage Logique/Modèle, Zéro-Path, Zéro-Delta".to_string(),
                "//".to_string(),
                "// Modes :".to_string(),
                "//   i      — mode insertion".to_string(),
                "//   Esc    — mode normal".to_string(),
                "//   :      — barre de commande".to_string(),
                "//   :w     — sauvegarder".to_string(),
                "//   :q     — quitter".to_string(),
                "//   :chaima — voir règles souveraines".to_string(),
                "//   :llm   — demander suggestion IA".to_string(),
                "//   Tab   — suggestion IA suivante".to_string(),
                "//".to_string(),
                "// Couche souveraine active — chaque action est vérifiée.".to_string(),
                String::new(),
            ];
            b
        };

        App {
            buffer,
            cursor: Cursor::new(),
            mode: Mode::Normal,
            chaima: ChaimaLayer::new(),
            llm: MockLLM::new(),
            command_input: String::new(),
            log: Vec::new(),
            should_quit: false,
            scroll_offset: 0,
            status_message: "Nexus Studio prêt — couche Chaima active".to_string(),
            last_suggestion: None,
            chaima_panel_width: 38,
        }
    }

    /// Évalue une action via la couche Chaima avant exécution.
    /// Si Zéro-Path, l'action est simplement ignorée — pas d'erreur visible.
    pub fn authorize(&mut self, action: &str, context: &str) -> bool {
        let verdict = self.chaima.evaluate_action(action, context);
        let authorized = verdict.is_authorized();

        self.log.push(LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            action: action.to_string(),
            verdict_authorized: authorized,
            reason: verdict.reason().to_string(),
        });

        // Garder seulement les 100 dernières entrées
        if self.log.len() > 100 {
            let drop = self.log.len() - 100;
            self.log.drain(0..drop);
        }

        if authorized {
            self.status_message = format!("✓ {} — chemin souverain vérifié", action);
        } else {
            self.status_message = format!("⊘ {} — Zéro-Path : action inexistante", action);
        }

        authorized
    }

    // ========================================================================
    // HANDLERS — MODE NORMAL
    // ========================================================================

    pub fn handle_normal(&mut self, key: &str) {
        let action = match key {
            "i" => Some(("insert_mode", "enter_insert")),
            "h" => Some(("cursor_move(left)", "move_left")),
            "l" => Some(("cursor_move(right)", "move_right")),
            "k" => Some(("cursor_move(up)", "move_up")),
            "j" => Some(("cursor_move(down)", "move_down")),
            "0" => Some(("cursor_move(line_start)", "move_line_start")),
            "$" => Some(("cursor_move(line_end)", "move_line_end")),
            "G" => Some(("cursor_move(buffer_end)", "move_buffer_end")),
            ":" => Some(("open_command_bar", "open_cmd")),
            "q" => Some(("quit_application", "quit")),
            _ => None,
        };

        if let Some((action_str, handler)) = action {
            if self.authorize(action_str, &self.buffer.full_text()) {
                match handler {
                    "enter_insert" => self.mode = Mode::Insert,
                    "move_left" => self.cursor.move_left(&self.buffer),
                    "move_right" => self.cursor.move_right(&self.buffer),
                    "move_up" => self.cursor.move_up(&self.buffer),
                    "move_down" => self.cursor.move_down(&self.buffer),
                    "move_line_start" => self.cursor.move_line_start(),
                    "move_line_end" => self.cursor.move_line_end(&self.buffer),
                    "move_buffer_end" => {
                        self.cursor.row = self.buffer.line_count().saturating_sub(1);
                        self.cursor.move_line_end(&self.buffer);
                    }
                    "open_cmd" => {
                        self.mode = Mode::Command;
                        self.command_input.clear();
                    }
                    "quit" => self.should_quit = true,
                    _ => {}
                }
            }
        }
    }

    // ========================================================================
    // HANDLERS — MODE INSERT
    // ========================================================================

    pub fn handle_insert_char(&mut self, c: char) {
        if self.authorize("buffer_insert(char)", &self.buffer.full_text()) {
            if c == '\n' {
                self.buffer.insert_newline(self.cursor.row, self.cursor.col);
                self.cursor.row += 1;
                self.cursor.col = 0;
            } else {
                self.buffer.insert_char(self.cursor.row, self.cursor.col, c);
                self.cursor.col += 1;
            }
        }
    }

    pub fn handle_insert_backspace(&mut self) {
        if self.authorize("buffer_delete", &self.buffer.full_text()) {
            if self.cursor.col > 0 {
                self.cursor.col -= 1;
                self.buffer.delete_char(self.cursor.row, self.cursor.col);
            } else if self.cursor.row > 0 {
                self.cursor.row -= 1;
                self.cursor.col = self.buffer.line(self.cursor.row).len();
                self.buffer.delete_char(self.cursor.row, self.cursor.col);
            }
        }
    }

    pub fn handle_insert_escape(&mut self) {
        if self.authorize("cursor_move(escape)", "escape_insert") {
            self.mode = Mode::Normal;
        }
    }

    // ========================================================================
    // HANDLERS — MODE COMMAND
    // ========================================================================

    pub fn handle_command_char(&mut self, c: char) {
        self.command_input.push(c);
    }

    pub fn handle_command_backspace(&mut self) {
        self.command_input.pop();
    }

    pub fn handle_command_enter(&mut self) {
        let cmd = self.command_input.clone();
        self.command_input.clear();
        self.mode = Mode::Normal;

        let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
        let command = parts[0];
        let arg = parts.get(1).copied().unwrap_or("");

        match command {
            "q" | "quit" => {
                if self.authorize("quit_application", "user command") {
                    self.should_quit = true;
                }
            }
            "w" | "write" => {
                if self.authorize("save_file", &self.buffer.full_text()) {
                    if !arg.is_empty() {
                        let path = std::path::PathBuf::from(arg);
                        match self.buffer.save_as(path) {
                            Ok(_) => self.status_message = format!("✓ Sauvegardé : {}", arg),
                            Err(e) => self.status_message = format!("✗ Erreur : {}", e),
                        }
                    } else {
                        match self.buffer.save() {
                            Ok(_) => self.status_message = format!("✓ Sauvegardé : {}", self.buffer.filename()),
                            Err(e) => self.status_message = format!("✗ Erreur : {}", e),
                        }
                    }
                }
            }
            "wq" => {
                if self.authorize("save_file", &self.buffer.full_text()) {
                    let _ = self.buffer.save();
                }
                if self.authorize("quit_application", "wq") {
                    self.should_quit = true;
                }
            }
            "chaima" => {
                self.status_message = format!(
                    "Chaima — autorisés: {}, zéro-path: {}, intentions: {}",
                    self.chaima.metrics().0,
                    self.chaima.metrics().1,
                    self.chaima.intent_chain().len()
                );
            }
            "llm" | "suggest" => {
                let context = if self.buffer.line_count() > 0 {
                    self.buffer.line(self.cursor.row).to_string()
                } else {
                    String::new()
                };
                let suggestion = self.llm.suggest(&context, &mut self.chaima);
                self.last_suggestion = Some(suggestion.code.clone());
                self.log.push(LogEntry {
                    timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                    action: format!("llm_suggest({})", &suggestion.code[..suggestion.code.len().min(40)]),
                    verdict_authorized: suggestion.verdict.is_authorized(),
                    reason: suggestion.verdict.reason().to_string(),
                });
                self.status_message = format!(
                    "LLM → {:?} — {}",
                    suggestion.verdict,
                    &suggestion.code[..suggestion.code.len().min(50)]
                );
            }
            "help" => {
                self.status_message = "Commandes : :w :q :wq :chaima :llm :help".to_string();
            }
            "" => {
                // commande vide — juste revenir
            }
            _ => {
                self.status_message = format!("Commande inconnue : {}", command);
            }
        }
    }

    pub fn handle_command_escape(&mut self) {
        self.command_input.clear();
        self.mode = Mode::Normal;
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    pub fn ensure_cursor_visible(&mut self, visible_rows: usize) {
        if self.cursor.row < self.scroll_offset {
            self.scroll_offset = self.cursor.row;
        } else if self.cursor.row >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.cursor.row.saturating_sub(visible_rows.saturating_sub(1));
        }
    }
}
