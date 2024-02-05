use std::io::Write;
use std::process::{Command, Stdio};

pub enum Menu {
    Bemenu { rows: String },
    Dmenu { rows: String },
    Rofi { rows: String },
}

impl Menu {
    pub fn new(menu_program: &str, rows: String) -> Result<Self, String> {
        match menu_program {
            "bemenu" => Ok(Self::Bemenu { rows }),
            "dmenu" => Ok(Self::Dmenu { rows }),
            "rofi" => Ok(Self::Rofi { rows }),
            _ => Err(format!("Unsupported menu program: {}", menu_program)),
        }
    }

    pub fn input(&self, prompt: &str) -> Result<String, String> {
        match self {
            Self::Bemenu { rows: _ } => self.run_command("bemenu", &["-p", prompt], None),
            Self::Dmenu { rows: _ } => self.run_command("dmenu", &["-p", prompt], None),
            Self::Rofi { rows: _ } => self.run_command("rofi", &["-dmenu", "-p", prompt], None),
        }
    }

    pub fn choose(&self, menu_items: &str, prompt: &str) -> Result<String, String> {
        match self {
            Self::Bemenu { rows } => self.run_command(
                "bemenu",
                &["-i", "-l", rows, "-p", prompt],
                Some(menu_items),
            ),
            Self::Dmenu { rows } => self.run_command(
                "dmenu",
                &["-i", "-l", rows, "-p", prompt],
                Some(menu_items),
            ),
            Self::Rofi { rows } => self.run_command(
                "rofi",
                &["-dmenu", "-i", "-l", rows, "-p", prompt],
                Some(menu_items),
            ),
        }
    }

    fn run_command(&self, cmd: &str, args: &[&str], input: Option<&str>) -> Result<String, String> {
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|_| format!("Failed to execute command: {}", cmd))?;

        if let Some(input) = input {
            if let Some(stdin) = child.stdin.as_mut() {
                stdin
                    .write_all(input.as_bytes())
                    .map_err(|_| "Failed to write to stdin".to_string())?;
            }
        }

        let output = child
            .wait_with_output()
            .map_err(|_| "Failed to wait on child".to_string())?;
        String::from_utf8(output.stdout)
            .map_err(|_| "Invalid UTF-8 sequence".to_string())
            .map(|v| v.trim().to_string())
    }
}
