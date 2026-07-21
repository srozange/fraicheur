# Intégration zsh pour fraîcheur.
#
# Installation : ajoutez à votre ~/.zshrc :
#     source /chemin/vers/fraicheur/shell/fraicheur.zsh
#
# `fraicheur` est ici une FONCTION shell qui enrobe le binaire : c'est elle qui
# exécute le `cd` (un binaire ne peut pas changer le cwd du shell parent).
#   fraicheur          → ouvre le TUI, puis cd dans le projet choisi
#   fraicheur "lang"   → ouvre pré-filtré sur « lang »
#
# Raccourci Ctrl+G : idem, mais le texte déjà tapé sur la ligne sert de requête.
#   Enter → cd   |   Ctrl+R → cd + claude --resume   |   Esc → annuler

# Chemin du binaire résolu UNE fois au chargement (indépendant du hash de
# commandes / du PATH au moment de l'appel). Repli sur l'install par défaut.
typeset -g _FRAICHEUR_BIN="${commands[fraicheur]:-$HOME/.local/bin/fraicheur}"

# Fonction du même nom que le binaire (elle l'appelle par chemin absolu).
fraicheur() {
  if [[ ! -x "$_FRAICHEUR_BIN" ]]; then
    print -u2 "fraicheur: binaire introuvable ($_FRAICHEUR_BIN)"
    return 1
  fi
  local out action path
  out=$("$_FRAICHEUR_BIN" "$@" </dev/tty) || return
  [[ -z "$out" ]] && return
  action="${out%%$'\t'*}"
  path="${out#*$'\t'}"
  cd "$path" || return
  [[ "$action" == "RESUME" ]] && claude --resume
}

# Widget ZLE : passe le BUFFER courant comme requête initiale.
fraicheur-widget() {
  if [[ ! -x "$_FRAICHEUR_BIN" ]]; then
    zle reset-prompt
    return
  fi
  local out action path
  out=$("$_FRAICHEUR_BIN" -- "$BUFFER" </dev/tty) || { zle reset-prompt; return }
  if [[ -z "$out" ]]; then
    zle reset-prompt
    return
  fi
  action="${out%%$'\t'*}"
  path="${out#*$'\t'}"
  BUFFER="cd ${(q)path}"
  [[ "$action" == "RESUME" ]] && BUFFER+=" && claude --resume"
  zle accept-line
}
zle -N fraicheur-widget
bindkey '^G' fraicheur-widget
