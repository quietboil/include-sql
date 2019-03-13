use oci_rs::{ connection::Connection, statement::Statement, types::ToSqlValue as ToSql };
use include_sql::include_sql;

include_sql!("proc-macro/examples/oracle_oci.sql",":");

type Result<T> = std::result::Result<T,Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let conn = connect_to_test_db()?;
    create_test_tables(&conn)?;
    insert_test_ship(&conn)?;
    let ship_id = select_test_ship(&conn)?;
    println!("Ship ID is {}", ship_id);
    insert_test_crew(&conn, ship_id)?;
    report_crew(&conn, ship_id)?;
    report_officers(&conn, ship_id)?;
    report_seamen(&conn, ship_id)?;
    Ok(())
}

fn connect_to_test_db() -> Result<Connection> {
    let mut args = std::env::args().skip(1);
    if let Some( user ) = args.next() {
        if let Some( pass ) = args.next() {
            if let Some( conn ) = args.next() {
                let conn = Connection::new(&conn, &user, &pass)?;
                return Ok(conn);
            }
        }
    }
    Err(string_error::static_err("expected 3 arguments: user password connect_id"))
}

fn create_test_tables(conn: &Connection) -> Result<()> {
    let mut stmt = conn.create_prepared_statement(CREATE_TEST_TABLES)?;
    stmt.execute()?;
    Ok(())
}

fn insert_test_ship(conn: &Connection) -> Result<()> {
    let mut stmt = conn.create_prepared_statement(INSERT_SHIP)?;
    stmt.bind(insert_ship_args! {
        name: &"Indefatigable",
        year: &"1784"
    })?;
    stmt.execute()?;
    Ok(())
}

fn select_test_ship(conn: &Connection) -> Result<i64> {
    let mut stmt = conn.create_prepared_statement(SELECT_SHIP_BY_NAME)?;
    stmt.bind(select_ship_by_name_args! {
        name: &"Indefatigable"
    })?;
    stmt.execute()?;
    let res = stmt.result_set()?;
    if res.len() == 1 {
        let cols = res[0].columns();
        if cols.len() >= 1 {
            if let Some( id ) = cols[0].value::<i64>() {
                return Ok(id);
            }
        }
    }
    Err(string_error::static_err("Ship ID was not found in the returned results"))
}

fn insert_test_crew(conn: &Connection, ship_id: i64) -> Result<()> {
    let mut stmt = conn.create_prepared_statement(INSERT_SAILOR)?;
    stmt.bind(insert_sailor_args! {
        ship: &ship_id,
        name: &"Edward Pellew",
        rank: &"captain"
    })?;
    stmt.execute()?;
    stmt.bind(insert_sailor_args! {
        ship: &ship_id,
        name: &"Horatio Hornblower",
        rank: &"midshipman"
    })?;
    stmt.execute()?;
    stmt.bind(insert_sailor_args! {
        ship: &ship_id,
        name: &"Archie Kennedy",
        rank: &"midshipman"
    })?;
    stmt.execute()?;
    stmt.bind(insert_sailor_args! {
        ship: &ship_id,
        name: &"Matthews",
        rank: &"seaman"
    })?;
    stmt.execute()?;
    stmt.bind(insert_sailor_args! {
        ship: &ship_id,
        name: &"Styles",
        rank: &"seaman"
    })?;
    stmt.execute()?;
    Ok(())
}

fn report_crew(conn: &Connection, ship_id: i64) -> Result<()> {
    let mut stmt = conn.create_prepared_statement(SELECT_SHIP_CREW)?;
    stmt.bind(select_ship_crew_args! {
        ship: &ship_id
    })?;
    stmt.execute()?;
    print_query_results("Ship crew", &mut stmt)?;
    Ok(())
}

fn report_officers(conn: &Connection, ship_id: i64) -> Result<()> {
    let (sql, args) = SelectShipCrewByRank {
        ship: &ship_id,
        ranks: &[ &"captain" as &ToSql, &"midshipman" ]
    }.into_sql_with_args();

    let mut stmt = conn.create_prepared_statement(&sql)?;
    stmt.bind(&args)?;
    stmt.execute()?;
    print_query_results("Officers", &mut stmt)?;
    Ok(())
}

fn report_seamen(conn: &Connection, ship_id: i64) -> Result<()> {
    let (sql, args) = SelectShipCrewByRank {
        ship: &ship_id,
        ranks: &[ &"seaman" as &ToSql ]
    }.into_sql_with_args();

    let mut stmt = conn.create_prepared_statement(&sql)?;
    stmt.bind(&args)?;
    stmt.execute()?;
    print_query_results("Seamen", &mut stmt)?;
    Ok(())
}

fn print_query_results(title: &str, stmt: &mut Statement) -> Result<()> {
    println!("{}:", title);
    for res in stmt.lazy_result_set() {
        match res {
            Ok(row) => {
                let cols = row.columns();
                let id   = if let Some( value ) = cols[0].value::<i64>() { value } else { 0 };
                let name = if let Some( value ) = cols[1].value::<String>() { value } else { String::from("?") };
                let rank = if let Some( value ) = cols[2].value::<String>() { value } else { String::from("?") };
                println!(" - {} {}, ID: {}", rank, name, id);
            }
            Err(err) => {
                println!(" Error {}", err);
            }
        }
    }
    Ok(())
}