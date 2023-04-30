use std::{io::{self, Stdout}, time::{Duration, Instant}};
use tui::{backend::CrosstermBackend, Terminal, layout::{Layout, Constraint, Rect}, widgets::{Borders, Block, Gauge, ListItem, List, ListState}, style::{Style, Color, Modifier}, text::{Spans, Span}};
use crate::sound::{Player};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

pub struct App {
    player: Player
}

impl App {
    pub fn new(player: Player) -> Self {
        let app = Self {
            player: player
        };
        return app;
    }

    pub fn run(&mut self, tick_rate: Duration) {
        // setup
        enable_raw_mode().unwrap();
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        // run
        let _result = self.mainloop(tick_rate, &mut terminal);

        // disable
        disable_raw_mode().unwrap();
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).unwrap();
        terminal.show_cursor().unwrap();
    }

    fn mainloop (&mut self, tick_rate: Duration, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        let mut last_tick = Instant::now();
        let mut should_quit = false;
        let mut was_down = false;
        loop {
            terminal.draw(|f: &mut tui::Frame<CrosstermBackend<Stdout>>| self.draw(f))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if !was_down {
                        was_down = true;
                        match key.code {
                            KeyCode::Char(c) => {
                                if c == ' ' {
                                    self.player.pause();
                                } else if c == 'q' {
                                    should_quit = true;
                                }
                            },
                            KeyCode::Right => {
                                self.player.skip_song();
                            },
                            _ => {}
                        }
                    }
                } else {
                    was_down = false;
                }
            } else {
                was_down = false;
            }
            if last_tick.elapsed() >= tick_rate {
                self.tick();
                last_tick = Instant::now();
            }
            if should_quit {
                return Ok(());
            }
        }
    }

    fn tick(&mut self) {
        self.player.update();
    }

    fn draw(&self, frame: &mut tui::Frame<CrosstermBackend<Stdout>>) {
        let chunks = Layout::default()
            .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
            .split(frame.size());
        self.draw_song_list(frame, chunks[0]);
        self.draw_progress_bar(frame, chunks[1]);
    }

    fn draw_progress_bar(&self, frame: &mut tui::Frame<CrosstermBackend<Stdout>>, area: Rect) {

        let progress = self.player.get_song_progress().min(1.0);
        let label = format!("{}: {:.2}%", self.player.state.song.name, progress * 100.0);
        let bar = Gauge::default()
            .block(Block::default().title("Song").borders(Borders::ALL))
            .gauge_style(
                Style::default()
                    .fg(Color::LightBlue)
                    .bg(Color::Black)
                    .add_modifier(Modifier::ITALIC | Modifier::BOLD),
            )
            .label(label)
            .ratio(progress)
            .use_unicode(true);
        frame.render_widget(bar, area);
    }

    fn draw_song_list(&self, frame: &mut tui::Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let tasks: Vec<ListItem> = self.player.song_selection
            .iter()
            .map(|i| ListItem::new(vec![Spans::from(Span::raw(&i.name))]))
            .collect();
        let tasks = List::new(tasks)
            .block(Block::default().borders(Borders::ALL).title("List"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))
            .highlight_symbol("> ");
        let mut state = ListState::default();
        state.select(Some(self.player.state.index));
        frame.render_stateful_widget(tasks, area, &mut state);
    }
}