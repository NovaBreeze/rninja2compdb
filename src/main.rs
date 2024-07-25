use core::panic;
use std::{
    io::{BufRead, BufReader, BufWriter},
    path::PathBuf,
    process::exit,
    str::FromStr,
};

use clap::{ArgAction, Parser};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CompDbEntry {
    directory: String,
    arguments: Vec<String>,
    file: String,
}

type CompDb = Vec<CompDbEntry>;

fn extract_commands_from_reader<R: BufRead>(rdr: R) -> Vec<String> {
    let re = regex::Regex::new(r#"^\s*command = /bin/bash -c "(PWD=.*\b(clang|clang\+\+)\b.*)"$"#)
        .unwrap();
    let commands: Vec<String> = rdr
        .lines()
        .filter_map(|l| {
            let l = l.unwrap();
            if let Some(capture) = re.captures(&l) {
                if let Some(matched) = capture.get(1) {
                    let extracted = matched.as_str();
                    let trimmed = extracted.replace("PWD=/proc/self/cwd ", "");
                    return Some(trimmed);
                }
            }
            return None;
        })
        .collect();
    commands
}

fn split_command_to_arguments(command: String) -> Vec<String> {
    shlex::Shlex::new(&command).collect()
}

fn extract_matched_patterns(db: CompDb, patterns: Vec<String>) -> CompDb {
    db.into_iter()
        .filter(move |x| {
            for p in &patterns {
                if x.file.contains(p) {
                    return true;
                }
            }
            return false;
        })
        .collect()
}

#[derive(Parser, Serialize, Deserialize)]
#[command(author = "tianyu", version = "0.1", about = "A simple binary program to generate the compile_command.json repository from ninja.", long_about = None)]
struct Cli {
    /// The path to the input file.
    /// If it's a .ninja file, it will be parsed to generate a clangd tag repository;
    /// if it's a .json repository file, it will extract entries that match the
    /// patterns parameter, or do nothing if no patterns are specified.
    #[arg(short = 'i', long = "input", value_name = "FILE")]
    input: Option<PathBuf>,

    /// Android root directory
    #[arg(short = 'r', long = "root", value_name = "DIR")]
    root: Option<PathBuf>,

    /// Output directory
    #[arg(short = 'o', long = "output", value_name = "DIR", default_value = ".")]
    output: PathBuf,

    /// Filename to process
    #[arg(
        short = 'f',
        long = "filename",
        value_name = "NAME",
        default_value = "compile_commands.json"
    )]
    filename: String,

    /// Pretty-print output
    #[arg(short = 'p', long = "on-pretty", action = ArgAction::SetFalse, default_value = "true")]
    pretty: bool,

    /// Patterns to match
    #[arg(short = 'P', long = "pattern", value_name = "PATTERN", action = ArgAction::Append)]
    patterns: Vec<String>,

    /// Parameter configuration file, if "-" is given,
    /// a template file is generated based on the current parameters.
    #[arg(short = 'c', long = "config", value_name = "FILE")]
    #[serde(skip_serializing)]
    config: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let cli: Cli = if let Some(cpath) = cli.config.clone() {
        if cpath.to_str().expect("Invalid config path!") == "-" {
            let wtr = std::fs::File::create("template.json").unwrap();
            let wtr = BufWriter::new(wtr);
            serde_json::to_writer_pretty(wtr, &cli).unwrap();
            exit(0);
        } else {
            let rdr = std::fs::File::open(cpath).unwrap();
            let rdr = BufReader::new(rdr);
            serde_json::from_reader(rdr).expect("Config file content is invalid!")
        }
    } else {
        cli
    };

    let input = match cli.input {
        Some(p) => p,
        None => panic!("Missing --input parameter!"),
    };

    let extension = input
        .extension()
        .expect("Not .ninja or .json input file!")
        .to_os_string()
        .to_string_lossy()
        .to_string();

    let compdb: CompDb = match extension.as_str() {
        "json" => {
            if cli.patterns.is_empty() {
                panic!("Missing --patterns parameter!");
            }
            let rdr = std::fs::File::open(input).unwrap();
            let rdr = BufReader::new(rdr);
            serde_json::from_reader(rdr).unwrap()
        }
        "ninja" => {
            let rdir = if let Some(r) = cli.root {
                r.to_str()
                    .expect("Invalid Android root directory!")
                    .to_string()
            } else {
                panic!("Missing --root parameter!");
            };
            let rdr = std::fs::File::open(input).unwrap();
            let rdr = BufReader::new(rdr);
            let commands = extract_commands_from_reader(rdr);
            commands
                .into_iter()
                .map(|x| {
                    let arguments = split_command_to_arguments(x);
                    let file = arguments.last().unwrap().to_string();
                    CompDbEntry {
                        directory: String::from_str(&rdir).unwrap(),
                        arguments,
                        file,
                    }
                })
                .collect()
        }
        _ => {
            panic!("Unsupport file type: {}", extension);
        }
    };

    if compdb.is_empty() {
        println!("Not found variable commands in ninja");
        return;
    }

    let compdb = if !cli.patterns.is_empty() {
        extract_matched_patterns(compdb, cli.patterns)
    } else {
        compdb
    };

    let mut output = cli.output.clone();
    output.push(cli.filename);
    let owtr = BufWriter::new(std::fs::File::create(output).unwrap());

    if cli.pretty {
        serde_json::to_writer_pretty(owtr, &compdb).unwrap();
    } else {
        serde_json::to_writer(owtr, &compdb).unwrap();
    }

    println!("Done");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_commands_from_reader() {
        let texts = r#"
command = /bin/bash -c "PWD=/proc/self/cwd vendor/qcom/proprietary/llvm-arm-toolchain-ship/14/bin/clang -I system/media/audio/include"
 command = /bin/bash -c "PWD=/proc/self/cwd vendor/qcom/proprietary/llvm-arm-toolchain-ship/14/bin/clang++ -I system/media/audio/include"
command = /bin/bash -c "PWD=/proc/self/cwd some other text"
command = /bin/bash -c "some other command"
random text that should be ignored
"#;
        let dest = vec![
            "vendor/qcom/proprietary/llvm-arm-toolchain-ship/14/bin/clang -I system/media/audio/include".to_string(),
            "vendor/qcom/proprietary/llvm-arm-toolchain-ship/14/bin/clang++ -I system/media/audio/include".to_string(),
        ];
        let commands = extract_commands_from_reader(BufReader::new(std::io::Cursor::new(texts)));
        assert_eq!(commands, dest);
    }

    #[test]
    fn test_split_command_to_arguments() {
        let text = "vendor/qcom/proprietary/llvm-arm-toolchain-ship/14/bin/clang -I system/media/audio/include";

        assert_eq!(
            split_command_to_arguments(text.to_string()),
            vec![
                "vendor/qcom/proprietary/llvm-arm-toolchain-ship/14/bin/clang".to_string(),
                "-I".to_string(),
                "system/media/audio/include".to_string(),
            ]
        );
    }
}
