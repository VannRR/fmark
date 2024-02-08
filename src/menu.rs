use std::io::Write;
use std::process::{Command, Stdio};

const CURRENT_MARKER: &str = " <-- current";

pub enum Menu {
    Bemenu { rows: String },
    Dmenu { rows: String },
    Rofi { rows: String },
    Fzf,
}

impl Menu {
    pub fn new(menu_program: &str, rows: String) -> Result<Self, String> {
        match menu_program {
            "bemenu" => Ok(Self::Bemenu { rows }),
            "dmenu" => Ok(Self::Dmenu { rows }),
            "rofi" => Ok(Self::Rofi { rows }),
            "fzf" => Ok(Self::Fzf),
            _ => Err(format!("Unsupported menu program: {}", menu_program)),
        }
    }

    pub fn choose(
        &self,
        menu_items: Option<&str>,
        default: Option<&str>,
        prompt: &str,
    ) -> Result<String, String> {
        let menu_items = match (menu_items, default) {
            (Some(items), Some(default)) => {
                Some(items.replace(default, &format!("{}{}", default, CURRENT_MARKER)))
            }
            (Some(items), None) => Some(items.to_string()),
            _ => None,
        };

        let output = match self {
            Self::Bemenu { rows } => {
                self.run_command("bemenu", &["-i", "-l", rows, "-p", prompt], menu_items)
            }
            Self::Dmenu { rows } => {
                self.run_command("dmenu", &["-i", "-l", rows, "-p", prompt], menu_items)
            }
            Self::Rofi { rows } => self.run_command(
                "rofi",
                &["-dmenu", "-i", "-l", rows, "-p", prompt],
                menu_items,
            ),
            Self::Fzf => {
                let prompt = format!("{}> ", prompt);
                let menu_items = menu_items.unwrap_or("".to_string());
                self.run_command(
                    "fzf",
                    &["-i", "--print-query", "--prompt", &prompt],
                    Some(menu_items),
                )
            }
        };
        match (output, default) {
            (Ok(mut output), Some(default)) => {
                output = output.replace(&format!("{}{}", default, CURRENT_MARKER), default);
                Ok(output)
            }
            (Ok(output), None) => Ok(output),
            (Err(e), _) => Err(e),
        }
    }

    fn run_command(
        &self,
        cmd: &str,
        args: &[&str],
        input: Option<String>,
    ) -> Result<String, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_new() {
        let menu = Menu::new("bemenu", "10".to_string());
        assert!(menu.is_ok());

        let menu = Menu::new("unsupported", "10".to_string());
        assert!(menu.is_err());
    }

    #[test]
    fn test_menu_choose() {
        let menu = Menu::new("bemenu", "10".to_string()).unwrap();
        let result = menu.choose(Some("item1\nitem2"), Some("item1"), "Choose an item");
        if let Ok(result) = result {
            assert!(result == "item1" || result == "item2");
        }
    }
}
