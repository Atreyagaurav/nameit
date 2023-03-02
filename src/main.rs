use directories::ProjectDirs;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

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
    for h in &mut *vec {
        println!("[{}] {}", i, h);
        i += 1;
    }
    if !manual {
        print!("Choose {}: ", prompt);
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut buf)?;
        choice = buf.trim().parse()?;
        if choice == 0 {
            manual = true;
        } else {
            choice -= 1;
        }
    }
    if manual {
        print!("Input {}: ", prompt);
        std::io::stdout().flush()?;
        buf.clear();
        std::io::stdin().read_line(&mut buf)?;
        vec.push(buf.trim().to_string());
        choice = vec.len() - 1;
    }
    Ok(vec[choice].clone())
}

fn main() -> Result<(), Box<dyn Error>> {
    let hist_file = ProjectDirs::from(
        "org",       /*qualifier*/
        "ZeroSofts", /*organization*/
        "nameit",    /*application*/
    )
    .unwrap()
    .data_dir()
    .join("histories.json");
    let mut hist = read_history(&hist_file)?;
    let fmt_str = choose("Format", &mut hist.formats)?;
    let vars: Vec<String> = fmt_str
        .split("_")
        .map(|v| match hist.values.get_mut(v) {
            Some(mut k) => choose(v, &mut k),
            None => {
                hist.variables.insert(v.to_string());
                let mut newvec = vec![];
                let var = choose(v, &mut newvec);
                hist.values.insert(v.to_string(), newvec);
                var
            }
        })
        .collect::<Result<Vec<String>, Box<dyn Error>>>()?;
    let filename = vars.join("_").replace(" ", "-");
    println!("{}", filename);
    save_history(&hist_file, &hist)?;
    Ok(())
}
