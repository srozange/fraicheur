//! État de l'application et gestion des événements clavier.

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;

use crate::index::Project;
use crate::search::Searcher;

/// Action choisie par l'utilisateur en sortie.
#[derive(Debug, Clone, Copy)]
pub enum Outcome {
    Cd,
    Resume,
}

pub struct App {
    pub projects: Vec<Project>,
    searcher: Searcher,
    pub query: String,
    /// Indices dans `projects`, ordonnés par pertinence.
    pub filtered: Vec<usize>,
    pub list_state: ListState,
    /// Résultat retenu (action + chemin) une fois validé.
    pub outcome: Option<(Outcome, PathBuf)>,
    pub quit: bool,
}

impl App {
    pub fn new(projects: Vec<Project>, initial_query: String) -> Self {
        let mut app = Self {
            projects,
            searcher: Searcher::new(),
            query: initial_query,
            filtered: Vec::new(),
            list_state: ListState::default(),
            outcome: None,
            quit: false,
        };
        app.refilter();
        app
    }

    /// Recalcule la liste filtrée et repositionne la sélection en tête.
    fn refilter(&mut self) {
        self.filtered = self.searcher.search(&self.projects, &self.query);
        if self.filtered.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    /// Projet actuellement surligné, le cas échéant.
    pub fn selected_project(&self) -> Option<&Project> {
        let sel = self.list_state.selected()?;
        let idx = *self.filtered.get(sel)?;
        self.projects.get(idx)
    }

    fn move_down(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) if i + 1 < self.filtered.len() => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn move_up(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(0) | None => self.filtered.len() - 1,
            Some(i) => i - 1,
        };
        self.list_state.select(Some(i));
    }

    fn confirm(&mut self, outcome: Outcome) {
        if let Some(p) = self.selected_project() {
            self.outcome = Some((outcome, p.path.clone()));
            self.quit = true;
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Esc => self.quit = true,
            KeyCode::Char('c') if ctrl => self.quit = true,
            KeyCode::Char('r') if ctrl => self.confirm(Outcome::Resume),
            KeyCode::Enter => self.confirm(Outcome::Cd),
            KeyCode::Down => self.move_down(),
            KeyCode::Up => self.move_up(),
            KeyCode::Char('n') if ctrl => self.move_down(),
            KeyCode::Char('p') if ctrl => self.move_up(),
            KeyCode::Backspace => {
                self.query.pop();
                self.refilter();
            }
            KeyCode::Char(c) if !ctrl => {
                self.query.push(c);
                self.refilter();
            }
            _ => {}
        }
    }
}
