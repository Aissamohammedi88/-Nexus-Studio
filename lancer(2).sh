#!/bin/bash
# ============================================================
#  NEXUS STUDIO v3 — DÉPLOIEMENT SOUVERAIN ANTI-FERMETURE
#  Protocole Chaima Architect v1.0
#
#  COMMENT LANCER (IMPORTANT) :
#    1. Enregistre ce fichier sous : ~/Téléchargements/lancer.sh
#       (clic droit → Enregistrer sous, ou copier dans un éditeur)
#    2. Ouvre un terminal et tape EXACTEMENT :
#         bash ~/Téléchargements/lancer.sh
#    3. Ne colle PAS le contenu directement dans le terminal :
#       ça cassera les commandes `read` et l'éditeur TUI.
#
#  Mode sûr (sans éditeur TUI, pour diagnostic) :
#         bash ~/Téléchargements/lancer.sh --safe
# ============================================================

# IMPORTANT : ni `set -e` ni `exit` — on veut survivre à toute erreur.
set -u

# --- Trap : quoiqu'il arrive, on restaure le terminal ---
restore_terminal() {
    # Restaure l'état du terminal si le TUI a laissé du bazar
    if [ -t 0 ]; then
        stty sane 2>/dev/null || true
        tput cnorm 2>/dev/null || true  # ré-afficher le curseur
        tput sgr0 2>/dev/null || true   # reset couleurs
    fi
    echo ""
    echo "[Déploiement terminé — retour shell normal]"
}
trap restore_terminal EXIT INT TERM

SAFE_MODE=0
if [ "${1:-}" = "--safe" ]; then
    SAFE_MODE=1
fi

# --- Fonction : pause non bloquante si stdin n'est pas un TTY ---
pause_lecture() {
    local msg="${1:-Appuie sur ENTRÉE pour continuer...}"
    if [ -t 0 ]; then
        read -r -p "$msg" _ 2>/dev/null || true
    else
        echo "$msg (stdin non interactif — on continue)"
    fi
}

main() {

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  NEXUS STUDIO v3 — Déploiement souverain (anti-suicide)     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "  Mode : $([ "$SAFE_MODE" = "1" ] && echo "SÛR (sans TUI)" || echo "COMPLET (avec TUI)")"
echo "  PID  : $$  (si ça bloque, tape :  kill $$)"
echo ""

# --- 1. Activer Rust dans ce terminal ---
if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
fi

if ! command -v cargo >/dev/null 2>&1 || ! cargo --version >/dev/null 2>&1; then
    echo "✗ Cargo inutilisable. Dans CE terminal, tape :"
    echo "    . \"\$HOME/.cargo/env\""
    echo "  puis relance :  bash ~/Téléchargements/lancer.sh"
    pause_lecture "Appuie sur ENTRÉE pour quitter proprement..."
    return 1
fi
echo "✓ Rust actif : $(cargo --version)"

# --- 2. Créer le dossier projet propre ---
PROJET="$HOME/nexus-studio"
mkdir -p "$PROJET/src"
echo "✓ Dossier projet : $PROJET"

# --- 3. Récupérer les 7 fichiers .rs où qu'ils soient dans le home ---
MODULES="main prolog chaima editor llm app ui"
TROUVES=0
for f in $MODULES; do
    SRC=$(find "$HOME" -maxdepth 4 -name "$f.rs" \
        -not -path "*/.cargo/*" -not -path "*/.rustup/*" \
        -not -path "*/nexus-studio/*" \
        -not -path "*/target/*" 2>/dev/null | head -1)
    if [ -n "$SRC" ]; then
        cp "$SRC" "$PROJET/src/$f.rs"
        TROUVES=$((TROUVES+1))
        echo "✓ $f.rs ← $SRC"
    else
        echo "✗ $f.rs introuvable dans ton home"
    fi
done

if [ "$TROUVES" -lt 7 ]; then
    echo ""
    echo "⚠ Seulement $TROUVES/7 modules trouvés."
    echo "  Télécharge les 7 fichiers .rs depuis la conversation Z.ai"
    echo "  (bouton Télécharger de chaque fichier), mets-les dans"
    echo "  ~/Téléchargements puis relance ce script."
    pause_lecture "Appuie sur ENTRÉE pour quitter proprement..."
    return 1
fi

# --- 4. Récupérer le Cargo.toml ---
if [ -f "$PROJET/Cargo.toml" ]; then
    echo "✓ Cargo.toml déjà en place (relance)"
else
    CARGO=$(find "$HOME" -maxdepth 4 \( -name "Cargo.toml" -o -name "Cargo.toml.bak" \) \
        -not -path "*/.cargo/*" -not -path "*/.rustup/*" \
        -not -path "*/nexus-studio/*" \
        -not -path "*/target/*" 2>/dev/null | head -1)
    if [ -n "$CARGO" ]; then
        cp "$CARGO" "$PROJET/Cargo.toml"
        echo "✓ Cargo.toml ← $CARGO"
    else
        echo "✗ Cargo.toml introuvable — télécharge-le depuis la conversation."
        pause_lecture "Appuie sur ENTRÉE pour quitter proprement..."
        return 1
    fi
fi

# --- 5. Nettoyer le Cargo.toml égaré dans home ---
if [ -f "$HOME/Cargo.toml" ] && [ "$HOME" != "$PROJET" ]; then
    mv "$HOME/Cargo.toml" "$HOME/Cargo.toml.bak" 2>/dev/null \
        && echo "✓ ~/Cargo.toml mis de côté (→ Cargo.toml.bak)"
fi

# --- 6. Compiler ---
echo ""
echo "── Compilation (release, LTO) — 1 à 2 minutes ──"
cd "$PROJET" || { pause_lecture; return 1; }
cargo build --release 2>&1 | tail -5
BIN="$PROJET/target/release/nexus"
if [ ! -x "$BIN" ]; then
    echo "✗ Échec compilation — binaire absent."
    echo "  Relance avec :  bash ~/Téléchargements/lancer.sh --safe"
    echo "  pour voir les logs complets :"
    echo "    cd $PROJET && cargo build --release"
    pause_lecture "Appuie sur ENTRÉE pour quitter proprement..."
    return 1
fi
echo "✓ Binaire prêt : $(ls -lh "$BIN" | awk '{print $5, $9}')"

# --- 7. Créer le fichier mission.rs ---
if [ ! -f "$PROJET/mission.rs" ]; then
    cat > "$PROJET/mission.rs" <<'MISSIONEOF'
// ============================================
//  MISSION NEXUS — fichier créé par le déploiement
//  Protocole Chaima Architect v1.0
// ============================================

fn souverain() {
    // La logique humaine contrôle l'exécution, pas l'inverse.
    let intention = "autorité";
    println!("Intention liée cryptographiquement : {}", intention);
}

fn main() {
    souverain();
    // Ouvre la couche souveraine : :chaima dans Nexus Studio
}
MISSIONEOF
    echo "✓ Fichier mission.rs créé dans $PROJET"
else
    echo "✓ mission.rs déjà présent (relance)"
fi

# --- 8. LANCER TOUT ---
echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  LANCEMENT COMPLET                                           ║"
echo "╚══════════════════════════════════════════════════════════════╝"

echo ""
echo "═══ [1/4] PROTOCOLE CHAIMA ═══"
"$BIN" --demo || echo "  (sortie code $? — continue)"

echo ""
echo "═══ [2/4] MOTEUR PROLOG ═══"
"$BIN" --prolog || echo "  (sortie code $? — continue)"

echo ""
echo "═══ [3/4] BENCHMARK vs VS CODE ═══"
"$BIN" --bench || echo "  (sortie code $? — continue)"

# --- 4/4 : TUI ou mode safe ---
if [ "$SAFE_MODE" = "1" ]; then
    echo ""
    echo "═══ [4/4] MODE SÛR — éditeur TUI ignoré ═══"
    echo "  Pour lancer l'éditeur plus tard :"
    echo "    $BIN $PROJET/mission.rs"
    echo ""
    echo "✓ Déploiement terminé en mode sûr, Commandant."
    return 0
fi

# Vérifier qu'on a un vrai TTY avant de lancer le TUI
if [ ! -t 0 ] || [ ! -t 1 ]; then
    echo ""
    echo "═══ [4/4] ÉDITEUR TUI — NON LANCÉ ═══"
    echo "  Terminal non interactif (redirigé ou pas de TTY)."
    echo "  Pour lancer l'éditeur manuellement :"
    echo "    $BIN $PROJET/mission.rs"
    return 0
fi

echo ""
pause_lecture "═══ [4/4] Appuie sur ENTRÉE pour ouvrir l'éditeur TUI sur mission.rs... ═══"
echo "  Raccourcis : i=insérer · Esc=normal · :w=sauver · :q=quitter · :chaima=métriques"
echo "  Si l'éditeur se fige, tape :q puis Entrée. Sinon Ctrl-C et le terminal"
echo "  sera restauré automatiquement."
echo ""

# || true : si le TUI crash, on continue au lieu de tout perdre
"$BIN" "$PROJET/mission.rs" || echo "  (éditeur sorti avec code $? — terminal restauré par trap)"

echo ""
echo "✓ Déploiement terminé, Commandant."
echo "  Relancer l'éditeur plus tard :"
echo "    $BIN $PROJET/mission.rs"
return 0
}

# ===== Exécution : return ≠ exit, le terminal survit toujours =====
main
RC=$?
echo ""
echo "[Script terminé — code retour $RC]"
# La trap EXIT s'occupe de stty sane + message final.
