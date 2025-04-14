use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent},
    execute,
    style::{PrintStyledContent, Stylize},
    terminal,
};
use std::{
    fs,
    io::{Write, stdout},
    time::Duration,
};

fn main() {
    let filename = "sample.txt";
    let content = fs::read_to_string(filename).expect("Failed to read file");
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    let mut stdout = stdout();
    terminal::enable_raw_mode().unwrap();

    let mut cursor_x = 0;
    let mut cursor_y = 0;
    let mut insert_mode = false;
    let mut help_mode = false; // Help menu toggle
    let mut command_buffer = String::new();

    redraw_screen(&mut stdout, &lines, cursor_x, cursor_y, insert_mode);

    loop {
        if let Ok(true) = event::poll(Duration::from_millis(500)) {
            if let Ok(event::Event::Key(KeyEvent { code, .. })) = event::read() {
                if help_mode {
                    if code == KeyCode::Esc {
                        help_mode = false;
                        redraw_screen(&mut stdout, &lines, cursor_x, cursor_y, insert_mode);
                    }
                    continue;
                }

                match code {
                    // 🔹 Insert Mode Toggle
                    KeyCode::Char('i') if !insert_mode => insert_mode = true,
                    KeyCode::Esc if insert_mode => insert_mode = false,

                    // 🔹 Move Cursor
                    KeyCode::Left if cursor_x > 0 => cursor_x -= 1,
                    KeyCode::Right if cursor_x < lines[cursor_y].len() => cursor_x += 1,
                    KeyCode::Up if cursor_y > 0 => cursor_y -= 1,
                    KeyCode::Down if cursor_y < lines.len() - 1 => cursor_y += 1,

                    // 🔹 Insert Mode Typing
                    KeyCode::Char(c) if insert_mode => {
                        lines[cursor_y].insert(cursor_x, c);
                        cursor_x += 1;
                    }

                    // 🔹 Backspace (In Insert Mode)
                    KeyCode::Backspace if insert_mode && cursor_x > 0 => {
                        lines[cursor_y].remove(cursor_x - 1);
                        cursor_x -= 1;
                    }

                    // 🔹 Command Mode (Start Typing `:`)
                    KeyCode::Char(':') if !insert_mode => {
                        command_buffer.clear();
                        print!(":");
                        stdout.flush().unwrap();

                        while let Ok(event::Event::Key(KeyEvent { code, .. })) = event::read() {
                            match code {
                                KeyCode::Enter => break,
                                KeyCode::Backspace => {
                                    command_buffer.pop();
                                }
                                KeyCode::Char(c) => command_buffer.push(c),
                                _ => {}
                            }
                        }

                        // 🔹 Open Help Menu if `:h` is entered
                        if command_buffer.trim() == "h" {
                            help_mode = true;
                            draw_help_menu(&mut stdout);
                            continue;
                        }

                        // 🔹 Print chart if `:p` is entered
                        if command_buffer.trim() == "p" {
                            print_chart(&lines);
                            continue;
                        }
                    }

                    // 🔹 Quit (`q`)
                    KeyCode::Char('q') => break,

                    _ => {}
                }

                // Redraw after any change
                redraw_screen(&mut stdout, &lines, cursor_x, cursor_y, insert_mode);
            }
        }
    }

    terminal::disable_raw_mode().unwrap();
}

// 🔹 Help Menu Display Function
fn draw_help_menu(stdout: &mut std::io::Stdout) {
    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();

    let help_text = [
        "📖 Spreadsheet Help Menu",
        "────────────────────────",
        "MOVEMENT:",
        "  ←, →, ↑, ↓   → Move cursor",
        "",
        "EDITING:",
        "  i           → Enter insert mode",
        "  ESC         → Exit insert mode",
        "",
        "COMMANDS:",
        "  :p          → Print chart",
        "  :h          → Open help menu",
        "  q           → Quit",
        "",
        "────────────────────────",
        "Press ESC to return to the spreadsheet.",
    ];

    for (i, line) in help_text.iter().enumerate() {
        execute!(stdout, cursor::MoveTo(0, i as u16)).unwrap();
        println!("{}", line);
    }

    stdout.flush().unwrap();
}

// 🔹 Print chart based on line lengths
fn print_chart(lines: &[String]) {
    println!("\n📊 Chart of Line Lengths:");
    println!("────────────────────────");
    for (i, line) in lines.iter().enumerate() {
        let length = line.len();
        println!("Line {}: {}", i + 1, "*".repeat(length));
    }
    println!("\n");
}

// 🔹 Redraw Screen Function
fn redraw_screen(
    stdout: &mut std::io::Stdout,
    lines: &[String],
    cursor_x: usize,
    cursor_y: usize,
    insert_mode: bool,
) {
    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();

    for (y, line) in lines.iter().enumerate() {
        execute!(stdout, cursor::MoveTo(0, y as u16)).unwrap();
        print!("{}", line);

        // Show cursor
        if y == cursor_y {
            execute!(stdout, cursor::MoveTo(cursor_x as u16, y as u16)).unwrap();
        }
    }

    // Mode indicator
    execute!(stdout, cursor::MoveTo(0, lines.len() as u16 + 1)).unwrap();
    if insert_mode {
        print!("-- INSERT --");
    } else {
        print!("NORMAL MODE");
    }

    stdout.flush().unwrap();
}
