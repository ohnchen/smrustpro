use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode, MouseButton,
        MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rusqlite::Connection;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

mod defaults;
mod world;

use world::World;

enum Event<Key, Pos> {
    KeyInput(Key),
    LeftClick(Pos),
    Tick,
}

// Keeps track of what the user wants to do
#[derive(PartialEq)]
pub enum Mode {
    Insert,
    Play,
    Load,
    Save,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(500);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                match event::read().expect("can read events") {
                    CEvent::Key(key) => tx.send(Event::KeyInput(key)).expect("can send keyevents"),
                    CEvent::Mouse(MouseEvent {
                        kind, column, row, ..
                    }) => match kind {
                        MouseEventKind::Down(MouseButton::Left) => tx
                            .send(Event::LeftClick((row, column)))
                            .expect("can send mouseevents"),
                        _ => {}
                    },
                    _ => {}
                };
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let conn = Connection::open("templates.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS templates (
           id integer primary key,
           name text not null,
           width integer not null,
           height integer not null,
           alive text not null
        )",
        [],
    )?;

    let mut stdout = io::stdout();

    execute!(stdout, EnableMouseCapture)?;

    let mut input: Input = "".into();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut should_play = false;
    let mut mode = Mode::Insert;

    let mut size = terminal.size()?;
    let mut world = World::default().width(size.width).height(size.height);
    let mut loaded_list: Vec<ListItem> = vec![];

    loop {
        if should_play == true {
            world.next_day();
        }

        terminal.draw(|f| {
            size = f.size();

            if world.alive.is_empty() && mode != Mode::Load {
                mode = Mode::Insert;
                should_play = false;
            }

            match mode {
                Mode::Load => {
                    let load_list = List::new(loaded_list.clone()).block(
                        Block::default()
                            .title("Load Templates")
                            .borders(Borders::ALL),
                    );
                    f.render_widget(load_list, size);
                }
                Mode::Insert | Mode::Play => {
                    let world_grided = world.get_grid(&mode, size.height - 2, size.width - 2);
                    let world_block = Paragraph::new(world_grided)
                        .block(
                            Block::default()
                                .title({
                                    match mode {
                                        Mode::Insert => "Editor - Game of Life",
                                        Mode::Play => "Conways - Game of Life",
                                        _ => "",
                                    }
                                })
                                .borders(Borders::ALL),
                        )
                        .wrap(Wrap { trim: true });

                    f.render_widget(world_block, size);
                }
                Mode::Save => {
                    let input_block =
                        Paragraph::new(input.value()).block(Block::default().borders(Borders::ALL));
                    f.render_widget(input_block, size);
                }
            }
        })?;

        match rx.recv()? {
            Event::KeyInput(event) => match mode {
                Mode::Save => match event.code {
                    KeyCode::Enter => {
                        world.save_current_state(&conn, input.value().to_owned())?;
                        input.reset();
                        mode = Mode::Insert;
                    },
                    _ => {
                        input.handle_event(&CEvent::Key(event));
                    }
                }
                _ => match event.code {
                    KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        terminal.show_cursor()?;
                        terminal.clear()?;
                        break;
                    }
                    KeyCode::Char(' ') => {
                        should_play = !should_play;
                    }
                    KeyCode::Char('i') => {
                        should_play = false;
                        mode = Mode::Insert;
                    }
                    KeyCode::Char('s') => mode = Mode::Save,
                    KeyCode::Char('l') => {
                        should_play = false;
                        mode = Mode::Load;
                        loaded_list = world
                            .load_alive(&conn)
                            .unwrap()
                            .iter()
                            .map(|i| ListItem::new(i.to_string()))
                            .collect::<Vec<ListItem>>();
                    }
                    KeyCode::Enter => {
                        should_play = true;
                        mode = Mode::Play;
                    }
                    KeyCode::Delete => {
                        world.alive.clear();
                    }
                    KeyCode::Char('1') => {
                        should_play = true;
                        mode = Mode::Play;
                        world.alive = World::pulsar().alive;
                    }
                    _ => {}
                },
            },
            Event::LeftClick(pos) => {
                if pos.0 as i32 - 1 < 0
                    || pos.0 as i32 - size.bottom() as i32 > 0
                    || pos.1 as i32 + 1 < 0
                    || pos.1 as i32 - size.right() as i32 > 0
                {
                    continue;
                }
                let position = (pos.0 - 1, pos.1 - 1);
                if !world
                    .alive
                    .iter()
                    .any(|x| x.0 == position.0 && x.1 == position.1)
                {
                    world.alive.push(position);
                } else {
                    world
                        .alive
                        .retain(|x| x.0 != position.0 || x.1 != position.1);
                }
            }
            Event::Tick => {}
        }
    }

    execute!(terminal.backend_mut(), DisableMouseCapture)?;
    terminal.show_cursor()?;
    terminal.clear()?;
    Ok(())
}
