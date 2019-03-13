use include_sql::include_sql;

include_sql!("proc-macro/tests/one_unnamed_stmt.sql",":");

#[test]
fn one_unnamed_statement_per_file() {
    assert_eq!(
        "select * from dual",
        collapce_whitespace(ONE_UNNAMED_STMT)
    );
}

include_sql!("proc-macro/tests/named_statements.sql", ":");

#[test]
fn multiple_named_statements_per_file() {
    assert_eq!(
        "select * from dual",
        collapce_whitespace(DUAL_OUTPUT)
    );
    assert_eq!(
        "select count(*) from user_tables",
        collapce_whitespace(USER_TABLES_COUNT)
    );
}

include_sql!("proc-macro/tests/default_stmt.sql", ":");

#[test]
fn include_with_default_unnamed_statement() {
    assert_eq!(
        "select * from dual",
        collapce_whitespace(DEFAULT_STMT)
    );
    assert_eq!(
        "select count(*) from user_tables", 
        collapce_whitespace(COUNT_TABLES)
    );
}

// Statements with parameters need ToSql trait
// The test version of the latter will be used to access argument value(s)
pub(crate) trait ToSql {
    fn to_sql(&self) -> &str;
}

impl ToSql for &str {
    fn to_sql(&self) -> &str {
        *self
    }
}

include_sql!("proc-macro/tests/stmt_with_params.sql", ":");

#[test]
fn statement_with_parameters() {
    assert_eq!(
        "select object_name from user_objects where object_type = :1 and status = 'INVALID'", 
        collapce_whitespace(SELECT_INVALID_OBJECTS)
    );
    let args = SelectInvalidObjects {
        object_type: &"FUNCTION"
    };
    let sql_args = args.into_iter().next();
    assert!(sql_args.is_some());
    if let Some( test_arg ) = sql_args {
        assert_eq!("FUNCTION", test_arg.to_sql());
    }
}

include_sql!("proc-macro/tests/stmt_with_in_params.sql", ":");

#[test]
fn statement_with_in_parameters() {
    assert_eq!(
        "select object_name, object_type from user_objects where object_type in ( ) and generated = :1 or object_type in ( ) and temporary = :2", 
        collapce_whitespace(SELECT_OBJECTS_BY_TYPE)
    );

    let (sql, args) = SelectObjectsByType {
        object_types: &[ &"FUNCTION" as &ToSql, &"TRIGGER" ],
        generated: &"N",
        temporary: &"N"
    }.into_sql_with_args();
    assert_eq!(
        "select object_name, object_type from user_objects where object_type in ( :3,:4 ) and generated = :1 or object_type in ( :3,:4 ) and temporary = :2",
        collapce_whitespace(&sql)
    );
    assert_eq!(4, args.len());
    assert_eq!("N", args[0].to_sql());
    assert_eq!("N", args[1].to_sql());
    assert_eq!("FUNCTION", args[2].to_sql());
    assert_eq!("TRIGGER", args[3].to_sql());
}

/// Removes consecutive whitespaces for easy comparison
fn collapce_whitespace(text: &str) -> String {
    let mut acc = String::with_capacity(text.len());
    let mut iter = text.split_whitespace();
    if let Some( fragment ) = iter.next() {
        acc.push_str(fragment);
        for fragment in iter {
            acc.push(' ');
            acc.push_str(fragment);
        }
    }
    acc
}
