use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
enum Command {
    Remove(RemoveOptions),
    Help,
    RemoveHelp,
}

#[derive(Debug, PartialEq, Default)]
struct RemoveOptions {
    paths: Vec<PathBuf>,
    recursive: bool,
    dry_run: bool,
}

#[derive(Debug, PartialEq)]
enum ParseError {
    MissingPath,
    UnknownCommand(OsString),
    UnknownOption(OsString),
}

fn parse_args(args: &[OsString]) -> Result<Command, ParseError> {
    match args.split_first() {
        Some((first, rest)) => {
            if first == "--help" {
                Ok(Command::Help)
            } else if first == "remove" {
                parse_remove_args(rest)
            } else {
                Err(ParseError::UnknownCommand(first.clone()))
            }
        }
        None => Ok(Command::Help),
    }
}

fn parse_remove_args(args: &[OsString]) -> Result<Command, ParseError> {
    let mut parsing_options = true;
    let mut recursive = false;
    let mut paths: Vec<PathBuf> = vec![];
    let mut dry_run = false;

    for arg in args {
        if parsing_options && arg == "--help" {
            return Ok(Command::RemoveHelp);
        }

        if parsing_options && arg == "--" {
            parsing_options = false;
            continue;
        }

        if parsing_options && (arg == "-r" || arg == "--recursive") {
            recursive = true;
        } else if parsing_options && arg == "--dry-run" {
            dry_run = true;
        } else if parsing_options && arg.as_encoded_bytes().starts_with(b"-") {
            return Err(ParseError::UnknownOption(arg.clone()));
        } else {
            parsing_options = false;
            paths.push(PathBuf::from(arg));
        }
    }

    if paths.is_empty() {
        Err(ParseError::MissingPath)
    } else {
        Ok(Command::Remove(RemoveOptions {
            paths,
            recursive,
            dry_run,
            ..RemoveOptions::default()
        }))
    }
}

#[cfg(test)]
mod tests {

    use super::{Command, OsString, ParseError, PathBuf, RemoveOptions, parse_args};

    #[test]
    fn remove_without_paths_returns_missing_path() {
        assert_eq!(
            parse_args(&[OsString::from("remove")]),
            Err(ParseError::MissingPath)
        );
    }

    #[test]
    fn remove_with_path_returns_remove_command() {
        assert_eq!(
            parse_args(&[OsString::from("remove"), OsString::from("a.txt")]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("a.txt")],
                recursive: false,
                ..RemoveOptions::default()
            }))
        );
    }

    #[test]
    fn unknown_command_returns_error() {
        assert_eq!(
            parse_args(&[OsString::from("other"), OsString::from("a.txt")]),
            Err(ParseError::UnknownCommand(OsString::from("other")))
        );
    }

    #[test]
    fn no_arguments_returns_help() {
        assert_eq!(parse_args(&[]), Ok(Command::Help));
    }

    #[test]
    fn help_option_returns_help() {
        assert_eq!(parse_args(&[OsString::from("--help")]), Ok(Command::Help));
    }

    #[test]
    fn remove_help_option_returns_remove_help() {
        assert_eq!(
            parse_args(&[OsString::from("remove"), OsString::from("--help")]),
            Ok(Command::RemoveHelp)
        );
    }

    #[test]
    fn remove_with_recursive_option_enables_recursion() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("-r"),
                OsString::from("logs"),
            ]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("logs")],
                recursive: true,
                ..RemoveOptions::default()
            }))
        );
    }

    #[test]
    fn recursive_option_after_path_is_treated_as_path() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("logs"),
                OsString::from("-r"),
            ]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("logs"), PathBuf::from("-r")],
                recursive: false,
                ..RemoveOptions::default()
            }))
        );
    }

    #[test]
    fn remove_with_long_recursive_option_enables_recursion() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("--recursive"),
                OsString::from("logs"),
            ]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("logs")],
                recursive: true,
                ..RemoveOptions::default()
            }))
        );
    }

    #[test]
    fn double_dash_ends_option_parsing() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("--"),
                OsString::from("-r"),
            ]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("-r")],
                recursive: false,
                ..RemoveOptions::default()
            }))
        );
    }

    #[test]
    fn double_dash_after_path_is_treated_as_path() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("logs"),
                OsString::from("--"),
            ]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("logs"), PathBuf::from("--")],
                recursive: false,
                ..RemoveOptions::default()
            }))
        );
    }

    #[test]
    fn double_dash_after_separator_is_treated_as_path() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("--"),
                OsString::from("--"),
            ]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("--")],
                recursive: false,
                ..RemoveOptions::default()
            }))
        );
    }

    #[test]
    fn return_help_regarding_recursive_options() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("-r"),
                OsString::from("--help")
            ]),
            Ok(Command::RemoveHelp)
        )
    }

    #[test]
    fn help_text_appears_after_the_path() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("logs"),
                OsString::from("--help"),
            ]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("logs"), PathBuf::from("--help")],
                recursive: false,
                ..RemoveOptions::default()
            }))
        )
    }

    #[test]
    fn help_text_appears_after_the_separator() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("--"),
                OsString::from("--help")
            ]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("--help")],
                recursive: false,
                ..RemoveOptions::default()
            }))
        )
    }

    #[test]
    fn unknown_option_returns_error() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("--unknown"),
                OsString::from("logs")
            ]),
            Err(ParseError::UnknownOption(OsString::from("--unknown")))
        )
    }

    #[test]
    fn unknown_option_after_recursive_option_returns_error() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("-r"),
                OsString::from("--unknown"),
                OsString::from("logs")
            ]),
            Err(ParseError::UnknownOption(OsString::from("--unknown")))
        )
    }

    #[test]
    fn unknown_option_after_path_is_treated_as_path() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("logs"),
                OsString::from("--unknown")
            ]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("logs"), PathBuf::from("--unknown")],
                recursive: false,
                ..RemoveOptions::default()
            }))
        )
    }

    #[test]
    fn unknown_option_after_separator_is_treated_as_path() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("--"),
                OsString::from("--unknown")
            ]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("--unknown")],
                recursive: false,
                ..RemoveOptions::default()
            }))
        )
    }

    #[test]
    fn dry_run_option_enables_preview() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("--dry-run"),
                OsString::from("logs")
            ]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("logs")],
                dry_run: true,
                ..RemoveOptions::default()
            }))
        )
    }

    #[test]
    fn dry_run_after_recursive_option_enables_both() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("-r"),
                OsString::from("--dry-run"),
                OsString::from("logs")
            ]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("logs")],
                recursive: true,
                dry_run: true,
                ..RemoveOptions::default()
            }))
        )
    }

    #[test]
    fn recursive_after_dry_run_option_enables_both() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("--dry-run"),
                OsString::from("-r"),
                OsString::from("logs")
            ]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("logs")],
                recursive: true,
                dry_run: true,
                ..RemoveOptions::default()
            }))
        )
    }

    #[test]
    fn dry_run_after_path_is_treated_as_path() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("logs"),
                OsString::from("--dry-run")
            ]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("logs"), PathBuf::from("--dry-run")],
                dry_run: false,
                ..RemoveOptions::default()
            }))
        )
    }

    #[test]
    fn dry_run_after_separator_is_treated_as_path() {
        assert_eq!(
            parse_args(&[
                OsString::from("remove"),
                OsString::from("--"),
                OsString::from("--dry-run")
            ]),
            Ok(Command::Remove(RemoveOptions {
                paths: vec![PathBuf::from("--dry-run")],
                ..RemoveOptions::default()
            }))
        )
    }

    #[test]
    fn dry_run_without_path_returns_missing_path() {
        assert_eq!(
            parse_args(&[OsString::from("remove"), OsString::from("--dry-run")]),
            Err(ParseError::MissingPath)
        )
    }
}
