use std::io::Write;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use zeroize::Zeroize;

pub fn prompt_user_for_bips(phrases: &mut Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Please enter your 28 phrase bips recovery key or phrase,\n\
                            one phrase at a time and press enter. the software will not echo your bips phrase.\n\
                            The sofware will only accept english lowercase letters"
    );
    let _ = std::io::stdout().flush().ok();
    let mut phrase = String::new();
    crossterm::terminal::enable_raw_mode()?;
    while crossterm::event::poll(std::time::Duration::ZERO)? {
        let _ = crossterm::event::read()?; // Discard the "stale" event
    }
    let _ = std::io::stdout().flush().ok();
    crossterm::execute!(
        std::io::stdout(),
        crossterm::style::Print(format!(
            "Word {0} of {1}\n",
            phrases.len() + 1,
            crate::bips::BIPS_WORDLEN_COUNT_ENTROPY_256_BITS
        ))
    )?;
    loop {
        // 256 bits entropy

        if phrases.len() >= crate::bips::BIPS_WORDLEN_COUNT_ENTROPY_256_BITS {
            break;
        }

        match crossterm::event::read()? {
            crossterm::event::Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers,
                ..
            }) if c == 'c' && modifiers.contains(KeyModifiers::CONTROL) => {
                crossterm::terminal::disable_raw_mode()?;
                return Err("Recieved ctrl+c".into());
            }
            crossterm::event::Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind,
                ..
            }) if kind == KeyEventKind::Release => {
                // user pressed enter
                if crate::bips::is_valid_word(&phrase) {
                    phrases.push(phrase.clone());
                    phrase.zeroize();
                    phrase.clear();
                    let _ = std::io::stdout().flush().ok();
                    crossterm::execute!(
                        std::io::stdout(),
                        crossterm::style::Print(format!(
                            "Word {0} of {1}\n",
                            phrases.len(),
                            crate::bips::BIPS_WORDLEN_COUNT_ENTROPY_256_BITS
                        ))
                    )?;
                    // feedback
                } else {
                    crossterm::execute!(
                        std::io::stdout(),
                        crossterm::style::Print(format!(
                            "That is not a valid BIP39 word. Try again\n",
                        ))
                    )?;
                    phrase.zeroize();
                    phrase.clear();
                }
            }
            crossterm::event::Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                kind,
                ..
            }) if kind == KeyEventKind::Press => {
                if let None = phrase.pop() {
                    let _ = crossterm::execute!(std::io::stdout(), crossterm::style::Print("\x07"))
                        .ok();
                }
            }
            crossterm::event::Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                kind,
                ..
            }) if kind == KeyEventKind::Press => {
                if c.is_ascii_alphabetic() {
                    phrase.push(c);
                } else {
                    let _ = crossterm::execute!(std::io::stdout(), crossterm::style::Print("\x07"))
                        .ok();
                }
            }
            _ => {}
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
