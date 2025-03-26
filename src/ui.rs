use crate::generation::TypeCase;
use crate::robot::Robot;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use std::io;
use std::sync::{Arc, Mutex};

pub fn run_ui(
    map: &Arc<Mutex<Vec<Vec<TypeCase>>>>,
    resources: &str,
    robots: &Arc<Mutex<Vec<Box<dyn Robot + Send>>>>,
) -> Result<(), io::Error> {
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

        let resources_paragraph = Paragraph::new(resources)
            .block(Block::default().borders(Borders::ALL).title("Resources"))
            .style(
                Style::default()
                    .fg(Color::Rgb(208, 191, 154))
                    .bg(Color::Rgb(27, 27, 34)),
            );

        f.render_widget(resources_paragraph, chunks[0]);

        // Create a copy of the map for display
        let map_guard = map.lock().unwrap();
        let mut displayed_map = (*map_guard).clone();
        drop(map_guard);

        // Update the map with the robots' positions
        if let Ok(robots_guard) = robots.lock() {
            for robot in robots_guard.iter() {
                let x = robot.get_position_x();
                let y = robot.get_position_y();
                if y < displayed_map.len() && x < displayed_map[0].len() {
                    if robot.get_type() == TypeCase::Collector {
                        if let Ok(map_guard) = map.lock() {
                            if map_guard[y][x] != TypeCase::Base {
                                displayed_map[y][x] = robot.get_type();
                            }
                        }
                    } else {
                        displayed_map[y][x] = robot.get_type();
                    }
                }
            }
        }

        let mut map_string = String::new();
        for row in displayed_map.iter() {
            for case in row {
                let symbol = match case {
                    TypeCase::Void => "  ",
                    TypeCase::Wall => "🪨",
                    TypeCase::Energy => "⚡",
                    TypeCase::Ore => "💎",
                    TypeCase::Science => "🧪",
                    TypeCase::Base => "🏠",
                    TypeCase::Explorer => "🛸",
                    TypeCase::Collector => "🤖",
                    TypeCase::Unknown => "▒▒",
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
    })?;

    Ok(())
}
