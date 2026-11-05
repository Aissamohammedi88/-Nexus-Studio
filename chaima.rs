//! Couche Souveraine Chaima — Protocole Chaima Architect v1.0
//!
//! Règle 1 : Découplage Logique/Modèle — le LLM propose, la Loi dispose.
//! Règle 2 : Zéro-Path — si l'action ne satisfait pas la règle, elle n'existe pas.
//! Règle 3 : Immunité Architecturale — aucune gestion post-hoc, prévention structurelle.
//! Règle 4 : Certitude Zéro-Delta — élimination de la dérive stochastique.
//! Règle 5 : Souveraineté de l'Intention — hash cryptographique liant intention à exécution.

use crate::prolog::{KnowledgeBase, Term, Parser};
use sha2::{Sha256, Digest};
use chrono::Utc;

// ============================================================================
// INTENTION — LIEN CRYPTOGRAPHIQUE
// ============================================================================

#[derive(Clone, Debug)]
pub struct Intent {
    pub id: u64,
    pub timestamp: String,
    pub action: String,
    pub context_hash: String,
    pub previous_hash: String,
    pub current_hash: String,
}

impl Intent {
    /// Calcule le hash chaîné : SHA256(id || timestamp || action || context || previous)
    ///
    /// Règle 5 : chaque exécution honore son créateur via hash lié.
    pub fn compute_hash(id: u64, timestamp: &str, action: &str, context_hash: &str, previous_hash: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(id.to_string());
        hasher.update(timestamp);
        hasher.update(action);
        hasher.update(context_hash);
        hasher.update(previous_hash);
        let result = hasher.finalize();
        result.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

// ============================================================================
// JOURNAL CHAIMA — CHAÎNE D'INTENTIONS
// ============================================================================

#[derive(Debug)]
pub struct IntentChain {
    intents: Vec<Intent>,
    next_id: u64,
    genesis_hash: String,
}

impl IntentChain {
    pub fn new() -> Self {
        // Hash genesis : SHA256("Chaima Architect v1.0")
        let mut hasher = Sha256::new();
        hasher.update(b"Chaima Architect v1.0 - Genesis");
        let genesis: String = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect();
        IntentChain {
            intents: Vec::new(),
            next_id: 1,
            genesis_hash: genesis[..16].to_string(),
        }
    }

    pub fn record(&mut self, action: &str, context: &str) -> Intent {
        let id = self.next_id;
        let timestamp = Utc::now().to_rfc3339();
        let mut ctx_hasher = Sha256::new();
        ctx_hasher.update(context.as_bytes());
        let context_hash: String = ctx_hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect();
        let context_hash = context_hash[..16].to_string();

        let previous_hash = self.intents.last().map(|i| i.current_hash.clone()).unwrap_or_else(|| self.genesis_hash.clone());

        let current_hash = Intent::compute_hash(id, &timestamp, action, &context_hash, &previous_hash);
        let current_hash = current_hash[..16].to_string();

        let intent = Intent {
            id,
            timestamp,
            action: action.to_string(),
            context_hash,
            previous_hash,
            current_hash: current_hash.clone(),
        };

        self.intents.push(intent.clone());
        self.next_id += 1;
        intent
    }

    pub fn last_n(&self, n: usize) -> Vec<&Intent> {
        let len = self.intents.len();
        if len <= n {
            self.intents.iter().collect()
        } else {
            self.intents[len - n..].iter().collect()
        }
    }

    pub fn len(&self) -> usize {
        self.intents.len()
    }
}

// ============================================================================
// VERDICT CHAIMA
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub enum Verdict {
    /// L'action est souveraine — chemin autorisé
    Authorized(String),
    /// L'action ne satisfait pas la règle — Zéro-Path : elle n'existe pas
    ZeroPath(String),
}

impl Verdict {
    pub fn is_authorized(&self) -> bool {
        matches!(self, Verdict::Authorized(_))
    }

    pub fn is_zeropath(&self) -> bool {
        matches!(self, Verdict::ZeroPath(_))
    }

    pub fn reason(&self) -> &str {
        match self {
            Verdict::Authorized(r) => r,
            Verdict::ZeroPath(r) => r,
        }
    }
}

// ============================================================================
// COUCHE SOUVERAINE CHAIMA
// ============================================================================

pub struct ChaimaLayer {
    kb: KnowledgeBase,
    chain: IntentChain,
    rules_source: String,
    /// Compteur de rejets Zéro-Path pour métriques
    zeropath_count: u64,
    /// Compteur d'autorisations
    authorized_count: u64,
}

impl ChaimaLayer {
    pub fn new() -> Self {
        let mut layer = ChaimaLayer {
            kb: KnowledgeBase::new(),
            chain: IntentChain::new(),
            rules_source: String::new(),
            zeropath_count: 0,
            authorized_count: 0,
        };
        layer.load_sovereign_rules();
        layer
    }

    /// Charge les règles souveraines fondatrices
    ///
    /// Ces règles incarnent le Protocole Chaima en logique déclarative.
    /// Elles ne peuvent PAS être contournées par la couche Modèle.
    fn load_sovereign_rules(&mut self) {
        let rules = r#"
            % === PROTOCOLE CHAIMA ARCHITECT v1.0 — RÈGLES SOUVERAINES ===
            % Variables Prolog = majuscule (X, Path, Code, Cmd)
            % Atomes = minuscule (tom, read_file, save_file)

            % Règle 2 : Zéro-Path — seul allowed/1 détermine l'existence d'un chemin.

            % Actions de base toujours autorisées (variables = jokers)
            allowed(read_file(Path)).
            allowed(cursor_move(Direction)).
            allowed(buffer_insert(Char)).
            allowed(buffer_delete).
            allowed(save_file).
            allowed(open_command_bar).
            allowed(quit_application).
            allowed(insert_mode).
            allowed(escape_insert).

            % Édition de fichier autorisée si le chemin n'est pas protégé
            allowed(edit_file(Path)) :- not_protected(Path).

            % Suggestion IA autorisée SEULEMENT si :
            %   - le code ne contient aucun motif interdit
            %   - le code correspond à une intention valide
            % Règle 1 (Découplage) : le modèle propose via llm_suggest(Code),
            % la couche Chaima décide souverainement.
            allowed(llm_suggest(Code)) :-
                safe_code(Code),
                matches_intent(Code).

            % Exécution shell nécessite confirmation humaine explicite
            allowed(shell_exec(Cmd)) :- human_confirmed(Cmd).

            % === DÉFINITIONS DES PREDICATS ===

            % Code sûr : ne contient AUCUN motif interdit (conjonction AND)
            safe_code(Code) :-
                atom_not_contains(Code, eval),
                atom_not_contains(Code, rm_rf),
                atom_not_contains(Code, system),
                atom_not_contains(Code, exec),
                atom_not_contains(Code, fork).

            % Intention valide : le code commence par un token autorisé
            % (simplification du lien cryptographique d'intention)
            matches_intent(Code) :-
                atom_starts_with(Code, fn),
                atom_not_contains(Code, eval).
            matches_intent(Code) :-
                atom_starts_with(Code, def).
            matches_intent(Code) :-
                atom_starts_with(Code, rule).
            matches_intent(Code) :-
                atom_starts_with(Code, let).

            % Chemin non protégé : ne contient aucun segment sensible
            not_protected(Path) :-
                atom_not_contains(Path, secrets),
                atom_not_contains(Path, credentials),
                atom_not_contains(Path, env_production),
                atom_not_contains(Path, system_config).

            % Confirmation humaine : seulement pour explicit_yes
            human_confirmed(explicit_yes).
        "#;

        let (clauses, _) = Parser::parse_program(rules);
        for clause in clauses {
            self.kb.add(clause);
        }
        self.rules_source = rules.to_string();
    }

    /// Évalue une action contre la couche souveraine.
    ///
    /// Règle 2 (Zéro-Path) : si `allowed(Action)` échoue, retourne ZeroPath.
    /// L'action n'est pas rejetée — elle n'existe simplement pas.
    pub fn evaluate(&mut self, action: &Term, context: &str) -> Verdict {
        let goal = Term::Struct("allowed".to_string(), vec![action.clone()]);

        let solutions = self.kb.query(&goal);
        let found = !solutions.is_empty();

        // Enregistrer l'intention dans la chaîne
        let action_str = action.to_string();
        let intent = self.chain.record(&format!("eval:{}", action_str), context);

        if found {
            self.authorized_count += 1;
            Verdict::Authorized(format!("[#{}] chemin souverain vérifié — Δ=0", intent.id))
        } else {
            self.zeropath_count += 1;
            Verdict::ZeroPath(format!("[#{}] Zéro-Path : la règle n'autorise pas ce chemin", intent.id))
        }
    }

    /// Version simplifiée : évalue une action donnée par son nom atomique.
    pub fn evaluate_action(&mut self, action_name: &str, context: &str) -> Verdict {
        let term = if action_name.contains('(') {
            Parser::parse_query(action_name)
                .unwrap_or_else(|| Term::Atom(action_name.to_string()))
        } else {
            Term::Atom(action_name.to_string())
        };
        self.evaluate(&term, context)
    }

    /// Vérifie si une suggestion LLM respecte la souveraineté.
    ///
    /// Règle 1 (Découplage) : le modèle propose via llm_suggest/1,
    /// la couche Chaima décide souverainement.
    pub fn gate_llm_suggestion(&mut self, code: &str, context: &str) -> Verdict {
        let code_term = Term::Atom(code.to_string());
        let action = Term::Struct("llm_suggest".to_string(), vec![code_term]);
        self.evaluate(&action, context)
    }

    pub fn intent_chain(&self) -> &IntentChain {
        &self.chain
    }

    pub fn metrics(&self) -> (u64, u64) {
        (self.authorized_count, self.zeropath_count)
    }

    pub fn rules_count(&self) -> usize {
        // Approximation : compte les clauses
        // (KnowledgeBase ne expose pas len directement, on estime)
        20
    }
}

impl Default for ChaimaLayer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeropath_blocks_shell_exec() {
        let mut chaima = ChaimaLayer::new();
        let verdict = chaima.evaluate_action("shell_exec(rm_file)", "test");
        assert!(verdict.is_zeropath(), "shell_exec non confirmé doit être Zéro-Path");
    }

    #[test]
    fn test_authorized_action() {
        let mut chaima = ChaimaLayer::new();
        let verdict = chaima.evaluate_action("cursor_move(left)", "test");
        assert!(verdict.is_authorized(), "cursor_move doit être autorisé");
    }

    #[test]
    fn test_intent_chain_integrity() {
        let mut chaima = ChaimaLayer::new();
        chaima.evaluate_action("save_file", "ctx1");
        chaima.evaluate_action("cursor_move(right)", "ctx2");
        let chain = chaima.intent_chain();
        assert_eq!(chain.len(), 2);
        // Vérifier chaînage : previous_hash du 2e == current_hash du 1er
        let intents = chain.last_n(2);
        assert_eq!(intents[1].previous_hash, intents[0].current_hash);
    }

    #[test]
    fn test_llm_gate_rejects_forbidden() {
        let mut chaima = ChaimaLayer::new();
        // Code contenant "eval" — pattern interdit
        let verdict = chaima.gate_llm_suggestion("eval(malicious)", "user input");
        assert!(verdict.is_zeropath(), "eval doit être bloqué par le gate");
    }

    #[test]
    fn test_llm_gate_accepts_safe() {
        let mut chaima = ChaimaLayer::new();
        let verdict = chaima.gate_llm_suggestion("fn hello()", "user input");
        assert!(verdict.is_authorized(), "fn hello() doit passer le gate");
    }
}
