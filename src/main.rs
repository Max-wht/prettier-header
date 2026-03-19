use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

const SAMPLE_TITLE: &str = "Auction Participation";
const PATTERN_ONE_ID: &str = "1";
const PATTERN_TWO_ID: &str = "2";
const PATTERN_ONE_BORDER_WIDTH: usize = 96;
const PATTERN_ONE_TEXT_WIDTH: usize = 94;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Pattern {
    One,
    Two,
}

impl Pattern {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            PATTERN_ONE_ID => Some(Self::One),
            PATTERN_TWO_ID => Some(Self::Two),
            _ => None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum AppCommand {
    List,
    Generate { pattern: Pattern, title: String },
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!("{}", usage());
        return;
    }

    let command = match parse_args(&args) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("{message}");
            eprintln!();
            eprintln!("{}", usage());
            std::process::exit(2);
        }
    };

    match command {
        AppCommand::List => {
            println!("{}", list_patterns());
        }
        AppCommand::Generate { pattern, title } => {
            let rendered = render_pattern(pattern, &title);
            println!("{rendered}");

            if let Err(message) = copy_to_clipboard(&rendered) {
                eprintln!("warning: failed to copy to clipboard: {message}");
            }
        }
    }
}

fn parse_args(args: &[String]) -> Result<AppCommand, String> {
    let mut list = false;
    let mut pattern = Pattern::One;
    let mut title_parts: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-l" | "--list" => {
                list = true;
                i += 1;
            }
            "-p" | "--pattern" => {
                let Some(value) = args.get(i + 1) else {
                    return Err(String::from("missing value for -p/--pattern"));
                };

                let Some(parsed) = Pattern::parse(value) else {
                    return Err(format!(
                        "invalid pattern '{value}', expected '{PATTERN_ONE_ID}' or '{PATTERN_TWO_ID}'"
                    ));
                };

                pattern = parsed;
                i += 2;
            }
            option if option.starts_with('-') => {
                return Err(format!("unknown option '{option}'"));
            }
            positional => {
                title_parts.push(positional.to_string());
                i += 1;
            }
        }
    }

    if list {
        if !title_parts.is_empty() {
            return Err(String::from("title is not allowed when using --list"));
        }
        return Ok(AppCommand::List);
    }

    let title = title_parts.join(" ");
    if title.trim().is_empty() {
        return Err(String::from("missing TITLE argument"));
    }

    Ok(AppCommand::Generate { pattern, title })
}

fn usage() -> &'static str {
    "Usage:
  phmx [OPTIONS] <TITLE>
  phmx -l | --list

Options:
  -p, --pattern <ID>   Pattern ID, accepts only '1' or '2' (default: '1')
  -l, --list           Show all pattern previews
  -h, --help           Show this help"
}

fn list_patterns() -> String {
    format!(
        "1. {id1}\n{preview1}\n\n2. {id2}\n{preview2}",
        id1 = PATTERN_ONE_ID,
        preview1 = render_pattern(Pattern::One, SAMPLE_TITLE),
        id2 = PATTERN_TWO_ID,
        preview2 = render_pattern(Pattern::Two, SAMPLE_TITLE)
    )
}

fn render_pattern(pattern: Pattern, title: &str) -> String {
    match pattern {
        Pattern::One => render_pattern_one(title),
        Pattern::Two => render_pattern_two(title),
    }
}

fn render_pattern_one(title: &str) -> String {
    let fitted_title = fit_text(title, PATTERN_ONE_TEXT_WIDTH);
    let centered = center_text(&fitted_title, PATTERN_ONE_TEXT_WIDTH);
    let border = "=".repeat(PATTERN_ONE_BORDER_WIDTH);
    format!("// {border}\n// │{centered}│\n// {border}")
}

fn render_pattern_two(title: &str) -> String {
    format!("<<<=========--{} --========>>>", title)
}

fn center_text(text: &str, width: usize) -> String {
    let text_width = text.chars().count();
    if text_width >= width {
        return text.to_string();
    }

    let total_padding = width - text_width;
    let left_padding = total_padding / 2;
    let right_padding = total_padding - left_padding;

    format!(
        "{}{}{}",
        " ".repeat(left_padding),
        text,
        " ".repeat(right_padding)
    )
}

fn fit_text(text: &str, width: usize) -> String {
    let text_width = text.chars().count();
    if text_width <= width {
        return text.to_string();
    }

    if width <= 3 {
        return text.chars().take(width).collect();
    }

    let mut result: String = text.chars().take(width - 3).collect();
    result.push_str("...");
    result
}

fn copy_to_clipboard(text: &str) -> Result<(), String> {
    if !cfg!(target_os = "macos") {
        return Err(String::from(
            "this version only supports clipboard copy on macOS",
        ));
    }

    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not start pbcopy: {error}"))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| String::from("failed to open pbcopy stdin"))?;

    stdin
        .write_all(text.as_bytes())
        .map_err(|error| format!("could not write to pbcopy stdin: {error}"))?;

    drop(stdin);

    let status = child
        .wait()
        .map_err(|error| format!("could not wait for pbcopy: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("pbcopy exited with status {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_list_command() {
        let args = vec!["--list".to_string()];
        assert_eq!(parse_args(&args), Ok(AppCommand::List));
    }

    #[test]
    fn parse_generate_command_default_pattern() {
        let args = vec!["Auction".to_string(), "Participation".to_string()];
        assert_eq!(
            parse_args(&args),
            Ok(AppCommand::Generate {
                pattern: Pattern::One,
                title: "Auction Participation".to_string(),
            })
        );
    }

    #[test]
    fn parse_generate_command_custom_pattern() {
        let args = vec![
            "-p".to_string(),
            "2".to_string(),
            "Auction Participation".to_string(),
        ];
        assert_eq!(
            parse_args(&args),
            Ok(AppCommand::Generate {
                pattern: Pattern::Two,
                title: "Auction Participation".to_string(),
            })
        );
    }

    #[test]
    fn reject_invalid_pattern() {
        let args = vec!["-p".to_string(), "3".to_string(), "Title".to_string()];
        let error = parse_args(&args).unwrap_err();
        assert!(error.contains("invalid pattern"));
    }

    #[test]
    fn render_pattern_one_shape() {
        let rendered = render_pattern(Pattern::One, SAMPLE_TITLE);
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].chars().count(), lines[1].chars().count());
        assert_eq!(lines[1].chars().count(), lines[2].chars().count());
    }

    #[test]
    fn render_pattern_two_shape() {
        let rendered = render_pattern(Pattern::Two, SAMPLE_TITLE);
        assert_eq!(
            rendered,
            "<<<=========--Auction Participation --========>>>"
        );
    }

    #[test]
    fn render_pattern_one_long_title_keeps_width() {
        let long_title = "X".repeat(120);
        let rendered = render_pattern(Pattern::One, &long_title);
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines[0].chars().count(), 99);
        assert_eq!(lines[1].chars().count(), 99);
        assert_eq!(lines[2].chars().count(), 99);
        assert!(lines[1].contains("..."));
    }
}
