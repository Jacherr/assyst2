#![feature(let_chains)]

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::token::Bracket;
use syn::{
    parse_macro_input, parse_quote, Expr, ExprArray, ExprLit, FnArg, Ident, Item, Lit, LitBool, LitStr, Meta, Pat, PatType, Token, Type
};

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
///         check_metadata()?;
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
        if let Meta::NameValue(meta) = attr {
            let ident = meta
                .path
                .get_ident()
                .expect("#[command] attribute key should be an identifier");

            fields.insert(ident.to_string(), meta.value);
        }
    }

    let mut parse_idents = Vec::new();
    let mut parse_exprs = Vec::new();
    let mut parse_usage = Vec::new();
    let mut interaction_parse_exprs = Vec::new();
    let mut command_option_exprs = Vec::new();

    // sanity check that the first parameter is the `ctxt`, and exclude it from the list of arguments
    // it wouldn't compile anyway since `CommandCtxt` can't be parsed as an argument (doesn't implement
    // the trait)
    // but this gives us a more useful error
    verify_input_is_ctxt(&item.sig.inputs);

    for (index, input) in item.sig.inputs.iter().skip(1).enumerate() {
        match input {
            FnArg::Receiver(_) => panic!("#[command] cannot have `self` arguments"),
            FnArg::Typed(PatType { ty, pat, .. }) => {
                if let Pat::Ident(ident) = &**pat {
                    let ident_string = ident.ident.to_string();

                    command_option_exprs.push(quote! {{
                            <#ty>::as_command_option(#ident_string)
                    }});
                }

                parse_idents.push(Ident::new(&format!("p{index}"), Span::call_site()));
                parse_exprs.push(quote!(<#ty>::parse_raw_message(&mut ctxt, Some((stringify!(#pat).to_string(), stringify!(#ty).to_string()))).await));
                parse_usage.push(quote!(<#ty as crate::command::arguments::ParseArgument>::usage(stringify!(#pat))));
                interaction_parse_exprs.push(quote!(<#ty>::parse_command_option(&mut ctxt).await));
            },
        }
    }

    let name = fields.remove("name").unwrap_or_else(|| str_expr(&fn_name.to_string()));
    let aliases = fields.remove("aliases").unwrap_or_else(empty_array_expr);
    let description = fields.remove("description").expect("missing description");
    let cooldown = fields.remove("cooldown").expect("missing cooldown");
    let access = fields.remove("access").expect("missing access");
    let category = fields.remove("category").expect("missing category");
    let examples = fields.remove("examples").unwrap_or_else(empty_array_expr);
    let usage: Expr = fields.remove("usage").map(|v| parse_quote!(String::from(#v))).unwrap_or_else(|| {
        parse_quote! {{
            let _v: Vec<String> = vec![#(#parse_usage),*];
            _v.join(" ")
        }}
    });
    let send_processing = fields.remove("send_processing").unwrap_or_else(false_expr);
    let age_restricted = fields.remove("age_restricted").unwrap_or_else(false_expr);
    let flag_descriptions = fields.remove("flag_descriptions").unwrap_or_else(empty_array_expr);

    let following = quote::quote! {
        #[allow(non_camel_case_types)]
        pub struct #struct_name;

        #[::async_trait::async_trait]
        impl crate::command::Command for #struct_name {
            fn metadata(&self) -> &'static crate::command::CommandMetadata {
                use std::collections::HashMap;
                let mut descriptions = HashMap::new();
                for (k, v) in #flag_descriptions {
                    descriptions.insert(k, v);
                }

                static META: std::sync::OnceLock<crate::command::CommandMetadata> = std::sync::OnceLock::new();
                META.get_or_init(|| crate::command::CommandMetadata {
                    description: #description,
                    cooldown: #cooldown,
                    access: #access,
                    name: #name,
                    aliases: &#aliases,
                    category: #category,
                    examples: &#examples,
                    usage: format!("{}", #usage),
                    send_processing: #send_processing,
                    age_restricted: #age_restricted,
                    flag_descriptions: descriptions
                })
            }

            fn subcommands(&self) -> Option<&'static [(&'static str, crate::command::TCommand)]> {
                None
            }

            fn as_interaction_command(&self) -> twilight_model::application::command::Command {
                let meta = self.metadata();
                let options = if let crate::command::CommandGroupingInteractionInfo::Command(x) = self.interaction_info() {
                    x.command_options
                } else {
                    unreachable!()
                };

                twilight_model::application::command::Command {
                    application_id: None,
                    default_member_permissions: None,
                    description: meta.description.to_owned(),
                    description_localizations: None,
                    // TODO: set based on if dms are allowed
                    // TODO: update to `contexts` once this is required
                    // (see https://discord.com/developers/docs/interactions/application-commands#create-global-application-command)
                    dm_permission: Some(false),
                    guild_id: None,
                    id: None,
                    kind: twilight_model::application::command::CommandType::ChatInput,
                    name: meta.name.to_owned(),
                    name_localizations: None,
                    nsfw: Some(meta.age_restricted),
                    options,
                    version: twilight_model::id::Id::new(1),
                }
            }

            fn interaction_info(&self) -> crate::command::CommandGroupingInteractionInfo {
                use crate::command::arguments::ParseArgument;

                let mut command_options = Vec::new();
                #(
                  command_options.push(#command_option_exprs);
                )*

                let command_info = crate::command::CommandInteractionInfo { command_options };
                crate::command::CommandGroupingInteractionInfo::Command(command_info)
            }

            async fn execute_raw_message(
                &self,
                mut ctxt:
                crate::command::RawMessageParseCtxt<'_>
            ) -> Result<(), crate::command::ExecutionError> {
                use crate::command::arguments::ParseArgument;

                crate::command::check_metadata(self.metadata(), &mut ctxt.cx).await?;

                #(
                    let #parse_idents = #parse_exprs.map_err(crate::command::ExecutionError::Parse)?;
                )*

                #fn_name(ctxt.cx, #(#parse_idents),*).await.map_err(crate::command::ExecutionError::Command)
            }

            async fn execute_interaction_command(
                &self,
                mut ctxt:
                crate::command::InteractionCommandParseCtxt<'_>
            ) -> Result<(), crate::command::ExecutionError> {
                use crate::command::arguments::ParseArgument;

                crate::command::check_metadata(self.metadata(), &mut ctxt.cx).await?;

                #(
                    let #parse_idents = #interaction_parse_exprs.map_err(crate::command::ExecutionError::Parse)?;
                )*

                #fn_name(ctxt.cx, #(#parse_idents),*).await.map_err(crate::command::ExecutionError::Command)
            }
        }
    };

    let mut output = item.into_token_stream();
    output.extend(following);

    //panic!("{}", output);

    output.into()
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
        lit: Lit::Str(LitStr::new(s, Span::call_site())),
    })
}

fn empty_array_expr() -> Expr {
    Expr::Array(ExprArray {
        attrs: Default::default(),
        bracket_token: Bracket::default(),
        elems: Default::default(),
    })
}

fn false_expr() -> Expr {
    Expr::Lit(ExprLit {
        attrs: Vec::new(),
        lit: Lit::Bool(LitBool::new(false, Span::call_site())),
    })
}
