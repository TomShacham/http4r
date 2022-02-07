use std::fs;
use std::fs::File;
use std::io::Write;

pub fn approve(actual: String, expected_file: &str) {
    let str = fs::read_to_string(expected_file);
    make_approval_file(&actual, expected_file);
    if str.unwrap() != actual.as_str() {
        panic!("Expected file not the same: {}", expected_file);
    }
}

fn make_approval_file(actual: &String, expected_file: &str) {
    let split_on_slash = expected_file.split("/").map(|it| it.to_string()).collect::<Vec<String>>();
    let iter = split_on_slash.iter();
    let up_to_file_name = iter.clone().take(split_on_slash.len() - 1).map(|it| it.to_string()).collect::<Vec<String>>().join("/");
    let file_name = iter.clone().last();
    let name_and_extension = file_name.map(|name| {
        let split = name.to_string().split(".").map(|x| x.to_string()).collect::<Vec<String>>();
        (split.get(0).unwrap().to_owned(), split.get(1).unwrap().to_owned())
    }).unwrap_or(("unknown_file_name".to_string(), "unknown_extension".to_string()));
    let approval_output_file_name = name_and_extension.0 + "_approval" + "." + name_and_extension.1.as_str();
    let create = File::create(up_to_file_name.clone() + "/" + approval_output_file_name.clone().as_str());
    if create.is_err() {
        panic!("{}", format!("Failed to create approval file named: {}", up_to_file_name.clone() + "/" + approval_output_file_name.clone().as_str()));
    }
    create.unwrap().write_all(actual.as_bytes());
}