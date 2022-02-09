#![cfg_attr(docsrs, doc = include_str!("../README.md"))]

pub use include_sql::include_sql;

/// Generates Rust code to use included SQL.
#[macro_export]
macro_rules! impl_sql {
    ( $sql_name:ident = $( { $kind:tt $name:ident ($($variant:tt $param:ident $ptype:tt)*) $doc:literal $s:tt $( $text:tt )+ } ),+ ) => {
        trait $sql_name {
            $( $crate::decl_method!{ $kind $name $doc () () $($param $variant $ptype)* } )+
        }
        impl $sql_name for postgres::Client {
            $( $crate::impl_method!{ $kind $name () () ($($param $variant $ptype)*) => ($($variant $param)*) $($text)+ } )+
        }
        impl $sql_name for postgres::Transaction<'_> {
            $( $crate::impl_method!{ $kind $name () () ($($param $variant $ptype)*) => ($($variant $param)*) $($text)+ } )+
        }
    };
}

#[macro_export]
macro_rules! decl_method {
    ( ? $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<$($gen_type : postgres::types::ToSql ,)* F>(&mut self $($fn_params)* , row_cb: F) -> std::result::Result<(),postgres::Error>
        where F: Fn(postgres::Row) -> std::result::Result<(),postgres::Error>;
    };
    ( ! $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<$($gen_type : postgres::types::ToSql),*>(&mut self $($fn_params)*) -> std::result::Result<u64,postgres::Error>;
    };
    ( -> $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) ) => {
        #[doc=$doc]
        fn $name<$($gen_type : postgres::types::ToSql),*>(&mut self $($fn_params)*) -> std::result::Result<postgres::Row,postgres::Error>;
    };
    ( $kind:tt $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) $param:ident : _ $($tail:tt)* ) => {
        $crate::decl_method!{
            $kind
            $name
            $doc
            ($($gen_type)*)
            ($($fn_params)* , $param : impl postgres::types::ToSql + Sync)
            $($tail)*
        }
    };
    ( $kind:tt $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) $param:ident : ($ptype:ty) $($tail:tt)* ) => {
        $crate::decl_method!{
            $kind
            $name
            $doc
            ($($gen_type)*)
            ($($fn_params)* , $param : $ptype)
            $($tail)*
        }
    };
    ( $kind:tt $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) $param:ident # _ $($tail:tt)* ) => {
        $crate::decl_method!{
            $kind
            $name
            $doc
            ($($gen_type)* include_sql::to_camel_case!($param))
            ($($fn_params)* , $param : & [ include_sql::to_camel_case!($param) ] )
            $($tail)*
        }
    };
    ( $kind:tt $name:ident $doc:literal ($($gen_type:ident)*) ($($fn_params:tt)*) $param:ident # ($ptype:ty) $($tail:tt)* ) => {
        $crate::decl_method!{
            $kind
            $name
            $doc
            ($($gen_type)*)
            ($($fn_params)* , $param : & [ $ptype ] )
            $($tail)*
        }
    };
}

#[macro_export]
macro_rules! impl_method {
    ( ? $name:ident () () () => () $text:literal ) => {
        fn $name<F>(&mut self, row_cb: F) -> std::result::Result<(),postgres::Error>
        where F: Fn(postgres::Row) -> std::result::Result<(),postgres::Error>
        {
            use postgres::fallible_iterator::FallibleIterator;

            let mut rows = self.query_raw( $text, [] as [&dyn postgres::types::ToSql; 0] )?;
            while let Some(row) = rows.next()? {
                row_cb(row)?;
            }
            Ok(())
        }
    };
    ( ? $name:ident () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name<F>(&mut self $($fn_params)+ , row_cb: F) -> std::result::Result<(),postgres::Error>
        where F: Fn(postgres::Row) -> std::result::Result<(),postgres::Error>
        {
            use postgres::fallible_iterator::FallibleIterator;

            let mut rows = self.query_raw(
                $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                [& $head as &(dyn postgres::types::ToSql + Sync) $(, & $tail)* ]
            )?;
            while let Some(row) = rows.next()? {
                row_cb(row)?;
            }
            Ok(())
        }
    };
    ( ? $name:ident ($($gen_type:ident)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<$($gen_type : postgres::types::ToSql ,)* F>(&mut self $($fn_params)+, row_cb: F) -> std::result::Result<(),postgres::Error>
        where F: Fn(postgres::Row) -> std::result::Result<(),postgres::Error>
        {
            use postgres::fallible_iterator::FallibleIterator;

            let mut stmt = String::with_capacity($crate::sql_len!($($text)+));
            let mut args = Vec::<&dyn postgres::types::ToSql>::with_capacity($crate::num_args!($($pv $param)+));
            let mut i = 0;
            $crate::dynamic_sql!(stmt args i $($text)+);
            let mut rows = self.query_raw(&stmt, args)?;
            while let Some(row) = rows.next()? {
                row_cb(row)?;
            }
            Ok(())
        }
    };
    ( ! $name:ident () () () => () $text:literal ) => {
        fn $name(&mut self) -> std::result::Result<u64,postgres::Error> {
            self.execute( $text, &[] )
        }
    };
    ( ! $name:ident () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name(&mut self $($fn_params)+ ) -> std::result::Result<u64,postgres::Error> {
            self.execute(
                $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                &[& $head as &(dyn postgres::types::ToSql + Sync) $(, & $tail)* ]
            )
        }
    };
    ( ! $name:ident ($($gen_type:ident)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<$($gen_type : postgres::types::ToSql),*>(&mut self $($fn_params)+ ) -> std::result::Result<u64,postgres::Error> {
            let mut stmt = String::with_capacity($crate::sql_len!($($text)+));
            let mut args = Vec::<&dyn postgres::types::ToSql>::with_capacity($crate::num_args!($($pv $param)+));
            let mut i = 0;
            $crate::dynamic_sql!(stmt args i $($text)+);
            self.execute(&stmt, &args)
        }
    };
    ( -> $name:ident () () () => () $text:literal ) => {
        fn $name(&mut self) -> std::result::Result<postgres::Row,postgres::Error> {
            self.query_one( $text, &[] )
        }
    };
    ( -> $name:ident () ($($fn_params:tt)+) () => (: $head:ident $(: $tail:ident)*) $($text:tt)+) => {
        fn $name(&mut self $($fn_params)+ ) -> std::result::Result<postgres::Row,postgres::Error> {
            self.query_one(
                $crate::sql_literal!( $head $($tail)* => $($text)+ ) ,
                &[& $head as &(dyn postgres::types::ToSql + Sync) $(, & $tail)* ]
            )
        }
    };
    ( -> $name:ident ($($gen_type:ident)*) ($($fn_params:tt)+) () => ($($pv:tt $param:ident)+) $($text:tt)+) => {
        fn $name<$($gen_type : postgres::types::ToSql),*>(&mut self $($fn_params)+ ) -> std::result::Result<postgres::Row,postgres::Error> {
            let mut stmt = String::with_capacity($crate::sql_len!($($text)+));
            let mut args = Vec::<&dyn postgres::types::ToSql>::with_capacity($crate::num_args!($($pv $param)+));
            let mut i = 0;
            $crate::dynamic_sql!(stmt args i $($text)+);
            self.query_one(&stmt, &args)
        }
    };
    ( $kind:tt $name:ident ($($gen_type:ident)*) ($($fn_params:tt)*) ($param:ident : _ $($tail:tt)*) => ($($pv:tt $param_name:ident)+) $($text:tt)+)  => {
        $crate::impl_method!{
            $kind
            $name
            ($($gen_type)*)
            ($($fn_params)* , $param : impl postgres::types::ToSql + Sync)
            ($($tail)*)
            =>
            ($($pv $param_name)+)
            $($text)+
        }
    };
    ( $kind:tt $name:ident ($($gen_type:ident)*) ($($fn_params:tt)*) ($param:ident : ($ptype:ty) $($tail:tt)*) => ($($pv:tt $param_name:ident)+) $($text:tt)+)  => {
        $crate::impl_method!{
            $kind
            $name
            ($($gen_type)*)
            ($($fn_params)* , $param : $ptype)
            ($($tail)*)
            =>
            ($($pv $param_name)+)
            $($text)+
        }
    };
    ( $kind:tt $name:ident ($($gen_type:ident)*) ($($fn_params:tt)*) ($param:ident # _ $($tail:tt)*) => ($($pv:tt $param_name:ident)+) $($text:tt)+)  => {
        $crate::impl_method!{
            $kind
            $name
            ($($gen_type)* include_sql::to_camel_case!($param))
            ($($fn_params)* , $param : & [ include_sql::to_camel_case!($param) ])
            ($($tail)*)
            =>
            ($($pv $param_name)+)
            $($text)+
        }
    };
    ( $kind:tt $name:ident ($($gen_type:ident)*) ($($fn_params:tt)*) ($param:ident # ($ptype:ty) $($tail:tt)*) => ($($pv:tt $param_name:ident)+) $($text:tt)+)  => {
        $crate::impl_method!{
            $kind
            $name
            ($($gen_type)*)
            ($($fn_params)* , $param : & [ $ptype ])
            ($($tail)*)
            =>
            ($($pv $param_name)+)
            $($text)+
        }
    };
}

#[macro_export]
macro_rules! sql_literal {
    ($($name:ident)+ => $text:literal) => {
        $text
    };
    ($($name:ident)+ => $text:literal : $param:ident) => {
        std::concat!( $text, '$', include_sql::index_of!($param in [ $( $name ),+ ] + 1) )
    };
    ($($name:ident)+ => $text:literal : $param:ident $($tail:tt)+) => {
        std::concat!(
            $text, '$', include_sql::index_of!($param in [ $( $name ),+ ] + 1),
            $crate::sql_literal!($($name)+ => $($tail)+)
        )
    };
}

#[macro_export]
macro_rules! num_args {
    () => { 0 };
    (: $head:ident $($tail:tt)*) => { 1 + $crate::num_args!($($tail)*) };
    (# $head:ident $($tail:tt)*) => { $head.len() + $crate::num_args!($($tail)*) };
}

#[macro_export]
macro_rules! sql_len {
    () => { 0 };
    ($text:literal $($tail:tt)*) => { $text.len() + $crate::sql_len!($($tail)*) };
    (: $head:ident $($tail:tt)*) => { 3 + $crate::sql_len!($($tail)*) };
    (# $head:ident $($tail:tt)*) => { $head.len() * 5 + $crate::sql_len!($($tail)*) };
}

#[macro_export]
macro_rules! dynamic_sql {
    ($stmt:ident $args:ident $i:ident) => {};
    ($stmt:ident $args:ident $i:ident $text:literal $($tail:tt)*) => {
        $stmt.push_str($text);
        $crate::dynamic_sql!($stmt $args $i $($tail)*);
    };
    ($stmt:ident $args:ident $i:ident : $param:ident $($tail:tt)*) => {
        $i += 1;
        $stmt.push_str(&format!("${}", $i));
        $args.push(&$param);
        $crate::dynamic_sql!($stmt $args $i $($tail)*);
    };
    ($stmt:ident $args:ident $i:ident # $param:ident $($tail:tt)*) => {
        let mut iter = $param.into_iter();
        if let Some(arg) = iter.next() {
            $i += 1;
            $stmt.push_str(&format!("${}", $i));
            $args.push(arg);
            while let Some(arg) = iter.next() {
                $i += 1;
                $stmt.push_str(&format!(", ${}", $i));
                $args.push(arg);
            }
        } else {
            $stmt.push_str("NULL");
        }
        $crate::dynamic_sql!($stmt $args $i $($tail)*);
    };
}
