use std::io;
use std::collections::HashMap;
use serde_json;
use serde_json::{Map, Value};

type Columns = Vec<String>; // consider a parallel Set to make contains cheaper
type Row = HashMap<String, String>;

fn ensure(columns: &mut Columns, name: &str) {
    let name = name.to_string();
    if !columns.contains(&name) {
        columns.push(name);
    }
}

fn singleton(path: &str, value: String) -> Vec<Row> {
    let mut row = HashMap::new();
    row.insert(path.to_string(), value);
    let mut ret = Vec::new();
    ret.push(row);
    ret
}

fn recurse_array(columns: &mut Columns, path: &str, array: Vec<Value>) -> Vec<Row> {
    array.iter().fold(Vec::new(), |mut acc, val| {
        acc.append(&mut recurse_value(columns, path, val.clone()));
        acc
    })
}

fn recurse_map(columns: &mut Columns, path: &str, map: Map<String, Value>) -> Vec<Row> {
    map.iter().fold(Vec::new(), |acc, (key, value)| {
        let path_key = format!("{}.{}", path, key);
        let rhs = recurse_value(columns, &path_key, value.clone());
        let mut ret = Vec::new();
        // merge each possible pair of partial rows
        for a in acc {
            for b in &rhs {
                let mut r = a.clone();
                r.extend(b.clone());
                ret.push(r);
            }
        }
        ret
    })
}

fn recurse_value(columns: &mut Columns, path: &str, value: Value) -> Vec<Row> {
    match value {
        Value::Null => Vec::new(),
        Value::String(s) => {
            ensure(columns, path);
            singleton(path, s)
        },
        Value::Bool(b) => {
            ensure(columns, path);
            singleton(path, format!("{}", b))
        },
        Value::Number(n) => {
            ensure(columns, path);
            singleton(path, format!("{}", n))
        },
        Value::Array(arr) => recurse_array(columns, path, arr),
        Value::Object(map) => recurse_map(columns, path, map),
    }
}

fn main() -> io::Result<()> {
    // parse args
    // parse stdin as json
    let json: Value = serde_json::from_reader(io::stdin())?;

    // recurse into json, building columns & rows as we go
    let mut columns: Columns = Vec::new();
    // TODO validate that outer is an array?
    let rows = recurse_value(&mut columns, "", json);

    // output
    println!("{}", columns.join(" "));
    for r in rows.iter() {
        let mut out = Vec::new();
        for c in columns.iter() {
            if let Some(s) = r.get(c) {
                out.push(s.clone())
            }
        }
        println!("{}", out.join(" "));
    }
    Ok(())
}
