//! Read in a YAML file and output a JSON file
extern crate docopt;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;
extern crate walkdir;

use std::{
    fs::{read_to_string, write},
    path::{PathBuf},
};

use docopt::{Docopt};

const HELP: &str = r#"
y2j (yaml to json) is a utility for converting yaml files into json files

Usage:
    y2j -f | --file <inpath> <outpath>
    y2j -d | --dir <inpath> <outpath>
    y2j -h | --help
    y2j -v | --version

Options:
    -h, --help     Print this message
    -v, --version  Print the current version
    -f, --file     Convert a single file
    -d, --dir      Convert all .yaml or .yml files in a directory
"#;
#[derive(Deserialize)]
struct Opts {
    pub flag_file: bool,
    pub flag_dir: bool,
    pub arg_inpath: PathBuf,
    pub arg_outpath: PathBuf,
}

fn main() {
    let args: Opts = Docopt::new(HELP)
                .and_then(|d| d.deserialize())
                .unwrap_or_else(|e| e.exit());
    let res = if args.flag_file {
        convert(&args.arg_inpath, &args.arg_outpath)
    } else if args.flag_dir {
        convert_dir(&args.arg_inpath, &args.arg_outpath)
    } else { 
        eprintln!("Error, you must use either the -f or -d flag when running");
        println!("{}", HELP);
        ::std::process::exit(1);
    };
    match res {
        Ok(_) => {
            println!("Successfully converted your files!")
        },
        Err(e) => {
            eprintln!("Error converting your files {:?}", e);
            println!("{}", HELP);
        }
    }
}

fn convert_dir(from_path: &PathBuf, to_path: &PathBuf) -> Result<(), Error> {
    println!("Converting the files from {} to {}", from_path.display(), to_path.display());
    for e in walkdir::WalkDir::new(&from_path).max_depth(1).min_depth(1) {
        println!("entry: {:?}", e);
        if let Ok(entry) = e {
            if entry.file_type().is_file() {
                let file_name = entry.file_name().to_string_lossy();
                if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
                    let mut target = entry.path().to_path_buf();
                    target.set_extension("json");
                    let new_name = target.file_name().ok_or(Error::Io("Failed to create an outfile with a .json extension".into()))?;
                    convert(&entry.path().to_path_buf(), &to_path.join(&new_name))?;
                }
            }
        }
    }
    Ok(())
}

fn convert(from_path: &PathBuf, to_path: &PathBuf) -> Result<(), Error> {
    println!("converting from {} to {}", &from_path.display(), &to_path.display());
    if !from_path.exists() {
        return Err(Error::Io(format!("infile does not exist\n{}", from_path.display())))
    }
    let to_dir = to_path.parent().ok_or(Error::Io("outfile doesn't have a parent".into()))?;
    if !to_dir.exists() {
        return Err(Error::Io(format!("outfile directory does not exists\n{}", to_path.display())))
    }
    let content = read_to_string(from_path)?;
    let notes = Notes::from_yaml(&content)?;
    let json = notes.to_json()?;
    write(to_path, &json)?;
    Ok(())
}
#[derive(Debug)]
enum Error {
    SerError(String),
    DeError(String),
    Io(String),
}

impl From<serde_yaml::Error> for Error {
    fn from(other: serde_yaml::Error) -> Self {
        Error::DeError(format!("Deserialization Error: {:?}", other))
    }
}

impl From<serde_json::Error> for Error {
    fn from(other: serde_json::Error) -> Self {
        Error::SerError(format!("Serialization Error: {:?}", other))
    }
}

impl From<::std::io::Error> for Error {
    fn from(other: ::std::io::Error) -> Self {
        Error::Io(format!("I/O Error: {:?}", other))
    }
}

impl From<walkdir::Error> for Error {
    fn from(other: walkdir::Error) -> Self {
        Error::Io(format!("I/O Error: {:?}", other))
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::SerError(msg) => msg,
            Error::DeError(msg) => msg,
            Error::Io(msg) => msg,
        }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let msg = match self {
            Error::SerError(msg) => msg,
            Error::DeError(msg) => msg,
            Error::Io(msg) => msg,
        };
        msg.fmt(f)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Notes {
    title: String,
    notes: Option<Vec<Notes>>
}

impl Notes {
    pub fn from_yaml(yaml: &str) -> Result<Notes, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}