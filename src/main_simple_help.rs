use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent, KeyModifiers},
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
    let mut clipboard: Option<String> = None;
    let mut command_buffer = String::new();
    let mut selected_text: Option<String> = None; // 🔹 Stores the search phrase
    let mut help_mode = false; // 🔹 Tracks if help menu is open

    redraw_screen(
        &mut stdout,
        &lines,
        cursor_x,
        cursor_y,
        insert_mode,
        selected_text.as_deref(),
    );

    loop {
        if let Ok(true) = event::poll(Duration::from_millis(500)) {
            if let Ok(event::Event::Key(KeyEvent {
                code, modifiers, ..
            })) = event::read()
            {
                if help_mode {
                    if code == KeyCode::Esc {
                        help_mode = false;
                        redraw_screen(
                            &mut stdout,
                            &lines,
                            cursor_x,
                            cursor_y,
                            insert_mode,
                            selected_text.as_deref(),
                        );
                    }
                    continue;
                }
                match code {
                    // 🔹 Insert Mode Toggle
                    KeyCode::Char('i') if !insert_mode => insert_mode = true,
                    KeyCode::Esc if insert_mode => insert_mode = false,

                    // 🔹 Move Cursor
                    KeyCode::Char('h') if !insert_mode && cursor_x > 0 => cursor_x -= 1,
                    KeyCode::Left if cursor_x > 0 => cursor_x -= 1,
                    KeyCode::Char('l') if !insert_mode && cursor_x < lines[cursor_y].len() => {
                        cursor_x += 1
                    }
                    KeyCode::Right if cursor_x < lines[cursor_y].len() => cursor_x += 1,
                    KeyCode::Char('k') if !insert_mode && cursor_y > 0 => cursor_y -= 1,
                    KeyCode::Up if cursor_y > 0 => cursor_y -= 1,
                    KeyCode::Char('j') if !insert_mode && cursor_y < lines.len() - 1 => {
                        cursor_y += 1
                    }
                    KeyCode::Down if cursor_y < lines.len() - 1 => cursor_y += 1,

                    // 🔹 Delete Character (`x`)
                    KeyCode::Char('x') if !insert_mode && cursor_x < lines[cursor_y].len() => {
                        lines[cursor_y].remove(cursor_x);
                    }

                    // 🔹 Backspace (In Insert Mode)
                    KeyCode::Backspace if insert_mode && cursor_x > 0 => {
                        lines[cursor_y].remove(cursor_x - 1);
                        cursor_x -= 1;
                    }

                    // 🔹 Insert Mode Typing
                    KeyCode::Char(c) if insert_mode => {
                        lines[cursor_y].insert(cursor_x, c);
                        cursor_x += 1;
                    }

                    // 🔹 Copy Line (`yy`)
                    KeyCode::Char('y') if !insert_mode => {
                        clipboard = Some(lines[cursor_y].clone());
                    }

                    // 🔹 Paste (`p`)
                    KeyCode::Char('p') if !insert_mode && clipboard.is_some() => {
                        lines.insert(cursor_y + 1, clipboard.clone().unwrap());
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

                        // 🔹 Check for `:select <phrase>`
                        if command_buffer.starts_with("select ") {
                            let phrase = command_buffer[7..].to_string();
                            selected_text = Some(phrase);
                        }

                        // 🔹 Open Help Menu if `:h` is entered
                        if command_buffer.trim() == "h" {
                            help_mode = true;
                            draw_help_menu(&mut stdout);
                            continue;
                        }
                    }

                    // 🔹 Quit (`q`)
                    KeyCode::Char('q') if modifiers == KeyModifiers::NONE => break,

                    _ => {}
                }

                // Redraw after any change
                redraw_screen(
                    &mut stdout,
                    &lines,
                    cursor_x,
                    cursor_y,
                    insert_mode,
                    selected_text.as_deref(),
                );
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
        "  h, ←        → Move left",
        "  l, →        → Move right",
        "  k, ↑        → Move up",
        "  j, ↓        → Move down",
        "",
        "EDITING:",
        "  i           → Enter insert mode",
        "  ESC         → Exit insert mode",
        "  x           → Delete character",
        "  yy          → Copy current line",
        "  p           → Paste copied line",
        "",
        "COMMANDS:",
        "  :select <phrase> → Highlight phrase",
        "  :h              → Open help menu",
        "  q               → Quit",
        "",
        "FORMAT COMMANDS:",
        "  :b           → Bold current character",
        "  :i           → Italicize current character",
        "  :u           → Underline current character",
        "  :color red   → Change text color to red",
        "  :color green → Change text color to green",
        "  :color blue  → Change text color to blue",
        "  :reset       → Remove formatting",
        "",
        "────────────────────────",
        "Press ESC to return to the spreadsheet.",
    ];

    for (i, line) in help_text.iter().enumerate() {
        execute!(stdout, cursor::MoveTo(0, i as u16)).unwrap(); // Move cursor to left margin before printing
        println!("{}", line);
    }

    stdout.flush().unwrap();
}

fn redraw_screen(
    stdout: &mut std::io::Stdout,
    lines: &[String],
    cursor_x: usize,
    cursor_y: usize,
    insert_mode: bool,
    selected_text: Option<&str>,
) {
    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();

    for (y, line) in lines.iter().enumerate() {
        execute!(stdout, cursor::MoveTo(0, y as u16)).unwrap();
        let mut i = 0;

        while i < line.len() {
            let mut matched = false;

            // 🔹 Check if cursor is at this position
            let is_cursor = (i == cursor_x) && (y == cursor_y);

            if let Some(search) = selected_text {
                if line[i..].starts_with(search) {
                    if is_cursor {
                        // 🔹 Cursor inside highlighted text
                        execute!(stdout, PrintStyledContent(format!("[{}]", search).red()))
                            .unwrap();
                    } else {
                        execute!(stdout, PrintStyledContent(search.red())).unwrap();
                    }
                    i += search.len();
                    matched = true;
                }
            }

            if !matched {
                if is_cursor {
                    // 🔹 Regular cursor display (without interfering with highlight)
                    print!("[{}]", &line[i..i + 1]);
                } else {
                    print!("{}", &line[i..i + 1]);
                }
                i += 1;
            }
        }

        // 🔹 Show cursor at the end of line if needed
        if cursor_x == line.len() && y == cursor_y {
            print!("[_]");
        }

        println!();
    }

    // 🔹 Show mode indicator
    execute!(stdout, cursor::MoveTo(0, lines.len() as u16 + 1)).unwrap();
    if insert_mode {
        print!("-- INSERT --");
    } else {
        print!("NORMAL MODE");
    }

    stdout.flush().unwrap();
}

// 🔹 Modified `redraw_screen` to highlight `selected_text`
// fn redraw_screen(
//     stdout: &mut std::io::Stdout,
//     lines: &[String],
//     cursor_x: usize,
//     cursor_y: usize,
//     insert_mode: bool,
//     selected_text: Option<&str>,
// ) {
//     execute!(
//         stdout,
//         terminal::Clear(terminal::ClearType::All),
//         cursor::MoveTo(0, 0)
//     )
//     .unwrap();

//     for (y, line) in lines.iter().enumerate() {
//         execute!(stdout, cursor::MoveTo(0, y as u16)).unwrap();
//         let mut i = 0;

//         while i < line.len() {
//             let mut matched = false;

//             if let Some(search) = selected_text {
//                 if line[i..].starts_with(search) {
//                     execute!(stdout, PrintStyledContent(search.red())).unwrap(); // 🔴 Highlight in red
//                     i += search.len();
//                     matched = true;
//                 }
//             }

//             if !matched {
//                 print!("{}", &line[i..i + 1]);
//                 i += 1;
//             }
//         }

//         if cursor_x == line.len() && y == cursor_y {
//             print!("[_]"); // Show cursor at end of line
//         }

//         println!();
//     }

//     // 🔹 Show mode indicator
//     execute!(stdout, cursor::MoveTo(0, lines.len() as u16 + 1)).unwrap();
//     if insert_mode {
//         print!("-- INSERT --");
//     } else {
//         print!("NORMAL MODE");
//     }

//     stdout.flush().unwrap();
// }
