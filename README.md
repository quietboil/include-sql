[![crates.io](https://img.shields.io/crates/v/include-sql)](https://crates.io/crates/include-sql)
[![Documentation](https://docs.rs/include-sql/badge.svg)](https://docs.rs/include-sql)
![MIT](https://img.shields.io/crates/l/include-sql.svg)
![MSRV](https://img.shields.io/crates/msrv/include-sql)


**include-sql** is a macro for *using* SQL in Rust.

include-sql was inspired by [Yesql][1]. It allows the programmer to write SQL queries in SQL, keep them separate from the Rust code, and easily embed them into Rust programs via the proc-macro that this library provides.

All by itself include-sql actually does very little - it reads and parses SQL file and transforms it into a call to the `impl_sql` macro. It is expected that `impl_sql` is provided either by the project that uses include-sql or by an external library. For example, there are several include-sql companion crates, like [include-postgres-sql][2], [include-sqlite-sql][3], and [include-oracle-sql][6], that implement `impl_sql`. They can simply be used directly if their approaches to embedding SQL are deemed appropriate and convenient. Alternatively, they can be used as a starting point when implementing your own `impl_sql`.

# Example

As include-sql is not intended to be used directly, to illustrate the workflow we'll use [include-sqlite-sql][3].

Add `include-sqlite-sql` as a dependency:

```toml
[dependencies]
include-sqlite-sql = "0.3"
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

include_sql!("/sql/library.sql");

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

> ℹ️ **Note** that the path to the SQL file can be specified either relative to the project root, i.e. relative to the `CARGO_MANIFEST_DIR`, or relative to the rust module that includes it.
> * To specify the path to the included SQL file relative to the project root start the path with the `/` character.
> * To specify the path to the included SQL file relative to the rust source file that included it start the path with the `./` characters.
> * ⚠️ For compatibility with the legacy code the path to the SQL file can also be specified without `/` or `./` prefix. In this case the path to it will be considered to be relative to the project root (as if it was specified with the leading `/`).

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

# Minimum Supported Rust Version

Since `include-sql` 0.3.2 the minimum supported rust version is 1.88 where [Span::file()][4] was stabilized.

> ⚠️ **Note** that [Span::file()][4] when it is called by [rust-analyzer][6], at the time of this writing (version 0.4.2535), returns empty string. This prevents `include-sql` determining the module that called it. While paths relative to the calling module work just fine when projects are compiled by cargo, until [Span::file()][4] is fully functional within [rust-analyzer][6], it is advisable to specify included SQL file paths relative to the project root.

[1]: https://github.com/krisajenkins/yesql
[2]: https://crates.io/crates/include-postgres-sql
[3]: https://crates.io/crates/include-sqlite-sql
[4]: https://doc.rust-lang.org/proc_macro/struct.Span.html#method.file
[5]: https://quietboil.github.io/include-sql
[6]: https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer