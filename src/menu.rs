use std::io::Write;
use std::process::{Command, Stdio};

pub enum Menu {
    Bemenu { rows: String },
    Dmenu { rows: String },
    Rofi { rows: String },
    Fzf,
}

impl Menu {
    pub fn new(menu_program: String, rows: String) -> Result<Self, String> {
        match menu_program.as_str() {
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
                let mut items = items.to_string();
                items = items.replace(&format!("{}\n", default), "");
                items = format!("{}\n{}", default, items);
                Some(items)
            }
            (Some(items), None) => Some(items.to_string()),
            _ => None,
        };

        let output = match self {
            Self::Bemenu { rows } => {
                self.run_command("bemenu", &["-i", "-l", rows, "-p", prompt], menu_items)?
            }
            Self::Dmenu { rows } => {
                self.run_command("dmenu", &["-i", "-l", rows, "-p", prompt], menu_items)?
            }
            Self::Rofi { rows } => self.run_command(
                "rofi",
                &["-dmenu", "-i", "-l", rows, "-p", prompt],
                menu_items,
            )?,
            Self::Fzf => {
                let prompt = format!("{}> ", prompt);
                let menu_items = menu_items.unwrap_or("".to_string());
                self.run_command(
                    "fzf",
                    &["-i", "--print-query", "--prompt", &prompt],
                    Some(menu_items),
                )?
            }
        };

        Ok(output)
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
        let menu = Menu::new("bemenu".to_string(), "10".to_string());
        assert!(menu.is_ok());

        let menu = Menu::new("unsupported".to_string(), "10".to_string());
        assert!(menu.is_err());
    }

    #[test]
    fn test_menu_choose() {
        let menu = Menu::new("bemenu".to_string(), "10".to_string()).unwrap();
        let result = menu.choose(Some("pass\nfail"), Some("pass"), "Choose an item");
        if let Ok(result) = result {
            assert_eq!(result, "pass");
        }
    }
}
