**include-sqlite-sql** is an extension of [include-sql][1] for using SQLite SQL in Rust. It completes include-sql by providing `impl_sql` macro to generate database access methods from the included SQL. include-sqlite-sql uses [Rusqlite][2] for database access.

# Usage

Include `include-sqlite-sql` as a dependency:

```toml
[dependencies]
include-sqlite-sql = "0.1"
```

Write your SQL and save it in a file. For example, let's say the following is the content of the `library.sql` file that is saved in the project's `src` folder:

```sql
-- name: get_loaned_books?
-- Returns the list of books loaned to a patron
-- # Parameters
-- param: user_id: &str - user ID
SELECT book_title
  FROM library
 WHERE loaned_to = :user_id
 ORDER BY 1;

-- name: loan_books!
-- Updates the book records to reflect loan to a patron
-- # Parameters
-- param: user_id: &str - user ID
-- param: book_ids: u32 - book IDs
UPDATE library
   SET loaned_to = :user_id
     , loaned_on = current_timestamp
 WHERE book_id IN (:book_ids);
```

And then use it in Rust as:

```rust , ignore
use include_sqlite_sql::{include_sql, impl_sql};
use rusqlite::{Result, Connection};

include_sql!("src/library.sql");

fn main() -> Result<()> {
    let args : Vec<String> = std::env::args().collect();
    let dbpath = &args[1];
    let user_id = &args[2];

    let db = Connection::open(dbpath)?;

    db.get_loaned_books(user_id, |row| {
        let book_title : &str = row.get_ref("book_title")?.as_str()?;
        println!("{}", book_title);
        Ok(())
    })?;

    Ok(())
}
```

> **Note** that the path to the SQL file must be specified relative to the project root, i.e. relative to `CARGO_MANIFEST_DIR`, even if you keep your SQL file alongside rust module that includes it. Because include-sql targets stable Rust this requirement will persist until [SourceFile][3] stabilizes.

# Anatomy of the Included SQL File

Please see the **Anatomy of the Included SQL File** in [include-sql][4] documentation for the description of the format that include-sql can parse.

# Generated Methods

**include-sqlite-sql** generates 3 variants of database access methods using the following selectors:
* `?` - methods that process rows retrieved by `SELECT`,
* `!` - methods that execute all other non-`SELECT` methods, and
* `->` - methods that execute `RETURNING` statements and provide access to returned data.

## Process Selected Rows

For the `SELECT` statement like:

```sql
-- name: get_loaned_books?
-- param: user_id: &str
SELECT book_title FROM library WHERE loaned_to = :user_id;
```

The method with the following signature is generated:

```rust , ignore
fn get_loaned_books<F>(&self, user_id: &str, row_callback: F) -> rusqlite::Result<()>
where F: Fn(&rusqlite::Row<'_>) -> rusqlite::Result<()>;
```

Where:
- `user_id` is a parameter that has the same name as the SQL parameter with the declared (in the SQL) type as `&str`.
- `F` is a type of a callback (closure) that the method implementation will call to process each row.

## Execute Non-Select Statements

For non-select statements - INSERT, UPDATE, DELETE, etc. - like the following:

```sql
-- name: loan_books!
-- param: user_id: &str
-- param: book_ids: u32
UPDATE library
   SET loaned_to = :user_id
     , loaned_on = current_timestamp
 WHERE book_id IN (:book_ids);
```

The method with the following signature is generated:

```rust , ignore
fn loan_books(&self, user_id: &str, book_ids: &[u32]) -> rusqlite::Result<usize>;
```

Where:
- `user_id` is a parameter that has the same name as the SQL parameter with the declared (in the SQL) type as `&str`,
- `book_ids` is a parameter for the matching IN-list parameter where each item in a collection has type `u32`.

## RETURNING Statements

For DELETE, INSERT, and UPDATE statements that return data via `RETURNING` clause like:

```sql
-- name: add_new_book->
-- param: isbn: &str
-- param: book_title: &str
INSERT INTO library (isbn, book_title)
VALUES (:isbn, :book_title)
RETURNING book_id;
```

The method with the following signature is generated:

```rust , ignore
fn add_new_book<F,R>(&self, isbn: &str, book_title: &str, row_callback: F) -> rusqlite::Result<R>
where F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<R>;
```

# Inferred Parameter Types

If a statement parameter type is not explicitly specified via `param:`, **include-sqlite-sql** will use `impl rusqlite::ToSql` for the corresponding method parameters. For example, if the SQL from the example above has not provided its parameter type:

```sql
-- name: get_loaned_books?
-- Returns the list of books loaned to a patron
SELECT book_title
  FROM library
 WHERE loaned_to = :user_id
 ORDER BY 1;
```

Then the signature of the generated method would be:

```rust , ignore
/// Returns the list of books loaned to a patron
fn get_loaned_books<F>(&self, user_id: impl rusqlite::ToSql, row_callback: F) -> rusqlite::Result<()>
where F: Fn(&rusqlite::Row<'_>) -> rusqlite::Result<()>;
```

[1]: https://crates.io/crates/include-sql
[2]: https://crates.io/crates/rusqlite
[3]: https://doc.rust-lang.org/proc_macro/struct.SourceFile.html
[4]: https://quietboil.github.io/include-sql
