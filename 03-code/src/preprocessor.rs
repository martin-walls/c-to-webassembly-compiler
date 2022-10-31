use std::fs;
use std::error::Error;
use std::io;
use std::io::{BufRead, Write};
use regex::Regex;
use lazy_static::lazy_static;
use std::process::{Command, Stdio};

pub fn preprocess(filepath: &String) -> Result<String, Box<dyn Error>> {
  println!("\n-- Running preprocessor --");

  let (file_contents, includes) = remove_include_directives(&filepath)?;

  let processed_source = run_c_preprocessor(file_contents)?;

  println!("Preprocessor output:");
  println!("{processed_source}");

  println!("Includes: {:?}", includes);

  Ok(processed_source)
}

fn remove_include_directives(filepath: &String) -> Result<(String, Vec<String>), Box<dyn Error>> {
  // store the includes we remove from the file
  let mut includes: Vec<String> = Vec::new();
  let mut output = String::new();

  if let Ok(lines) = read_lines(&filepath) {
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
  }

  Ok((output, includes))
}

/// Checks if the given line is a #include directive.
/// If so, returns the name of the header file included, or None otherwise
fn check_for_include(line: &String) -> Option<String> {
  lazy_static! {
    static ref INCLUDE_RE: Regex = Regex::new(r#"#include\s[<"](.*)[>"]"#).unwrap();
  }

  if let Some(captures) = INCLUDE_RE.captures(&line) {
    // store what the include was that we're removing
    if let Some(captured_include) = captures.get(1) {
      return Some(captured_include.as_str().to_owned());
    }
  }

  None
}

/// Returns an iterator over the lines of the file
fn read_lines(filepath: &String) -> io::Result<io::Lines<io::BufReader<fs::File>>> {
  let file = fs::File::open(filepath)?;
  Ok(io::BufReader::new(file).lines())
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
  cpp_child.stdin
    .as_mut()
    .ok_or("Child process stdin hasn't been captured")?
    .write_all(&source.as_bytes())?;

  // capture the output from cpp
  let output = cpp_child.wait_with_output()?;

  if output.status.success() {
    Ok(String::from_utf8(output.stdout)?)
  } else {
    // terminate the program if the preprocessor can't parse the file
    let err = String::from_utf8(output.stderr)?;
    panic!("C preprocessor failed:\n{}", err);
  }
}