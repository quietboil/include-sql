#![allow(dead_code)]

use std::collections::HashMap;

use once_cell::sync::Lazy;
use regex::Regex;
use crate::err::{self, Result};
use crate::conv::to_snake_case;

/// Parses the content of the included SQL file.
pub(super) fn parse(text: &str, file_name: &str) -> Result<IncludedSql> {
    let file_name = file_name.to_string();
    let stmt_list = parse_text(text);
    check_stmt_names(&stmt_list)?;
    check_parameters(&stmt_list)?;
    Ok(IncludedSql { file_name, stmt_list })
}

static LINE_COMMENT : Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*--").expect("full line comment pattern"));
static TAIL_COMMENT : Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*--").expect("line tail comment pattern"));
static STMT_NAME    : Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*name:\s*([[:alpha:]][[:word:]]*)([!#$%&*+./:<=>?@^|~-]*)").expect("statement name pattern"));
static STMT_PARAM   : Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*param:\s*([[:alpha:]][[:word:]]*)\s*:\s*(\S+)\s*(.*)").expect("statement parameter pattern"));
static BIND_NAME    : Lazy<Regex> = Lazy::new(|| Regex::new(r"\b[Ii][Nn]\s*\(\s*(:[[:alpha:]][[:word:]]*)\s*\)|(:[[:alpha:]][[:word:]]*)").expect("parameter placeholder pattern"));
static INTO_TOKEN   : Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?:[@,#$?;~_.]|[+^/*!%]=?|&[&=]?|=[=>]?|>[>=]?|<[<=-]?|[|][=|]?|-[=>]?|::?|[.][.][.=]?|>>=|<<=)$").expect("punctuation token pattern"));

fn parse_text(text: &str) -> Vec<Stmt> {
    let mut stmt_list = Vec::new();
    let mut stmt_name = None;
    let mut stmt_into = None;
    let mut stmt_text = String::with_capacity(250);
    let mut stmt_docs = String::with_capacity(250);
    let mut stmt_params = HashMap::new();

    for line in text.lines() {
        let line = line.trim_end();
        if line.len() == 0 { continue; }

        if let Some( comment_prefix ) = LINE_COMMENT.find(line) {

            let comment = &line[comment_prefix.end()..];
            if let Some( name ) = STMT_NAME.captures( comment ) {
                if !stmt_text.is_empty() {
                    // found a new name, while the current statement is not saved yet
                    let stmt = Stmt::new(stmt_name, stmt_into, stmt_params, &stmt_text, &stmt_docs);
                    stmt_list.push(stmt);

                    stmt_text.clear();
                }
                stmt_docs.clear();
                stmt_params = HashMap::new();

                stmt_name = name.get(1).map(|name_match| name_match.as_str().to_string());
                stmt_into = name.get(2).map(|into_match| into_match.as_str()).filter(|into| into.len() > 0).map(|into| into.to_string());

            } else if !stmt_text.is_empty() {
                // Then the line is a statement inner comment
                continue;

            } else if let Some( param ) = STMT_PARAM.captures( comment ) {
                let param_name = to_snake_case(&param[1]);
                let param_type = param[2].to_string();
                // build a doc-comment line for this parameter
                if !stmt_docs.is_empty() {
                    stmt_docs.push('\n');
                }
                stmt_docs.push_str(" * `");
                stmt_docs.push_str(param_name.as_str());
                stmt_docs.push_str("` ");
                stmt_docs.push_str(&param[3]);

                stmt_params.insert(param_name, param_type);
            } else {
                // A comment or a doc-comment line.
                // It depends on whether this statement's name: has been parsed already
                if !stmt_docs.is_empty() {
                    stmt_docs.push('\n');
                }
                stmt_docs.push_str(comment);
            }

        } else {
            if !stmt_text.is_empty() {
                stmt_text.push('\n');
            }
            let line = if let Some( comment ) = TAIL_COMMENT.find(line) {
                &line[0..comment.start()]
            } else {
                line.trim_end()
            };
            if let Some( last_line ) = line.strip_suffix(';') {
                // statement is explicitly terminated
                stmt_text.push_str(last_line.trim_end());
                if !stmt_text.is_empty() {
                    let stmt = Stmt::new(stmt_name, stmt_into, stmt_params, &stmt_text, &stmt_docs);
                    stmt_list.push(stmt);

                    stmt_text.clear();
                    stmt_params = HashMap::new();
                } else {
                    stmt_params.clear();
                }
                stmt_name = None;
                stmt_into = None;
                stmt_docs.clear();
            } else {
                stmt_text.push_str(&line);
            }
        }
    }
    if !stmt_text.is_empty() {
        let stmt = Stmt::new(stmt_name, stmt_into, stmt_params, &stmt_text, &stmt_docs);
        stmt_list.push(stmt);
    }
    stmt_list
}

fn check_stmt_names(stmt_list: &[Stmt]) -> Result<()> {
    for stmt in stmt_list {
        if !stmt.items.is_empty() {
            if stmt.name.is_empty() {
                let text = match &stmt.items[0] {
                    StmtItem::Text(text) => String::from(text),
                    StmtItem::Bind(name) => String::from(":") + &name,
                    StmtItem::List(name) => String::from(":") + &name,
                };
                return Err(err::new(format!("statement `{}...` must have a name", text)));
            }
            if !INTO_TOKEN.is_match(&stmt.into) {
                return Err(err::new(format!("statement `{}` variant selector `{}` is not a single punctuation token", &stmt.name, &stmt.into)));
            }
        }
    }
    Ok(())
}

fn check_parameters(stmt_list: &[Stmt]) -> Result<()> {
    for stmt in stmt_list {
        for param_name in stmt.params.keys() {
            if !stmt.items.iter().any(|item| item.is_bind(param_name)) {
                return Err(err::new(format!("param `{}` is not found in `{}`", param_name, &stmt.name)))
            }
        }
    }
    Ok(())
}

/// Represents the included SQL file.
#[derive(Debug)]
pub(crate) struct IncludedSql {
    pub(crate) file_name: String,
    pub(crate) stmt_list: Vec<Stmt>
}

/// Represents a single SQL statement that was found in the included SQL file.
#[derive(Debug)]
pub(crate) struct Stmt {
    pub(crate) name: String,
    /// Indicates how (into what) the statement should be transformed.
    /// For example, ? might indicate that the generated macro would
    /// retrieve and process rows returned by a query, etc.
    pub(crate) into: String,
    pub(crate) docs: Option<String>,
    pub(crate) params: HashMap<String,String>,
    pub(crate) items: Vec<StmtItem>,
}

/// Represents an element of the SQL statement
#[derive(Debug)]
pub(crate) enum StmtItem {
    /// Text portion of the SQL statement
    Text(String),
    /// Parameter placeholder
    Bind(String),
    /// IN-list parameter placeholder
    List(String),
}

impl Stmt {
    fn new(name: Option<String>, into: Option<String>, params: HashMap<String,String>, stmt_text: &str, stmt_docs: &str) -> Self {
        let name = name.unwrap_or_default();
        let into = into.unwrap_or_else(|| "!".to_string());
        let items = Self::parse_text(stmt_text);
        let docs = if stmt_docs.is_empty() { None } else { Some(stmt_docs.to_string()) };
        Self { name, params, into, docs, items }
    }

    fn parse_text(text: &str) -> Vec<StmtItem> {
        let mut items = Vec::new();
        let mut text_start = 0;
        for caps in BIND_NAME.captures_iter(text) {
            if let Some( name ) = caps.get(1) {
                let bind_range = name.range();
                items.push(StmtItem::Text(text[text_start..bind_range.start].to_string()));
                items.push(StmtItem::List(text[(bind_range.start + 1)..bind_range.end].to_snake_case()));
                text_start = bind_range.end;
            } else if let Some( bind ) = caps.get(2) {
                let bind_range = bind.range();
                items.push(StmtItem::Text(text[text_start..bind_range.start].to_string()));
                items.push(StmtItem::Bind(text[(bind_range.start + 1)..bind_range.end].to_snake_case()));
                text_start = bind_range.end;
            }
        }
        let stmt_tail = text[text_start..].trim_end();
        if !stmt_tail.is_empty() {
            items.push(StmtItem::Text(stmt_tail.to_string()));
        }
        items
    }

    pub(crate) fn unique_binds(&self) -> Vec<&StmtItem> {
        let mut binds = Vec::with_capacity(self.items.len());
        let mut names = Vec::with_capacity(self.items.len());
        for item in &self.items {
            match item {
                StmtItem::Bind(name) if !names.contains(&name) => {
                    names.push(name);
                    binds.push(item)
                },
                StmtItem::List(name) if !names.contains(&name) => {
                    names.push(name);
                    binds.push(item)
                },
                _ => {}
            }
        }
        binds
    }
}

impl StmtItem {
    fn is_bind(&self, name: &str) -> bool {
        match self {
            Self::Text(_) => { false },
            Self::Bind(param_name) => { param_name == name },
            Self::List(param_name) => { param_name == name }
        }
    }
}

trait StrExt {
    fn to_snake_case(&self) -> String;
}

impl StrExt for str {
    fn to_snake_case(&self) -> String {
        crate::conv::to_snake_case(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::sql::StmtItem;

    #[test]
    fn bind_name() {
        use super::BIND_NAME;

        let mut iter = BIND_NAME.captures_iter("where col1 = :val1 and col2 in ( :val2 ) and col3 = :val3 and ...");
        let cap = iter.next();
        assert!(cap.is_some());
        let cap = cap.unwrap();
        let g = cap.get(1);
        assert!(g.is_none());
        let g = cap.get(2);
        assert!(g.is_some());
        let g = g.unwrap();
        assert_eq!(g.as_str(), ":val1");

        let cap = iter.next();
        assert!(cap.is_some());
        let cap = cap.unwrap();
        let g = cap.get(1);
        assert!(g.is_some());
        let g = g.unwrap();
        assert_eq!(g.as_str(), ":val2");
        let g = cap.get(2);
        assert!(g.is_none());

        let cap = iter.next();
        assert!(cap.is_some());
        let cap = cap.unwrap();
        let g = cap.get(1);
        assert!(g.is_none());
        let g = cap.get(2);
        assert!(g.is_some());
        let g = g.unwrap();
        assert_eq!(g.as_str(), ":val3");

        let cap = iter.next();
        assert!(cap.is_none());
    }

    #[test]
    fn parse_empty_sql_file() {
        let text = "\n  \n  \n";
        let sql = super::parse(text, "empty_sql_file").unwrap();
        assert!(sql.stmt_list.is_empty());
    }

    #[test]
    fn parse_comments_only_text() {
        let text = "
-- name: commented_out_statement?
-- SELECT *
--   FROM some_table
--  WHERE column_a = :val1
--    AND column_b LIKE :val2;
        ";
        let sql = super::parse(text, "comments_only_sql").unwrap();
        assert!(sql.stmt_list.is_empty());
    }

    #[test]
    fn parse_empty_stmt() {
        let text = "
-- there is no name
-- not even SQL
-- maybe it's a work in progress
    ;
        ";
        let sql = super::parse(text, "comments_only_sql").unwrap();
        assert!(sql.stmt_list.is_empty());
    }

    #[test]
    #[should_panic(expected = "statement `SELECT Count(*) FROM some_table WHERE num_column > 0...` must have a name")]
    fn parse_unnamed_stmt() {
        use super::parse;

        let text = "
-- name: select_something?
SELECT something FROM somewhere WHERE col = :val;

SELECT Count(*) FROM some_table WHERE num_column > 0;
        ";
        parse(text, "unnamed_statement").unwrap();
    }

    #[test]
    fn parse_no_variant_stmt() {
        use super::parse;

        let text = "
-- name: update_something
UPDATE something SET a_thing = :VAL WHERE col = :COL_VAL;
        ";
        let sql = parse(text, "unnamed_statement").unwrap();
        assert_eq!(1, sql.stmt_list.len());
        let stmt = &sql.stmt_list[0];
        assert_eq!(stmt.name, "update_something");
        assert_eq!(stmt.into, "!");
    }

    #[test]
    #[should_panic(expected = "statement `select_something` variant selector `>?` is not a single punctuation token")]
    fn parse_bad_variant_stmt() {
        use super::parse;

        let text = "
-- name: select_something>?
SELECT something FROM somewhere WHERE col = :val;
        ";
        parse(text, "unnamed_statement").unwrap();
    }

    #[test]
    fn parse_named_stmt_with_a_param() {
        use super::{parse, StmtItem};

        let text = "
-- A comment line (not part of the statement)
-- name: count_positives?
-- A doc-comment line
-- # Parameters
-- param: rec_type : &str - record type
-- Final doc-comment line
SELECT Count(*)
       -- line level comments inside the statement are discarded too
  FROM some_table      -- as well as trailing comments
 WHERE num_column > 0
   AND record_type = :rec_type
;
        ";
        let sql = parse(text, "named_stmt_with_a_param").unwrap();
        assert_eq!(1, sql.stmt_list.len());
        let stmt = &sql.stmt_list[0];
        assert_eq!(stmt.name, "count_positives");
        assert_eq!(stmt.into, "?");
        assert_eq!(stmt.docs.as_ref().unwrap(), " A doc-comment line\n # Parameters\n * `rec_type` - record type\n Final doc-comment line");

        let stmt_items = &stmt.items;
        assert_eq!(2, stmt_items.len());
        let item = &stmt_items[0];
        if let StmtItem::Text(text) = item {
            assert_eq!("SELECT Count(*)\n  FROM some_table\n WHERE num_column > 0\n   AND record_type = ", text)
        } else {
            panic!("expected text item");
        }
        let item = &stmt_items[1];
        if let StmtItem::Bind(name) = item {
            assert_eq!("rec_type", name);
        } else {
            panic!("expected bind item");
        }

        assert_eq!(stmt.params.len(), 1);
        let ty = stmt.params.get("rec_type");
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert_eq!(ty, "&str");
    }

    #[test]
    #[should_panic(expected = "param `record_type` is not found in `count_positives`")]
    fn unknown_parameter() {
        use super::parse;

        let text = "
-- name: count_positives?
-- # Parameters
-- param: record_type: &str - record type
SELECT Count(*)
  FROM some_table
 WHERE num_column > 0
   AND record_type = :rec_type
;
        ";
        parse(text, "unknown_parameter").unwrap();
    }

    #[test]
    fn parse_multiple_stmts() {
        use super::{parse, StmtItem};

        let text = "
-- comment before a `name:` is a file comment

-- name: count_stuff?
-- param: rec_type: &str - record type
SELECT Count(*)
  FROM some_table
 WHERE record_type = :rec_type
;

-- comment after the terminating semicolon is also a file comment

-- name: find_stuff?
-- param: min_qty: usize - minimum quantity
SELECT *
  FROM some_table
 WHERE some_qty >= :min_qty

-- if the statement is not explicitly terminated with a semicolon
-- this comment will be considered as an inner statement comment
-- and ignored as such too
        ";
        let sql = parse(text, "multiple_stmts").unwrap();
        assert_eq!(2, sql.stmt_list.len());

        let stmt = &sql.stmt_list[0];
        assert_eq!(stmt.name, "count_stuff");
        assert_eq!(stmt.into, "?");
        assert_eq!(stmt.params.len(), 1);
        let param = stmt.params.get("rec_type");
        assert!(param.is_some());
        let ptype = param.unwrap();
        assert_eq!(ptype, "&str");
        assert_eq!(stmt.docs.as_ref().unwrap(), " * `rec_type` - record type");

        let stmt_items = &stmt.items;
        assert_eq!(2, stmt_items.len());

        let item = &stmt_items[0];
        if let StmtItem::Text(text) = item {
            assert_eq!("SELECT Count(*)\n  FROM some_table\n WHERE record_type = ", text)
        } else {
            panic!("expected text item")
        }
        let item = &stmt_items[1];
        if let StmtItem::Bind(name) = item {
            assert_eq!("rec_type", name);

        } else {
            panic!("expected bind item")
        }

        let stmt = &sql.stmt_list[1];
        assert_eq!(stmt.name, "find_stuff");
        assert_eq!(stmt.into, "?");
        assert_eq!(stmt.params.len(), 1);
        let param = stmt.params.get("min_qty");
        assert!(param.is_some());
        let ptype = param.unwrap();
        assert_eq!(ptype, "usize");
        assert_eq!(stmt.docs.as_ref().unwrap(), " * `min_qty` - minimum quantity");

        let stmt_items = &stmt.items;
        assert_eq!(2, stmt_items.len());

        let item = &stmt_items[0];
        if let StmtItem::Text(text) = item {
            assert_eq!("SELECT *\n  FROM some_table\n WHERE some_qty >= ", text)
        } else {
            panic!("expected text item")
        }
        let item = &stmt_items[1];
        if let StmtItem::Bind(name) = item {
            assert_eq!("min_qty", name);

        } else {
            panic!("expected bind item")
        }
    }

    #[test]
    fn unique_binds() {
        let text = "
-- name: unique_binds
INSERT INTO some_table VALUES (:v1, :v2, :v3, :v1, :v3, :v1, :v2, :v4, :v3, :v1);
        ";
        let sql = super::parse(text, "find_unique_parameter_names").unwrap();
        assert_eq!(1, sql.stmt_list.len());
        let stmt = &sql.stmt_list[0];
        let binds = stmt.unique_binds();
        assert_eq!(binds.len(), 4);
        if let StmtItem::Bind(name) = binds[0] {
            assert_eq!(name, "v1");
        } else {
            panic!("expected v1");
        }
        if let StmtItem::Bind(name) = binds[1] {
            assert_eq!(name, "v2");
        } else {
            panic!("expected v2");
        }
        if let StmtItem::Bind(name) = binds[2] {
            assert_eq!(name, "v3");
        } else {
            panic!("expected v3");
        }
        if let StmtItem::Bind(name) = binds[3] {
            assert_eq!(name, "v4");
        } else {
            panic!("expected v4");
        }
    }

    #[test]
    fn snake_case_parameter_names() {
        use super::{parse, StmtItem};

        let text = "
--name: snake_case_parameter_names->
INSERT INTO invoice (InvoiceId, CustomerId, InvoiceDate, Total) VALLUES (:InvoiceId, :customerId, :Invoice_Date, :TOTAL);
        ";
        let sql = parse(text, "snake_case_parameter_names").unwrap();
        assert_eq!(sql.stmt_list.len(), 1);
        assert_eq!(sql.stmt_list[0].into, "->");
        let stmt_items = &sql.stmt_list[0].items;
        assert_eq!(stmt_items.len(), 9);

        match &stmt_items[1] {
            StmtItem::Bind(name) => {
                assert_eq!(name, "invoice_id");

            }
            _ => { panic!("unexpected {:?}", &stmt_items[1]); }
        }
        match &stmt_items[3] {
            StmtItem::Bind(name) => {
                assert_eq!(name, "customer_id");

            }
            _ => { panic!("unexpected {:?}", &stmt_items[1]); }
        }
        match &stmt_items[5] {
            StmtItem::Bind(name) => {
                assert_eq!(name, "invoice_date");

            }
            _ => { panic!("unexpected {:?}", &stmt_items[1]); }
        }
        match &stmt_items[7] {
            StmtItem::Bind(name) => {
                assert_eq!(name, "total");

            }
            _ => { panic!("unexpected {:?}", &stmt_items[1]); }
        }
    }

    #[test]
    fn into_token() {
        use super::INTO_TOKEN;

        assert!(INTO_TOKEN.is_match("+"));
        assert!(INTO_TOKEN.is_match("+="));
        assert!(INTO_TOKEN.is_match("&"));
        assert!(INTO_TOKEN.is_match("&&"));
        assert!(INTO_TOKEN.is_match("&="));
        assert!(INTO_TOKEN.is_match("@"));
        assert!(INTO_TOKEN.is_match("!"));
        assert!(INTO_TOKEN.is_match("^"));
        assert!(INTO_TOKEN.is_match("^="));
        assert!(INTO_TOKEN.is_match(":"));
        assert!(INTO_TOKEN.is_match("::"));
        assert!(INTO_TOKEN.is_match(","));
        assert!(INTO_TOKEN.is_match("/"));
        assert!(INTO_TOKEN.is_match("/="));
        assert!(INTO_TOKEN.is_match("$"));
        assert!(INTO_TOKEN.is_match("."));
        assert!(INTO_TOKEN.is_match(".."));
        assert!(INTO_TOKEN.is_match("..."));
        assert!(INTO_TOKEN.is_match("..="));
        assert!(INTO_TOKEN.is_match("="));
        assert!(INTO_TOKEN.is_match("=="));
        assert!(INTO_TOKEN.is_match(">="));
        assert!(INTO_TOKEN.is_match(">"));
        assert!(INTO_TOKEN.is_match("<="));
        assert!(INTO_TOKEN.is_match("<"));
        assert!(INTO_TOKEN.is_match("*="));
        assert!(INTO_TOKEN.is_match("!="));
        assert!(INTO_TOKEN.is_match("|"));
        assert!(INTO_TOKEN.is_match("|="));
        assert!(INTO_TOKEN.is_match("||"));
        assert!(INTO_TOKEN.is_match("#"));
        assert!(INTO_TOKEN.is_match("?"));
        assert!(INTO_TOKEN.is_match("->"));
        assert!(INTO_TOKEN.is_match("<-"));
        assert!(INTO_TOKEN.is_match("%"));
        assert!(INTO_TOKEN.is_match("%="));
        assert!(INTO_TOKEN.is_match("=>"));
        assert!(INTO_TOKEN.is_match(";"));
        assert!(INTO_TOKEN.is_match("<<"));
        assert!(INTO_TOKEN.is_match("<<="));
        assert!(INTO_TOKEN.is_match(">>"));
        assert!(INTO_TOKEN.is_match(">>="));
        assert!(INTO_TOKEN.is_match("*"));
        assert!(INTO_TOKEN.is_match("-"));
        assert!(INTO_TOKEN.is_match("-="));
        assert!(INTO_TOKEN.is_match("~"));
        assert!(INTO_TOKEN.is_match("_"));

        assert!(!INTO_TOKEN.is_match("!>"));
        assert!(!INTO_TOKEN.is_match(":>"));
        assert!(!INTO_TOKEN.is_match("?!"));
        assert!(!INTO_TOKEN.is_match("!?"));
    }
}
