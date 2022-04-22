use proc_macro2::{TokenStream, Span, Group, Delimiter, Literal, Ident, Punct, Spacing};
use quote::{ToTokens, TokenStreamExt};
use crate::sql::{IncludedSql, Stmt, StmtItem};
use crate::conv::StringExt;

impl ToTokens for IncludedSql {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new("impl_sql", Span::call_site()));
        tokens.append(Punct::new('!', Spacing::Alone));

        let mut macro_args = TokenStream::new();
        let mut name = self.file_name.to_camel_case(); // crate::conv::to_camel_case(&self.file_name);
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
                StmtItem::Bind(name) => {
                    stmt_params.append(Punct::new(':', Spacing::Alone));
                    stmt_params.append(Ident::new(name, Span::call_site()));
                    let type_tree = self.params.get(name)
                        .and_then(|type_name| syn::parse_str::<syn::Type>(type_name).ok())
                        .map(|param_type| {
                            let mut type_tokens = TokenStream::new();
                            #[cfg(feature = "async")]
                            if let syn::Type::Reference(_) = &param_type {
                                lifetime(name).to_tokens(&mut type_tokens);
                            }
                            param_type.to_tokens(&mut type_tokens);
                            Group::new(Delimiter::Parenthesis, type_tokens)
                        })
                    ;
                    if let Some(tt) = type_tree {
                        stmt_params.append(tt);
                    } else {
                        stmt_params.append(Ident::new("_", Span::call_site()));
                    }
                },
                StmtItem::List(name) => {
                    stmt_params.append(Punct::new('#', Spacing::Alone));
                    stmt_params.append(Ident::new(name, Span::call_site()));
                    let type_tree = self.params.get(name)
                        .and_then(|type_name| syn::parse_str::<syn::Type>(type_name).ok())
                        .map(|param_type| {
                            let mut type_tokens = TokenStream::new();                            
                            #[cfg(feature = "async")] {
                                lifetime(name).to_tokens(&mut type_tokens);
                                if let syn::Type::Reference(_) = &param_type {
                                    let mut item = String::with_capacity(name.len() + 5);
                                    item.push_str(name);
                                    item.push_str("_item");
                                    lifetime(&item).to_tokens(&mut type_tokens);
                                }
                            }
                            param_type.to_tokens(&mut type_tokens);
                            Group::new(Delimiter::Parenthesis, type_tokens)
                        })
                    ;
                    if let Some(tt) = type_tree {
                        stmt_params.append(tt);
                    } else {
                        let type_name = name.to_camel_case();
                        let mut type_tokens = TokenStream::new();
                        #[cfg(feature = "async")]
                        lifetime(name).to_tokens(&mut type_tokens);
                        type_tokens.append(Ident::new(&type_name, Span::call_site()));
                        stmt_params.append(Group::new(Delimiter::Bracket, type_tokens));
                    }
                },
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

#[cfg(feature = "async")]
fn lifetime(name: &str) -> syn::Lifetime {
    syn::Lifetime { apostrophe: Span::call_site(), ident: Ident::new(name, Span::call_site()) }
}