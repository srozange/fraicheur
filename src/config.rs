//! Chargement de la configuration utilisateur (liste des workspaces).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

/// Un workspace = une racine sous laquelle vivent des projets.
#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub root: String,
    /// Profondeur de scan sous la racine (1 = sous-répertoires directs).
    #[serde(default = "default_depth")]
    pub depth: usize,
}

fn default_depth() -> usize {
    1
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default, rename = "workspace")]
    pub workspaces: Vec<Workspace>,
}

/// Contenu écrit lorsqu'aucune config n'existe encore.
pub const EXAMPLE: &str = r#"# Configuration fraîcheur.
# Chaque [[workspace]] est une racine ; chaque sous-répertoire devient un projet.

[[workspace]]
name  = "internal"
root  = "~/workspace/internal"
depth = 1

# [[workspace]]
# name = "cnam"
# root = "~/workspace/cnam"
"#;

/// Chemin du fichier de config : `$FRAICHEUR_CONFIG` sinon
/// `~/.config/fraicheur/config.toml`.
pub fn config_path() -> PathBuf {
    if let Ok(p) = std::env::var("FRAICHEUR_CONFIG") {
        return PathBuf::from(p);
    }
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| expand("~/.config"));
    base.join("fraicheur").join("config.toml")
}

/// Expansion d'un `~` en tête de chemin.
pub fn expand(path: &str) -> PathBuf {
    PathBuf::from(shellexpand::tilde(path).into_owned())
}

impl Config {
    /// Charge la config, ou renvoie une erreur `NotFound` si le fichier manque.
    pub fn load(path: &Path) -> Result<Config> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("lecture de {}", path.display()))?;
        let cfg: Config =
            toml::from_str(&text).with_context(|| format!("parsing TOML de {}", path.display()))?;
        Ok(cfg)
    }
}
