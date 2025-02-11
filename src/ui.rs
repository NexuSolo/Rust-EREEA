use crate::TypeCase;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use std::io;

pub fn run_ui(carte: Vec<Vec<TypeCase>>, ressources: &str) -> Result<(), io::Error> {
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
            ); // Modifier le background et le foreground en même temps

        f.render_widget(ressources_paragraph, chunks[0]);

        let mut map_string = String::new();
        for row in &carte {
            for case in row {
                let symbol = match case {
                    TypeCase::Vide => ' ',
                    TypeCase::Base => '🔵',
                    TypeCase::Mur => '🪨',
                    TypeCase::Energie => '⚡',
                    TypeCase::Minerais => '💎',
                    TypeCase::Science => 'S',
                    TypeCase::Explorateur => '🛸',
                    TypeCase::Collecteur => '🤖',
                };
                map_string.push(symbol);
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
    })?;

    Ok(())
}
