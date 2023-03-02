use clap::Parser;
use colored::Colorize;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Parser)]
struct Cli {
    /// Format to rename the file in
    ///
    /// formats given in CLI are not saved in history, it helps when
    /// batch processing a list of files with similar format at once,
    /// use `NNN` character format for zero padded numbers.
    #[arg(short, long)]
    format: Option<String>,
    /// Repeat Last choice
    #[arg(short, long, action)]
    last: bool,
    /// Rename given file
    #[arg(short, long, action)]
    rename: bool,
    /// Paths to rename
    paths: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
struct History {
    formats: Vec<String>,
    variables: HashSet<String>,
    values: HashMap<String, Vec<String>>,
}

impl Default for History {
    fn default() -> Self {
        History {
            formats: Vec::new(),
            variables: HashSet::new(),
            values: HashMap::new(),
        }
    }
}

fn save_history(fname: &PathBuf, history: &History) -> Result<(), Box<dyn Error>> {
    let par = fname.parent().unwrap();
    if !par.exists() {
        std::fs::create_dir_all(&par)?;
    }
    let file = File::create(fname)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, history)?;
    Ok(())
}

fn read_history(path: &PathBuf) -> Result<History, Box<dyn Error>> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Ok(History::default());
            } else {
                return Err(Box::new(e));
            }
        }
    };
    let reader = BufReader::new(file);
    let hist = serde_json::from_reader(reader)?;
    Ok(hist)
}

fn choose(prompt: &str, vec: &mut Vec<String>) -> Result<String, Box<dyn Error>> {
    let mut i = 1;
    let mut manual = vec.len() == 0;
    let mut buf = String::new();
    let mut choice: usize = 0;

    if !manual {
        println!("{} {}:", "Choices for".bold().blue(), prompt.bold().blue());
        for h in &mut *vec {
            println!("  [{}] {}", i, h);
            i += 1;
        }
        loop {
            print!("{} <1>: ", "Choice".on_blue().bold());
            std::io::stdout().flush()?;
            std::io::stdin().read_line(&mut buf)?;
            if buf.trim() == "" {
                choice = 0
            } else {
                choice = match buf.trim().parse() {
                    Ok(c) => {
                        if c > vec.len() {
                            eprintln!("{}: Enter from 0 to {} only", "Error".red(), vec.len());
                            buf.clear();
                            continue;
                        } else {
                            c
                        }
                    }
                    Err(e) => {
                        eprintln!("{}: {:?}", "Error".red(), e.kind());
                        buf.clear();
                        continue;
                    }
                };
                if choice == 0 {
                    manual = true;
                } else {
                    choice -= 1;
                }
            }
            break;
        }
    }
    if manual {
        print!(
            "{}{}: ",
            "Input ".on_bright_green().black().bold(),
            prompt.on_bright_green().black().bold()
        );
        std::io::stdout().flush()?;
        buf.clear();
        std::io::stdin().read_line(&mut buf)?;
        vec.push(buf.trim().to_string());
        choice = vec.len() - 1;
    }
    // moves the choosen option to the front and returns it
    let choice = vec.remove(choice);
    vec.insert(0, choice.clone());
    Ok(choice)
}

fn rename_filename(
    hist: &mut History,
    fmt_str: &str,
    num: usize,
    last: bool,
) -> Result<String, Box<dyn Error>> {
    let vars: Vec<String> = fmt_str
        .split("_")
        .map(|v| match hist.values.get_mut(v) {
            Some(mut k) => {
                if last {
                    Ok(k[0].clone())
                } else {
                    choose(v, &mut k)
                }
            }
            None => {
                if v.chars().all(|c| c == 'N') {
                    Ok(format!("{0:01$}", num, v.len()))
                } else {
                    hist.variables.insert(v.to_string());
                    let mut newvec = vec![];
                    // here since the variable is not new when --last
                    // is used it won't happen, so I'll leave it be
                    // interactive.
                    let var = choose(v, &mut newvec);
                    hist.values.insert(v.to_string(), newvec);
                    var
                }
            }
        })
        .collect::<Result<Vec<String>, Box<dyn Error>>>()?;
    Ok(vars.join("_").replace(" ", "-"))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let hist_file = ProjectDirs::from(
        "org",       /*qualifier*/
        "ZeroSofts", /*organization*/
        "nameit",    /*application*/
    )
    .unwrap()
    .data_dir()
    .join("histories.json");
    let mut hist = read_history(&hist_file)?;
    let fmt_str = if let Some(f) = args.format {
        f
    } else {
        if args.last {
            hist.formats[0].clone()
        } else {
            choose("Format", &mut hist.formats)?
        }
    };

    for (i, filename) in args.paths.iter().enumerate() {
        let new_name = filename
            .with_file_name(rename_filename(&mut hist, &fmt_str, i + 1, args.last)?)
            .with_extension(filename.extension().unwrap_or_default());
        println!(
            "{}: {:?} -> {:?}",
            "Copy".green().bold(),
            filename,
            new_name
        );
        if args.rename {
            std::fs::rename(filename, new_name)?;
        } else {
            std::fs::copy(filename, new_name)?;
        }
    }
    save_history(&hist_file, &hist)?;
    Ok(())
}
