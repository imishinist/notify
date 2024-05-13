use std::fmt;
use std::io;
use std::process::{self, Command};

use log;
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Sound {
    Basso,
    Blow,
    Bottle,
    Frog,
    Funk,
    Glass,
    Hero,
    Morse,
    Ping,
    Pop,
    Purr,
    Sosumi,
    Submarine,
    Tink,
}

impl fmt::Display for Sound {
    fn fmt(&self, w: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Sound::Basso => write!(w, "Basso"),
            Sound::Blow => write!(w, "Blow"),
            Sound::Bottle => write!(w, "Bottle"),
            Sound::Frog => write!(w, "Frog"),
            Sound::Funk => write!(w, "Funk"),
            Sound::Glass => write!(w, "Glass"),
            Sound::Hero => write!(w, "Hero"),
            Sound::Morse => write!(w, "Morse"),
            Sound::Ping => write!(w, "Ping"),
            Sound::Pop => write!(w, "Pop"),
            Sound::Purr => write!(w, "Purr"),
            Sound::Sosumi => write!(w, "Sosumi"),
            Sound::Submarine => write!(w, "Submarine"),
            Sound::Tink => write!(w, "Tink"),
        }
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
#[clap(group(clap::ArgGroup::new("notify-message")
        .required(true)
        .args(&["message", "command"])))]
struct NotifyCmd {
    #[clap(short, long, default_value = "blow")]
    sound: Sound,

    #[clap(short, long)]
    message: Option<String>,

    #[clap(name = "command")]
    rest: Vec<String>,
}

#[derive(thiserror::Error, Debug)]
enum NotifyError {
    #[error("Failed to run command: {status}")]
    CommandError { status: i32 },

    #[error(transparent)]
    IoError(#[from] io::Error),
}

fn main() {
    env_logger::init();
    let cmd = NotifyCmd::parse();
    
    if let Some(message) = cmd.message {
        if let Err(e) = notify(None, &message, cmd.sound) {
            let _ = notify(Some("Notify Failed"), format!("message: {} exit {:?}", message, e), cmd.sound);
            process::exit(1);
        }
        return;
    }

    let child_command = cmd.rest.join(" ");
    if let Err(e) = run_command(&child_command) {
        match e {
            NotifyError::CommandError { status } => {
                let _ = notify(Some("Run Failed"), format!("command: {} exit {:?}", child_command, e), cmd.sound);
                process::exit(status);
            }
            _ => (),
        }
        let _ = notify(Some("Run Failed"), format!("command: {} exit {:?}", child_command, e), cmd.sound);
        process::exit(1);
    }

    let _ = notify(None, &child_command, cmd.sound);
}

fn run_command(cmd: &str) -> Result<(), NotifyError> {
    let mut child = Command::new("sh").arg("-c").arg(cmd).spawn()?;
    log::info!("run command: {:?}", cmd);
    let status = child.wait()?;

    if !status.success() {
        return Err(NotifyError::CommandError {
            status: status.code().unwrap_or(1),
        });
    }
    Ok(())
}

fn notify<S: AsRef<str>>(title: Option<&str>, message: S, sound: Sound) -> io::Result<()> {
    let title = title.unwrap_or("from notify");
    log::info!("notify {{.title=\"{}\", .message={:?}}}", title, message.as_ref());

    let osascript = format!(
        r#"display notification "{}" with title "{}" sound name "{}""#,
        message.as_ref(),
        title,
        sound
    );

    let _ = Command::new("osascript")
        .arg("-e")
        .arg(osascript)
        .output()?;
    Ok(())
}
