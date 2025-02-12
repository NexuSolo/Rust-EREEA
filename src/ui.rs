use crate::generation::TypeCase;
use crate::robot::Robot;
use log::{debug, info, trace, warn};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use std::io;
use std::sync::{Arc, Mutex};

pub fn run_ui(
    carte: &Vec<Vec<TypeCase>>,
    ressources: &str,
    robots: &Arc<Mutex<Vec<Box<dyn Robot + Send>>>>,
) -> Result<(), io::Error> {
    info!("[UI] Démarrage de l'interface utilisateur");

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let size = f.area();
        let reduced_height = size.height.saturating_sub(5);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(reduced_height)].as_ref())
            .split(size);

        let ressources_paragraph = Paragraph::new(ressources)
            .block(Block::default().borders(Borders::ALL).title("Ressources"))
            .style(
                Style::default()
                    .fg(Color::Rgb(208, 191, 154))
                    .bg(Color::Rgb(27, 27, 34)),
            );

        f.render_widget(ressources_paragraph, chunks[0]);

        // Créer une copie de la carte pour l'affichage
        let mut carte_affichage = carte.clone();

        // Mettre à jour la carte avec les positions des robots
        if let Ok(robots_guard) = robots.lock() {
            for robot in robots_guard.iter() {
                let x = robot.get_position_x();
                let y = robot.get_position_y();
                if y < carte_affichage.len() && x < carte_affichage[0].len() {
                    carte_affichage[y][x] = robot.get_type();
                }
            }
        }

        let mut map_string = String::new();
        for ligne in carte_affichage.iter() {
            for case in ligne {
                let symbol = match case {
                    TypeCase::Vide => "  ",
                    TypeCase::Mur => "🪨",
                    TypeCase::Energie => "⚡",
                    TypeCase::Minerais => "💎",
                    TypeCase::Science => "S ",
                    TypeCase::Explorateur => "🛸",
                    TypeCase::Collecteur => "🤖",
                    TypeCase::Base => "🏠",
                    TypeCase::Inconnu => "▒▒",
                };
                map_string.push_str(symbol);
            }
            map_string.push('\n');
        }

        let map_paragraph = Paragraph::new(map_string)
            .block(Block::default().borders(Borders::ALL).title("Carte"))
            .style(
                Style::default()
                    .fg(Color::Rgb(208, 191, 154))
                    .bg(Color::Rgb(27, 27, 34)),
            );

        f.render_widget(map_paragraph, chunks[1]);
        info!("[UI] Interface utilisateur mise à jour");
    })?;

    Ok(())
}
