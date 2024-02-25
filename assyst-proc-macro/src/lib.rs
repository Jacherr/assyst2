#![feature(let_chains)]

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned, ToTokens};
use syn::punctuated::Punctuated;
use syn::token::Bracket;
use syn::{parse_macro_input, Expr, ExprArray, ExprLit, FnArg, Ident, Item, Lit, LitStr, Meta, PatType, Token, Type};

struct CommandAttributes(syn::punctuated::Punctuated<syn::Meta, Token![,]>);

impl syn::parse::Parse for CommandAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse_terminated(syn::Meta::parse, Token![,])?))
    }
}

/// A proc macro applied to functions that will create a type that implements the `Command` trait.
/// In its `execute` method, it will call `parse` on all the parameter's types and finally forward
/// them to the annotated function:
///
/// ```ignore
/// #[command]
/// fn remind(ctxt: &mut CommandCtxt<'_>, time: Time, rest: Rest) {}
/// ```
///
/// becomes roughly...
///
/// ```ignore
/// struct remind_command;
///
/// impl Command for remind_command {
///     fn execute(&mut self, ctxt: &mut CommandCtxt<'_>) {
///         let p1 = Time::parse(ctxt)?;
///         let p2 = Rest::parse(ctxt)?;
///         remind(p1, p2)
///     }
/// }
///
/// fn remind(ctxt: &mut CommandCtxt<'_>, time: Time, rest: Rest) {}
/// ```
#[proc_macro_attribute]
pub fn command(attrs: TokenStream, func: TokenStream) -> TokenStream {
    let CommandAttributes(attrs) = syn::parse_macro_input!(attrs as CommandAttributes);

    let Item::Fn(item) = parse_macro_input!(func as syn::Item) else {
        panic!("#[command] applied to non-function")
    };

    let fn_name = &item.sig.ident;
    let struct_name = Ident::new(&format!("{}_command", item.sig.ident), Span::call_site());

    let mut fields = HashMap::new();

    for attr in attrs {
        match attr {
            Meta::NameValue(meta) => {
                let ident = meta
                    .path
                    .get_ident()
                    .expect("#[command] attribute key should be an identifier");

                fields.insert(ident.to_string(), meta.value);
            },
            _ => {},
        }
    }

    let mut parse_idents = Vec::new();
    let mut parse_exprs = Vec::new();

    // sanity check that the first parameter is the `ctxt`, and exclude it from the list of arguments
    // it wouldn't compile anyway since `CommandCtxt` can't be parsed as an argument (doesn't implement
    // the trait)
    // but this gives us a more useful error
    verify_input_is_ctxt(&item.sig.inputs);

    // used for sanity checking that `Rest` only ever appears as the last type
    let mut has_rest_ty = None;

    for (index, input) in item.sig.inputs.iter().skip(1).enumerate() {
        if let Some(span) = has_rest_ty {
            return quote_spanned!(span => compile_error!("`Rest` must be the last argument");).into();
        }

        match input {
            FnArg::Receiver(_) => panic!("#[command] cannot have `self` arguments"),
            FnArg::Typed(PatType { ty, .. }) => {
                if let Some(span) = is_rest_type(ty) {
                    has_rest_ty = Some(span);
                }

                parse_idents.push(Ident::new(&format!("p{index}"), Span::call_site()));
                parse_exprs.push(quote!(<#ty>::parse(&mut ctxt).await));
            },
        }
    }

    let name = fields.remove("name").unwrap_or_else(|| str_expr(&fn_name.to_string()));
    let aliases = fields.remove("aliases").unwrap_or_else(empty_array_expr);
    let description = fields.remove("description").expect("missing description");
    let cooldown = fields.remove("cooldown").expect("missing cooldown");
    let access = fields.remove("access").expect("missing access");
    let category = fields.remove("category").expect("missing category");
    let examples = fields.remove("examples").unwrap_or(empty_array_expr());
    let usage = fields.remove("usage").expect("missing usage");

    let following = quote::quote! {
        pub struct #struct_name;

        #[::async_trait::async_trait]
        impl crate::command::Command for #struct_name {
            fn metadata(&self) -> &'static crate::command::CommandMetadata {
                static META: crate::command::CommandMetadata = crate::command::CommandMetadata {
                    description: #description,
                    cooldown: #cooldown,
                    access: #access,
                    name: #name,
                    aliases: &#aliases,
                    category: #category,
                    examples: &#examples,
                    usage: #usage
                };
                &META
            }

            async fn execute(&self, mut ctxt: crate::command::CommandCtxt<'_>) -> Result<(), crate::command::ExecutionError> {
                use crate::command::arguments::ParseArgument;

                #(
                    let #parse_idents = #parse_exprs.map_err(crate::command::ExecutionError::Parse)?;
                )*

                #fn_name(ctxt, #(#parse_idents),*).await.map_err(crate::command::ExecutionError::Command)
            }
        }
    };

    let mut output = item.into_token_stream();
    output.extend(following);

    output.into()
}

fn is_rest_type(ty: &Type) -> Option<Span> {
    if let Type::Path(p) = ty
        && let Some(ident) = p.path.get_ident()
        && ident.to_string() == "Rest"
    {
        Some(ident.span())
    } else {
        None
    }
}

fn verify_input_is_ctxt(inputs: &Punctuated<FnArg, Token![,]>) {
    if let Some(FnArg::Typed(PatType { ty, .. })) = inputs.first()
        && let Type::Path(path) = &**ty
        && let Some(seg) = path.path.segments.last()
        && seg.ident == "CommandCtxt"
    {
        return;
    }

    panic!("first parameter of a #[command] annotated function should be the `CommandCtxt`");
}

fn str_expr(s: &str) -> Expr {
    Expr::Lit(ExprLit {
        attrs: Vec::new(),
        lit: Lit::Str(LitStr::new(s, Span::call_site()))
    })
}

fn empty_array_expr() -> Expr {
    Expr::Array(ExprArray {
        attrs: Default::default(),
        bracket_token: Bracket::default(),
        elems: Default::default()
    })
}
