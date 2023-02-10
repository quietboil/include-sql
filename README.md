[![crates.io](https://img.shields.io/crates/v/include-sql)](https://crates.io/crates/include-sql)
[![Documentation](https://docs.rs/include-sql/badge.svg)](https://docs.rs/include-sql)
![MIT](https://img.shields.io/crates/l/include-sql.svg)

**include-sql** is a macro for *using* SQL in Rust.

include-sql was inspired by [Yesql][1]. It allows the programmer to write SQL queries in SQL, keep them separate from the Rust code, and easily embed them into Rust programs via the proc-macro that this library provides.

All by itself include-sql actually does very little - it reads and parses SQL file and transforms it into a call to the `impl_sql` macro. It is expected that `impl_sql` is provided either by the project that uses include-sql or by an external library. For example, there are several include-sql companion crates, like [include-postgres-sql][2], [include-sqlite-sql][3], and [include-oracle-sql][6], that implement `impl_sql`. They can simply be used directly if their approaches to embedding SQL are deemed appropriate and convenient. Alternatively, they can be used as a starting point when implementing your own `impl_sql`.

# Example

As include-sql is not intended to be used directly, to illustrate the workflow we'll use [include-sqlite-sql][3].

Add `include-sqlite-sql` as a dependency:

```toml
[dependencies]
include-sqlite-sql = "0.2"
```

Write your SQL and save it in a file. For example, let's say the following is saved as `library.sql` in the project's `sql` folder:

```sql
-- name: get_loaned_books?
-- Returns the list of books loaned to a patron
-- # Parameters
-- param: user_id: &str - user ID
SELECT book_title
  FROM library
 WHERE loaned_to = :user_id
 ORDER BY 1
/
-- name: loan_books!
-- Updates the book record to reflect loan to a patron
-- # Parameters
-- param: book_titles: &str - book titles
-- param: user_id: &str - user ID
UPDATE library
   SET loaned_to = :user_id
     , loaned_on = current_timestamp
 WHERE book_title IN (:book_titles)
/
```

> **Note** that the parameter order is defined by the `param` declarations.


And then use it in Rust as:

```rust , ignore
use include_sqlite_sql::{include_sql, impl_sql};
use rusqlite::{Result, Connection};

include_sql!("sql/library.sql");

fn main() -> Result<()> {
    let db = Connection::open("library.db")?;

    db.loan_books(&["Where the Sidewalk Ends", "A Wrinkle in Time", "Dune"], "Penny Teller")?;

    db.get_loaned_books("Leonard Hofstadter", |row| {
        let book_title : &str = row.get_ref("book_title")?.as_str()?;
        println!("{book_title}");
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
    ? get_loaned_books (: user_id (&str))
    " Returns the list of books loaned to a patron\n # Parameters\n * `user_id` - user ID"
    $ "SELECT book_title\n  FROM library\n WHERE loaned_to = " :user_id "\n ORDER BY 1"
  },
  {
    ! loan_books (# book_titles (&str) : user_id (&str))
    " Updates the book records to reflect loan to a patron\n # Parameters\n * `user_id` - user ID\n * `book_titles` - book titles"
    $ "UPDATE library\n   SET loaned_to = " : user_id "\n,     loaned_on = current_timestamp\n WHERE book_title IN (" # book_titles ")"
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
    where F: FnMut(&rusqlite::Row) -> rusqlite::Result<()>;

    /// Updates the book records to reflect loan to a patron
    /// # Parameters
    /// * `book_titles` - book titles
    /// * `user_id` - user ID
    fn loan_books(&self, book_ids: &[&str], user_id: &str) -> rusqlite::Result<usize>;
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

# 💥 Breaking Changes in Version 0.3

* Order of the parameters for the generated method is defined by the order of the `param` descriptors. This is a potentially breaking change as previously parameters of the generated method followed the order of the parameters in the SQL statement. When SQL statement header does not use `param` descriptors, then the generated generic method parameters will be ordered according to their appearance in the SQL statement.
* Statements are terminated with the slash `/` instead of the semicolon `;`. This was implemented to allow declaration and use of [batches][7] of statements for SQLite and PL/SQL blocks for Oracle. Note that statement terminator is optional when the statement is the last statement in the file or when it is followed by another statement, which header will auto-terminate the preceding statement.

[1]: https://github.com/krisajenkins/yesql
[2]: https://crates.io/crates/include-postgres-sql
[3]: https://crates.io/crates/include-sqlite-sql
[4]: https://doc.rust-lang.org/proc_macro/struct.SourceFile.html
[5]: https://quietboil.github.io/include-sql
[6]: https://crates.io/crates/include-oracle-sql
[7]: https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html#method.execute_batch
