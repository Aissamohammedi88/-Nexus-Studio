//! Couche Modèle — Mock LLM
//!
//! Règle 1 (Découplage Logique/Modèle) : le modèle propose, la couche Chaima dispose.
//! Ce mock simule un LLM sans dépendance cloud — souveraineté totale (Règle 5).
//!
//! Le mock génère des suggestions basées sur le contexte du buffer.
//! Chaque suggestion passe par le gate Chaima avant d'être présentée.

use crate::chaima::{ChaimaLayer, Verdict};

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub code: String,
    pub confidence: f32,
    pub verdict: Verdict,
    pub reason: String,
}

pub struct MockLLM {
    /// Compteur de suggestions générées
    generated_count: u64,
    /// Compteur de suggestions acceptées par Chaima
    accepted_count: u64,
    /// Compteur de suggestions rejetées (Zéro-Path)
    rejected_count: u64,
}

impl MockLLM {
    pub fn new() -> Self {
        MockLLM {
            generated_count: 0,
            accepted_count: 0,
            rejected_count: 0,
        }
    }

    /// Génère une suggestion de complétion pour le contexte donné.
    ///
    /// Le modèle est volontairement simplifié — l'objectif n'est pas l'intelligence
    /// mais la démonstration du découplage Chaima.
    pub fn suggest(&mut self, context: &str, chaima: &mut ChaimaLayer) -> Suggestion {
        self.generated_count += 1;

        // Heuristique simple : détecter le langage par mots-clés
        let suggestion_code = self.generate_candidate(context);
        let confidence = self.estimate_confidence(context);

        // Gate Chaima : la couche souveraine décide souverainement
        let verdict = chaima.gate_llm_suggestion(&suggestion_code, context);

        match &verdict {
            Verdict::Authorized(_) => {
                self.accepted_count += 1;
            }
            Verdict::ZeroPath(_) => {
                self.rejected_count += 1;
            }
        }

        Suggestion {
            code: suggestion_code,
            confidence,
            verdict,
            reason: "Modèle découplé — Chaima a décidé souverainement".to_string(),
        }
    }

    fn generate_candidate(&self, context: &str) -> String {
        let trimmed = context.trim();
        if trimmed.is_empty() {
            return "fn main() {}".to_string();
        }

        // Détection heuristique du langage
        if trimmed.contains("fn ") || trimmed.contains("let mut") {
            // Rust
            if trimmed.ends_with("{") {
                return "fn generated() -> Result<(), String> { Ok(()) }".to_string();
            }
            if trimmed.ends_with(".") {
                return "fn helper() -> i32 { 42 }".to_string();
            }
            return "fn new_function() {}".to_string();
        }

        if trimmed.contains("def ") || trimmed.contains("import ") {
            // Python
            return "def generated(): pass".to_string();
        }

        if trimmed.contains(":-") || trimmed.contains("?-") {
            // Prolog
            return "rule(X) :- fact(X).".to_string();
        }

        // Texte par défaut — volontairement sûr
        "fn generated() {}".to_string()
    }

    fn estimate_confidence(&self, context: &str) -> f32 {
        let len = context.len();
        if len == 0 {
            0.3
        } else if len < 20 {
            0.5
        } else if len < 100 {
            0.75
        } else {
            0.85
        }
    }

    pub fn stats(&self) -> (u64, u64, u64) {
        (self.generated_count, self.accepted_count, self.rejected_count)
    }
}

impl Default for MockLLM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoupling_chaima_rejects_eval() {
        let mut llm = MockLLM::new();
        let mut chaima = ChaimaLayer::new();

        // Forcer une suggestion contenant "eval" — doit être bloquée
        let verdict = chaima.gate_llm_suggestion("eval(injection)", "context");
        assert!(verdict.is_zeropath());
    }

    #[test]
    fn test_safe_suggestion_passes() {
        let mut llm = MockLLM::new();
        let mut chaima = ChaimaLayer::new();
        let s = llm.suggest("fn hello()", &mut chaima);
        assert!(s.verdict.is_authorized());
    }
}
