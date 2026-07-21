//! Filtrage flou des projets via nucleo-matcher.

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};

use crate::index::Project;

pub struct Searcher {
    matcher: Matcher,
    buf: Vec<char>,
}

impl Searcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT),
            buf: Vec::new(),
        }
    }

    /// Renvoie les indices des projets correspondant à `query`, triés par
    /// pertinence décroissante. Query vide → tous les projets, ordre d'origine.
    pub fn search(&mut self, projects: &[Project], query: &str) -> Vec<usize> {
        let query = query.trim();
        if query.is_empty() {
            return (0..projects.len()).collect();
        }

        let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);
        let mut scored: Vec<(usize, u32)> = Vec::new();

        for (i, p) in projects.iter().enumerate() {
            // Score sur le haystack complet (nom + docs).
            let hay_score = pattern.score(
                Utf32Str::new(&p.haystack, &mut self.buf),
                &mut self.matcher,
            );
            let Some(mut score) = hay_score else {
                continue;
            };
            // Boost si le nom du projet matche aussi (proximité forte).
            if let Some(name_score) =
                pattern.score(Utf32Str::new(&p.name, &mut self.buf), &mut self.matcher)
            {
                score = score.saturating_add(name_score.saturating_mul(2));
            }
            scored.push((i, score));
        }

        // Tri par score desc, puis index asc pour un ordre stable.
        scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        scored.into_iter().map(|(i, _)| i).collect()
    }
}

impl Default for Searcher {
    fn default() -> Self {
        Self::new()
    }
}
