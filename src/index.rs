//! Découverte des projets et lecture des documents indexés.

use std::path::PathBuf;

use walkdir::WalkDir;

use crate::config::{expand, Config};

/// Taille max de contenu doc indexé par fichier (octets).
const MAX_DOC_BYTES: usize = 8 * 1024;
/// Taille max de l'extrait de preview.
const MAX_PREVIEW_BYTES: usize = 4 * 1024;

/// Fichiers considérés comme marqueurs/sources de doc d'un projet.
const DOC_FILES: &[&str] = &["README.md", "CLAUDE.md", "AGENT.md", "AGENTS.md"];

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub workspace: String,
    pub path: PathBuf,
    /// Texte concaténé servant à la recherche floue (nom + docs).
    pub haystack: String,
    /// Extrait affiché dans le panneau de preview.
    pub preview: String,
}

/// Un répertoire est un projet s'il contient `.git/` ou un des fichiers doc.
fn is_project(dir: &std::path::Path) -> bool {
    if dir.join(".git").exists() {
        return true;
    }
    DOC_FILES.iter().any(|f| dir.join(f).exists())
}

/// Lit un fichier doc s'il existe, tronqué à `max` octets (sur une frontière char).
fn read_doc(path: &std::path::Path, max: usize) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    if content.len() <= max {
        Some(content)
    } else {
        let mut end = max;
        while !content.is_char_boundary(end) {
            end -= 1;
        }
        Some(content[..end].to_string())
    }
}

/// Construit un `Project` à partir d'un répertoire découvert.
fn build_project(name: String, workspace: String, dir: PathBuf) -> Project {
    let mut haystack = name.clone();
    let mut preview = String::new();

    for f in DOC_FILES {
        let p = dir.join(f);
        if let Some(doc) = read_doc(&p, MAX_DOC_BYTES) {
            haystack.push('\n');
            haystack.push_str(&doc);
            if preview.len() < MAX_PREVIEW_BYTES {
                preview.push_str(&format!("── {f} ──\n"));
                preview.push_str(&doc);
                preview.push('\n');
            }
        }
    }

    Project {
        name,
        workspace,
        path: dir,
        haystack,
        preview,
    }
}

/// Parcourt tous les workspaces et renvoie les projets, triés par (workspace, nom).
///
/// Règle d'inclusion : si au moins un répertoire enfant d'un parent qualifie
/// (`is_project`), alors *tous* les répertoires enfants directs de ce parent
/// sont inclus — peu importe qu'ils aient eux-mêmes un marqueur.
pub fn index(config: &Config) -> Vec<Project> {
    use std::collections::{HashMap, HashSet};

    let mut projects = Vec::new();

    for ws in &config.workspaces {
        let root = expand(&ws.root);
        if !root.is_dir() {
            continue;
        }
        let depth = ws.depth.max(1);

        // Regrouper tous les dossiers par leur parent direct.
        // filter_entry élagage : WalkDir ne descend pas dans les dossiers cachés.
        let mut by_parent: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
        for entry in WalkDir::new(&root)
            .min_depth(1)
            .max_depth(depth)
            .into_iter()
            .filter_entry(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| !n.starts_with('.'))
                    .unwrap_or(true)
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path().to_path_buf();
            if !path.is_dir() {
                continue;
            }
            if let Some(parent) = path.parent() {
                by_parent.entry(parent.to_path_buf()).or_default().push(path);
            }
        }

        // Si au moins un enfant qualifie, inclure tous les enfants du même parent.
        let mut to_include: HashSet<PathBuf> = HashSet::new();
        for children in by_parent.values() {
            if children.iter().any(|d| is_project(d)) {
                to_include.extend(children.iter().cloned());
            }
        }

        for dir in to_include {
            let name = dir
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            projects.push(build_project(name, ws.name.clone(), dir));
        }
    }

    projects.sort_by(|a, b| {
        (a.workspace.as_str(), a.name.as_str()).cmp(&(b.workspace.as_str(), b.name.as_str()))
    });
    projects
}
