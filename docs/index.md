# Usage

If you decided to write your own `impl_sql`, you would need to include `include-sql` as a dependency:

```toml
[dependencies]
include-sql = "0.2"
```

It might also be prudent to re-export `include-sql` macro to make the use of your crate more ergonomic:

```rust
pub use include_sql::include_sql;
```

The users of your implementation would then be able to generate API to access database queries as:

```rust
use your_crate::{include_sql, impl_sql};

include_sql!("src/queries.sql");
```

# Anatomy of the Included SQL File

For illustration, let's assume that the following file is called `library.sql`:

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
 WHERE book_id IN (:book_ids)   -- Note that :book_ids names a collection
                                -- of values rather than a single one
;
```

An SQL file can include one or more SQL statements. Each statement must have a preceding doc-comment. The latter has a dual purpose - it provides the doc-comment text for the generated method, and it also embeds meta data about the statement that follows it:

* `name:` a mandatory meta comment that defines an `ident` that is used to generate the database access artifact. For example, [include-postgres-sql][1] and [include-sqlite-sql][2] generate a trait method from it.

> **Note** that include-sql will use the name as-is. If you want to avoid Rust complaining about it, use the appropriate (snake) case for it.

* `?` or `!` is a mandatory statement variant tag. It directs `impl_sql` to generate a specific implementation. This tag can be any sequence of Rust punctuation characters as long as they form a valid Rust [token][4].

> For example, [include-postgres-sql][1] and [include-sqlite-sql][2] recognize `?`, `!`, and `->` tags. For `? they generate methods that process selected rows, for `!` - methods that execute all other - non-select - statements, and for `->` - methods that read data from `RETURNING` statements.

* `param:` is an optional description of a statement parameter. It is expressed in `parameter_name : parameter_type` format. Text that follows `parameter_type` is used as a doc-comment for this parameter.

> include-sql uses `param:` to gather parameter types and to generate the Rust doc-comment for the parameter. For example, this line in SQL: `param: user_id: &str - user ID` tells include-sql that the type of `user_id` is `&str`. It is then converted into `` * `user_id` - user ID`` text line and included into the Rust doc-comment for the generated method.

> **Note** that include-sql expects `parameter_type` to be a Rust [type][3] and will fail if it cannot be parsed as such.

> **Note** that because `param:` is optional `include-sql` will create a synthetic parameter description using an inferred type `_` for those SQL parameters that are not explicitly described by `param:`. `impl_sql` must be prepared to handle cases when a parameter type should be inferred and generate method parameter that is typed generically. See [include-postgres-sql][1] or [include-sqlite-sql][2] for an example of how it can be done.

* The rest of the statement doc-comment lines are gathered together to form a Rust doc-comment text for the generated method.

* `:user_id` and `:book_ids` are statement parameters. Each parameter starts with `:` and can be anything the can be an `ident` in Rust. However, as they might be used to name method parameters in Rust, `include-sql` forces them into snake-case.

* The inner statement comments are allowed and will be discarded by include-sql.

Statements should be terminated with semicolons. However, this is optional as the following `name:` meta comment would also auto-terminate the preceding statement.

# Generated `impl_sql` Call

For the SQL above include-sql would generate:

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

Where:

* `LibrarySql` is a camel-cased `ident` derived from the SQL file name. It might be used by `impl_sql` to generate a trait (like [include-postgres-sql][1] and [include-sqlite-sql][2] do).

* `?` or `!` is a statement variant selector

* `get_loaned_books` and `loan_books` are `ident`s created from the statement names that can be used to name generated methods

* `user_id` and `book_ids` are `ident`s that represent parameter names.

* `:` and `#` in front of the parameter names are parameter variant tags:
  - `:` indicates that the following parameter is a scalar
  - `#` tags IN-list parameters.

* The following `(&str)` and `(usize)` are Rust parameter types as declared in the SQL.

> **Note** that types are passed as parenthesized types. This is done to allow `impl_sql` match them as token trees. If a parameter type is not defined in SQL, `_` will be used in its place (this `_` drives the need to match parameter types as token trees) for which `impl_sql` is expected to generate an appropriate generic type.

* `$` is a helper token that could be used to generate repetitions if generated artifacts are macros.

# Implementation Examples

As a picture is worth a thousand words before you start implementing your own `impl_sql` macro it would be advisable to review existing implementations like [include-postgres-sql][1] and [include-sqlite-sql][1], and maybe even use one of them as a starting point.

[1]: https://crates.io/crates/include-postgres-sql
[2]: https://crates.io/crates/include-sqlite-sql
[3]: https://docs.rs/syn/latest/syn/enum.Type.html
[4]: https://docs.rs/syn/latest/syn/macro.Token.html