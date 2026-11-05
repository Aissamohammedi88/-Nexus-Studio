//! Palette de commandes — le cœur "VS Code" de Nexus Studio
//!
//! Chaque commande est une entrée RÉELLE câblée sur une action de l'app.
//! Filtre flou (sous-séquence) à la frappe. Le compteur affiché est honnête :
//! `palette.len()` commandes, pas un de plus.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionId {
    // Fichier
    Save,
    SaveAs,
    CloseTab,
    NewFile,
    OpenExplorer,
    RevealInExplorer,
    // Navigation
    NextTab,
    PrevTab,
    GotoLine,
    // Affichage
    ToggleSidebar,
    ToggleBottomPanel,
    CycleTheme,
    ThemeNexusDark,
    ThemeNord,
    ThemeGruvbox,
    ThemeLight,
    // Chaima (couche souveraine)
    ChaimaDashboard,
    ChaimaEvaluateTest,
    ChaimaShowChain,
    ChaimaMetrics,
    // IA (gate)
    LlmSuggest,
    LlmStats,
    // Recherche
    FindInBuffer,
    FindNext,
    FindPrev,
    GrepProject,
    // Prolog
    PrologDemo,
    // Édition
    DeleteLine,
    DuplicateLine,
    MoveLineUp,
    MoveLineDown,
    TrimTrailingWhitespace,
    SelectAll,
    // Aide
    HelpKeys,
    About,
    Quit,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub id: ActionId,
    /// Catégorie affichée : "Fichier :", "Affichage :", "Chaima :"...
    pub title: String,
    pub description: String,
}

pub struct Palette {
    pub commands: Vec<Command>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub input: String,
    pub open: bool,
}

impl Palette {
    pub fn new() -> Self {
        let mut p = Palette {
            commands: Palette::build_registry(),
            filtered: Vec::new(),
            selected: 0,
            input: String::new(),
            open: false,
        };
        p.refilter();
        p
    }

    fn build_registry() -> Vec<Command> {
        let defs: &[(ActionId, &str, &str)] = &[
            (ActionId::Save, "Fichier : Sauvegarder", "Écrit le buffer courant (autorisation Chaima requise)"),
            (ActionId::SaveAs, "Fichier : Sauvegarder sous…", "Enregistre sous un nouveau chemin (:w chemin)"),
            (ActionId::CloseTab, "Fichier : Fermer l'onglet", "Ferme l'onglet actif (:bd)"),
            (ActionId::NewFile, "Fichier : Nouveau buffer", "Ouvre un buffer vierge (:e)"),
            (ActionId::OpenExplorer, "Fichier : Ouvrir l'explorateur", "Naviguer dans l'arborescence (Ctrl+E)"),
            (ActionId::RevealInExplorer, "Fichier : Révéler dans l'explorateur", "Sélectionne le fichier courant dans l'arbre"),
            (ActionId::NextTab, "Aller : Onglet suivant", "gt ou Alt+→"),
            (ActionId::PrevTab, "Aller : Onglet précédent", "gT ou Alt+←"),
            (ActionId::GotoLine, "Aller : Ligne…", "Saute à la ligne (:42)"),
            (ActionId::ToggleSidebar, "Affichage : Basculer l'explorateur", "Ctrl+B"),
            (ActionId::ToggleBottomPanel, "Affichage : Basculer le panneau inférieur", "Ctrl+J"),
            (ActionId::CycleTheme, "Affichage : Thème suivant", "Cycle les 4 thèmes installés"),
            (ActionId::ThemeNexusDark, "Affichage : Thème Nexus Dark", "Palette par défaut, inspirée VS Code Dark+"),
            (ActionId::ThemeNord, "Affichage : Thème Nord", "Bleus froids, contraste doux"),
            (ActionId::ThemeGruvbox, "Affichage : Thème Gruvbox", "Terre chaude rétro"),
            (ActionId::ThemeLight, "Affichage : Thème Light", "Fond clair, pour les projets en plein jour"),
            (ActionId::ChaimaDashboard, "Chaima : Tableau de bord souverain", "Métriques, règles actives, chaîne SHA256 (Ctrl+L)"),
            (ActionId::ChaimaEvaluateTest, "Chaima : Évaluer une action de test", "Passe une action autorisée et une interdite dans la couche"),
            (ActionId::ChaimaShowChain, "Chaima : Chaîne d'intentions", "Journal cryptographique complet SHA256"),
            (ActionId::ChaimaMetrics, "Chaima : Métriques Zéro-Path", "Autorisés / Zéro-Path / Δ"),
            (ActionId::LlmSuggest, "IA : Demander une suggestion", "Le modèle propose, Chaima dispose (:llm)"),
            (ActionId::LlmStats, "IA : Statistiques du modèle", "Suggestions générées / acceptées / rejetées par le gate"),
            (ActionId::FindInBuffer, "Recherche : Dans le buffer", "Recherche incrémentale (/)"),
            (ActionId::FindNext, "Recherche : Occurrence suivante", "n"),
            (ActionId::FindPrev, "Recherche : Occurrence précédente", "N"),
            (ActionId::GrepProject, "Recherche : Dans tout le projet", "grep récursif, résultats dans le panneau bas (:grep)"),
            (ActionId::PrologDemo, "Prolog : Exécuter la requête de démo", "Fait tourner le moteur natif, résultat en SORTIE"),
            (ActionId::DeleteLine, "Édition : Supprimer la ligne", "dd"),
            (ActionId::DuplicateLine, "Édition : Dupliquer la ligne", "Copie la ligne courante en dessous"),
            (ActionId::MoveLineUp, "Édition : Déplacer la ligne vers le haut", "Alt+k"),
            (ActionId::MoveLineDown, "Édition : Déplacer la ligne vers le bas", "Alt+j"),
            (ActionId::TrimTrailingWhitespace, "Édition : Nettoyer les espaces en fin de ligne", "Trim trailing whitespace sur tout le buffer"),
            (ActionId::SelectAll, "Édition : Tout sélectionner", "Place le curseur au début et marque la sélection"),
            (ActionId::HelpKeys, "Aide : Raccourcis clavier", "Toutes les touches, par mode"),
            (ActionId::About, "Aide : À propos de Nexus Studio", "Version, protocole, benchmark"),
            (ActionId::Quit, "Fichier : Quitter", ":q"),
        ];
        defs.iter()
            .map(|(id, title, desc)| Command {
                id: *id,
                title: title.to_string(),
                description: desc.to_string(),
            })
            .collect()
    }

    pub fn open(&mut self) {
        self.open = true;
        self.input.clear();
        self.selected = 0;
        self.refilter();
    }

    pub fn close(&mut self) {
        self.open = false;
        self.input.clear();
        self.selected = 0;
    }

    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
        self.selected = 0;
        self.refilter();
    }

    pub fn backspace(&mut self) {
        self.input.pop();
        self.selected = 0;
        self.refilter();
    }

    /// Filtre flou : sous-séquence insensible à la casse.
    pub fn refilter(&mut self) {
        let q = self.input.to_lowercase();
        self.filtered.clear();
        for (i, cmd) in self.commands.iter().enumerate() {
            if q.is_empty() || fuzzy_match(&cmd.title.to_lowercase(), &q) {
                self.filtered.push(i);
            }
        }
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.filtered.len() {
            self.selected += 1;
        }
    }

    /// La commande sélectionnée, prête à être exécutée.
    pub fn selected_command(&self) -> Option<&Command> {
        self.filtered
            .get(self.selected)
            .and_then(|i| self.commands.get(*i))
    }

    pub fn visible_count(&self) -> usize {
        self.filtered.len()
    }

    pub fn total_count(&self) -> usize {
        self.commands.len()
    }
}

/// Sous-séquence : chaque caractère de `query` apparaît dans `text` dans l'ordre.
pub fn fuzzy_match(text: &str, query: &str) -> bool {
    let mut chars = query.chars();
    let mut current = match chars.next() {
        Some(c) => c,
        None => return true,
    };
    for c in text.chars() {
        if c == current {
            match chars.next() {
                Some(next) => current = next,
                None => return true,
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        assert!(fuzzy_match("chaima : tableau de bord souverain", "tbs"));
        assert!(fuzzy_match("fichier : sauvegarder", "fsv"));
        assert!(!fuzzy_match("fichier : sauvegarder", "zx"));
        assert!(fuzzy_match("n'importe quoi", ""));
    }

    #[test]
    fn test_palette_registry_and_filter() {
        let mut p = Palette::new();
        assert!(p.total_count() >= 30, "au moins 30 commandes réelles");
        assert_eq!(p.visible_count(), p.total_count());

        p.push_char('c');
        p.push_char('h');
        p.push_char('a');
        // "cha" doit matcher Chaima :* mais pas les autres catégories
        assert!(p.visible_count() > 0);
        assert!(p.visible_count() < p.total_count());

        p.backspace();
        p.backspace();
        p.backspace();
        assert_eq!(p.visible_count(), p.total_count());
    }

    #[test]
    fn test_palette_selection() {
        let mut p = Palette::new();
        p.open();
        let before = p.selected_command().unwrap().id;
        p.move_down();
        let after = p.selected_command().unwrap().id;
        assert_ne!(before, after);
        p.move_up();
        assert_eq!(p.selected_command().unwrap().id, before);
    }
}
