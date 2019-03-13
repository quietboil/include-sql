use rusqlite::{Connection, Result, NO_PARAMS};
use rusqlite::types::ToSql;
use include_sql::include_sql;

include_sql!("proc-macro/examples/sqlite.sql","?");

#[derive(Debug)]
struct Sailor {
    id: i32,
    name: String,
    rank: String
}

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    conn.execute(CREATE_TABLE_SHIPS, NO_PARAMS)?;
    conn.execute(CREATE_TABLE_SAILORS, NO_PARAMS)?;

    let mut stmt = conn.prepare(INSERT_SHIP)?;
    stmt.execute(InsertShip {
        name: &"Indefatigable",
        year: &"1784"
    })?;

    // a bit of cheating - we "know" that the ship ID will be 1 here
    let ship_id = 1;

    let mut stmt = conn.prepare(INSERT_SAILOR)?;
    stmt.execute(InsertSailor {
        ship: &ship_id,
        name: &"Edward Pellew",
        rank: &"captain"
    })?;
    stmt.execute(InsertSailor {
        ship: &ship_id,
        name: &"Horatio Hornblower",
        rank: &"midshipman"
    })?;
    stmt.execute(InsertSailor {
        ship: &ship_id,
        name: &"Archie Kennedy",
        rank: &"midshipman"
    })?;
    stmt.execute(InsertSailor {
        ship: &ship_id,
        name: &"Matthews",
        rank: &"seaman"
    })?;
    stmt.execute(InsertSailor {
        ship: &ship_id,
        name: &"Styles",
        rank: &"seaman"
    })?;

    let mut stmt = conn.prepare(SELECT_SHIP_CREW)?;
    let iter = stmt.query_map(SelectShipCrew {
        ship: &ship_id
    }, |row|
        Ok(Sailor {
            id:     row.get(0)?,
            name:   row.get(1)?,
            rank:   row.get(2)?
        })
    )?;

    println!("Ship crew:");
    for item in iter {
        match item {
            Ok(person) => println!(" - {:?}", person),
            Err(error) => eprintln!("Error {}", error),
        }
    }

    // "select_ship_crew_by_rank" uses IN parameter list and thus
    // its SQL has to be generated dynamically for each execution
    let (sql, args) = SelectShipCrewByRank {
        ship: &ship_id,
        ranks: &[ &"captain" as &ToSql, &"midshipman" ]
    }.into_sql_with_args();

    let mut stmt = conn.prepare(&sql)?;
    let iter = stmt.query_map(&args, |row|
        Ok(Sailor {
            id:     row.get(0)?,
            name:   row.get(1)?,
            rank:   row.get(2)?
        })
    )?;

    println!("Officers:");
    for item in iter {
        match item {
            Ok(person) => println!(" - {:?}", person),
            Err(error) => eprintln!("Error {}", error),
        }
    }

    let (sql, args) = SelectShipCrewByRank {
        ship: &ship_id,
        ranks: &[ &"seaman" as &ToSql ]
    }.into_sql_with_args();

    let mut stmt = conn.prepare(&sql)?;
    let iter = stmt.query_map(&args, |row|
        Ok(Sailor {
            id:     row.get(0)?,
            name:   row.get(1)?,
            rank:   row.get(2)?
        })
    )?;

    println!("Seamen:");
    for item in iter {
        match item {
            Ok(person) => println!(" - {:?}", person),
            Err(error) => eprintln!("Error {}", error),
        }
    }

    Ok(())
}