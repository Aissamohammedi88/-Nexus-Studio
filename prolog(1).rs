//! Mini-moteur Prolog natif — unification + backtracking + règles non-déterministes
//!
//! Implémentation souveraine : aucune dépendance externe, aucune fuite de contrôle.
//! Supporte : atomes, variables, entiers, termes composés, règles (:-), requêtes (?-).
//!
//! Règle Chaima 3 (Immunité Architecturale) : le moteur est entièrement maîtrisé,
//! aucune attaque par dépendance possible.

use std::collections::HashMap;
use std::fmt;

// ============================================================================
// TERME PROLOG
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Term {
    /// Atome : `tom`, `write_file`, `+/+`
    Atom(String),
    /// Variable : `X`, `Path`, `_`
    Var(String),
    /// Entier
    Int(i64),
    /// Terme composé : `foo(a, b, c)`
    Struct(String, Vec<Term>),
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Atom(s) => write!(f, "{}", s),
            Term::Var(s) => write!(f, "{}", s),
            Term::Int(i) => write!(f, "{}", i),
            Term::Struct(name, args) => {
                if args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(f, "{}({})", name, args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", "))
                }
            }
        }
    }
}

impl Term {
    /// Substitution d'une variable par sa valeur liée
    pub fn substitute(&self, bindings: &HashMap<String, Term>) -> Term {
        match self {
            Term::Var(name) => {
                if let Some(value) = bindings.get(name) {
                    value.substitute(bindings)
                } else {
                    self.clone()
                }
            }
            Term::Struct(name, args) => {
                Term::Struct(name.clone(), args.iter().map(|a| a.substitute(bindings)).collect())
            }
            _ => self.clone(),
        }
    }

    /// Récupère toutes les variables libres du terme
    pub fn variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_vars(&mut vars);
        vars
    }

    fn collect_vars(&self, vars: &mut Vec<String>) {
        match self {
            Term::Var(name) => {
                if !vars.contains(name) {
                    vars.push(name.clone());
                }
            }
            Term::Struct(_, args) => {
                for a in args {
                    a.collect_vars(vars);
                }
            }
            _ => {}
        }
    }
}

// ============================================================================
// UNIFICATION
// ============================================================================

/// Tente d'unifier deux termes. Retourne les bindings nécessaires, ou None si échec.
///
/// Règle Chaima 4 (Certitude Zéro-Delta) : l'unification est déterministe,
/// pas de stochasticité, Δ=0 garanti.
pub fn unify(a: &Term, b: &Term, bindings: &mut HashMap<String, Term>) -> bool {
    let a = resolve(a, bindings);
    let b = resolve(b, bindings);

    match (a, b) {
        (Term::Atom(x), Term::Atom(y)) => x == y,
        (Term::Int(x), Term::Int(y)) => x == y,
        (Term::Var(name), other) | (other, Term::Var(name)) => {
            bindings.insert(name, other);
            true
        }
        (Term::Struct(n1, a1), Term::Struct(n2, a2)) => {
            if n1 != n2 || a1.len() != a2.len() {
                return false;
            }
            a1.iter().zip(a2.iter()).all(|(x, y)| unify(x, y, bindings))
        }
        _ => false,
    }
}

/// Résout une variable vers sa valeur liée si possible
fn resolve(term: &Term, bindings: &HashMap<String, Term>) -> Term {
    let mut current = term.clone();
    while let Term::Var(name) = &current {
        if let Some(value) = bindings.get(name) {
            if value == &current {
                break;
            }
            current = value.clone();
        } else {
            break;
        }
    }
    current
}

// ============================================================================
// CLAUSE ET BASE DE CONNAISSANCE
// ============================================================================

#[derive(Clone, Debug)]
pub struct Clause {
    /// Tête de clause : `allowed(X)`
    pub head: Term,
    /// Corps : liste de buts (vide pour un fait)
    pub body: Vec<Term>,
}

impl Clause {
    pub fn fact(head: Term) -> Self {
        Clause { head, body: vec![] }
    }

    pub fn rule(head: Term, body: Vec<Term>) -> Self {
        Clause { head, body }
    }
}

#[derive(Debug, Default)]
pub struct KnowledgeBase {
    clauses: Vec<Clause>,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, clause: Clause) {
        self.clauses.push(clause);
    }

    pub fn add_fact(&mut self, head: Term) {
        self.add(Clause::fact(head));
    }

    pub fn add_rule(&mut self, head: Term, body: Vec<Term>) {
        self.add(Clause::rule(head, body));
    }

    /// Requête : retourne toutes les solutions (DFS récursif avec backtracking).
    ///
    /// Implémentation standard SLD : pour chaque but, essaie chaque clause
    /// correspondante, unifie, et résout le corps récursivement.
    /// Limite à 1000 solutions pour éviter boucles infinies.
    pub fn query(&self, goal: &Term) -> Vec<HashMap<String, Term>> {
        let mut solutions = Vec::new();
        let mut counter: u64 = 0;
        self.solve_goals(&[goal.clone()], &HashMap::new(), &mut solutions, &mut counter);
        solutions
    }

    /// Solveur récursif SLD
    fn solve_goals(
        &self,
        goals: &[Term],
        bindings: &HashMap<String, Term>,
        solutions: &mut Vec<HashMap<String, Term>>,
        counter: &mut u64,
    ) {
        // Limite anti-boucle
        if solutions.len() >= 1000 || *counter > 50000 {
            return;
        }

        if goals.is_empty() {
            solutions.push(bindings.clone());
            return;
        }

        let goal = goals[0].substitute(bindings);
        let rest = &goals[1..];

        // 1. Prédicats built-in (évalués en Rust, pas en clauses)
        if let Some(succeeded) = self.eval_builtin(&goal, bindings) {
            if succeeded {
                self.solve_goals(rest, bindings, solutions, counter);
            }
            return;
        }

        // 2. Clauses utilisateur
        let functor = functor_of(&goal);
        for clause in &self.clauses {
            if functor_of(&clause.head) != functor {
                continue;
            }
            *counter += 1;
            let suffix = format!("_{}", counter);
            let renamed_head = rename_vars(&clause.head, &suffix);
            let renamed_body: Vec<Term> = clause.body.iter().map(|t| rename_vars(t, &suffix)).collect();

            let mut new_bindings = bindings.clone();
            if unify(&goal, &renamed_head, &mut new_bindings) {
                let mut new_goals = renamed_body;
                new_goals.extend_from_slice(rest);
                self.solve_goals(&new_goals, &new_bindings, solutions, counter);
            }
        }
    }

    /// Évaluation des prédicats built-in — Règle Chaima 3 (Immunité Architecturale) :
    /// les vérifications critiques sont en Rust, non en clauses user-modifiables.
    fn eval_builtin(&self, goal: &Term, bindings: &HashMap<String, Term>) -> Option<bool> {
        match goal {
            Term::Struct(name, args) if name == "atom_contains" && args.len() == 2 => {
                let haystack = args[0].substitute(bindings).to_string();
                let needle = args[1].substitute(bindings).to_string();
                Some(haystack.contains(&needle))
            }
            Term::Struct(name, args) if name == "atom_not_contains" && args.len() == 2 => {
                let haystack = args[0].substitute(bindings).to_string();
                let needle = args[1].substitute(bindings).to_string();
                Some(!haystack.contains(&needle))
            }
            Term::Struct(name, args) if name == "atom_starts_with" && args.len() == 2 => {
                let s = args[0].substitute(bindings).to_string();
                let prefix = args[1].substitute(bindings).to_string();
                Some(s.starts_with(&prefix))
            }
            _ => None,
        }
    }

    fn matching_clauses(&self, goal: &Term) -> impl Iterator<Item = &Clause> {
        let functor = functor_of(goal);
        self.clauses.iter().filter(move |c| functor_of(&c.head) == functor)
    }
}

fn functor_of(term: &Term) -> (String, usize) {
    match term {
        Term::Atom(s) => (s.clone(), 0),
        Term::Struct(name, args) => (name.clone(), args.len()),
        _ => (String::new(), 0),
    }
}

fn rename_vars(term: &Term, suffix: &str) -> Term {
    match term {
        Term::Var(name) => Term::Var(format!("{}{}", name, suffix)),
        Term::Struct(name, args) => {
            Term::Struct(name.clone(), args.iter().map(|a| rename_vars(a, suffix)).collect())
        }
        other => other.clone(),
    }
}

// ============================================================================
// PARSER PROLOG (simplifié)
// ============================================================================

pub struct Parser {
    tokens: Vec<String>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let tokens = tokenize(input);
        Parser { tokens, pos: 0 }
    }

    pub fn parse_program(input: &str) -> (Vec<Clause>, Vec<Term>) {
        let mut parser = Parser::new(input);
        let mut clauses = Vec::new();
        let mut queries = Vec::new();

        while parser.pos < parser.tokens.len() {
            if parser.peek() == Some("?") {
                parser.advance();
                let goal = parser.parse_term().unwrap();
                parser.expect(".");
                queries.push(goal);
            } else {
                let head = parser.parse_term().unwrap();
                let mut body = Vec::new();
                if parser.peek() == Some(":=") || parser.peek() == Some(":-") {
                    parser.advance();
                    loop {
                        let g = parser.parse_term().unwrap();
                        body.push(g);
                        if parser.peek() == Some(",") {
                            parser.advance();
                        } else {
                            break;
                        }
                    }
                }
                parser.expect(".");
                clauses.push(Clause { head, body });
            }
        }

        (clauses, queries)
    }

    pub fn parse_query(input: &str) -> Option<Term> {
        let mut parser = Parser::new(input);
        if parser.peek() == Some("?-") {
            parser.advance();
        }
        let term = parser.parse_term()?;
        Some(term)
    }

    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.pos).map(|s| s.as_str())
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, tok: &str) {
        if self.peek() == Some(tok) {
            self.advance();
        }
    }

    fn parse_term(&mut self) -> Option<Term> {
        let tok = self.tokens.get(self.pos)?.clone();
        self.pos += 1;

        // Variable : commence par majuscule ou _
        if tok.starts_with(|c: char| c.is_uppercase()) || tok.starts_with('_') {
            return Some(Term::Var(tok));
        }

        // Entier
        if let Ok(i) = tok.parse::<i64>() {
            return Some(Term::Int(i));
        }

        // Atome simple
        if self.peek() != Some(&"(".to_string()) {
            return Some(Term::Atom(tok));
        }

        // Terme composé : name(args)
        self.advance(); // consomme (
        let mut args = Vec::new();
        if self.peek() != Some(&")".to_string()) {
            loop {
                args.push(self.parse_term()?);
                if self.peek() == Some(&",".to_string()) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(")");
        Some(Term::Struct(tok, args))
    }
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }

        // Ponctuation multi-caractères
        let rest: String = chars.clone().collect();
        if rest.starts_with(":-") {
            tokens.push(":-".to_string());
            for _ in 0..2 { chars.next(); }
            continue;
        }
        if rest.starts_with("?-") {
            tokens.push("?-".to_string());
            for _ in 0..2 { chars.next(); }
            continue;
        }

        // Ponctuation simple
        if "(,).".contains(c) {
            tokens.push(c.to_string());
            chars.next();
            continue;
        }

        // Identificateur / atome / variable / nombre
        let mut buf = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() || "(,).".contains(c) {
                break;
            }
            buf.push(c);
            chars.next();
        }
        if !buf.is_empty() {
            tokens.push(buf);
        }
    }

    tokens
}

// ============================================================================
// TESTS UNITAIRES
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unification_atom() {
        let mut b = HashMap::new();
        assert!(unify(&Term::Atom("foo".to_string()), &Term::Atom("foo".to_string()), &mut b));
        assert!(!unify(&Term::Atom("foo".to_string()), &Term::Atom("bar".to_string()), &mut b));
    }

    #[test]
    fn test_unification_var() {
        let mut b = HashMap::new();
        assert!(unify(&Term::Var("X".to_string()), &Term::Atom("foo".to_string()), &mut b));
        assert_eq!(b.get("X"), Some(&Term::Atom("foo".to_string())));
    }

    #[test]
    fn test_query_basic() {
        let mut kb = KnowledgeBase::new();
        kb.add_fact(Term::Struct("parent".to_string(), vec![Term::Atom("tom".to_string()), Term::Atom("bob".to_string())]));
        kb.add_fact(Term::Struct("parent".to_string(), vec![Term::Atom("bob".to_string()), Term::Atom("alice".to_string())]));
        kb.add_rule(
            Term::Struct("ancestor".to_string(), vec![Term::Var("X".to_string()), Term::Var("Z".to_string())]),
            vec![
                Term::Struct("parent".to_string(), vec![Term::Var("X".to_string()), Term::Var("Y".to_string())]),
                Term::Struct("ancestor_or_self".to_string(), vec![Term::Var("Y".to_string()), Term::Var("Z".to_string())]),
            ],
        );
        kb.add_rule(
            Term::Struct("ancestor_or_self".to_string(), vec![Term::Var("X".to_string()), Term::Var("X".to_string())]),
            vec![],
        );
        kb.add_rule(
            Term::Struct("ancestor_or_self".to_string(), vec![Term::Var("X".to_string()), Term::Var("Z".to_string())]),
            vec![
                Term::Struct("parent".to_string(), vec![Term::Var("X".to_string()), Term::Var("Y".to_string())]),
                Term::Struct("ancestor_or_self".to_string(), vec![Term::Var("Y".to_string()), Term::Var("Z".to_string())]),
            ],
        );

        let goal = Term::Struct("ancestor".to_string(), vec![Term::Atom("tom".to_string()), Term::Var("Who".to_string())]);
        let solutions = kb.query(&goal);
        assert!(!solutions.is_empty(), "doit trouver au moins une solution");
    }

    #[test]
    fn test_zeropath_rejection() {
        // Si aucune clause ne correspond, aucune solution — Zéro-Path
        let kb = KnowledgeBase::new();
        let goal = Term::Struct("forbidden".to_string(), vec![Term::Atom("x".to_string())]);
        let solutions = kb.query(&goal);
        assert!(solutions.is_empty(), "Zéro-Path : le chemin n'existe pas");
    }
}
