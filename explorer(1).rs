//! Explorateur de fichiers — arborescence VS Code-like
//!
//! Arbre à expansion paresseuse : les répertoires se développent à la demande.
//! Le rendu est une liste aplatie d'entrées avec profondeur.

use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Filter {
    All,
    Code,
}

#[derive(Debug)]
pub struct Explorer {
    pub root: PathBuf,
    pub entries: Vec<TreeEntry>,
    pub selected: usize,
    pub visible: bool,
    pub filter: Filter,
    pub error: Option<String>,
}

/// Répertoires systématiquement exclus de l'arbre
const EXCLUDED_DIRS: &[&str] = &[".git", "target", "__pycache__", "node_modules"];

impl Explorer {
    /// Construit l'explorateur sur la racine ; charge le premier niveau.
    pub fn new(root: PathBuf) -> Self {
        let mut ex = Explorer {
            root,
            entries: Vec::new(),
            selected: 0,
            visible: true,
            filter: Filter::All,
            error: None,
        };
        ex.reload_root();
        ex
    }

    pub fn reload_root(&mut self) {
        self.entries.clear();
        self.selected = 0;
        self.error = None;
        if !self.root.exists() {
            self.error = Some(format!("introuvable : {}", self.root.display()));
            return;
        }
        let children = self.read_children(&self.root, 0);
        self.entries = children;
    }

    /// Change la racine (ex : :cd) et recharge l'arbre.
    pub fn set_root(&mut self, root: PathBuf) {
        self.root = root;
        self.reload_root();
    }

    fn read_children(&self, dir: &PathBuf, depth: usize) -> Vec<TreeEntry> {
        let mut dirs: Vec<TreeEntry> = Vec::new();
        let mut files: Vec<TreeEntry> = Vec::new();

        if let Ok(read) = fs::read_dir(dir) {
            for entry in read.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = path.is_dir();

                if is_dir && EXCLUDED_DIRS.contains(&name.as_str()) {
                    continue;
                }
                if !is_dir && !self.file_passes_filter(&name) {
                    continue;
                }

                let e = TreeEntry {
                    name,
                    path: path.clone(),
                    is_dir,
                    depth,
                    expanded: false,
                };
                if is_dir {
                    dirs.push(e);
                } else {
                    files.push(e);
                }
            }
        }
        dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        dirs.extend(files);
        dirs
    }

    fn file_passes_filter(&self, name: &str) -> bool {
        match self.filter {
            Filter::All => true,
            Filter::Code => {
                name.ends_with(".rs")
                    || name.ends_with(".py")
                    || name.ends_with(".pl")
                    || name.ends_with(".prolog")
                    || name.ends_with(".md")
                    || name.ends_with(".toml")
                    || name.ends_with(".json")
                    || name.ends_with(".txt")
                    || name.ends_with(".sh")
            }
        }
    }

    /// Liste aplatie affichée : entrées visibles uniquement.
    ///
    /// Les enfants (insérés contigus après leur parent) sont masqués si un
    /// ancêtre est replié : on saute toute entrée de profondeur >= d+1 tant
    /// qu'aucune entrée de profondeur <= d ne réapparaît.
    pub fn visible_entries(&self) -> Vec<(usize, &TreeEntry)> {
        let mut out = Vec::new();
        let mut skip_above: Option<usize> = None;
        for (idx, e) in self.entries.iter().enumerate() {
            if let Some(s) = skip_above {
                if e.depth >= s {
                    continue;
                }
                skip_above = None;
            }
            out.push((idx, e));
            if e.is_dir && !e.expanded {
                skip_above = Some(e.depth + 1);
            }
        }
        out
    }

    /// L'entrée sélectionnée (dans la liste visible).
    pub fn selected_entry(&self) -> Option<&TreeEntry> {
        let vis = self.visible_entries();
        vis.get(self.selected).map(|(_, e)| *e)
    }

    pub fn move_down(&mut self, by: usize) {
        let count = self.visible_entries().len();
        if count == 0 {
            return;
        }
        self.selected = (self.selected + by).min(count - 1);
    }

    pub fn move_up(&mut self, by: usize) {
        self.selected = self.selected.saturating_sub(by);
    }

    pub fn move_top(&mut self) {
        self.selected = 0;
    }

    pub fn move_bottom(&mut self) {
        let count = self.visible_entries().len();
        self.selected = count.saturating_sub(1);
    }

    /// Enter/Espace : développe/réduit un répertoire ; rien pour un fichier
    /// (l'ouverture du fichier est gérée par l'app). Retourne Some(true) si
    /// un répertoire a été basculé, Some(false) si c'était un fichier.
    pub fn toggle_expanded(&mut self) -> Option<bool> {
        let vis = self.visible_entries();
        let (idx, entry) = match vis.get(self.selected) {
            Some((i, e)) => (*i, (*e).clone()),
            None => return None,
        };
        if !entry.is_dir {
            return Some(false);
        }
        let now_expanded = !entry.expanded;
        self.entries[idx].expanded = now_expanded;

        if now_expanded {
            // Insérer les enfants juste après le répertoire
            let children = self.read_children(&entry.path, entry.depth + 1);
            for (i, child) in children.into_iter().enumerate() {
                self.entries.insert(idx + 1 + i, child);
            }
        } else {
            // Retirer tout le sous-arbre : les entrées contiguës de
            // profondeur > entry.depth qui suivent immédiatement
            let mut remove_count = 0;
            while idx + 1 + remove_count < self.entries.len()
                && self.entries[idx + 1 + remove_count].depth > entry.depth
            {
                remove_count += 1;
            }
            self.entries.drain(idx + 1..idx + 1 + remove_count);
        }
        Some(true)
    }

    /// Déplie les parents d'un chemin pour que le fichier soit visible,
    /// et sélectionne l'entrée correspondante (style VS Code "reveal").
    pub fn reveal(&mut self, path: &PathBuf) {
        // Collecter les ancêtres de la racine vers le fichier
        let mut ancestors: Vec<PathBuf> = Vec::new();
        let mut cur = path.clone();
        while cur.starts_with(&self.root) && cur != self.root {
            ancestors.push(cur.clone());
            match cur.parent() {
                Some(p) => cur = p.to_path_buf(),
                None => break,
            }
        }
        ancestors.reverse();

        // Déplier successivement chaque niveau
        for ancestor in ancestors {
            // chercher l'entrée correspondante
            let mut target_idx: Option<usize> = None;
            for (i, e) in self.entries.iter().enumerate() {
                if e.path == ancestor && e.is_dir && !e.expanded {
                    target_idx = Some(i);
                    break;
                }
            }
            if let Some(i) = target_idx {
                self.entries[i].expanded = true;
                let entry = self.entries[i].clone();
                let children = self.read_children(&entry.path, entry.depth + 1);
                for (j, child) in children.into_iter().enumerate() {
                    self.entries.insert(i + 1 + j, child);
                }
            }
        }

        // Sélectionner le fichier
        for (pos, (_, e)) in self.visible_entries().iter().enumerate() {
            if e.path == *path {
                self.selected = pos;
                break;
            }
        }
    }

    /// Compte de fichiers projet (bilan statistique).
    pub fn file_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.is_dir).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_explorer_build_and_expand() {
        let base = std::env::temp_dir().join(format!("nexus_explorer_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("src")).unwrap();
        fs::write(base.join("src/a.rs"), "fn a() {}\n").unwrap();
        fs::write(base.join("Cargo.toml"), "[package]\n").unwrap();

        let mut ex = Explorer::new(base.clone());
        assert!(ex.entries.len() >= 2, "au moins src/ et Cargo.toml");
        // Les répertoires d'abord
        assert!(ex.entries[0].is_dir);

        // Déplier src/
        ex.selected = 0;
        let was_dir = ex.toggle_expanded();
        assert_eq!(was_dir, Some(true));
        let vis = ex.visible_entries();
        assert!(vis.iter().any(|(_, e)| e.name == "a.rs"));

        // Reveal
        ex.reveal(&base.join("src/a.rs"));
        let sel = ex.selected_entry().unwrap();
        assert_eq!(sel.name, "a.rs");

        let _ = fs::remove_dir_all(&base);
    }
}
