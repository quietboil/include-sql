use proc_macro2::{TokenStream, Span, Group, Delimiter, Literal, Ident, Punct, Spacing};
use quote::{ToTokens, TokenStreamExt};
use crate::sql::{IncludedSql, Stmt, StmtItem};

impl ToTokens for IncludedSql {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new("impl_sql", Span::call_site()));
        tokens.append(Punct::new('!', Spacing::Alone));

        let mut macro_args = TokenStream::new();
        let mut name = crate::conv::to_camel_case(&self.file_name);
        name.push_str("Sql");
        macro_args.append(Ident::new(&name, Span::call_site()));
        macro_args.append(Punct::new('=', Spacing::Alone));

        macro_args.append_separated(&self.stmt_list, Punct::new(',', Spacing::Alone));

        tokens.append(Group::new(Delimiter::Brace, macro_args));
    }
}

impl ToTokens for Stmt {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut stmt_tokens = TokenStream::new();

        let last = self.into.len() - 1;
        for (i, ch) in self.into.chars().enumerate() {
            let spacing = if i < last { Spacing::Joint } else { Spacing::Alone };
            stmt_tokens.append(Punct::new(ch, spacing));
        }

        stmt_tokens.append(Ident::new(&self.name, Span::call_site()));

        let mut stmt_params = TokenStream::new();
        for param in self.unique_binds() {
            match param {
                StmtItem::Bind(_) => {
                    stmt_params.append(Punct::new(':', Spacing::Alone));
                },
                StmtItem::List(_) => {
                    stmt_params.append(Punct::new('#', Spacing::Alone));
                },
                _ => {},
            }
            match param {
                StmtItem::Bind(name) | StmtItem::List(name) => {
                    stmt_params.append(Ident::new(name, Span::call_site()));

                    if let Some(param_type) = self.params.get(name) {
                        if let Ok(param_type) = syn::parse_str::<syn::Type>(param_type) {
                            let mut type_tokens = TokenStream::new();
                            param_type.to_tokens(&mut type_tokens);
                            stmt_params.append(Group::new(Delimiter::Parenthesis, type_tokens));
                        } else {
                            stmt_params.append(Ident::new("_", Span::call_site()));
                        }
                    } else {
                        stmt_params.append(Ident::new("_", Span::call_site()));
                    }
                }
                _ => {},
            }
        }
        stmt_tokens.append(Group::new(Delimiter::Parenthesis, stmt_params));

        if let Some(doc_comment) = self.docs.as_ref() {
            stmt_tokens.append(Literal::string(doc_comment));
        } else {
            stmt_tokens.append(Literal::string(""));
        }
        stmt_tokens.append(Punct::new('$', Spacing::Alone));
        stmt_tokens.append_all(&self.items);

        tokens.append(Group::new(Delimiter::Brace, stmt_tokens));
    }
}

impl ToTokens for StmtItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Text( text ) => {
                tokens.append(Literal::string(text));
            },
            Self::Bind( name ) => {
                tokens.append(Punct::new(':', Spacing::Alone));
                tokens.append(Ident::new(name, Span::call_site()));
            },
            Self::List( name ) => {
                tokens.append(Punct::new('#', Spacing::Alone));
                tokens.append(Ident::new(name, Span::call_site()));
            }
        }
    }
}
