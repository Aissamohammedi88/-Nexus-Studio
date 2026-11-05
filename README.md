# -Nexus-Studio
 Nexus Studio v1.0: The Sovereign IDE. 107x smaller than VS Code (745 KB). Built in Rust with a native Prolog engine for deterministic, logic-based security. Zero-Cloud, Zero-Path architecture. The Chaima Architect Protocol.
<div align="center">
  <h1>🛡️ NEXUS STUDIO v1.0</h1>
  <p><strong>The Sovereign IDE. 107x Smaller than VS Code. 100% Deterministic.</strong></p>
  
  [![Rust](https://img.shields.io/badge/Core-Rust_|_1500_LoC-000000?style=for-the-badge&logo=rust&logoColor=white)](#)
  [![Prolog](https://img.shields.io/badge/Logic-Prolog_Engine-00ff9d?style=for-the-badge)](#)
  [![Size](https://img.shields.io/badge/Binary-745_KB-blue?style=for-the-badge)](#)
  [![License](https://img.shields.io/badge/License-MIT-purple?style=for-the-badge)](#)
</div>

---

## 🧬 Overview

**Nexus Studio** n'est pas un éditeur de code classique. C'est un **environnement de développement souverain** qui applique une logique formelle (Prolog) pour garantir que chaque action de l'IA est mathématiquement sûre.

Inspiré par le **Protocole Chaima Architect**, Nexus Studio résout le problème fondamental des IDE modernes : l'IA génère du code, mais personne ne peut garantir qu'il est sécurisé. Ici, la logique décide, pas le modèle.

**Règle de Nexus** : *Si l'action n'est pas logiquement prouvée, elle n'existe pas.*

---

## 🧠 Le Protocole Chaima Architect (5 Règles)

L'architecture repose sur 5 piliers implémentés en Rust et en Prolog :

### 1. Découplage Logique/Modèle
- **MockLLM Proposer** : Le modèle propose des suggestions.
- **ChaimaLayer.gate_llm_suggestion()** : La couche logique valide ou rejette. **Si la suggestion est dangereuse, elle est éliminée.**

### 2. Zéro-Path
- Si une action viole les règles Prolog (`allowed(Action)` échoue), elle retourne `ZeroPath`.
- **Ce n'est pas une erreur, c'est une impossibilité logique.** L'action n'existe pas.

### 3. Immunité Architecturale
- Le moteur Prolog natif est écrit en Rust.
- Les prédicats intégrés (comme `atom_not_contains`) sont **non-modifiables par l'utilisateur**.

### 4. Certitude Zéro-Delta
- Le solveur SLD (Selective Linear Definite) est récursif et déterministe.
- **Performance** : 12 microsecondes par évaluation. Pas de hasard, pas de probabilité.

### 5. Souveraineté de l'Intention
- Chaque action est liée à la précédente par une **chaîne SHA-256**.
- Impossible de faire une action sans laisser une trace cryptographique.

---

## 🚀 Performance & Preuves

| Métrique | Nexus Studio | VS Code |
| :--- | :--- | :--- |
| **Taille du binaire** | **745 KB** | ~80 MB |
| **Langage core** | Rust + Prolog | Electron/TypeScript |
| **Déterminisme** | ✅ 100% (SLD) | ⚠️ Probabiliste |
| **Temps d'évaluation** | 12 μs | N/A |

### Tests Validés (14/14 + 3 Démos)

- **14 tests unitaires** : ✅ PASS.
- **Mode `--demo`** : ✅ Démonstration des actions autorisées/rejetées.
- **Mode `--prolog`** : ✅ Résolution de requêtes Prolog (ancestor/backtracking).
- **Mode `--bench`** : ✅ Bench du solveur (12 μs).

### Démonstrations Prouvées

| Action | Résultat | Explication |
| :--- | :--- | :--- |
| `eval(...)` | ⊘ **Zéro-Path** | Bloqué par le gate (danger). |
| `system_call(...)` | ⊘ **Zéro-Path** | Bloqué par le gate (danger). |
| `exec(...)` | ⊘ **Zéro-Path** | Bloqué par le gate (danger). |
| `fn hello()` | ✓ **Autorisé** | Code sûr. |
| `def greet()` | ✓ **Autorisé** | Code sûr. |
| `ancestor(tom, Who)` | **bob, alice, charlie, dave** | Backtracking Prolog fonctionnel. |

---

## 🛠️ Installation & Utilisation

### Prérequis
- `rustc` (Rust compiler).

### Compilation
```bash
# 1. Installer Rust si pas déjà fait
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 2. Compiler
cd ~/Téléchargements/nexus-studio
cargo build --release

# 3. Lancer
./target/release/nexus --demo
git clone https://github.com/Aissamohammedi88/nexus-studio.git
cd nexus-studio
cargo build --release
