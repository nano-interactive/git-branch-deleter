use crossterm::style::Colorize;
use std::fmt::Display;
use std::io::{Read, Result, Write};

#[derive(Debug, PartialEq)]
pub enum BranchAction {
    Show,
    Keep,
    Delete,
    Quit,
    Invalid,
}

impl Display for BranchAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BranchAction::Show => f.write_str("Show"),
            BranchAction::Keep => f.write_str("Keep"),
            BranchAction::Delete => f.write_str("Delete"),
            BranchAction::Quit => f.write_str("Quit"),
            BranchAction::Invalid => f.write_str("Invalid"),
        }
    }
}

impl From<u8> for BranchAction {
    fn from(c: u8) -> Self {
        match c {
            b'd' => Self::Delete,
            b'k' => Self::Keep,
            b's' => Self::Show,
            b'q' => Self::Quit,
            _ => Self::Invalid,
        }
    }
}

pub fn get_action<R: Read, W: Write>(out: &mut W, input: &mut R) -> Result<BranchAction> {
    let action = "Action".green();
    let mut buf: [u8; 3] = [0; 3];
    write!(out, "{}", action)?;

    out.flush()?;
    input.read(&mut buf)?;

    Ok(BranchAction::from(buf[0].to_ascii_lowercase()))
}
