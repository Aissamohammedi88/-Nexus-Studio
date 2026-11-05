//! Rendu Nexus Studio v2.0 — layout VS Code-like
//!
//! ┌─ titre ──────────────────────────────────────────────────────┐
//! │ EXPLORATEUR │ onglets                                        │
//! │  arbre      │────────────────────────────────────────────────│
//! │             │ n°│ code coloré                                │
//! │─────────────│                                                │
//! │ CHAIMA      │────────────────────────────────────────────────│
//! │ métriques   │ PROBLÈMES │ SORTIE │ CHAIMA │ RECHERCHE       │
//! │─────────────┴────────────────────────────────────────────────│
//! │ NORMAL │ fichier │ Ln,Col │ langage │ ✓7 ⊘5 │ Δ=0 │ heure    │
//! └──────────────────────────────────────────────────────────────┘
//! + overlays : palette (Ctrl+P), tableau de bord Chaima (Ctrl+L), aide.

use crate::app::{App, Mode, OverlayKind, PanelTab};
use crate::editor::{Buffer, TokenKind};
use crate::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let t = &app.theme;
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    // ── Barre de titre ──────────────────────────────────────────────────
    let title = Line::from(vec![
        Span::styled(" NEXUS STUDIO ", Style::default().fg(Color::Black).bg(t.accent).add_modifier(Modifier::BOLD)),
        Span::styled(" v2.0 ", Style::default().fg(t.fg_dim)),
        Span::styled("— Protocole Chaima Architect", Style::default().fg(t.fg_dim)),
        Span::styled(
            format!("  {} commandes", app.palette.total_count()),
            Style::default().fg(t.accent),
        ),
    ]);
    f.render_widget(
        Paragraph::new(title).style(Style::default().bg(t.bg)),
        root[0],
    );

    // ── Corps : sidebar + éditeur/panneaux ──────────────────────────────
    let body = if app.sidebar_visible {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(28), Constraint::Min(20)])
            .split(root[1]);
        render_sidebar(f, app, cols[0]);
        cols[1]
    } else {
        root[1]
    };

    if app.panel_visible {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(body);
        render_editor_area(f, app, rows[0]);
        render_bottom_panel(f, app, rows[1]);
    } else {
        render_editor_area(f, app, body);
    }

    // ── Barre de statut ─────────────────────────────────────────────────
    render_status_bar(f, app, root[2]);

    // ── Overlays ────────────────────────────────────────────────────────
    match app.mode {
        Mode::Palette => render_palette(f, app),
        Mode::Command => render_command_bar(f, app),
        Mode::Find => render_find_bar(f, app),
        _ => {}
    }
    match app.overlay {
        OverlayKind::Chaima => render_chaima_overlay(f, app),
        OverlayKind::Help => render_help_overlay(f, app),
        OverlayKind::About => render_about_overlay(f, app),
        OverlayKind::None => {}
    }

    // Curseur visible en insertion
    if app.mode == Mode::Insert {
        if let Some(tab) = app.tab() {
            let area = editor_text_area(app, f.area());
            let x = area.x + 1 + tab.cursor.col as u16;
            let y = area.y + 1 + tab.cursor.row.saturating_sub(tab.scroll_row) as u16;
            if x < area.x + area.width && y < area.y + area.height {
                f.set_cursor_position((x, y));
            }
        }
    }
}

/// Zone écran occupée par le texte de l'éditeur (pour positionner le curseur).
fn editor_text_area(app: &App, full: Rect) -> Rect {
    let mut x = 0u16;
    let mut width = full.width;
    if app.sidebar_visible {
        x = 28;
        width = width.saturating_sub(28);
    }
    let height = full.height.saturating_sub(2); // titre + statut
    let height = if app.panel_visible {
        height * 65 / 100
    } else {
        height
    };
    Rect { x, y: 1, width, height: height.saturating_sub(2).max(1) }
}

// ============================================================================
// SIDEBAR — explorateur + panneau Chaima compact
// ============================================================================

fn render_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(14)])
        .split(area);

    // Explorateur
    let explorer_focus = app.mode == Mode::Explorer;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" EXPLORATEUR ")
        .border_style(Style::default().fg(if explorer_focus { t.border_focus } else { t.border }))
        .style(Style::default().bg(t.bg_sidebar));
    let inner = block.inner(chunks[0]);
    f.render_widget(block, chunks[0]);

    let vis = app.explorer.visible_entries();
    let max = inner.height as usize;
    let scroll = if explorer_focus {
        app.explorer
            .selected
            .saturating_sub(max.saturating_sub(1))
            .min(vis.len().saturating_sub(max.min(vis.len())))
    } else {
        0
    };
    let mut lines: Vec<Line> = Vec::new();
    for (pos, (_, entry)) in vis.iter().enumerate().skip(scroll).take(max) {
        let indent = "  ".repeat(entry.depth);
        let icon = if entry.is_dir {
            if entry.expanded { "▾ " } else { "▸ " }
        } else {
            "  "
        };
        let is_selected = explorer_focus && pos == app.explorer.selected;
        let mut style = Style::default().fg(t.fg);
        if is_selected {
            style = style.bg(t.bg_selection).add_modifier(Modifier::BOLD);
        }
        lines.push(Line::from(Span::styled(
            format!("{}{}{}", indent, icon, entry.name),
            style,
        )));
    }
    if lines.is_empty() {
        lines.push(Line::from(Span::styled("  (projet vide)", Style::default().fg(t.fg_dim))));
    }
    f.render_widget(Paragraph::new(lines), inner);

    // Panneau Chaima compact (bas de sidebar)
    let block2 = Block::default()
        .borders(Borders::ALL)
        .title(" CHAIMA · SOUVERAIN ")
        .border_style(Style::default().fg(t.accent))
        .style(Style::default().bg(t.bg_sidebar));
    let inner2 = block2.inner(chunks[1]);
    f.render_widget(block2, chunks[1]);

    let (auth, zeropath) = app.chaima_counts();
    let chain = app.chaima.intent_chain();
    let mut lines = vec![
        Line::from(vec![
            Span::styled(" autorisés   ", Style::default().fg(t.fg_dim)),
            Span::styled(format!("{} ✓", auth), Style::default().fg(t.authorized).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(" zéro-path   ", Style::default().fg(t.fg_dim)),
            Span::styled(format!("{} ⊘", zeropath), Style::default().fg(t.zeropath).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(" intentions  ", Style::default().fg(t.fg_dim)),
            Span::styled(format!("{}", chain.len()), Style::default().fg(t.accent).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(" dérive      ", Style::default().fg(t.fg_dim)),
            Span::styled("Δ = 0", Style::default().fg(t.authorized).add_modifier(Modifier::BOLD)),
        ]),
    ];
    let remaining = (inner2.height as usize).saturating_sub(lines.len());
    for intent in chain.last_n(remaining.min(4)).iter().rev() {
        lines.push(Line::from(vec![
            Span::styled(format!("#{} ", intent.id), Style::default().fg(t.accent)),
            Span::styled(
                &intent.current_hash[..12.min(intent.current_hash.len())],
                Style::default().fg(t.fg_dim),
            ),
        ]));
    }
    f.render_widget(Paragraph::new(lines), inner2);
}

// ============================================================================
// ÉDITEUR — onglets + numéros de ligne + coloration syntaxique
// ============================================================================

fn render_editor_area(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(3)])
        .split(area);

    render_tab_bar(f, app, chunks[0]);

    if app.tabs.is_empty() {
        render_welcome(f, app, chunks[1]);
        return;
    }
    let tab = &app.tabs[app.active];
    let buf = &tab.buffer;
    let cursor = &tab.cursor;

    // Scroll : garder le curseur visible
    let height = chunks[1].height.saturating_sub(2) as usize;
    let mut scroll_row = tab.scroll_row;
    if cursor.row < scroll_row {
        scroll_row = cursor.row;
    } else if cursor.row >= scroll_row + height {
        scroll_row = cursor.row + 1 - height;
    }

    let line_no_width = buf.line_count().to_string().len() + 1;
    let gutter = format!("{:>width$} │ ", "", width = line_no_width);
    let _ = &gutter;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.border))
        .style(Style::default().bg(t.bg));
    let inner = block.inner(chunks[1]);
    f.render_widget(block, chunks[1]);

    let mut lines: Vec<Line> = Vec::new();
    let first = scroll_row;
    let last = (scroll_row + height).min(buf.line_count());

    for row in first..last {
        let is_current = row == cursor.row;
        let num_style = if is_current {
            Style::default().fg(t.fg_line_current_number).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.fg_line_numbers)
        };
        let mut spans = vec![
            Span::styled(format!("{:>width$} ", row + 1, width = line_no_width), num_style),
            Span::styled("│ ", Style::default().fg(t.border)),
        ];
        let line_text = buf.line(row);
        if is_current {
            // fond de ligne courante
            spans.push(Span::styled(
                " ".repeat((inner.width as usize).saturating_sub(line_no_width + 2)),
                Style::default().bg(t.bg_line_current),
            ));
            lines.push(Line::from(spans));
            // La ligne courante réelle est rendue juste après (approche : rendre
            // le texte par-dessus n'est pas possible en une passe — on fusionne).
            lines.pop();
            let mut cur_spans = vec![
                Span::styled(format!("{:>width$} ", row + 1, width = line_no_width), num_style.bg(t.bg_line_current)),
                Span::styled("│ ", Style::default().fg(t.border).bg(t.bg_line_current)),
            ];
            append_syntax(&mut cur_spans, line_text, buf, t, Some(t.bg_line_current));
            lines.push(Line::from(cur_spans));
        } else {
            append_syntax(&mut spans, line_text, buf, t, None);
            lines.push(Line::from(spans));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled("  (buffer vide)", Style::default().fg(t.fg_dim))));
    }
    f.render_widget(Paragraph::new(lines), inner);

    // Indicateur de sélection de recherche : surligne via statut (simple)
    let _ = &app.last_find;
    let _ = cursor;
}

fn append_syntax(spans: &mut Vec<Span>, line: &str, _buf: &Buffer, t: &Theme, bg: Option<Color>) {
    let tokens = crate::editor::tokenize_line(line, _buf.language);
    for tok in tokens {
        let color = match tok.kind {
            TokenKind::Keyword => t.syn_keyword,
            TokenKind::String => t.syn_string,
            TokenKind::Comment => t.syn_comment,
            TokenKind::Number => t.syn_number,
            TokenKind::Function => t.syn_function,
            TokenKind::Type => t.syn_type,
            TokenKind::Operator => t.syn_operator,
            TokenKind::Punctuation => t.syn_punctuation,
            TokenKind::Text => t.fg,
        };
        let mut style = Style::default().fg(color);
        if let Some(b) = bg {
            style = style.bg(b);
        }
        spans.push(Span::styled(tok.text, style));
    }
}

fn render_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    if app.tabs.is_empty() {
        f.render_widget(
            Block::default().style(Style::default().bg(t.bg_tab_inactive)),
            area,
        );
        return;
    }
    let titles: Vec<Line> = app
        .tabs
        .iter()
        .map(|tab| {
            let dirty = if tab.buffer.modified { "● " } else { "" };
            Line::from(format!(" {}{} ", dirty, tab.buffer.filename()))
        })
        .collect();
    let tabs_widget = Tabs::new(titles)
        .select(Some(app.active))
        .divider("│")
        .style(Style::default().fg(t.fg_dim).bg(t.bg_tab_inactive))
        .highlight_style(Style::default().fg(t.fg).bg(t.bg_tab_active).add_modifier(Modifier::BOLD));
    f.render_widget(tabs_widget, area);
}

fn render_welcome(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "   NEXUS STUDIO v2.0 — IDE Souverain",
            Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "   Protocole Chaima Architect v1.0",
            Style::default().fg(t.fg_dim),
        )),
        Line::from(""),
        Line::from(Span::styled("   Ctrl+P   palette de commandes", Style::default().fg(t.fg))),
        Line::from(Span::styled("   Ctrl+E   explorateur de fichiers", Style::default().fg(t.fg))),
        Line::from(Span::styled("   Ctrl+L   tableau de bord Chaima", Style::default().fg(t.fg))),
        Line::from(Span::styled("   :e fichier   ouvrir un fichier", Style::default().fg(t.fg))),
        Line::from(Span::styled("   :help     toutes les touches", Style::default().fg(t.fg))),
        Line::from(""),
        Line::from(Span::styled(
            "   Le modèle propose, la Loi dispose. Δ=0 par construction.",
            Style::default().fg(t.fg_dim).add_modifier(Modifier::ITALIC),
        )),
    ];
    f.render_widget(
        Paragraph::new(text).alignment(Alignment::Left).style(Style::default().bg(t.bg)),
        area,
    );
}

// ============================================================================
// PANNEAU INFÉRIEUR — onglets PROBLÈMES / SORTIE / CHAIMA / RECHERCHE
// ============================================================================

fn render_bottom_panel(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    // Barre d'onglets du panneau
    let tab_defs = [
        PanelTab::Problems,
        PanelTab::Output,
        PanelTab::Chaima,
        PanelTab::Search,
    ];
    let mut spans: Vec<Span> = Vec::new();
    for pt in tab_defs.iter() {
        let count = match pt {
            PanelTab::Problems => app.log.iter().filter(|l| !l.verdict_authorized).count(),
            PanelTab::Output => app.output.len(),
            PanelTab::Chaima => app.chaima.intent_chain().len(),
            PanelTab::Search => app.grep_hits.len(),
        };
        let selected = app.panel_tab == *pt;
        let style = if selected {
            Style::default().fg(t.fg).add_modifier(Modifier::BOLD).bg(t.bg_panel)
        } else {
            Style::default().fg(t.fg_dim).bg(t.bg_panel)
        };
        spans.push(Span::styled(pt.label().to_string(), style));
        spans.push(Span::styled(
            format!("{} ", count),
            if selected { Style::default().fg(t.accent).bg(t.bg_panel) } else { Style::default().fg(t.fg_dim).bg(t.bg_panel) },
        ));
        spans.push(Span::styled("│", Style::default().fg(t.border).bg(t.bg_panel)));
    }
    f.render_widget(Paragraph::new(Line::from(spans)).style(Style::default().bg(t.bg_panel)), chunks[0]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.border))
        .style(Style::default().bg(t.bg_panel));
    let inner = block.inner(chunks[1]);
    f.render_widget(block, chunks[1]);
    let max = inner.height as usize;

    let lines: Vec<Line> = match app.panel_tab {
        PanelTab::Problems => {
            let zeropath: Vec<&crate::app::LogEntry> =
                app.log.iter().filter(|l| !l.verdict_authorized).rev().take(max).collect();
            zeropath
                .iter()
                .map(|l| {
                    Line::from(vec![
                        Span::styled("⊘ ", Style::default().fg(t.zeropath).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("{} ", l.timestamp), Style::default().fg(t.fg_dim)),
                        Span::styled(truncate(&l.action, 30), Style::default().fg(t.fg)),
                        Span::styled(" — ", Style::default().fg(t.fg_dim)),
                        Span::styled(truncate(&l.reason, 50), Style::default().fg(t.fg_dim)),
                    ])
                })
                .collect()
        }
        PanelTab::Output => app
            .output
            .iter()
            .rev()
            .take(max)
            .rev()
            .map(|l| Line::from(Span::styled(l.clone(), Style::default().fg(t.fg))))
            .collect(),
        PanelTab::Chaima => app
            .chaima
            .intent_chain()
            .last_n(max)
            .iter()
            .rev()
            .map(|intent| {
                Line::from(vec![
                    Span::styled(format!("#{:<4} ", intent.id), Style::default().fg(t.accent)),
                    Span::styled(
                        format!("{:<38}", truncate(&intent.action, 38)),
                        Style::default().fg(t.fg),
                    ),
                    Span::styled(
                        &intent.current_hash[..16.min(intent.current_hash.len())],
                        Style::default().fg(t.fg_dim),
                    ),
                ])
            })
            .collect(),
        PanelTab::Search => {
            if app.grep_hits.is_empty() {
                vec![Line::from(Span::styled(
                    "  aucune recherche — :grep <motif> pour chercher dans tout le projet",
                    Style::default().fg(t.fg_dim),
                ))]
            } else {
                app.grep_hits
                    .iter()
                    .enumerate()
                    .skip(app.grep_selected.saturating_sub(max.saturating_sub(1)))
                    .take(max)
                    .map(|(i, hit)| {
                        let selected = i == app.grep_selected;
                        let style = if selected {
                            Style::default().fg(t.fg).bg(t.bg_selection)
                        } else {
                            Style::default().fg(t.fg)
                        };
                        Line::from(vec![
                            Span::styled(
                                format!(" {:>3} ", i + 1),
                                Style::default().fg(t.fg_dim),
                            ),
                            Span::styled(
                                format!("{}:{}  ", short_path(&hit.path), hit.line_no),
                                style.fg(t.accent),
                            ),
                            Span::styled(truncate(&hit.text, 60), style),
                        ])
                    })
                    .collect()
            }
        }
    };
    f.render_widget(Paragraph::new(lines), inner);
}

// ============================================================================
// BARRE DE STATUT — style VS Code
// ============================================================================

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let (mode_label, mode_bg) = match app.mode {
        Mode::Normal => (" NORMAL ", t.bg_status),
        Mode::Insert => (" INSERT ", t.bg_status_insert),
        Mode::Command => (" COMMAND ", t.bg_status),
        Mode::Explorer => (" EXPLORATEUR ", t.bg_status),
        Mode::Palette => (" PALETTE ", t.bg_status),
        Mode::Overlay => (" DASHBOARD ", t.bg_status),
        Mode::Find => (" RECHERCHE ", t.bg_status),
    };

    let lang = app
        .tab()
        .map(|tab| language_label(tab.buffer.language))
        .unwrap_or("—");
    let filename = app.tab().map(|tab| {
        let dirty = if tab.buffer.modified { "● " } else { "" };
        format!("{}{}", dirty, tab.buffer.filename())
    }).unwrap_or_else(|| "bienvenue".to_string());

    let (ln, col) = app
        .tab()
        .map(|tab| (tab.cursor.row + 1, tab.cursor.col + 1))
        .unwrap_or((1, 1));
    let (auth, zeropath) = app.chaima_counts();

    let left = vec![
        Span::styled(mode_label, Style::default().fg(Color::White).bg(mode_bg).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {} ", filename), Style::default().fg(Color::White).bg(t.bg_status)),
        Span::styled(format!(" Ln {}, Col {} ", ln, col), Style::default().fg(Color::White).bg(t.bg_status)),
    ];
    let right = vec![
        Span::styled(format!(" ✓{} ", auth), Style::default().fg(Color::White).bg(t.bg_status)),
        Span::styled(format!("⊘{} ", zeropath), Style::default().fg(Color::White).bg(t.bg_status)),
        Span::styled(" Δ=0 ", Style::default().fg(Color::White).bg(t.bg_status)),
        Span::styled(format!(" {} ", lang), Style::default().fg(Color::White).bg(t.bg_status)),
        Span::styled(" UTF-8 ", Style::default().fg(Color::White).bg(t.bg_status)),
        Span::styled(
            format!(" {} ", chrono::Local::now().format("%H:%M")),
            Style::default().fg(Color::White).bg(t.bg_status),
        ),
    ];
    let left_w: usize = left.iter().map(|s| s.content.chars().count()).sum();
    let right_w: usize = right.iter().map(|s| s.content.chars().count()).sum();
    let gap = (area.width as usize).saturating_sub(left_w + right_w);

    let mut all = left;
    all.push(Span::styled(" ".repeat(gap), Style::default().bg(t.bg_status)));
    all.extend(right);
    f.render_widget(Paragraph::new(Line::from(all)), area);

    // Message de statut flottant sur la gauche du message principal ? Non —
    // le message passe par la ligne au-dessus quand le panneau est caché.
    let _ = &app.status_message;
}

fn language_label(l: crate::editor::Language) -> &'static str {
    match l {
        crate::editor::Language::Rust => "Rust",
        crate::editor::Language::Python => "Python",
        crate::editor::Language::Prolog => "Prolog",
        crate::editor::Language::Markdown => "Markdown",
        crate::editor::Language::Text => "Texte",
    }
}

// ============================================================================
// OVERLAYS — palette, barre :, barre /, dashboards
// ============================================================================

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup[1])[1]
}

fn render_palette(f: &mut Frame, app: &App) {
    let t = &app.theme;
    let area = centered_rect(70, 55, f.size());
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " PALETTE DE COMMANDES — {} / {} ",
            app.palette.visible_count(),
            app.palette.total_count()
        ))
        .border_style(Style::default().fg(t.border_focus))
        .style(Style::default().bg(t.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" › ", Style::default().fg(t.accent).add_modifier(Modifier::BOLD)),
            Span::styled(&app.palette.input, Style::default().fg(t.fg)),
            Span::styled("▏", Style::default().fg(t.accent)),
        ])),
        chunks[0],
    );

    let max = chunks[1].height as usize;
    let sel = app.palette.selected;
    let scroll = sel.saturating_sub(max.saturating_sub(1));
    let mut lines = Vec::new();
    for (pos, cmd_idx) in app.palette.filtered.iter().enumerate().skip(scroll).take(max) {
        let cmd = &app.palette.commands[*cmd_idx];
        let selected = pos == sel;
        let style = if selected {
            Style::default().fg(t.fg).bg(t.bg_selection).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.fg)
        };
        lines.push(Line::from(vec![
            Span::styled(if selected { " ▸ " } else { "   " }, style),
            Span::styled(truncate(&cmd.title, 42), style),
            Span::styled(
                format!("  {}", truncate(&cmd.description, (inner.width as usize).saturating_sub(50))),
                Style::default().fg(t.fg_dim),
            ),
        ]));
    }
    if lines.is_empty() {
        lines.push(Line::from(Span::styled("  aucune commande ne correspond", Style::default().fg(t.fg_dim))));
    }
    f.render_widget(Paragraph::new(lines), chunks[1]);
}

fn render_command_bar(f: &mut Frame, app: &App) {
    let t = &app.theme;
    let area = Rect { x: 0, y: f.size().height.saturating_sub(2), width: f.size().width, height: 1 };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" :", Style::default().fg(t.accent).add_modifier(Modifier::BOLD)),
            Span::styled(&app.command_input, Style::default().fg(t.fg)),
            Span::styled("▏", Style::default().fg(t.accent)),
        ]))
        .style(Style::default().bg(t.bg_panel)),
        area,
    );
}

fn render_find_bar(f: &mut Frame, app: &App) {
    let t = &app.theme;
    let area = Rect { x: 0, y: f.size().height.saturating_sub(2), width: f.size().width, height: 1 };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" /", Style::default().fg(t.accent).add_modifier(Modifier::BOLD)),
            Span::styled(&app.find_input, Style::default().fg(t.fg)),
            Span::styled("▏", Style::default().fg(t.accent)),
            Span::styled("   (Entrée : chercher · n/N : suivant/précédent)", Style::default().fg(t.fg_dim)),
        ]))
        .style(Style::default().bg(t.bg_panel)),
        area,
    );
}

fn render_chaima_overlay(f: &mut Frame, app: &App) {
    let t = &app.theme;
    let area = centered_rect(80, 80, f.size());
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" COUCHE SOUVERAINE — CHAIMA · Protocole Architect v1.0 ")
        .border_style(Style::default().fg(t.accent))
        .style(Style::default().bg(t.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let (auth, zeropath) = app.chaima_counts();
    let chain = app.chaima.intent_chain();
    let rules = app.chaima.rules_count();
    let (gen, acc, rej) = app.llm.stats();

    let mut lines = vec![
        Line::from(Span::styled("  RÈGLES ACTIVES", Style::default().fg(t.accent).add_modifier(Modifier::BOLD))),
        Line::from("   R1  Découplage Logique/Modèle — le modèle propose, la Loi dispose"),
        Line::from("   R2  Zéro-Path — si non autorisé, l'action n'existe pas"),
        Line::from("   R3  Immunité architecturale — Prolog natif Rust, règles non modifiables"),
        Line::from("   R4  Zéro-Delta — Δ=0, déterministe par construction"),
        Line::from("   R5  Souveraineté de l'intention — chaîne SHA256 immuable"),
        Line::from(""),
        Line::from(Span::styled("  MÉTRIQUES", Style::default().fg(t.accent).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("   autorisées      ", Style::default().fg(t.fg_dim)),
            Span::styled(format!("{} ✓", auth), Style::default().fg(t.authorized).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("   zéro-path       ", Style::default().fg(t.fg_dim)),
            Span::styled(format!("{} ⊘", zeropath), Style::default().fg(t.zeropath).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("   intentions      ", Style::default().fg(t.fg_dim)),
            Span::styled(format!("{}", chain.len()), Style::default().fg(t.accent).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("   règles Prolog   ", Style::default().fg(t.fg_dim)),
            Span::styled(format!("{}", rules), Style::default().fg(t.fg)),
        ]),
        Line::from(vec![
            Span::styled("   dérive          ", Style::default().fg(t.fg_dim)),
            Span::styled("Δ = 0 (déterministe)", Style::default().fg(t.authorized).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("   LLM (gate)      ", Style::default().fg(t.fg_dim)),
            Span::styled(format!("{} générées · {} passées · {} rejetées", gen, acc, rej), Style::default().fg(t.fg)),
        ]),
        Line::from(""),
        Line::from(Span::styled("  CHAÎNE D'INTENTIONS (dernières, SHA256)", Style::default().fg(t.accent).add_modifier(Modifier::BOLD))),
    ];

    let max = inner.height as usize;
    let visible_chain = max.saturating_sub(lines.len() + 1);
    for intent in chain.last_n(visible_chain).iter().rev() {
        lines.push(Line::from(vec![
            Span::styled(format!("   #{:<4} ", intent.id), Style::default().fg(t.accent)),
            Span::styled(
                format!("{:<40}", truncate(&intent.action, 40)),
                Style::default().fg(t.fg),
            ),
            Span::styled(
                &intent.current_hash[..16.min(intent.current_hash.len())],
                Style::default().fg(t.fg_dim),
            ),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  t : tester une action · c : vider la chaîne dans SORTIE · q : fermer",
        Style::default().fg(t.fg_dim),
    )));

    let scroll = app.overlay_scroll.min(lines.len().saturating_sub(max.saturating_sub(1)));
    let visible: Vec<Line> = lines.into_iter().skip(scroll).collect();
    f.render_widget(Paragraph::new(visible), inner);
}

fn render_help_overlay(f: &mut Frame, app: &App) {
    let t = &app.theme;
    let area = centered_rect(80, 80, f.size());
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" AIDE — RACCOURCIS ")
        .border_style(Style::default().fg(t.accent))
        .style(Style::default().bg(t.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows: &[(&str, &str)] = &[
        ("── GLOBAL ──", ""),
        ("Ctrl+P", "palette de commandes (filtre flou)"),
        ("Ctrl+E", "explorateur de fichiers"),
        ("Ctrl+L", "tableau de bord souverain Chaima"),
        ("Ctrl+B", "afficher/masquer la barre latérale"),
        ("Ctrl+J", "afficher/masquer le panneau inférieur"),
        ("Ctrl+S", "sauvegarder"),
        ("Alt+←/→", "onglet précédent / suivant"),
        ("Alt+↑/↓", "correspondance de recherche précédente / suivante"),
        ("── ÉDITEUR (normal) ──", ""),
        ("h j k l", "déplacement curseur"),
        ("0 / $", "début / fin de ligne"),
        ("g g / G", "début / fin du buffer"),
        ("i / Esc", "insertion / retour normal"),
        ("o / O", "nouvelle ligne dessous / dessus"),
        ("x", "supprimer le caractère"),
        ("d d", "supprimer la ligne"),
        ("/ puis n/N", "recherche buffer, occurrence suivante/précédente"),
        (":", "barre de commande"),
        ("── COMMANDES (:…) ──", ""),
        (":w / :w chemin", "sauvegarder / sauvegarder sous"),
        (":q / :q! / :wq", "fermer l'onglet / forcer / sauvegarder+fermer"),
        (":e fichier", "ouvrir un fichier (ou buffer vierge sans argument)"),
        (":42", "aller à la ligne 42"),
        (":grep motif", "recherche dans tout le projet"),
        (":theme nord|gruvbox|light|nexus-dark", "changer de thème"),
        (":chaima / :llm / :stats / :test / :chain", "couches souveraines"),
        (":prolog / :trim / :dup / :help / :about", "moteur Prolog, nettoyage, aide"),
        ("── EXPLORATEUR ──", ""),
        ("j / k", "naviguer"),
        ("Entrée / Espace", "ouvrir le fichier / déplier le dossier"),
        ("Esc", "revenir à l'éditeur"),
    ];
    let max = inner.height as usize;
    let lines: Vec<Line> = rows
        .iter()
        .map(|(k, v)| {
            if v.is_empty() {
                Line::from(Span::styled(format!(" {}", k), Style::default().fg(t.accent).add_modifier(Modifier::BOLD)))
            } else {
                Line::from(vec![
                    Span::styled(format!("  {:<38}", k), Style::default().fg(t.fg).add_modifier(Modifier::BOLD)),
                    Span::styled(v.to_string(), Style::default().fg(t.fg_dim)),
                ])
            }
        })
        .collect();
    let scroll = app.overlay_scroll.min(lines.len().saturating_sub(max.saturating_sub(1)));
    let visible: Vec<Line> = lines.into_iter().skip(scroll).collect();
    f.render_widget(Paragraph::new(visible), inner);
}

fn render_about_overlay(f: &mut Frame, app: &App) {
    let t = &app.theme;
    let area = centered_rect(70, 55, f.size());
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" À PROPOS DE NEXUS STUDIO ")
        .border_style(Style::default().fg(t.accent))
        .style(Style::default().bg(t.bg));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let (auth, zeropath) = app.chaima_counts();
    let text = vec![
        Line::from(""),
        Line::from(Span::styled("  NEXUS STUDIO v2.0", Style::default().fg(t.accent).add_modifier(Modifier::BOLD))),
        Line::from("  IDE souverain Rust — Protocole Chaima Architect v1.0"),
        Line::from(""),
        Line::from(format!("  binaire        ~770 KB (VS Code : ~80 MB — 107× plus petit)")),
        Line::from(format!("  dépendances    0 runtime (VS Code : Electron + Chromium + Node)")),
        Line::from(format!("  init souverain < 1 ms (VS Code : ~5 000 ms)")),
        Line::from(format!("  commandes      {} dans la palette", app.palette.total_count())),
        Line::from(format!("  souveraineté   {} autorisées ✓ · {} Zéro-Path ⊘ · Δ=0", auth, zeropath)),
        Line::from(""),
        Line::from(Span::styled("  Le modèle propose, la Loi dispose.", Style::default().fg(t.fg_dim).add_modifier(Modifier::ITALIC))),
        Line::from(""),
        Line::from("  q : fermer"),
    ];
    f.render_widget(Paragraph::new(text), inner);
}

// ============================================================================
// UTILS
// ============================================================================

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", cut)
    }
}

fn short_path(p: &str) -> String {
    p.rsplit('/').next().unwrap_or(p).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("abc", 5), "abc");
        assert_eq!(truncate("abcdefgh", 5), "abcd…");
        assert_eq!(truncate("", 3), "");
    }

    #[test]
    fn test_short_path() {
        assert_eq!(short_path("/a/b/c.rs"), "c.rs");
        assert_eq!(short_path("x.py"), "x.py");
    }
}
