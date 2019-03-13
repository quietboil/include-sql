use postgres::{Connection, TlsMode};
use postgres::types::ToSql;
use include_sql::include_sql;

include_sql!("proc-macro/examples/pgsql.sql","$");

const NO_ARGS: &[&dyn ToSql] = &[];

type Result<T> = std::result::Result<T,Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let conn = connect_to_test_db()?;

    conn.execute(CREATE_TABLE_SHIPS, NO_ARGS)?;
    conn.execute(CREATE_TABLE_SAILORS, NO_ARGS)?;

    let res = conn.query(SELECT_SHIP_BY_NAME, using_select_ship_by_name_args! {
        name: &"Indefatigable"
    })?;

    let ship_id = if res.is_empty() {
        // initialize data set
        let res = conn.query(INSERT_SHIP, using_insert_ship_args! {
            name: &"Indefatigable",
            year: &"1784"
        })?;
        // let it panic - this is a demo after all
        let row = res.get(0);
        let ship_id : i32 = row.get(0);

        let stmt = conn.prepare(INSERT_SAILOR)?;
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
        ship_id
    } else {
        let row = res.get(0);
        let ship_id : i32 = row.get(0);
        ship_id
    };

    let res = conn.query(SELECT_SHIP_CREW, using_select_ship_crew_args! {
        ship: &ship_id
    })?;
    print_query_results("Ship crew", &res);

    // "select_ship_crew_by_rank" uses IN parameter list and thus
    // its SQL has to be generated dynamically for each execution
    let (sql, args) = SelectShipCrewByRank {
        ship: &ship_id,
        ranks: &[ &"captain" as &ToSql, &"midshipman" ]
    }.into_sql_with_args();

    let res = conn.query(&sql, &args)?;
    print_query_results("Officers", &res);

    let (sql, args) = SelectShipCrewByRank {
        ship: &ship_id,
        ranks: &[ &"seaman" as &ToSql ]
    }.into_sql_with_args();

    let res = conn.query(&sql, &args)?;
    print_query_results("Seamen", &res);

    Ok(())
}

fn connect_to_test_db() -> Result<Connection> {
    let mut args = std::env::args().skip(1);
    if let Some( params ) = args.next() {
        let url = String::from("postgres://") + &params;
        let conn = Connection::connect(url, TlsMode::None)?;
        return Ok(conn);
    }
    Err(string_error::static_err("expected postgres connect parameter"))
}

fn print_query_results(title: &str, rows: &postgres::rows::Rows) {
    println!("{}:", title);
    for row in rows {
        let id : i32     = row.get(0);
        let name: String = row.get(1);
        let rank: String = row.get(2);
        println!(" - {} {}, ID: {}", rank, name, id);
    }
}