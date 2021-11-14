use std::error::Error;
use std::fs;

pub fn run<'a>(query: &str, filename: &'a str) -> Result<String, Box<dyn Error>> {
    let contents = fs::read_to_string(filename)?;

    Ok(
        contents.lines()
            .filter(|line| line.contains(query))
            .collect::<Vec<&str>>()
            .join("\n"))
}

pub fn parse(args: &Vec<String>) -> Result<(&str, &str), &str> {
    if args.len() < 3 {
        return Err("Expect two arguments, search string and file path.")
    }
    let query = &args[1];
    let filename = &args[2];

    Ok((query, filename))
}

