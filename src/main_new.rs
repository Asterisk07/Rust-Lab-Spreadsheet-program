use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{PrintStyledContent, Stylize},
    terminal,
};
use std::{
    collections::HashMap,
    fs,
    io::{stdout, Write},
    time::Duration,
};

// Formatting options struct
#[derive(Clone, Default)]
struct FormattingOptions {
    bold: bool,
    italic: bool,
    underline: bool,
    color: Option<String>, // "red", "green", "blue"
}

fn main() {
    let filename = "sample.txt";
    let content = fs::read_to_string(filename).expect("Failed to read file");
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    let mut stdout = stdout();
    terminal::enable_raw_mode().unwrap();

    let mut cursor_x = 0;
    let mut cursor_y = 0;
    let mut formatting: HashMap<(usize, usize), FormattingOptions> = HashMap::new(); // 🔹 Track formatting per character
    let mut command_buffer = String::new();
    let mut help_mode = false; // 🔹 Tracks if help menu is open

    redraw_screen(&mut stdout, &lines, cursor_x, cursor_y, &formatting);

    loop {
        if let Ok(true) = event::poll(Duration::from_millis(500)) {
            if let Ok(event::Event::Key(KeyEvent {
                code, modifiers, ..
            })) = event::read()
            {
                // 🔹 If Help Mode is Open, Only Allow Esc to Close
                if help_mode {
                    if code == KeyCode::Esc {
                        help_mode = false;
                        redraw_screen(&mut stdout, &lines, cursor_x, cursor_y, &formatting);
                    }
                    continue;
                }

                match code {
                    // 🔹 Move Cursor
                    KeyCode::Char('h') if cursor_x > 0 => cursor_x -= 1,
                    KeyCode::Left if cursor_x > 0 => cursor_x -= 1,
                    KeyCode::Char('l') if cursor_x < lines[cursor_y].len() => cursor_x += 1,
                    KeyCode::Right if cursor_x < lines[cursor_y].len() => cursor_x += 1,
                    KeyCode::Char('k') if cursor_y > 0 => cursor_y -= 1,
                    KeyCode::Up if cursor_y > 0 => cursor_y -= 1,
                    KeyCode::Char('j') if cursor_y < lines.len() - 1 => cursor_y += 1,
                    KeyCode::Down if cursor_y < lines.len() - 1 => cursor_y += 1,

                    // 🔹 Delete Character (`x`)
                    KeyCode::Char('x') if cursor_x < lines[cursor_y].len() => {
                        lines[cursor_y].remove(cursor_x);
                        formatting.remove(&(cursor_y, cursor_x)); // Remove formatting too
                    }

                    // 🔹 Command Mode (Start Typing `:`)
                    KeyCode::Char(':') => {
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

                        process_command(&command_buffer, cursor_x, cursor_y, &mut formatting);
                    }

                    // 🔹 Quit (`q`)
                    KeyCode::Char('q') if modifiers == KeyModifiers::NONE => break,

                    _ => {}
                }

                // Redraw after any change
                redraw_screen(&mut stdout, &lines, cursor_x, cursor_y, &formatting);
            }
        }
    }

    terminal::disable_raw_mode().unwrap();
}

// 🔹 Function to process formatting commands
fn process_command(
    command: &str,
    cursor_x: usize,
    cursor_y: usize,
    formatting: &mut HashMap<(usize, usize), FormattingOptions>,
) {
    if command == "b" {
        let entry = formatting.entry((cursor_y, cursor_x)).or_default();
        entry.bold = !entry.bold;
    } else if command == "i" {
        let entry = formatting.entry((cursor_y, cursor_x)).or_default();
        entry.italic = !entry.italic;
    } else if command == "u" {
        let entry = formatting.entry((cursor_y, cursor_x)).or_default();
        entry.underline = !entry.underline;
    } else if command.starts_with("color ") {
        let color = command[6..].trim();
        let entry = formatting.entry((cursor_y, cursor_x)).or_default();
        if ["red", "green", "blue"].contains(&color) {
            entry.color = Some(color.to_string());
        }
    } else if command == "reset" {
        formatting.remove(&(cursor_y, cursor_x));
    }
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
        "h, l, j, k  → Move cursor",
        "x            → Delete character",
        ":b           → Bold current character",
        ":i           → Italicize current character",
        ":u           → Underline current character",
        ":color red   → Change text color to red",
        ":color green → Change text color to green",
        ":color blue  → Change text color to blue",
        ":reset       → Remove formatting",
        "q            → Quit",
        "────────────────────────",
        "Press ESC to return to the spreadsheet.",
    ];

    for (i, line) in help_text.iter().enumerate() {
        execute!(stdout, cursor::MoveTo(0, i as u16)).unwrap(); // Move cursor to left margin before printing
        println!("{}", line);
    }

    stdout.flush().unwrap();
}

// // 🔹 Help Menu Display Function
// fn draw_help_menu(stdout: &mut std::io::Stdout) {
//     execute!(
//         stdout,
//         terminal::Clear(terminal::ClearType::All),
//         cursor::MoveTo(0, 0)
//     )
//     .unwrap();
//     println!("📖 Spreadsheet Help Menu");
//     println!("────────────────────────");
//     println!("h, l, j, k  → Move cursor");
//     println!("x            → Delete character");
//     println!(":b           → Bold current character");
//     println!(":i           → Italicize current character");
//     println!(":u           → Underline current character");
//     println!(":color red   → Change text color to red");
//     println!(":color green → Change text color to green");
//     println!(":color blue  → Change text color to blue");
//     println!(":reset       → Remove formatting");
//     println!("q            → Quit");
//     println!("────────────────────────");
//     println!("Press ESC to return to the spreadsheet.");

//     stdout.flush().unwrap();
// }

// 🔹 Modify `redraw_screen` to apply formatting
fn redraw_screen(
    stdout: &mut std::io::Stdout,
    lines: &[String],
    cursor_x: usize,
    cursor_y: usize,
    formatting: &HashMap<(usize, usize), FormattingOptions>,
) {
    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();

    for (y, line) in lines.iter().enumerate() {
        execute!(stdout, cursor::MoveTo(0, y as u16)).unwrap();

        for (x, ch) in line.chars().enumerate() {
            let format = formatting.get(&(y, x)).cloned().unwrap_or_default();
            use crossterm::style::{Color, PrintStyledContent, Stylize};

            // Get the character as a `StyledContent`
            let mut styled_content = ch.stylize(); // Convert character to StyledContent

            if format.bold {
                styled_content = styled_content.bold();
            }
            if format.italic {
                styled_content = styled_content.italic();
            }
            if format.underline {
                styled_content = styled_content.underlined();
            }
            if let Some(color) = &format.color {
                styled_content = match color.as_str() {
                    "red" => styled_content.with(Color::Red),
                    "green" => styled_content.with(Color::Green),
                    "blue" => styled_content.with(Color::Blue),
                    _ => styled_content,
                };
            }

            // 🔹 Ensure cursor visibility
            if x == cursor_x && y == cursor_y {
                let cursor_char = format!("[{}]", ch); // Cursor with brackets
                execute!(stdout, PrintStyledContent(cursor_char.stylize().bold())).unwrap();
            // Apply bold to cursor
            } else {
                execute!(stdout, PrintStyledContent(styled_content)).unwrap(); // Correctly print formatted text
            }
        }
        println!();
    }

    stdout.flush().unwrap();
}
