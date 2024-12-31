use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use std::io::{self, stdout};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    let top_perc = 80;
    let bot_perc = 20;
    // List available ports
    let ports = serialport::available_ports().expect("No ports found!");
    if ports.is_empty() {
        eprintln!("No available serial ports.");
        return Ok(());
    }

    // Connect to the first available port
    let port_name = &ports[0].port_name;
    println!("Connecting to {}...", port_name);

    let baud_rate = 115200;
    let timeout = Duration::from_millis(1000);
    let mut port = serialport::new(port_name, baud_rate)
        .timeout(timeout)
        .open()
        .expect("Failed to open port");

    println!("Connected to {} at {} baud.", port_name, baud_rate);

    // Channel for sending data from the serial port to the UI
    let (tx, rx) = mpsc::channel();

    // Spawn a thread to read from the serial port
    thread::spawn(move || {
        let mut buffer: [u8; 1024] = [0; 1024];
        let mut partial_line = String::new();
        loop {
            match port.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        let data = String::from_utf8_lossy(&buffer[..bytes_read]);
                        for chunk in data.split_inclusive(['\r', '\n'].as_ref()) {
                            // Check if the chunk ends with \n (either alone or with \r before it)
                            if chunk.ends_with("\n") {
                                // If it ends with both \r\n, trim \r before sending
                                partial_line.push_str(chunk.trim_end_matches('\r'));

                                // Send the complete line through the channel
                                if tx.send(partial_line.clone()).is_err() {
                                    break;
                                }
                                partial_line.clear();
                            } else {
                                // Otherwise, accumulate the chunk
                                partial_line.push_str(chunk);
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                    // Ignore timeout errors
                }
                Err(_) => {
                    break;
                }
            }
        }
    });

    // Initialize the terminal UI
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut received_data: Vec<String> = Vec::new();
    let mut error_warn_data: Vec<(String, Color)> = Vec::new(); // Store both message and color
    let mut scroll_offset = 0;
    let mut error_warn_scroll_offset = 0; // Add a scroll offset for errors and warnings
    let mut is_scrolled = false; // Track if user manually scrolled
    let mut is_error_warn_scrolled = false; // Track if user manually scrolled the error/warn section

    loop {
        // Handle UI events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => {
                        if scroll_offset < received_data.len().saturating_sub(1) {
                            scroll_offset += 1;
                            is_scrolled = true;
                        }
                    }
                    KeyCode::Up => {
                        if scroll_offset > 0 {
                            scroll_offset -= 1;
                            is_scrolled = true;
                        }
                    }
                    KeyCode::Char('a') => {
                        // Reset to auto-scrolling
                        is_scrolled = false;
                    }
                    KeyCode::Char('w') => {
                        if error_warn_scroll_offset < error_warn_data.len().saturating_sub(1) {
                            error_warn_scroll_offset += 1;
                            is_error_warn_scrolled = true;
                        }
                    }
                    KeyCode::Char('s') => {
                        if error_warn_scroll_offset > 0 {
                            error_warn_scroll_offset -= 1;
                            is_error_warn_scrolled = true;
                        }
                    }
                    KeyCode::Char('d') => {
                        // Reset to auto-scrolling for error/warnings
                        is_error_warn_scrolled = false;
                    }
                    _ => {}
                }
            }
        }

        // Receive data from the serial port
        if let Ok(data) = rx.try_recv() {
            // Convert data to lowercase to perform case-insensitive comparison
            let data_lower = data.to_lowercase();

            // Check if the data contains any variation of "ERR", "ERROR", "WRN", or "WARN"
            if data_lower.contains("err") || data_lower.contains("error") {
                error_warn_data.push((data, Color::Red)); // Red color for errors
            } else if data_lower.contains("wrn") || data_lower.contains("warn") {
                error_warn_data.push((data, Color::Yellow)); // Yellow color for warnings
            } else {
                received_data.push(data);
            }
        }

        // Prevent buffers from growing indefinitely
        if received_data.len() > 1000 {
            received_data.drain(..received_data.len().saturating_sub(1000));
        }
        if error_warn_data.len() > 1000 {
            error_warn_data.drain(..error_warn_data.len().saturating_sub(1000));
        }

        // Auto-scroll to the latest entry if not manually scrolled for the main data
        if !is_scrolled {
            let serial_pane_height = terminal.size()?.height as usize * top_perc / 100; // Calculate 70% height
            scroll_offset = received_data.len().saturating_sub(serial_pane_height);
        }

        // Auto-scroll to the latest entry if not manually scrolled for the error/warning data
        if !is_error_warn_scrolled {
            let error_warn_pane_height = terminal.size()?.height as usize * bot_perc / 100; // Calculate 30% height
            error_warn_scroll_offset = error_warn_data.len().saturating_sub(error_warn_pane_height);
        }

        // Draw the UI
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(top_perc as u16), Constraint::Percentage(bot_perc as u16)].as_ref())
                .split(f.size());

            let text = Paragraph::new(
                received_data
                    .iter()
                    .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::Green))))
                    .collect::<Vec<Line>>(),
            )
            .block(
                Block::default()
                    .title("Serial Monitor")
                    .borders(Borders::ALL),
            )
            .scroll((scroll_offset as u16, 0));

            // Combine error and warning data in the same pane, coloring each appropriately
            let error_warn_text = Paragraph::new(
                error_warn_data
                    .iter()
                    .map(|(line, color)| {
                        Line::from(Span::styled(line, Style::default().fg(*color)))
                    })
                    .collect::<Vec<Line>>(),
            )
            .block(
                Block::default()
                    .title("Errors and Warnings")
                    .borders(Borders::ALL),
            )
            .scroll((error_warn_scroll_offset as u16, 0)); // Add scrolling for the error/warning pane

            f.render_widget(text, chunks[0]);
            f.render_widget(error_warn_text, chunks[1]);
        })?;
    }

    // Restore the terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}

