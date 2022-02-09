**include-sql** is a macro for *using* SQL in Rust.

include-sql was inspired by [Yesql][1]. It allows the programmer to write SQL queries in SQL, keep them separate from the Rust code, and easily embed them into Rust programs via the proc-macro that this library provides.

All by itself include-sql actually does very little - it reads and parses SQL file and transforms it into a call to the `impl_sql` macro. It is expected that `impl_sql` is provided either by the project that uses include-sql or by an external library. For example, there are several include-sql companion crates, like [include-postgres-sql][2] and [include-sqlite-sql][3], that implement `impl_sql`. They can simply be used directly if their approaches to embedding SQL are deemed appropriate and convenient. Alternatively, they can be used as a starting point when implementing your own `impl_sql`.

# Example

As include-sql is not intended to be used directly, to illustrate the workflow we'll use [include-sqlite-sql][3].

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

> **Note** that the path to the SQL file must be specified relative to the project root, i.e. relative to `CARGO_MANIFEST_DIR`, even if you keep your SQL file alongside rust module that includes it. Because include-sql targets stable Rust this requirement will persist until [SourceFile][4] stabilizes.

# Under the Hood

After parsing and validating the content of the SQL file `include-sql` generates the following call:

```rust , ignore
impl_sql!{ LibrarySql =
  {
    ? get_loaned_books (:user_id (&str)) 
    " Returns the list of books loaned to a patron\n # Parameters\n * `user_id` - user ID"
    $ "SELECT book_title\n  FROM library\n WHERE loaned_to = " :user_id "\n ORDER BY 1"
  },
  {
    ! loan_books (:user_id (&str) #book_ids (u32)) 
    " Updates the book records to reflect loan to a patron\n # Parameters\n * `user_id` - user ID\n * `book_ids` - book IDs"
    $ "UPDATE library\n   SET loaned_to = " :user_id "\n,     loaned_on = current_timestamp\n WHERE book_id IN (" #book_ids ")"
  }
}
```

Which `include_sqlite_sql::impl_sql` transforms into the following implementation:

```rust , ignore
trait LibrarySql {
    /// Returns the list of books loaned to a patron
    /// # Parameters
    /// * `user_id` - user ID
    fn get_loaned_books<F>(&self, user_id: &str, row_callback: F) -> rusqlite::Result<()>
    where F: Fn(&rusqlite::Row<'_>) -> rusqlite::Result<()>;

    /// Updates the book records to reflect loan to a patron
    /// # Parameters
    /// * `user_id` - user ID
    /// * `book_ids` - book IDs
    fn loan_books(&self, user_id: &str, book_ids: &[u32]) -> rusqlite::Result<usize>;
}
```

And, of course, it also implements the trait:

```rust , ignore
impl LibrarySql for rusqlite::Connection {
    /// ...
}
```

# Documentation

The included [documentation][5] describes the supported SQL file format and provides instructions on writing your own `impl_sql` macro.

[1]: https://github.com/krisajenkins/yesql
[2]: https://crates.io/crates/include-postgres-sql
[3]: https://crates.io/crates/include-sqlite-sql
[4]: https://doc.rust-lang.org/proc_macro/struct.SourceFile.html
[5]: https://quietboil.github.io/include-sql
