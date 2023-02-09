use std::error::Error;
use std::io;
use std::io::{BufRead, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fmt, fs};

use lazy_static::lazy_static;
use log::info;
use regex::Regex;

pub fn preprocess(filepath: &Path) -> Result<String, PreprocessorError> {
    info!("Running preprocessor");

    let (mut file_contents, includes) = remove_include_directives(filepath)?;

    info!("Includes: {:?}", includes);

    file_contents = include_headers(file_contents, includes, filepath)?;

    info!("Included headers:\n{file_contents}");

    let processed_source = match run_c_preprocessor(file_contents) {
        Ok(s) => s,
        Err(e) => return Err(PreprocessorError::CppError(e)),
    };

    info!("Preprocessor output:\n{processed_source}");

    Ok(processed_source)
}

fn remove_include_directives(filepath: &Path) -> Result<(String, Vec<String>), PreprocessorError> {
    // store the includes we remove from the file
    let mut includes: Vec<String> = Vec::new();
    let mut output = String::new();

    match read_lines(filepath) {
        Ok(lines) => {
            for line in lines {
                if let Ok(l) = line {
                    // check if line is a #include directive
                    if let Some(include) = check_for_include(&l) {
                        includes.push(include);
                        continue;
                    }
                    output.push_str(&l);
                    output.push('\n');
                }
            }
            Ok((output, includes))
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Err(PreprocessorError::FileNotFound(
                filepath.to_str().unwrap().to_owned(),
            )),
            _ => Err(PreprocessorError::IoError(e)),
        },
    }
}

/// Checks if the given line is a #include directive.
/// If so, returns the name of the header file included, or None otherwise
fn check_for_include(line: &String) -> Option<String> {
    lazy_static! {
        static ref INCLUDE_RE: Regex = Regex::new(r#"^#include\s[<"](.*)[>"]"#).unwrap();
    }

    if let Some(captures) = INCLUDE_RE.captures(line) {
        // store what the include was that we're removing
        if let Some(captured_include) = captures.get(1) {
            return Some(captured_include.as_str().to_owned());
        }
    }

    None
}

/// Returns an iterator over the lines of the file
fn read_lines(filepath: &Path) -> io::Result<io::Lines<io::BufReader<fs::File>>> {
    let file = fs::File::open(filepath)?;
    Ok(io::BufReader::new(file).lines())
}

fn include_headers(
    mut source: String,
    mut includes: Vec<String>,
    filepath: &Path,
) -> Result<String, PreprocessorError> {
    loop {
        if includes.is_empty() {
            break;
        }
        let include_name = includes.pop().unwrap();
        let header =
            if include_name == format!("{}.h", filepath.file_stem().unwrap().to_str().unwrap()) {
                match load_program_header(filepath, &include_name) {
                    Err(e) => {
                        return match e {
                            PreprocessorError::FileNotFound(f) => {
                                Err(PreprocessorError::UnsupportedHeaderInclude(f))
                            }
                            _ => Err(e),
                        }
                    }
                    Ok(s) => s,
                }
            } else {
                match load_header_file(&include_name) {
                    Err(e) => {
                        return match e {
                            PreprocessorError::FileNotFound(f) => {
                                Err(PreprocessorError::UnsupportedHeaderInclude(f))
                            }
                            _ => Err(e),
                        }
                    }
                    Ok(s) => s,
                }
            };
        let (header_source, mut new_includes) = header;
        info!("New includes: {:?}", new_includes);
        includes.append(&mut new_includes);
        source.insert(0, '\n');
        source.insert_str(0, header_source.as_str())
    }
    Ok(source)
}

fn load_program_header(
    source_filepath: &Path,
    header_name: &String,
) -> Result<(String, Vec<String>), PreprocessorError> {
    let path = match source_filepath.parent() {
        Some(p) => {
            let mut path = PathBuf::from(p);
            path.push(header_name);
            path
        }
        None => PathBuf::from(&header_name),
    };
    remove_include_directives(path.as_path())
}

fn load_header_file(header_name: &String) -> Result<(String, Vec<String>), PreprocessorError> {
    let mut path = PathBuf::from("headers");
    path.push(header_name);
    remove_include_directives(path.as_path())
}

/// Runs the C preprocessor over the given source.
fn run_c_preprocessor(source: String) -> Result<String, Box<dyn Error>> {
    let mut cpp_child = Command::new("cpp")
        // capture input, output, and error
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // args to cpp
        .arg("-") // process from stdin
        .arg("-") // output to stdout
        .arg("-P") // don't include line marker information
        .spawn()?;

    // write the contents of source to stdin for cpp
    cpp_child
        .stdin
        .as_mut()
        .ok_or("Child process stdin hasn't been captured")?
        .write_all(source.as_bytes())?;

    // capture the output from cpp
    let output = cpp_child.wait_with_output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        // terminate the program if the preprocessor can't parse the file
        let err = String::from_utf8(output.stderr)?;
        panic!("C preprocessor failed:\n{err}");
    }
}

#[derive(Debug)]
pub enum PreprocessorError {
    FileNotFound(String),
    UnsupportedHeaderInclude(String),
    IoError(io::Error),
    CppError(Box<dyn Error>),
}

impl fmt::Display for PreprocessorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PreprocessorError::UnsupportedHeaderInclude(h) => {
                write!(f, "Header \"{h}\" not supported")
            }
            PreprocessorError::IoError(e) => {
                write!(f, "IO Error occurred during preprocessor: {e}")
            }
            PreprocessorError::CppError(e) => {
                write!(f, "Error occurred running C preprocessor: {e}")
            }
            PreprocessorError::FileNotFound(n) => {
                write!(f, "File not found: {n}")
            }
        }
    }
}

impl Error for PreprocessorError {}
