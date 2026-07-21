//! fraîcheur — TUI pour indexer et sauter dans ses projets de travail.

mod app;
mod config;
mod index;
mod search;
mod ui;

use std::fs::OpenOptions;

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::{App, Outcome};
use config::Config;

const HELP: &str = "\
fraîcheur — recherche floue dans vos projets de travail

USAGE:
    fraicheur [QUERY]
    fraicheur --query QUERY
    fraicheur -- QUERY        (tout ce qui suit `--` est la requête)

Config : $FRAICHEUR_CONFIG sinon ~/.config/fraicheur/config.toml
Sortie : `CD<TAB>chemin` ou `RESUME<TAB>chemin` sur stdout (rien si annulé).
";

/// Extrait la requête initiale des arguments de ligne de commande.
fn parse_query(args: &[String]) -> Option<String> {
    let mut query: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--" => return Some(args[i + 1..].join(" ")),
            "--query" | "-q" => {
                if let Some(v) = args.get(i + 1) {
                    query = Some(v.clone());
                    i += 1;
                }
            }
            "-h" | "--help" => {
                print!("{HELP}");
                std::process::exit(0);
            }
            other => {
                if query.is_none() {
                    query = Some(other.to_string());
                }
            }
        }
        i += 1;
    }
    query
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let query = parse_query(&args).unwrap_or_default();

    // --- Config ---
    let cfg_path = config::config_path();
    if !cfg_path.exists() {
        if let Some(parent) = cfg_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&cfg_path, config::EXAMPLE)
            .with_context(|| format!("écriture config exemple {}", cfg_path.display()))?;
        eprintln!(
            "Aucune config trouvée. Un exemple a été créé :\n  {}\n\
             Éditez-le pour lister vos workspaces, puis relancez.",
            cfg_path.display()
        );
        std::process::exit(1);
    }
    let cfg = Config::load(&cfg_path)?;

    // --- Indexation ---
    let projects = index::index(&cfg);
    if projects.is_empty() {
        eprintln!(
            "Aucun projet trouvé sous les workspaces de {}.\n\
             Vérifiez les chemins `root` et leur contenu.",
            cfg_path.display()
        );
        std::process::exit(1);
    }

    let mut app = App::new(projects, query);
    let outcome = run_tui(&mut app)?;

    // --- Sortie machine sur stdout ---
    match outcome {
        Some((Outcome::Cd, path)) => println!("CD\t{}", path.display()),
        Some((Outcome::Resume, path)) => println!("RESUME\t{}", path.display()),
        None => std::process::exit(1),
    }
    Ok(())
}

/// Lance le TUI sur `/dev/tty` (stdout reste réservé au résultat).
fn run_tui(app: &mut App) -> Result<Option<(Outcome, std::path::PathBuf)>> {
    let tty = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .context("ouverture de /dev/tty")?;
    let mut writer = tty.try_clone().context("clone /dev/tty")?;

    enable_raw_mode()?;
    execute!(writer, EnterAlternateScreen)?;

    // Restaure le terminal même en cas de panique.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stderr(), LeaveAlternateScreen);
        original_hook(info);
    }));

    let backend = CrosstermBackend::new(writer);
    let mut terminal = Terminal::new(backend)?;

    let loop_result = event_loop(&mut terminal, app);

    // Restauration propre.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    let _ = std::panic::take_hook();

    loop_result?;
    Ok(app.outcome.clone())
}

fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    while !app.quit {
        terminal.draw(|f| ui::render(f, app))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.on_key(key);
            }
        }
    }
    Ok(())
}
