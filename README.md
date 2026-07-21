# fraîcheur

TUI Rust pour retrouver et sauter dans ses projets de travail : recherche floue
*live* sur le **nom**, le **README.md** et les **CLAUDE.md / AGENT.md**, puis
`cd` (ou `cd` + `claude --resume`) depuis zsh.

## Installation

```sh
cargo build --release
cp target/release/fraicheur ~/.local/bin/          # doit être dans le PATH
```

Intégration zsh — ajoutez à `~/.zshrc` :

```sh
source /chemin/vers/fraicheur/shell/fraicheur.zsh   # définit `fraicheur` + Ctrl+G
```

## Config

Copiez `config.example.toml` vers `~/.config/fraicheur/config.toml`
(ou pointez `$FRAICHEUR_CONFIG` dessus). Chaque sous-répertoire d'une racine
contenant `.git/`, un `README.md`, un `CLAUDE.md` ou un `AGENT.md` est indexé.

```toml
[[workspace]]
name  = "internal"
root  = "~/workspace/internal"
depth = 1
```

## Usage

- `fraicheur` (ou **Ctrl+G**) → ouvre le TUI ; le texte déjà tapé sert de filtre.
- `fraicheur "lang"` → ouvre pré-filtré.

| Touche           | Action                                       |
|------------------|----------------------------------------------|
| frappe           | filtre les résultats en direct               |
| `↑` / `↓`        | naviguer (aussi `Ctrl+P` / `Ctrl+N`)         |
| `Enter`          | `cd` dans le projet                          |
| `Ctrl+R`         | `cd` dans le projet **+ `claude --resume`**  |
| `Esc` / `Ctrl+C` | annuler                                      |

## Fonctionnement

Un binaire ne peut pas changer le `cwd` du shell parent : le TUI dessine sur
`/dev/tty` et imprime `CD<TAB>chemin` sur stdout ; la fonction zsh `fraicheur`
capture cette sortie et exécute le `cd` (même principe que `fzf`).

## Licence

MIT
