//! This crate exports items that [include-sql](https://github.com/quietboil/include-sql), being a
//! proc-macro library, cannot export.

/// Pushes a list of values into the query argument list.
/// 
/// This is a helper function that `include-sql` uses when it generates `into_sql_with_args`.
/// 
pub fn push<'a,T: ?Sized>(arg: &[&'a T], param_prefix: &str, sql: &mut String, args: &mut Vec<&'a T>) {
    let start = args.len() + 1;
    let end = start + arg.len();
    let range = start..end;
    let mut iter = range.into_iter();
    if let Some( n ) = iter.next() {
        sql.push_str(param_prefix);
        sql.push_str(&n.to_string());
        for n in iter {
            sql.push(',');
            sql.push_str(param_prefix);
            sql.push_str(&n.to_string());
        }
    }
    args.extend_from_slice(arg);
}

/// Generates a macro that convers an argument struct into a slice that can be passed to
/// database interfaces that require the latter.
/// 
#[macro_export]
macro_rules! def_args {
    ($s:tt => $macro_name:ident : $args_struct:ident = $($field:ident),+) => {
        #[allow(unused_macros)]
        macro_rules! $macro_name {
            ($s($s name:ident : $s value:expr),+) => {{
                let args = $args_struct { $s( $s name : $s value ),+ };
                &[ $(args.$field),+ ]
            }};
        }
    };
}
