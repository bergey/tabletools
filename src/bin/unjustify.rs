use clap::{Parser, ValueEnum};
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
    #[arg(long, short, help="additional column delimiters", default_value="")]
    delimiters: String,
    #[arg(long, short, help="whitespace delimited?", default_value_t=SplitWhitespace::Any)]
    whitespace: SplitWhitespace,
    #[arg(long, short='H', help="pick columns from first row only")]
    header: bool,
    #[arg(long, short, help="count +-| and other border drawing characters as delimiters")]
    border: bool,
    #[arg(long="output", short='O', help="output delimiter", default_value=",")]
    output_delimiter: String,
}

impl Default for Cli {
    fn default() -> Self {
        Cli {
            delimiters: "".to_string(),
            whitespace: SplitWhitespace::Any,
            header: false,
            border: false,
            output_delimiter: ",".to_string(),
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
    ret
}

fn main() -> io::Result<()> {
    let mut args = Cli::parse();
    if args.border { args.delimiters.push_str("+-|│"); }
    
    let stdin = io::stdin();
    let in_handle = stdin.lock();

    let lines = in_handle.lines().collect::<io::Result<Vec<String>>>()?;
    let spaces = if args.header && lines.len() >= 1 {
        update_spaces(&args, Vec::new(), &lines[0])
    } else {
        lines.iter().fold(Vec::new(), |spaces, string| { update_spaces(&args, spaces, string) })
    };
    let columns = columns(&spaces);

    let mut outln: Vec<String> = Vec::new();
    for string in lines {
        let line: Vec<char> = string.chars().collect();
        for (s, e) in &columns {
            if *s >= line.len() { continue; }
            let e_ = (*e).min(line.len());
            outln.push(line[*s..e_].iter().collect::<String>().trim().to_string());
        }
        println!("{}", &outln.join(&args.output_delimiter));
        outln.clear();
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
}
