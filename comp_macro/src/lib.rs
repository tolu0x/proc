use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream}, parse_macro_input, Token
};
use proc_macro2::TokenStream;

// comp: mapping for_if_clause+ (+ meaning one or more)
//
// mapping: expression
//
// for_if_clause:
//     | 'for' pattern 'in' expression ('if' expression)* (* meaning zero or more)
//
// pattern: name (, name)*

struct Proc {
    mapping: Mapping,
    for_if_clause: ForIfClause,
}

impl Parse for Proc {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            mapping: input.parse()?,
            for_if_clause: input.parse()?,
        })
    }
}

impl quote::ToTokens for Proc {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // core::iter::IntoIterator::into_iter(sequence).filter_map(move |pattern| {
        //      (true && ...).then(|| mapping)
        //})
        let Mapping(mapping) = &self.mapping;
        let ForIfClause{
            pattern,
            sequence,
            conditions,
        } = &self.for_if_clause;

        let conditions = conditions.iter().map(|c| {
            let inner = &c.0;
            quote! { #inner }
        });
        
        tokens.extend(quote!{
            core::iter::IntoIterator::into_iter(#sequence).filter_map(move |#pattern| {
                (true #(&& (#conditions))*).then(|| #mapping)
            })
        })
    }
}

struct Mapping(syn::Expr);

impl Parse for Mapping {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse()?))
    }
}

impl ToTokens for Mapping {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

struct ForIfClause {
    pattern: Pattern,
    sequence: syn::Expr,
    conditions: Vec<Condition>,
}

impl Parse for ForIfClause {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        _ = input.parse::<Token![for]>()?;
        let pattern = input.parse()?;
        _ = input.parse::<Token![in]>()?;
        let expression = input.parse()?;
        let conditions = parse_zero_or_more(input);

        Ok(Self {
            pattern,
            sequence: expression,
            conditions,
        })
    }
}

fn parse_zero_or_more<T: Parse>(input: ParseStream) -> Vec<T> {
    let mut result = Vec::new();
    while let Ok(item) = input.parse() {
        result.push(item);
    }
    result
}

struct Pattern(syn::Pat);

impl Parse for Pattern {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // syn::Pat::parse_single(input).map(Self)
        input.call(syn::Pat::parse_single).map(Self)
    }
}

impl ToTokens for Pattern {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

struct Condition(syn::Expr);

impl Parse for Condition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        _ = input.parse::<syn::Token![if]>()?;
        input.parse::<syn::Expr>().map(Self)
    }
}

#[proc_macro]
pub fn comp(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let c = parse_macro_input!(input as Proc);
    quote! { #c }.into()
}