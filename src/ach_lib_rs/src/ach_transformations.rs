use crate::ach_file::{AchFile, AchRecord, AchRecordType, Field, Header};
use log::error;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, ErrorKind};
use std::path::Path;
use std::str::Split;

#[derive(Debug)]
pub struct Transformations {
    base_ach_file: AchFile,
    transformations: Vec<Transformation>,
}

impl Transformations {
    fn with_ach_file(mut self, base_ach_file: AchFile) {
        self.base_ach_file = base_ach_file;
    }
}

impl TryFrom<&Path> for Transformations {
    type Error = io::Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let lines: Vec<String> = BufReader::new(match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                error!(
                    "Could not open transform file: {} \n Path: {}",
                    e,
                    path.to_str().unwrap()
                );
                return Err(e);
            }
        })
        .lines()
        .filter(|l: &io::Result<String>| {
            let l = l.as_ref().to_owned().unwrap().trim();
            !(l.starts_with("#") || l.len() == 0)
        })
        .map(|l| l.unwrap())
        .collect();

        Ok(Transformations::try_from(lines)?)
    }
}

impl TryFrom<Vec<String>> for Transformations {
    type Error = io::Error;

    fn try_from(mut lines: Vec<String>) -> Result<Self, Self::Error> {
        // first line should be a YAML beginning of file tag, but we should ignore it.
        if lines[0] == "---" {
            lines.remove(0);
        }

        let base_indent_size = lines[0].len() - lines[0].trim().len();
        let mut transformation_lines = vec![];
        let mut transformations = vec![];

        for line in lines {
            let indent_size = line.len() - line.trim().len();
            if indent_size <= base_indent_size && transformation_lines.len() > 0 {
                transformations.push(Transformation::try_from(transformation_lines.to_owned())?);
                transformation_lines.clear();
            }
            transformation_lines.push(line.trim().to_string());
        }
        Ok(Transformations {
            base_ach_file: Default::default(),
            transformations,
        })
    }
}

#[derive(Debug)]
struct Transformation {
    label: String,
    operation: Vec<Operation>,
    on: Vec<AchRecordType>,
    conditions: Vec<Condition>,
    replacments: Vec<Replacement>,
}

impl TryFrom<Vec<String>> for Transformation {
    type Error = io::Error;
    fn try_from(mut lines: Vec<String>) -> Result<Self, Self::Error> {
        // first line should be the label, always
        let mut transformation = Transformation {
            label: lines[0].trim().trim_end_matches(":").to_string(),
            operation: vec![],
            on: vec![],
            conditions: vec![],
            replacments: vec![],
        };
        lines.remove(0);

        let mut broken_line_content = "".to_string();
        for line in lines {
            let mut tmp_line = "".to_string();
            if line.trim_end().ends_with(",") {
                broken_line_content = format!("{} {}", broken_line_content, line.trim_end());
                continue;
            } else if !broken_line_content.is_empty() {
                tmp_line.push_str(&*broken_line_content);
                tmp_line.push_str(&*line);
            }
            let line = if !tmp_line.is_empty() { tmp_line } else { line };
            println!("{}", line);

            let line_data = line.split(":").map(|s| s.trim()).collect::<Vec<&str>>();
            if line_data.len() > 2 {
                error!(
                    "Malformed config file! Error on following line: \n {}",
                    line
                );
                return Err(Self::Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Malformed config file! Error on following line: \n {}",
                        line
                    ),
                ));
            }
            match line_data[0] {
                "operation" => {
                    let ops = Operation::new_vec(line_data[1]);
                    let mut unwrapped_ops: Vec<Operation> = vec![];
                    for op in ops {
                        // Return any errors if they exist so we map unwrap() over the members without worrying about panics.
                        unwrapped_ops.push(op?);
                    }

                    transformation.operation.append(&mut unwrapped_ops)
                }
                "on" => transformation.on.append(&mut {
                    let mut st = line_data[1].clone();
                    parse_config_value(st, |s| AchRecordType::from(s.trim()))
                }),
                _ => {
                    error!("unknown key '{}'", line_data[0]);
                    return Err(Self::Error::new(
                        ErrorKind::InvalidData,
                        format!("unknown key '{}'", line_data[0]),
                    ));
                }
            };
            println!("{:?}", transformation)
        }

        Ok(transformation)
    }
}

#[derive(Debug)]
enum Operation {
    SPLIT,
    REPLACE,
}

impl Operation {
    fn new_vec(s: &str) -> Vec<io::Result<Self>> {
        parse_config_value(s, |st: &str| match st.trim() {
            "split" => Ok(Operation::SPLIT),
            "replace" => Ok(Operation::REPLACE),
            _ => {
                error!("Unknown operation: {}", st);
                Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("Unknown operation: {}", st),
                ))
            }
        })
    }
}

trait Conditions: std::fmt::Debug {}

#[derive(Debug)]
struct Condition {
    condition: Box<dyn Conditions>,
    result: bool,
}

#[derive(Debug)]
struct FieldCondition {
    field: Field,
}
impl Conditions for FieldCondition {}

#[derive(Debug)]
struct FieldArrayCondition {
    field_list: Vec<Field>,
}
impl Conditions for FieldArrayCondition {}

#[derive(Debug)]
enum Conjunction {
    AND,
    OR,
}

#[derive(Debug)]
struct ConjunctionCondition {
    conjunction: Conjunction,
    conditions: Vec<Box<dyn Conditions>>,
}
impl Conditions for ConjunctionCondition {}

#[derive(Debug)]
struct NotCondition<'a> {
    condition: &'a dyn Conditions,
}
impl Conditions for NotCondition<'_> {}

#[derive(Debug)]
struct Replacement {
    record: AchRecordType,
    replace_with: Field,
}

fn parse_config_value<T, F>(input_string: &str, map_closure: F) -> Vec<T>
    where
        F: Fn(&str) -> T,
{
    match input_string.find("[") {
                Some(_) => input_string
                    .trim_matches(|c| c == '[' || c == ']')
                    .split(',')
                    .map(map_closure)
                    .collect::<Vec<T>>(),
                None => vec![map_closure(input_string)],
            }
    
    
}
