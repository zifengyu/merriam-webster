use colored::*;
use itertools::Itertools;
use serde_json::Value;

fn print_style_text(text: &str, bold: bool, italic: bool, underline: bool) {
    let mut st: ColoredString = text.into();
    if bold {
        st = st.bold();
    }
    if italic {
        st = st.italic();
    }
    if underline {
        st = st.underline();
    }
    print!("{}", st);
}

fn print_running_text(mut text: &str) {
    let mut bold = false;
    let mut italic = false;
    while text.len() > 0 {
        match text.find('{') {
            Some(idx) => {
                print_style_text(text.get(0..idx).unwrap(), bold, italic, false);
                text = text.get(idx..).unwrap();
                match text.find('}') {
                    Some(ridx) => {
                        let token = text.get(1..ridx).unwrap();
                        match token {
                            "bc" => {
                                print!("{}", ": ".bold());
                            }
                            "b" => {
                                bold = true;
                            }
                            "\\/b" => {
                                bold = false;
                            }
                            "it" => {
                                italic = true;
                            }
                            "\\/it" => {
                                italic = false;
                            }
                            _ => {
                                if token.starts_with("a_link|") {
                                    if let Some(link_text) = token.split('|').nth(1) {
                                        print_style_text(link_text, bold, italic, true);
                                    }
                                } else if token.starts_with("sx|") {
                                    if let Some(link_text) = token.split('|').nth(1) {
                                        print_style_text(
                                            &link_text.to_uppercase(),
                                            bold,
                                            italic,
                                            true,
                                        );
                                    }
                                } else {
                                    print!("{token}");
                                }
                            }
                        }
                        text = text.get(ridx + 1..).unwrap();
                    }
                    None => {
                        print_style_text(text, bold, italic, false);
                        return;
                    }
                }
            }
            None => {
                print_style_text(text, bold, italic, false);
                return;
            }
        }
    }
}

fn print_dt_element(elem: &Value) {
    if let Value::Array(elem_arr) = elem {
        match &elem_arr[0] {
            Value::String(s) => match s.as_str() {
                "text" => {
                    if let Value::String(content) = &elem[1] {
                        print_running_text(content);
                        println!();
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }
}

fn print_sense(sense: &Value) {
    if let Value::String(sn) = &sense["sn"] {
        let ident = match sn.chars().next() {
            Some('(') => 4,
            Some(c) => if c.is_alphabetic() { 2 } else { 0 },
            _ => 0, 
        };
        for _ in 0..ident {
            print!(" ");
        }
        print!("{} ", sn.bold().yellow());
    }

    if let Value::Array(sls) = &sense["sls"] {
        let sls_text = sls
            .iter()
            .map(|v| match v {
                Value::String(s) => s,
                _ => "",
            })
            .join(",");
        print!("{} ", sls_text.on_bright_black());
    }

    let dt = match &sense["dt"] {
        Value::Array(dt) => dt,
        _ => return,
    };

    for elem in dt.iter() {
        print_dt_element(elem);
    }
}

fn print_dt_array(arr: &Value) {
    if let Value::String(t) = &arr[0] {
        match t.as_str() {
            "pseq" => print_dt_array(&arr[1]),
            "sense" => print_sense(&arr[1]),
            "bs" => print_sense(&arr[1]["sense"]),
            _ => (),
        }
        return;
    }

    if let Value::Array(arrays) = &arr {
        for a in arrays.iter() {
            print_dt_array(a);
        }
    }
}

fn print_def(def: &Value) {
    if let Value::Object(def) = def {
        match def.get("vd") {
            Some(Value::String(s)) => println!("{}", s.italic().bright_blue()),
            _ => (),
        };
        print_dt_array(&def["sseq"]);
    }
}

fn print_entry(entry: &Value) {
    let head_word = match &entry["hwi"]["hw"] {
        Value::String(s) => s,
        _ => return,
    };

    let head_word = &head_word.replace("*", "\u{2022}");

    let functional_label = match &entry["fl"] {
        Value::String(s) => s,
        _ => "",
    };

    println!("{} {}", head_word.bold().green(), functional_label.dimmed());

    if let Value::Array(defs) = &entry["def"] {
        for def in defs.iter() {
            print_def(def);
        }
    }

    println!();
}

fn main() {
    let args : Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <word>", args[0]);
        return;
    }
    
    let key = "9f29b177-8135-4752-a7a7-8ee2f1f79d17";
    let word = &args[1];    
    let body: Value = reqwest::blocking::get(format!(
        "https://dictionaryapi.com/api/v3/references/collegiate/json/{}?key={}",
        word, key
    ))
    .unwrap()
    .json()
    .unwrap();

    if let Value::Array(entries) = body {
        for entry in entries.iter() {
            print_entry(entry);            
        }
    }
}
