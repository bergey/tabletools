use clap::{Parser, ValueEnum};
use std::collections::HashMap;
use std::process;
use std::io;
use std::io::BufRead;
use std::fmt;

#[derive(Debug, Clone, ValueEnum)]
enum SplitWhitespace {
    Any,
    Double,
    Ignore,
}

impl fmt::Display for SplitWhitespace {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use SplitWhitespace::*;
        match self {
            Any => write!(out, "{}", "any"),
            Double => write!(out, "{}", "double"),
            Ignore => write!(out, "{}", "ignore"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command()]
    output_columns: Vec<String>,
    #[arg(long, short, help="case insensitive match for column names")]
    insensitive: bool,
    #[arg(long, short, help="additional column delimiters", default_value="")]
    delimiters: String,
    #[arg(long, short, help="whitespace delimited?", default_value_t=SplitWhitespace::Any)]
    whitespace: SplitWhitespace,
    #[arg(long, short, help="count +-| and other border drawing characters as delimiters")]
    border: bool,
    #[arg(long="output", short='O', help="output delimiter (default ,)")]
    output_delimiter: Option<String>,
    #[arg(long, help="ascii unit separator character (overrides output delimiter)")]
    unit_separator: bool,
    #[arg(long, help="line delimiter (default newline)")]
    line_delimiter: Option<String>,
    #[arg(long, help="ascii record separator character (overrides line delimiter)")]
    record_separator: bool,
    #[arg(short='0', help="null (overrides line delimiter)")]
    null_end_line: bool,
    #[arg(long, short='H', help="pick columns from first row only")]
    header: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Cli {
            output_columns: Vec::new(),
            insensitive: false,
            header: false,
            delimiters: "".to_string(),
            whitespace: SplitWhitespace::Any,
            border: false,
            output_delimiter: None,
            unit_separator: false,
            line_delimiter: None,
            record_separator: false,
            null_end_line: false,
        }
    }
}

impl Cli {
    fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.unit_separator && self.output_delimiter.is_some() {
            errors.push("--unit-separator is incompatible with --output".to_string());
        }
        if self.null_end_line && self.line_delimiter.is_some() {
            errors.push("--line-delimiter is incompatible with -0".to_string())
        }
        if self.record_separator && self.line_delimiter.is_some() {
            errors.push("--line-delimiter is incompatible with --record-separator".to_string())
        }
        if self.record_separator && self.null_end_line {
            errors.push("--record-separator is incompatible with -0".to_string())
        }
       errors
    }

    fn computed_output_delimiter(&self) -> String {
        // by now we know that there are no conflicting options, so order doesn't matter
        if self.unit_separator {
            "\x1F".to_string()
        } else if let Some(s) = &self.output_delimiter {
            s.clone()
        } else {
            ",".to_string()
        }
    }

    fn computed_line_delimiter(&self) -> String {
        if self.null_end_line {
            "\0".to_string()
        } else if self.record_separator {
            "\x1E".to_string()
        } else if let Some(s) = &self.line_delimiter {
            s.clone()
        } else {
            "\n".to_string()
        }
    }
}

fn some_whitespace(c: Option<char>) -> bool {
    match c {
        Some(c) => c.is_whitespace(),
        None => false,
    }
}

fn is_delimiter(args: &Cli, before: Option<char>, c: char, after: Option<char>) -> bool {
    let matches_whitespace = match args.whitespace {
        SplitWhitespace::Any => c.is_whitespace(),
        SplitWhitespace::Ignore => false,
        SplitWhitespace::Double => c.is_whitespace() && (some_whitespace(before) || some_whitespace(after))
    };
    let matches_delimiters = args.delimiters.contains(c);
    matches_whitespace || matches_delimiters
}

fn update_spaces(args: &Cli, mut spaces: Vec<bool>, string: &String) -> Vec<bool> {
    let chars: Vec<char> = string.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        let before = if i > 0 { Some(chars[i-1]) } else { None };
        let after = if i + 1 < chars.len() { Some(chars[i+1])} else { None };
        match spaces.get_mut(i) {
            Some(space) => *space = *space && is_delimiter(args, before, *c, after),
            None => spaces.push(is_delimiter(args, before, *c, after)),
        }
    }
    spaces
}

fn columns(spaces: &Vec<bool>) -> Vec<(usize, usize)> {
    let mut ret = Vec::new();
    let mut start: Option<usize> = None;
    for (i, space) in spaces.iter().enumerate() {
        match (start, space) {
            (None,false) => {
                start = Some(i);
            },
            (Some(pos), true) => {
                ret.push((pos, i));
                start = None;
            }
            (_, _) => (),
        }
    }
    if let Some(pos) = start {
        ret.push((pos, spaces.len()));
    }
    ret
}

fn split_line(columns: &[(usize, usize)], line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let line: Vec<char> = line.chars().collect();
    for (s, e) in columns {
        if *s >= line.len() { continue; }
        let e_ = (*e).min(line.len());
        out.push(line[*s..e_].iter().collect::<String>().trim().to_string());
    }
    out
}

fn output_columns(columns: &[(usize, usize)], header: &str, desired: &[String], insensitive: bool) -> Vec<(usize, usize)>{
    if desired.len() == 0 {
        return columns.to_vec();
    }

    let headings = split_line(columns, header);
    let mut mapping = HashMap::new();
    for (head, range) in std::iter::zip(headings, columns) {
        if insensitive {
            mapping.insert(head.to_lowercase(), range);
        } else {
            mapping.insert(head, range);
        }
    }

    let mut ret = Vec::new();
    for head in desired {
        let o_range = if insensitive {
            let h = head.to_lowercase();
            mapping.get(&h)
        } else {
            mapping.get(head)
        };
        match o_range {
            Some(range) => ret.push(**range),
            None => (), // TODO error?
        }
    }
    ret
}

fn main() -> io::Result<()> {
    let mut args = Cli::parse();
    if args.border { args.delimiters.push_str("+-|│"); }
    let validation_errors = args.validate();
    if validation_errors.len() > 0 {
        for e in validation_errors {
            eprintln!("{}", e);
        }
        process::exit(1);
    }

    let stdin = io::stdin();
    let in_handle = stdin.lock();

    let lines = in_handle.lines().collect::<io::Result<Vec<String>>>()?;
    if lines.len() == 0 {
        process::exit(2);
    }
    let spaces = if args.header {
        update_spaces(&args, Vec::new(), &lines[0])
    } else {
        lines.iter().fold(Vec::new(), |spaces, string| { update_spaces(&args, spaces, string) })
    };
    let columns = columns(&spaces);
    let output_columns = output_columns(&columns, &lines[0], args.output_columns.as_ref(), args.insensitive);

    for string in lines {
        let outln = split_line(&output_columns, &string);
        print!("{}{}", &outln.join(&args.computed_output_delimiter()), args.computed_line_delimiter());
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn update_spaces_first_line() {
        let mut line = String::new();
        line.push_str("  a bb  ccc ");
        let spaces = update_spaces(&Default::default(), Vec::new(), &line);
        assert_eq!(spaces, vec![true, true, false, true, false, false, true, true, false, false, false, true])
    }

    #[test]
    fn update_spaces_two_lines() {
        let args = Default::default();
        let mut line = String::new();
        line.push_str("  a bb  ccc ");
        let spaces = update_spaces(&args, Vec::new(), &line);
        line.clear();
        line.push_str(" aa  b ccc  ");
        let spaces = update_spaces(&args, spaces, &line);
        assert_eq!(spaces, vec![true, false, false, true, false, false, true, false, false, false, false, true])
    }

    #[test]
    fn update_spaces_comma() {
        let mut args: Cli = Default::default();
        args.delimiters.push_str(",");
        let mut line = String::new();
        line.push_str(",,a,bb,,ccc,");
        let spaces = update_spaces(&args, Vec::new(), &line);
        assert_eq!(spaces, vec![true, true, false, true, false, false, true, true, false, false, false, true])
    }

    #[test]
    fn update_spaces_mixed_delims() {
        let mut args: Cli = Default::default();
        args.delimiters.push_str(",");
        let mut line = String::new();
        line.push_str(", a,bb, ccc,");
        let spaces = update_spaces(&args, Vec::new(), &line);
        assert_eq!(spaces, vec![true, true, false, true, false, false, true, true, false, false, false, true])
    }

    #[test]
    fn update_spaces_double() {
        let mut args: Cli = Default::default();
        args.whitespace = SplitWhitespace::Double;
        let mut line = String::new();
        line.push_str(" a  b b   c c ");
        let spaces = update_spaces(&args, Vec::new(), &line);
        assert_eq!(spaces, vec![false, false, true, true, false, false, false, true, true, true, false, false, false, false])
    }

    #[test]
    fn update_spaces_double_ends() {
        let mut args: Cli = Default::default();
        args.whitespace = SplitWhitespace::Double;
        let mut line = String::new();
        line.push_str("  a  b b   c c  ");
        let spaces = update_spaces(&args, Vec::new(), &line);
        assert_eq!(spaces, vec![true, true, false, true, true, false, false, false, true, true, true, false, false, false, true, true])
    }

    #[test]
    fn update_spaces_double_starts() {
        let mut args: Cli = Default::default();
        args.whitespace = SplitWhitespace::Double;
        let mut line = String::new();
        line.push_str("a  b b   c c");
        let spaces = update_spaces(&args, Vec::new(), &line);
        assert_eq!(spaces, vec![false, true, true, false, false, false, true, true, true, false, false, false])
    }

    #[test]
    fn columns_first_line() {
        let runs = columns(&vec![true, true, false, true, false, false, true, true, false, false, false, true]);
        assert_eq!(runs, vec![(2, 3), (4, 6), (8, 11)]);
    }

    #[test]
    fn columns_two_lines() {
        let runs = columns(&vec![true, false, false, true, false, false, true, false, false, false, false, true]);
        assert_eq!(runs, vec![(1, 3), (4, 6), (7, 11)]);
    }

    #[test]
    fn columns_no_trailing_whitespace() {
        let runs = columns(&vec![true, false, false, true, false, false, true, false, false, false, false]);
        assert_eq!(runs, vec![(1, 3), (4, 6), (7, 11)]);
    }
}
