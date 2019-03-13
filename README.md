# Rust include-sql

**include-sql** is a macro for *using* SQL from Rust.

**include-sql** was inspired by [Yesql](https://github.com/krisajenkins/yesql). However **include-sql** is not a Yesql implementated in Rust as there is one key difference - **include-sql** *assists* in using externally defined SQL, but it offloads the actual work to the database interface. Unlike Yesql it does not generate functions that abstract database access.

## Features

Most of **include-sql** features are derivatives of the Yesql approach to deal with the SQL:
- SQL and Rust are kept separately. This usually improves maintainability of both code bases.
- The SQL parameters are named even for databases that do not support them natively.
 
Some of the features are specific to Rust:
- Arguments are passed to queries via generated structs. This makes it impossible to miss an argument or tack an extra one to the argument list as the compiler will complain. Auto generated argument lists (slices or Iterators, depending on the database interface) will always put arguments in the correct order.

## Usage

Add **include-sql** as a dependency:
```toml
[dependencies]
include-sql = "0.1"
```

## Example

Start with the SQL.
```sql
 -- name: select_ship_crew
 -- Selects ship crew (sailors of a given ship)
 SELECT id, name, rank
   FROM sailors
  WHERE ship_id = :ship
 
 -- name: select_ship_crew_by_rank
 -- Selects sailors of a given ship that also have
 -- specific ranks
 SELECT id, name, rank
   FROM sailors
  WHERE ship_id = :ship
    AND rank IN (:ranks)
 ```
The SQL file might contain one or more SQL statements. Usually all of them are named - see the meta-comment `name:` line. However the top one can remain unnamed. **include-sql** will use the name of the file itself to give it a name. This is mostly useful when there is only one statement in the included SQL file.

Let's assume the above file is stored alongside the `.rs` that is using it and both are named `crew`. Include this SQL into the module that will execute these queries:
```rust
use postgres::{ Connection, Result };
use postgres::types::ToSql;
use include_sql::include_sql;

include_sql!("src/crew.sql","$");
```
There are 2 important points that the example above shows:
1. `include_sql!` macro takes 2 arguments:
   - First argument is the path to the `.sql` file that is relative to the package root.
   - The second one is a prefix that is used by the database to mark positional parameters. For example, SQLite uses `?`, Postgres - `$`, Oracle - `:`.
2. The generated code expects that the database interface provides a trait to convert Rust values into database values. It also expects that it can refer to that trait by the `ToSql` name. Thus the appropriate trait needs to be brought into scope and maybe renamed via `as` into `ToSql`.

As the SQL is being imported **include-sql** generates the following code:
1. The `&str` const that is named after the included statement and contains a pre-processed text of the statement - named arguments are replaced with the positional ones. Using the first statement from the `crew.sql`:
```rust
const SELECT_SHIP_CREW : &str = "SELECT id, name, rank FROM sailors WHERE ship_id = ?1";
```
2. The `struct` that is used to collect query arguments and then pass them to the database interface:
```rust
struct SelectShipCrew<'a> {
    ship: &'a dyn ToSql
}
```
3. Two macros to convert argument struct into a slice. These macros are used when the argument struct cannot be used directly by the database API:
```rust
macro_rules! using_select_ship_crew_args {
    // ...
}
macro_rules! select_ship_crew_args {
    // ...
}
```

> Notes:
> - The arguments struct also implements the [IntoIterator](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html) trait. This it can be passed directly to functions that accept it. SQLite is one of those that can benefit from this.
> - The two argument conversion macros are the same macro that is created with 2 different names. Depending on the database API one will "sound" better than the other. Pick whatever appeals to you (they *are* the same macro).

Finally let's execute the first query:
```rust
fn print_ship_crew(conn: &Connection, ship_id: i32) -> Result<()> {
    let rows = conn.query(SELECT_SHIP_CREW, using_select_ship_crew_args! {
        ship: &ship_id
    })?;

    println!("Ship crew:");
    for row in &rows {
        let id : i32     = row.get(0);
        let name: String = row.get(1);
        let rank: String = row.get(2);
        println!(" - {} {}, ID: {}", rank, name, id);
    }
    Ok(())
}
```

The second query is a bit different. As it uses the `IN (:list)` parameter it cannot be prepared from a static string becuase the length of that list and consequently the number of argument placeholders is not known until run time. These argument structs for these statements therefore implement neither `IntoIterator` trait, nor the macros to make slices. Instead they implement the `into_sql_with_args` method that returns a tuple with 2 elements - the generated SQL and the slice of ordered arguments.

Let's see this in the next example:
```rust
println!("Officers:");

let (sql, args) = SelectShipCrewByRank {
    ship: &ship_id,
    ranks: &[ &"captain" as &ToSql, &"midshipman" ]
}.into_sql_with_args();

let rows = conn.query(&sql, &args)?;
for row in rows {
    let id : i32     = row.get(0);
    let name: String = row.get(1);
    let rank: String = row.get(2);
    println!(" - {} {}, ID: {}", rank, name, id);
}
```

There is also a recurring demo in the `examples` directory. It is more or less the same application, but implemented for 4 different database interfaces.
> Note that to run a demo all except SQLite, which is created in memory, need access to a runnint database with a precreated user/schema. Both Oracle examples need 3 arguments - user, password and connect ID. For example, to run the `oracle` example, you would execute:
```sh
$ cargo run --example oracle -- user pass //host:port/svc
```
> The postgres, or `pgsql` example needs 1 argument - the URL portion of the connect string after "postgres://". In other words something like this might be used to execute it:
```sh
$ cargo run --example pgsql -- user:pass@host
```
