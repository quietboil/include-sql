#![cfg_attr(docsrs, doc = include_str!("../README.md"))]

use proc_macro;
use syn::{ self, Token, parse::{ Parse, ParseStream } };
use proc_macro2::{
    TokenStream, Span, Group, Delimiter, Literal, Ident, Punct, Spacing
};
use quote::{ ToTokens, TokenStreamExt };

mod err;
mod sql;
mod gen;
mod conv;

/**
Reads and parses the specified SQL file, and generates `impl_sql` macro call.

For example, if the SQL file "library.sql" has these 2 statements:

```sql
-- name: get_loaned_books?
-- Returns the list of books loaned to a patron
-- # Parameters
-- param: user_id: &str - user ID
SELECT book_title FROM library WHERE loaned_to = :user_id ORDER BY 1;

-- name: loan_books
-- Updates the book records to reflect loan to a patron
-- # Parameters
-- param: user_id: &str - user ID
-- param: book_ids: usize - book IDs
UPDATE library SET loaned_to = :user_id, loaned_on = current_timestamp WHERE book_id IN ( :book_ids );
```

This method would generate:

```rust,no_run
# macro_rules! impl_sql { ($($t:tt)+) => {}; }
impl_sql!{ LibrarySql =
  {
    ? get_loaned_books (:user_id (&str))
    " Returns the list of books loaned to a patron\n # Parameters\n * `user_id` - user ID"
    $ "SELECT book_title FROM library WHERE loaned_to = " :user_id "ORDER BY 1"
  },
  {
    ! loan_books (:user_id (&str) #book_ids (usize))
    " Updates the book records to reflect loan to a patron\n # Parameters\n * `user_id` - user ID\n * `book_ids` - book IDs"
    $ "UPDATE library SET loaned_to = " :user_id ", loaned_on = current_timestamp WHERE book_id IN ( " #book_ids " )"
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
* `$` is a helper token that could be used to generate repetitions if generated artifacts are macros.

> **Note** that types are passed as parenthesized types. This is done to allow `impl_sql` match them as token trees. If a parameter type is not defined in SQL, `_` will be used in its place (this `_` drives the need to match parameter types as token trees) for which `impl_sql` is expected to generate an appropriate generic type.

[1]: https://crates.io/crates/include-postgres-sql
[2]: https://crates.io/crates/include-sqlite-sql
*/
#[proc_macro]
pub fn include_sql(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let file_path = syn::parse_macro_input!(input as syn::LitStr);
    let path = file_path.value();
    match read_and_parse_sql_file(&path) {
        Ok(included_sql) => {
            let mut tokens = TokenStream::new();
            output_include_bytes(&path, &mut tokens);
            if !included_sql.stmt_list.is_empty() {
                included_sql.to_tokens(&mut tokens);
            }
            tokens.into()
        }
        Err(err) => {
            syn::Error::new(file_path.span(), err.to_string()).to_compile_error().into()
        }
    }
}

/// Reads the content of the file at the `path` and parses its content.
fn read_and_parse_sql_file(file_path: &str) -> err::Result<sql::IncludedSql> {
    use std::path::PathBuf;
    use std::fs;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let mut path = PathBuf::from(&manifest_dir);
    path.push(file_path);
    let text = fs::read_to_string(&path)?;
    let file_name = path.file_stem().unwrap_or_default().to_str().unwrap_or_default().replace('-', "_");
    sql::parse(&text, &file_name)
}

/// Writes a phantom call to `include_bytes` to make compiler aware of the external dependency.
fn output_include_bytes(file_path: &str, tokens: &mut TokenStream) {
    use std::path::PathBuf;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let mut path = PathBuf::from(&manifest_dir);
    path.push(file_path);

    tokens.append(Ident::new("const", Span::call_site()));
    tokens.append(Ident::new("_", Span::call_site()));
    tokens.append(Punct::new(':', Spacing::Alone));
    tokens.append(Punct::new('&', Spacing::Alone));

    let mut type_tokens = TokenStream::new();
    type_tokens.append(Ident::new("u8", Span::call_site()));
    tokens.append(Group::new(Delimiter::Bracket, type_tokens));

    tokens.append(Punct::new('=', Spacing::Alone));
    tokens.append(Ident::new("include_bytes", Span::call_site()));
    tokens.append(Punct::new('!', Spacing::Alone));

    let mut macro_tokens = TokenStream::new();
    macro_tokens.append(Literal::string(path.to_str().unwrap()));
    tokens.append(Group::new(Delimiter::Parenthesis, macro_tokens));

    tokens.append(Punct::new(';', Spacing::Alone));
}

/**
Finds the specified item (`ident`) in a list (of `idents`).

Returns the item's index offset by the number after `+`.

```
let idx = include_sql::index_of!(id in [name, flag, id] + 1);
assert_eq!(idx, 3);
```
*/
#[proc_macro]
pub fn index_of(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let IndexOfArgs { param_name, start_index, stmt_params } = syn::parse_macro_input!(input as IndexOfArgs);
    let param_lookup = stmt_params.iter().position(|param| param == &param_name);
    if let Some( pos ) = param_lookup {
        let mut tokens = TokenStream::new();
        tokens.append(Literal::usize_unsuffixed(start_index + pos));
        tokens.into()
    } else {
        syn::Error::new(param_name.span(), "no such parameter").to_compile_error().into()
    }
}

struct IndexOfArgs {
    param_name : syn::Ident,
    stmt_params : syn::punctuated::Punctuated<syn::Ident, Token![,]>,
    start_index : usize,
}

impl Parse for IndexOfArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let param_name = input.parse()?;
        input.parse::<Token![in]>()?;
        let param_list;
        syn::bracketed!(param_list in input);
        input.parse::<Token![+]>()?;
        let start_index : syn::LitInt = input.parse()?;
        let start_index = start_index.base10_parse()?;
        let stmt_params = param_list.parse_terminated(syn::Ident::parse)?;
        Ok(Self { param_name, stmt_params, start_index })
    }
}

/**
Converts an `ident` into a camel-case `ident`.

Conversion uses the followng rules:
- First character is capitalized
- Underscores are removed
- Character that used to follow the removed underscore is capitalized
- Other charactes are left unchanged
*/
#[proc_macro]
pub fn to_camel_case(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let in_arg = syn::parse_macro_input!(input as syn::Ident);
    let in_name = in_arg.to_string();
    let out_name = conv::to_camel_case(&in_name);
    let mut tokens = TokenStream::new();
    tokens.append(Ident::new(&out_name, in_arg.span()));
    tokens.into()
}

/**
Converts an `ident` into a snake-case `ident`.

Conversion uses the followng rules:
- Leading and trailing underscores are removed
- All letters are lowered
- An underscore is inserted before a letter (except before the first one):
  - if it used to be an uppercase one and
  - if it does not already have an underscore in front of it
*/
#[proc_macro]
pub fn to_snake_case(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let in_arg = syn::parse_macro_input!(input as syn::Ident);
    let in_name = in_arg.to_string();
    let out_name = conv::to_snake_case(&in_name);
    let mut tokens = TokenStream::new();
    tokens.append(Ident::new(&out_name, in_arg.span()));
    tokens.into()
}
