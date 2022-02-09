/*!
This example uses sample [chinook](https://github.com/lerocha/chinook-database) database.
*/
use include_sqlite_sql::{include_sql, impl_sql};
use rusqlite::{Result, Connection};

include_sql!("examples/chinook.sql");

fn main() -> Result<()> {
    let args : Vec<String> = std::env::args().collect();
    let dbname = &args[1];
    let artist = &args[2];

    let db = Connection::open(dbname)?;

    db.get_artist_albums(artist, |row| {
        let artist_name : &str = row.get_ref("artist_name")?.as_str()?;
        let album_title : &str = row.get_ref("album_title")?.as_str()?;
        println!("{}: {}", artist_name, album_title);
        Ok(())
    })?;

    db.count_albums(artist, |row| {
        let artist_name : &str = row.get_ref("artist_name")?.as_str()?;
        let num_albums : u32   = row.get("num_albums")?;
        println!("{}: {}", artist_name, num_albums);
        Ok(())
    })?;

    db.get_customers("CA", &["Apple Inc.", "Google Inc."], |row| {
        let first_name : &str = row.get_ref("first_name")?.as_str()?;
        let last_name  : &str = row.get_ref("last_name")?.as_str()?;
        println!("{}, {}", last_name, first_name);
        Ok(())
    })?;

    db.begin_transaction()?;
    db.create_new_genre(99, "New Age")?;
    // RETURNING is not available before 3.35.0
    println!("sqlite version = {}", rusqlite::version());
    let name = db.delete_genre(99, |row| {
        let name : String = row.get("Name")?;
        Ok(name)
    })?;
    println!("deleted: {}", name);
    db.rollback()?;

    Ok(())
}
