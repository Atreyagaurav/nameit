use chrono::Local;
use clap::{ArgGroup, Parser};
use colored::Colorize;
use directories::ProjectDirs;
use number_range::NumberRangeOptions;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use term_grid;
use terminal_size::{terminal_size, Width};

#[derive(Parser)]
#[command(group = ArgGroup::new("action").required(false).multiple(false))]
struct Cli {
    /// Format to rename the file in
    ///
    /// formats given in CLI are not saved in history, it helps when
    /// batch processing a list of files with similar format at once,
    /// use `NNN` character format for zero padded numbers. If not
    /// given asks interactively.
    #[arg(short, long)]
    format: Option<String>,
    /// Destination directory
    ///
    /// Move or Rename the file to the destination directory instead
    /// of the current one.
    #[arg(short, long)]
    destination: Option<PathBuf>,
    /// Repeat Last choice
    ///
    /// Choose the first option for all the interactive choices. Be
    /// careful using this one on formats without the number (N, NN,
    /// etc.) variable, as all the files will be names the same. And
    /// the number format will only work for a single execution, even
    /// if you've used it before for different files, it'll restart
    /// from 1.
    #[arg(short, long, action)]
    last: bool,
    /// Replace a file if same name is generated
    #[arg(short = 'R', long, action)]
    replace: bool,
    /// Rename given file instead of copying
    ///
    /// Only works for files in the same mount point, if you have
    /// files in different mount points, you have to use `--move`
    #[arg(short, long, action, group = "action")]
    rename: bool,
    /// Move a file instead of copying
    ///
    /// Unlike rename it works even in different mount point, but
    /// moving is done by first copying the file and then removing the
    /// original, so it'll take time, while rename is just changing
    /// the name so it's fast
    #[arg(short, long, action, group = "action")]
    r#move: bool,
    /// Edit saved choices
    ///
    /// Gives you interactive options to edit the choices. Use it to
    /// permanently filter the options.
    #[arg(short, long, action)]
    edit: bool,
    /// Print the new filename and do nothing
    #[arg(short, long, action)]
    test: bool,
    /// Number of choices to show from history
    #[arg(short, long, default_value = "20")]
    choices: usize,
    /// Paths to rename
    ///
    /// If you have more than one path then any number of character
    /// `N` in the format string will be replaced with the loop index
    /// (starting at 1), you can use that system to batch rename
    /// files.
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

fn choose(
    prompt: &str,
    vec: &mut Vec<String>,
    filter: bool,
    max_choice: usize,
) -> Result<String, Box<dyn Error>> {
    let mut manual = vec.len() == 0;
    let mut buf = String::new();
    let mut choice: usize = 0;

    if !manual {
        println!("{} {}:", "Choices for".bold().blue(), prompt.bold().blue());
        let mut grid = term_grid::Grid::new(term_grid::GridOptions {
            filling: term_grid::Filling::Spaces(2),
            direction: term_grid::Direction::LeftToRight,
        });
        if !filter {
            grid.add(term_grid::Cell::from(format!(
                "[0] {}",
                "<new entry>".bold().yellow()
            )));
        }

        let mut i = 1;
        for h in &mut *vec {
            grid.add(term_grid::Cell::from(format!("  [{}] {}", i, h)));
            i += 1;
            if i > max_choice {
                break;
            }
        }
        let width: usize = if let Some((Width(w), _)) = terminal_size() {
            w.into()
        } else {
            100
        };
        println!("{}", grid.fit_into_width(width).unwrap());
        let def = if filter {
            format!("1-{}", vec.len())
        } else {
            "1".to_string()
        };
        loop {
            print!("{} <{}>: ", "Select".on_blue().bold(), def);
            std::io::stdout().flush()?;
            std::io::stdin().read_line(&mut buf)?;
            match (buf.trim(), filter) {
                ("", true) => return Ok(def),
                ("", false) => choice = 0,
                (b, true) => {
                    let choices: HashSet<usize> = NumberRangeOptions::default()
                        .with_list_sep(',')
                        .with_range_sep('-')
                        .with_default_start(1)
                        .with_default_end(vec.len())
                        .parse(&b)?
                        .collect();
                    let mut new_vec: Vec<String> = vec
                        .into_iter()
                        .enumerate()
                        .filter_map(|(i, f)| {
                            if choices.contains(&(i + 1)) {
                                Some(f.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                    vec.clear();
                    vec.append(&mut new_vec);
                    return Ok(buf);
                }
                (b, false) => {
                    choice = match b.parse() {
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
            }
            break;
        }
    }
    if manual {
        if filter {
            return Ok("0".to_string());
        }
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
    cur: &str,
    hist: &mut History,
    fmt_str: &str,
    num: usize,
    last: bool,
    max_choice: usize,
) -> Result<String, Box<dyn Error>> {
    let vars: Vec<String> = fmt_str
        .split("_")
        .map(|v| match hist.values.get_mut(v) {
            Some(mut k) => {
                if last {
                    Ok(k[0].clone())
                } else {
                    choose(v, &mut k, false, max_choice)
                }
            }
            None => {
                if v.chars().all(|c| c == '#') {
                    Ok(format!("{0:01$}", num, v.len()))
                } else if v == "?" {
                    Ok(cur.to_string())
                } else if v.starts_with("%") {
                    Ok(Local::now().format(&v).to_string())
                } else if v.chars().all(|c| c == '*') {
                    Ok(format!(
                        "{}",
                        cur.split("_")
                            .take(v.len())
                            .collect::<Vec<&str>>()
                            .join("_")
                    ))
                } else {
                    hist.variables.insert(v.to_string());
                    let mut newvec = vec![];
                    // here since the variable is not new when --last
                    // is used it won't happen, so I'll leave it be
                    // interactive. Is manual format is given from
                    // TUI, it'll need one time input.
                    let var = choose(v, &mut newvec, false, max_choice);
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

    if args.edit {
        choose("Formats", &mut hist.formats, true, args.choices)?;
        let new_vars: HashSet<&str> = hist
            .formats
            .iter()
            .map(|s| s.split("_"))
            .flatten()
            .collect();
        let mut new_values = HashMap::<String, Vec<String>>::new();
        for (k, v) in hist.values {
            if !new_vars.contains(k.as_str()) {
                println!("{} {}", k, "variable doesn't appear in any formats".red());
            }
            let mut v = v;
            choose(&k, &mut v, true, args.choices)?;
            if v.len() == 0 {
                continue;
            }
            new_values.insert(k.to_string(), v.to_vec());
        }
        hist.variables = new_values.keys().map(|s| s.to_string()).collect();
        hist.values = new_values;
        save_history(&hist_file, &hist)?;
        return Ok(());
    }

    if args.paths.len() == 0 {
        return Ok(());
    }

    let fmt_str = if let Some(f) = args.format {
        f
    } else {
        if args.last {
            hist.formats[0].clone()
        } else {
            choose("Format", &mut hist.formats, false, args.choices)?
        }
    };

    for (i, filename) in args.paths.iter().enumerate() {
        let ext = filename.extension();
        let fname = rename_filename(
            &filename.file_stem().unwrap_or_default().to_string_lossy(),
            &mut hist,
            &fmt_str,
            i + 1,
            args.last,
            args.choices,
        )?;
        let mut new_name = match ext {
            None => filename.with_file_name(fname),
            Some(e) => filename.with_file_name(format!(
                // .with_extension() thing didn't work as it removes any
                // part of the name after first '.' in filename
                "{}.{}",
                fname,
                e.to_string_lossy(),
            )),
        };
        if let Some(d) = &args.destination {
            // if destination is given discard the parent directory information
            new_name = d.join(new_name.file_name().unwrap());
        }
        println!(
            "{}: {:?} -> {:?}",
            (match (args.rename, args.r#move) {
                (true, false) => "Rename",
                (false, true) => "Move",
                (false, false) => "Copy",
                _ => "Error",
            })
            .green()
            .bold(),
            filename,
            new_name
        );
        if args.test {
            continue;
        }
        if new_name.exists() {
            if !args.replace {
                print!(
                    "{}: {:?} will be replaced by file {:?}, continue <y/N>? ",
                    "Warning".on_yellow().bold(),
                    filename,
                    new_name
                );
                std::io::stdout().flush()?;
                let mut buf = String::new();
                std::io::stdin().read_line(&mut buf)?;
                if !(buf.trim().to_lowercase() == "y") {
                    continue;
                }
            }
        }
        if args.rename {
            std::fs::rename(filename, new_name)?;
        } else {
            std::fs::copy(filename, new_name)?;
            if args.r#move {
                std::fs::remove_file(filename)?;
            }
        }
    }
    save_history(&hist_file, &hist)?;
    Ok(())
}
