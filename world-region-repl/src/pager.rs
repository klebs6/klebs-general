// ---------------- [ File: src/pager.rs ]
crate::ix!();


use std::io::{stdout, Write};
use std::time::Duration;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};

/// A small RAII guard that ensures we disable raw mode and show the cursor on drop.
struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), Show);
    }
}

/// This pager allows vertical AND horizontal scrolling (left/right).
/// It uses normal screen (NOT alternate screen), so you can still
/// mouse‐select text in tmux. We enable raw mode for capturing arrow keys,
/// pageUp/pageDown, etc. We clear the screen on each redraw, so you may see
/// slight flicker if you scroll rapidly, but typically it’s fine.
pub fn scrolling_pager(lines: &[String]) {
    // Determine terminal size
    let (mut cols, mut rows) = match size() {
        Ok((c, r)) => (c as usize, r as usize),
        Err(_e) => (80, 24),
    };
    // We will reserve one line for a status message at the bottom
    let mut screen_height = rows.saturating_sub(1);

    // If content fits in one screen vertically, just print and return
    if lines.len() <= screen_height {
        for line in lines {
            println!("{}", line);
        }
        return;
    }

    // Try enabling raw mode
    if let Err(_e) = enable_raw_mode() {
        // fallback: just print
        for line in lines {
            println!("{}", line);
        }
        return;
    }

    let _guard = RawModeGuard; // Disables raw mode & shows cursor on drop

    let mut stdout_handle = stdout();
    let _ = execute!(stdout_handle, Hide); // hide cursor during interactive paging

    // Vertical offset (top line being displayed)
    let mut top = 0_usize;
    // Horizontal offset (leftmost column being displayed)
    let mut left_offset = 0_usize;

    // Figure out the maximum width among all lines so we know how far we can scroll horizontally
    let max_line_width = lines.iter().map(|l| l.len()).max().unwrap_or(0);

    loop {
        // Re-check terminal size each loop, in case the user resized
        if let Ok((c, r)) = size() {
            cols = c as usize;
            rows = r as usize;
            screen_height = rows.saturating_sub(1);
        }

        // Clear the entire screen, move cursor to (0,0)
        let _ = queue!(stdout_handle, MoveTo(0, 0), Clear(ClearType::All));

        // We'll display up to screen_height lines of text
        let end = top.saturating_add(screen_height).min(lines.len());

        // For horizontal scrolling, we only show from left_offset..left_offset+cols
        // We also clamp left_offset so it doesn't exceed the maximum needed offset:
        let max_horiz_offset = max_line_width.saturating_sub(cols);
        if left_offset > max_horiz_offset {
            left_offset = max_horiz_offset;
        }

        for (row, line) in lines[top..end].iter().enumerate() {
            let start = left_offset.min(line.len());
            let stop = (start + cols).min(line.len());
            // Substring the line according to current left_offset and terminal width
            let visible_slice = &line[start..stop];

            let _ = queue!(stdout_handle, MoveTo(0, row as u16));
            let _ = write!(stdout_handle, "{}", visible_slice);
        }

        // Show a small status line at the bottom
        let status_row = screen_height.min(rows.saturating_sub(1));
        let percent_scrolled = (end as f64 / lines.len() as f64) * 100.0;

        let horiz_info = if max_line_width > cols {
            format!(" Hscroll {}/{} ", left_offset, max_horiz_offset)
        } else {
            // No horizontal scrolling needed
            "".to_string()
        };

        let msg = format!(
            "[Up/Down/j/k, PageUp/PageDown, Left/Right/h/l, q to quit]  {}/{} lines ({:.0}%){}",
            end,
            lines.len(),
            percent_scrolled,
            horiz_info
        );

        let _ = queue!(
            stdout_handle,
            MoveTo(0, status_row as u16),
            Clear(ClearType::CurrentLine)
        );
        let _ = write!(stdout_handle, "{}", msg);

        let _ = stdout_handle.flush();

        // Wait for key
        match event::poll(Duration::from_millis(500)) {
            Ok(true) => {
                if let Ok(ev) = event::read() {
                    if let Event::Key(KeyEvent { code, .. }) = ev {
                        match code {
                            // Quit
                            KeyCode::Char('q') | KeyCode::Esc => break,

                            // Scroll up one line
                            KeyCode::Up | KeyCode::Char('k') => {
                                if top > 0 {
                                    top -= 1;
                                }
                            }
                            // Scroll down one line
                            KeyCode::Down | KeyCode::Char('j') => {
                                if top < lines.len().saturating_sub(1) {
                                    top += 1;
                                }
                            }
                            // Page Up
                            KeyCode::Char('f') | KeyCode::PageUp => {
                                top = top.saturating_sub(screen_height);
                            }
                            // Page Down
                            KeyCode::Char('d') | KeyCode::PageDown => {
                                top = (top + screen_height).min(lines.len().saturating_sub(1));
                            }
                            // Home
                            KeyCode::Home => {
                                top = 0;
                            }
                            // End
                            KeyCode::End => {
                                if lines.len() >= screen_height {
                                    top = lines.len().saturating_sub(screen_height);
                                } else {
                                    top = 0;
                                }
                            }

                            // Horizontal scroll left
                            KeyCode::Left | KeyCode::Char('h') => {
                                if left_offset > 0 {
                                    left_offset -= 1;
                                }
                            }
                            // Horizontal scroll right
                            KeyCode::Right | KeyCode::Char('l') => {
                                if left_offset < max_horiz_offset {
                                    left_offset += 1;
                                }
                            }

                            _ => {}
                        }
                    }
                }
            }
            _ => {
                // no event => loop again
            }
        }

        // If we've scrolled past the bottom, clamp
        if top + screen_height >= lines.len() {
            top = lines.len().saturating_sub(screen_height);
        }
    }

    // Once we exit the loop, raw mode + hidden cursor are undone by _guard's Drop.
}

#[cfg(test)]
mod test_scrolling_pager_with_horizontal {
    use super::scrolling_pager_with_horizontal;

    #[test]
    fn test_it_prints_when_fewer_lines_than_screen() {
        let lines = vec![
            "Short line 1".to_string(),
            "Short line 2".to_string(),
        ];
        // We can't truly test user input in raw mode, but we can call the function
        // to ensure it doesn't panic on short input.
        scrolling_pager_with_horizontal(&lines);
    }

    #[test]
    fn test_it_handles_some_long_lines() {
        let lines = vec![
            "This is a quite long line that should require horizontal scrolling if the terminal width is small.".to_string(),
            "Another extremely long line that might also need horizontal movement.".to_string(),
            "Third line is also fairly lengthy to test horizontal scrolling boundaries.".to_string(),
        ];
        scrolling_pager_with_horizontal(&lines);
    }
}
