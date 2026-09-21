# fraîcheur

TUI (Rust/ratatui) de recherche floue et de saut dans les projets de travail.

## Binaire & installation

- Le binaire utilisé au runtime est **`~/.local/bin/fraicheur`** — c'est là que pointe
  la fonction shell (`shell/fraicheur.zsh:16`, via `_FRAICHEUR_BIN`).
- **Ne pas** se fier à `cargo install --path .` : il écrit dans `~/.cargo/bin/`, qui
  n'est pas le binaire lancé. Pour déployer un changement en local :

  ```sh
  cargo build --release
  cp target/release/fraicheur ~/.local/bin/
  ```

- fraîcheur indexe au démarrage : relancer l'appli après un `cp` pour voir l'effet.

## Config

- `~/.config/fraicheur/config.toml` (ou `$FRAICHEUR_CONFIG`) liste les `[[workspace]]`
  (name, root, depth).
