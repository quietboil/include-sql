/*!
This example uses modified sample [chinook](https://github.com/lerocha/chinook-database) database.
The [Chinook_PostgreSql.sql](https://github.com/lerocha/chinook-database/blob/master/ChinookDatabase/DataSources/Chinook_PostgreSql.sql?raw=true)
was modified - table and column names were changed from camel-case to snake-case and, consequently, double-quotes around names were removed.
*/
use postgres::{Config, NoTls, Error};
use include_postgres_sql::{include_sql, impl_sql};

include_sql!("examples/chinook.sql");

fn main() -> Result<(),Error> {
    let mut args = std::env::args();
    let _ = args.next();
    let artist = args.next().expect("match artist pattern");

    let mut db = Config::new()
        .host("localhost")
        .user("postgres")
        .dbname("chinook")
        .connect(NoTls)?
    ;

    db.get_artist_albums(&artist, |row| {
        let artist_name : &str = row.try_get("artist_name")?;
        let album_title : &str = row.try_get("album_title")?;
        println!("{}: {}", artist_name, album_title);
        Ok(())
    })?;

    db.count_albums(&artist, |row| {
        let artist_name : &str = row.try_get("artist_name")?;
        let num_albums : i64   = row.try_get("num_albums")?;
        println!("{}: {}", artist_name, num_albums);
        Ok(())
    })?;

    db.get_customers("CA", &["Apple Inc.", "Google Inc."], |row| {
        let first_name : &str = row.try_get("first_name")?;
        let last_name  : &str = row.try_get("last_name")?;
        println!("{}, {}", last_name, first_name);
        Ok(())
    })?;

    let mut tr = db.transaction()?;
    tr.add_new_genre(99, "New Age")?;
    let row = tr.delete_genre(99)?;
    let name : &str = row.try_get("name")?;
    println!("deleted {}", name);
    tr.rollback()?;

    Ok(())
}
