//! Rendu UI — ratatui
//!
//! Layout :
//!   ┌──────────────────────────────┬──────────────┐
//!   │  Header (Nexus Studio v1.0)                 │
//!   ├──────────────────────────────┼──────────────┤
//!   │                              │              │
//!   │   Éditeur (buffer + syntaxe) │  Panneau     │
//!   │                              │  Chaima      │
//!   │                              │  (log)       │
//!   │                              │              │
//!   ├──────────────────────────────┴──────────────┤
//!   │  Barre commande / status                     │
//!   └─────────────────────────────────────────────┘

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Mode};
use crate::editor::{tokenize_line, TokenKind};

// Palette Chaima — sobre, technique, souverain
const COLOR_BG: Color = Color::Black;
const COLOR_FG: Color = Color::Gray;
const COLOR_KEYWORD: Color = Color::Magenta;
const COLOR_STRING: Color = Color::Green;
const COLOR_COMMENT: Color = Color::DarkGray;
const COLOR_NUMBER: Color = Color::Yellow;
const COLOR_FUNCTION: Color = Color::Cyan;
const COLOR_TYPE: Color = Color::Blue;
const COLOR_OPERATOR: Color = Color::LightRed;
const COLOR_PUNCT: Color = Color::Gray;
const COLOR_ACCENT: Color = Color::Cyan;
const COLOR_AUTHORIZED: Color = Color::Green;
const COLOR_ZEROPATH: Color = Color::Red;
const COLOR_CURSOR: Color = Color::White;
const COLOR_HEADER_BG: Color = Color::DarkGray;

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Min(5),    // body
            Constraint::Length(3), // footer (command/status)
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_body(f, app, chunks[1]);
    render_footer(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let (authorized, zeropath) = app.chaima.metrics();
    let title = format!(
        " NEXUS STUDIO v1.0 — Protocole Chaima Architect  |  {}  |  Mode: {:?}  |  ✓ {}  ⊘ {}  |  Intentions: {} ",
        app.buffer.filename(),
        app.mode,
        authorized,
        zeropath,
        app.chaima.intent_chain().len(),
    );
    let header = Paragraph::new(title)
        .style(
            Style::default()
                .fg(COLOR_ACCENT)
                .bg(COLOR_HEADER_BG)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).style(Style::default().fg(COLOR_ACCENT)));
    f.render_widget(header, area);
}

fn render_body(f: &mut Frame, app: &mut App, area: Rect) {
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20),
            Constraint::Length(app.chaima_panel_width),
        ])
        .split(area);

    render_editor(f, app, body_chunks[0]);
    render_chaima_panel(f, app, body_chunks[1]);
}

fn render_editor(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} {} ", app.buffer.filename(), if app.buffer.modified { "[*]"} else {""}))
        .style(Style::default().fg(COLOR_FG));

    // Calculer la zone éditable (avec marges de bordure)
    let inner = block.inner(area);
    f.render_widget(block, area);

    let visible_height = inner.height as usize;
    app.ensure_cursor_visible(visible_height.saturating_sub(1));

    let line_count = app.buffer.line_count();
    let line_num_width = line_count.to_string().len().max(3) + 1;

    let editor_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(line_num_width as u16),
            Constraint::Min(1),
        ])
        .split(inner);

    // Numéros de ligne
    let mut lines_text = String::new();
    let start = app.scroll_offset;
    let end = (start + visible_height).min(line_count);
    for i in start..end {
        lines_text.push_str(&format!("{:>width$}\n", i + 1, width = line_num_width - 1));
    }
    let line_nums = Paragraph::new(lines_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Right);
    f.render_widget(line_nums, editor_chunks[0]);

    // Contenu éditable avec coloration syntaxique
    let mut content: Vec<Line> = Vec::new();
    for i in start..end {
        let line = app.buffer.line(i);
        let tokens = tokenize_line(line, app.buffer.language);
        let spans: Vec<Span> = tokens.iter().map(|t| Span::styled(t.text.clone(), token_style(t.kind))).collect();
        if spans.is_empty() {
            content.push(Line::from(""));
        } else {
            content.push(Line::from(spans));
        }
    }
    // Remplir les lignes manquantes
    while content.len() < visible_height {
        content.push(Line::from(""));
    }

    let content_para = Paragraph::new(content);
    f.render_widget(content_para, editor_chunks[1]);

    // Curseur
    let cursor_y = app.cursor.row.saturating_sub(app.scroll_offset);
    if cursor_y < visible_height {
        let cursor_x = app.cursor.col as u16 + line_num_width as u16 + 1; // +1 pour bordure
        f.set_cursor_position((
            area.x + cursor_x,
            area.y + 1 + cursor_y as u16,
        ));
    }
}

fn token_style(kind: TokenKind) -> Style {
    match kind {
        TokenKind::Keyword => Style::default().fg(COLOR_KEYWORD).add_modifier(Modifier::BOLD),
        TokenKind::String => Style::default().fg(COLOR_STRING),
        TokenKind::Comment => Style::default().fg(COLOR_COMMENT),
        TokenKind::Number => Style::default().fg(COLOR_NUMBER),
        TokenKind::Function => Style::default().fg(COLOR_FUNCTION),
        TokenKind::Type => Style::default().fg(COLOR_TYPE),
        TokenKind::Operator => Style::default().fg(COLOR_OPERATOR),
        TokenKind::Punctuation => Style::default().fg(COLOR_PUNCT),
        TokenKind::Text => Style::default().fg(COLOR_FG),
    }
}

fn render_chaima_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Couche Souveraine — Chaima ")
        .style(Style::default().fg(COLOR_ACCENT));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // métriques
            Constraint::Min(3),   // log
            Constraint::Length(4), // chaîne intentions
        ])
        .split(inner);

    // Métriques
    let (authorized, zeropath) = app.chaima.metrics();
    let metrics = format!(
        "Autorisés: {} (✓ vert)\nZéro-Path: {} (⊘ rouge)\nTotal intentions: {}",
        authorized,
        zeropath,
        app.chaima.intent_chain().len(),
    );
    let metrics_para = Paragraph::new(metrics)
        .style(Style::default().fg(COLOR_FG))
        .wrap(Wrap { trim: false });
    f.render_widget(metrics_para, inner_chunks[0]);

    // Journal des actions (log) — derniers N
    let log_entries: Vec<&crate::app::LogEntry> = app.log.iter().rev().take(inner_chunks[1].height as usize).collect();
    let mut log_lines: Vec<Line> = Vec::new();
    for entry in log_entries.iter().rev() {
        let icon = if entry.verdict_authorized { "✓" } else { "⊘" };
        let color = if entry.verdict_authorized { COLOR_AUTHORIZED } else { COLOR_ZEROPATH };
        let action_short = if entry.action.len() > 22 {
            format!("{}..", &entry.action[..20])
        } else {
            entry.action.clone()
        };
        log_lines.push(Line::from(vec![
            Span::styled(format!("{} ", entry.timestamp), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} ", icon), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(action_short, Style::default().fg(COLOR_FG)),
        ]));
    }
    let log_block = Block::default()
        .borders(Borders::TOP)
        .title(" Journal des verdicts ")
        .style(Style::default().fg(Color::DarkGray));
    let log_para = Paragraph::new(log_lines).block(log_block);
    f.render_widget(log_para, inner_chunks[1]);

    // Chaîne d'intentions (hash chain)
    let chain = app.chaima.intent_chain();
    let last_intents = chain.last_n(3);
    let mut chain_text = String::from("Chaîne cryptographique:\n");
    if last_intents.is_empty() {
        chain_text.push_str("(vide — aucune action évaluée)\n");
    }
    for intent in last_intents.iter() {
        chain_text.push_str(&format!("#{} h:{}\n", intent.id, &intent.current_hash));
    }
    let chain_para = Paragraph::new(chain_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title(" Intention chain ")
                .style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(chain_para, inner_chunks[2]);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(12), Constraint::Min(1)])
        .split(area);

    // Indicateur de mode
    let mode_str = match app.mode {
        Mode::Normal => " NORMAL ",
        Mode::Insert => " INSERT ",
        Mode::Command => " COMMAND ",
    };
    let mode_style = match app.mode {
        Mode::Normal => Style::default().fg(Color::Black).bg(Color::Blue),
        Mode::Insert => Style::default().fg(Color::Black).bg(Color::Green),
        Mode::Command => Style::default().fg(Color::Black).bg(Color::Yellow),
    };
    let mode_para = Paragraph::new(mode_str).style(mode_style);
    f.render_widget(mode_para, chunks[0]);

    // Status ou entrée commande
    let content = match app.mode {
        Mode::Command => format!(":{}", app.command_input),
        _ => app.status_message.clone(),
    };
    let style = if app.status_message.starts_with('✓') {
        Style::default().fg(COLOR_AUTHORIZED)
    } else if app.status_message.starts_with('⊘') || app.status_message.starts_with('✗') {
        Style::default().fg(COLOR_ZEROPATH)
    } else {
        Style::default().fg(COLOR_FG)
    };
    let status = Paragraph::new(format!(" {} ", content)).style(style);
    f.render_widget(status, chunks[1]);
}
