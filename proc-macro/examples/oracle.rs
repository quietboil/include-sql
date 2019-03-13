use oracle::{Connection, ToSql, ResultSet, Row};

use include_sql::include_sql;

include_sql!("proc-macro/examples/oracle.sql",":");

type Result<T> = std::result::Result<T,Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let conn = connect_to_test_db()?;

    conn.execute(CREATE_TEST_TABLES, &[])?;
    conn.execute(INSERT_SHIP, using_insert_ship_args! {
        name: &"Indefatigable",
        year: &"1784"
    })?;
    let (ship_id, _year_launched) = conn.query_row_as::<(i32,String)>(SELECT_SHIP_BY_NAME, using_select_ship_by_name_args! {
        name: &"Indefatigable"
    })?;
    let mut stmt = conn.prepare(INSERT_SAILOR, &[])?;
    stmt.execute(using_insert_sailor_args! {
        ship: &ship_id,
        name: &"Edward Pellew",
        rank: &"captain"
    })?;
    stmt.execute(using_insert_sailor_args! {
        ship: &ship_id,
        name: &"Horatio Hornblower",
        rank: &"midshipman"
    })?;
    stmt.execute(using_insert_sailor_args! {
        ship: &ship_id,
        name: &"Archie Kennedy",
        rank: &"midshipman"
    })?;
    stmt.execute(using_insert_sailor_args! {
        ship: &ship_id,
        name: &"Matthews",
        rank: &"seaman"
    })?;
    stmt.execute(using_insert_sailor_args! {
        ship: &ship_id,
        name: &"Styles",
        rank: &"seaman"
    })?;

    let res = conn.query(SELECT_SHIP_CREW, using_select_ship_crew_args! {
        ship: &ship_id
    })?;
    print_query_results("Ship crew", &res)?;

    let (sql, args) = SelectShipCrewByRank {
        ship: &ship_id,
        ranks: &[ &"captain" as &ToSql, &"midshipman" ]
    }.into_sql_with_args();

    let res = conn.query(&sql, &args)?;
    print_query_results("Officers", &res)?;

    let (sql, args) = SelectShipCrewByRank {
        ship: &ship_id,
        ranks: &[ &"seaman" as &ToSql ]
    }.into_sql_with_args();

    let res = conn.query(&sql, &args)?;
    print_query_results("Seamen", &res)?;

    Ok(())
}

fn connect_to_test_db() -> Result<Connection> {
    let mut args = std::env::args().skip(1);
    if let Some( user ) = args.next() {
        if let Some( pass ) = args.next() {
            if let Some( conn ) = args.next() {
                let conn = Connection::connect(&user, &pass, &conn, &[])?;
                return Ok(conn);
            }
        }
    }
    Err(string_error::static_err("expected 3 arguments: user password connect_id"))
}

fn print_query_results(title: &str, res: &ResultSet<Row>) -> Result<()> {
    println!("{}:", title);
    for row in res {
        let row = row?;
        let id : i32     = row.get(0)?;
        let name: String = row.get(1)?;
        let rank: String = row.get(2)?;
        println!(" - {} {}, ID: {}", rank, name, id);
    }
    Ok(())
}