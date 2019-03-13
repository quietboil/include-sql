use std::fs::File;
use std::path::Path;
use std::io::{self, BufRead, BufReader};
use lazy_static::lazy_static;
use regex::Regex;
use syn::Ident;
use proc_macro2::Span;

pub(crate) struct Stmt {
    pub(crate) name: String,
    pub(crate) const_name: Ident,
    pub(crate) text: String,
    pub(crate) params: Option<StmtParams>
}

pub(crate) struct StmtParams {
    pub(crate) struct_name: Ident,
    pub(crate) pos_params: Vec<Ident>,
    pub(crate) lst_params: Vec<LstParam>
}

pub(crate) struct LstParam {
    pub(crate) name: Ident,
    pub(crate) position: usize
}

pub(crate) fn parse_sql_file(path: &str, param_prefix: &str) -> io::Result<Vec<Stmt>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let file_name = Path::new(&path)
        .file_stem().unwrap_or_default()
        .to_str().unwrap_or_default();
    parse_sql(file_name, reader, param_prefix)
}

fn parse_sql(file_name: &str, mut reader: impl BufRead, param_prefix: &str) -> io::Result<Vec<Stmt>> {
    let mut all_stmt = Vec::new();
    let mut sql_name = String::with_capacity(50);
    let mut sql_text = String::with_capacity(500);

    let mut buffer = String::with_capacity(100);
    loop {
        let num_read = reader.read_line(&mut buffer)?;
        if num_read == 0 {
            break;
        }
        let line = buffer.trim_end();
        if line.len() > 0 {
            if line.starts_with("--") {
                if let Some( caps ) = STMT_NAME.captures(line) {
                    let name = &caps[1];
                    if !sql_text.is_empty() {
                        let name = if sql_name.is_empty() { file_name } else { &sql_name };
                        let stmt = Stmt::new(name, &sql_text, param_prefix);
                        all_stmt.push(stmt);
                        sql_text.clear();
                    }
                    sql_name.clear();
                    sql_name.push_str(name);
                }
            } else {
                if !sql_text.is_empty() {
                    sql_text.push('\n');
                }
                sql_text.push_str(&line);
            }
        }
        buffer.clear();
    }
    if !sql_text.is_empty() {
        let name = if sql_name.is_empty() { file_name } else { &sql_name };
        let stmt = Stmt::new(name, &sql_text, param_prefix);
        all_stmt.push(stmt);
    }
    Ok(all_stmt)
}

macro_rules! ident {
    ($s:expr) => {
        Ident::new($s, Span::call_site())
    };
}

impl Stmt {
    fn new(stmt_name: &str, stmt_text: &str, param_prefix: &str) -> Self {
        let name = ident!(&stmt_name.to_uppercase());
        let (text, pos_params, lst_params) = parse_sql_text(stmt_text, param_prefix);
        let params = if !pos_params.is_empty() || !lst_params.is_empty() {
            Some( StmtParams::new(stmt_name, pos_params, lst_params) )
        } else {
            None
        };
        Stmt { name: stmt_name.to_string(), const_name: name, text, params }
    }
}

impl StmtParams {
    fn new(stmt_name: &str, pos_params: Vec<Ident>, lst_params: Vec<LstParam>) -> Self {
        StmtParams { struct_name: ident!(&to_camel_case(stmt_name)), pos_params, lst_params }
    }
}

fn parse_sql_text(stmt_text: &str, param_prefix: &str) -> (String, Vec<Ident>, Vec<LstParam>) {
    let mut text = String::with_capacity(stmt_text.len());
    let mut sql_in_params = Vec::new();
    for caps in SQL_IN_PARAM.captures_iter(stmt_text) {
        let param_name = &caps[1];
        if !sql_in_params.iter().any(|name| name == param_name) {
            sql_in_params.push(param_name.to_string());
        }
    }
    let mut from = 0;
    let mut pos_params = Vec::new();
    let mut lst_params = Vec::new();
    for caps in SQL_PARAM.captures_iter(stmt_text) {
        if let Some( param_match ) = caps.get(0) {
            let text_end = param_match.start();
            text.push_str(&stmt_text[from..text_end]);
            let param_name = &caps[1];
            if sql_in_params.iter().any(|name| name == param_name) {
                let param = LstParam { name: ident!(param_name), position: text.len() };
                lst_params.push(param);
            } else {
                let param_no = if let Some( idx ) = pos_params.iter().position(|name| name == param_name) {
                    idx + 1
                } else {
                    pos_params.push(param_name.to_string());
                    pos_params.len()
                };
                text.push_str(param_prefix);
                text.push_str(&param_no.to_string());
            }
            from = param_match.end();
        }
    }
    text.push_str(&stmt_text[from..]);

    let pos_params : Vec<_> = pos_params.into_iter().map(|name| ident!(&name)).collect();

    (text, pos_params, lst_params)
}

fn to_camel_case(stmt_name: &str) -> String {
    let mut name = String::with_capacity(stmt_name.len());
    for name_fragment in stmt_name.split("_") {
        let mut chars = name_fragment.chars();
        if let Some( first ) = chars.next() {
            if let Some( c ) = first.to_uppercase().next() {
                name.push(c);
            }
            for c in chars {
                if let Some( c ) = c.to_lowercase().next() {
                    name.push(c);
                }
            }
        }
    }
    name
}

lazy_static! {
    static ref STMT_NAME : Regex = Regex::new(r"^--\s*name:\s*([[:word:]]+)").expect("bad statement name line pattern");
    static ref SQL_PARAM : Regex = Regex::new(r":([[:word:]]+)").expect("bad parameter name pattern");
    static ref SQL_IN_PARAM : Regex = Regex::new(r"\b[Ii][Nn]\s*\(\s*:([[:word:]]+)\s*\)").expect("bad IN parameter pattern");
}
