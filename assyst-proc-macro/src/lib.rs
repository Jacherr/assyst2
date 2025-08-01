use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::token::{Bracket, Comma};
use syn::{
    Expr, ExprArray, ExprLit, FnArg, Ident, Item, Lit, LitBool, LitStr, Meta, Pat, PatType, Token, Type,
    parse_macro_input, parse_quote,
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

    let Item::Fn(mut item) = parse_macro_input!(func as syn::Item) else {
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
    let mut parse_attrs = Vec::new();
    let mut interaction_parse_exprs = Vec::new();
    let mut command_option_exprs = Vec::new();

    // sanity check that the first parameter is the `ctxt`, and exclude it from the list of
    // arguments it wouldn't compile anyway since `CommandCtxt` can't be parsed as an argument
    // (doesn't implement the trait)
    // but this gives us a more useful error
    verify_input_is_ctxt(&item.sig.inputs);

    for (index, input) in item.sig.inputs.iter_mut().skip(1).enumerate() {
        match input {
            FnArg::Receiver(_) => panic!("#[command] cannot have `self` arguments"),
            FnArg::Typed(PatType { ty, pat, attrs, .. }) => {
                if let Pat::Ident(ident) = &**pat {
                    let ident_string = ident.ident.to_string();

                    command_option_exprs.push(quote! {{
                            <#ty>::as_command_options(#ident_string)
                    }});

                    parse_attrs.push((ident_string, attrs.clone(), ty.clone()));
                }

                attrs.clear();
                parse_idents.push(Ident::new(&format!("p{index}"), Span::call_site()));
                parse_exprs.push(quote!(<#ty>::parse_raw_message(&mut ctxt, Some((stringify!(#pat).to_string(), stringify!(#ty).to_string()))).await));
                parse_usage.push(quote!(<#ty as crate::command::arguments::ParseArgument>::usage(stringify!(#pat))));
                interaction_parse_exprs.push(quote!(<#ty>::parse_command_option(&mut ctxt, Some((stringify!(#pat).to_string(), stringify!(#ty).to_string()))).await));
            },
        }
    }

    struct AutocompleteVisitable(bool);
    impl<'ast> syn::visit::Visit<'ast> for AutocompleteVisitable {
        fn visit_type(&mut self, i: &'ast syn::Type) {
            if let Type::Path(p) = i
                && let Some(seg) = p.path.segments.last()
                && seg.ident == "WordAutocomplete"
            {
                self.0 = true;
            } else {
                syn::visit::visit_type(self, i);
            }
        }
    }

    // collect stuff from argument attributes
    // add more here as required
    // todo: support parameter descriptions later
    let mut autocomplete_fns: Punctuated<proc_macro2::TokenStream, Comma> = Punctuated::new();

    for param in parse_attrs {
        use syn::visit::Visit;

        if param.1.is_empty() {
            let mut visitor = AutocompleteVisitable(false);
            visitor.visit_type(param.2.as_ref());

            assert!(
                !visitor.0,
                "autocomplete attr must be defined on WordAutocomplete arg type"
            );
        }

        for attr in param.1 {
            if let Meta::NameValue(n) = attr.meta.clone()
                && let Some(s) = n.path.segments.first()
            {
                if s.ident == "autocomplete" {
                    if let Expr::Lit(ref l) = n.value
                        && let Lit::Str(ref s) = l.lit
                    {
                        let mut visitor = AutocompleteVisitable(false);
                        visitor.visit_type(param.2.as_ref());

                        if visitor.0 {
                            let path = s.parse::<syn::Path>().expect("autocomplete: invalid path");
                            let arg = param.0.clone();
                            autocomplete_fns.push(quote::quote!(#arg => #path(assyst, data).await));
                        } else {
                            panic!("autocomplete attr is only valid on WordAutocomplete arg type");
                        }
                    } else {
                        panic!("autocomplete: invalid value ({:?})", n.value);
                    }
                } else {
                    panic!("fn arg attr: invalid name ({:?})", s.ident.to_string());
                }
            } else if let Meta::Path(p) = attr.meta {
                // add any value-less attrs here
                panic!(
                    "fn arg attr: invalid attr ({:?})",
                    p.get_ident().map(std::string::ToString::to_string)
                );
            }
        }
    }

    autocomplete_fns
        .push(quote::quote!(_ => panic!("unhandled autocomplete arg name {arg_name:?} for command {}", meta.name)));

    let name = fields.remove("name").unwrap_or_else(|| str_expr(&fn_name.to_string()));
    let aliases = fields.remove("aliases").unwrap_or_else(empty_array_expr);
    let description = fields.remove("description").expect("missing description");
    let cooldown = fields.remove("cooldown").expect("missing cooldown");
    let access = fields.remove("access").expect("missing access");
    let category = fields.remove("category").expect("missing category");
    let examples = fields.remove("examples").unwrap_or_else(empty_array_expr);
    let usage: Expr = fields
        .remove("usage")
        .map(|v| parse_quote!(String::from(#v)))
        .unwrap_or_else(|| {
            parse_quote! {{
                let _v: Vec<String> = vec![#(#parse_usage),*];
                _v.join(" ")
            }}
        });
    let send_processing = fields.remove("send_processing").unwrap_or_else(false_expr);
    let age_restricted = fields.remove("age_restricted").unwrap_or_else(false_expr);
    let context_menu_message_command = fields
        .remove("context_menu_message_command")
        .unwrap_or_else(|| str_expr(""));
    let context_menu_user_command = fields
        .remove("context_menu_user_command")
        .unwrap_or_else(|| str_expr(""));
    let group_parent_name = fields.remove("group_parent_name").unwrap_or_else(|| str_expr(""));

    assert!(
        !(context_menu_message_command != str_expr("") && context_menu_user_command != str_expr("")),
        "command cannot be a context message and user command at the same time"
    );

    let flag_descriptions = fields.remove("flag_descriptions").unwrap_or_else(empty_array_expr);
    let guild_only = fields.remove("guild_only").unwrap_or_else(false_expr);

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
                    flag_descriptions: descriptions,
                    context_menu_message_command: #context_menu_message_command,
                    context_menu_user_command: #context_menu_user_command,
                    guild_only: #guild_only,
                    group_parent_name: #group_parent_name
                })
            }

            fn subcommands(&self) -> Option<&'static [(&'static str, crate::command::TCommand)]> {
                None
            }

            fn as_interaction_command(&self) -> twilight_model::application::command::Command {
                let meta = self.metadata();
                let options = if let crate::command::CommandGroupingInteractionInfo::Command(x) = self.interaction_info() {
                    let mut ops = x.command_options;
                    ops.sort_by(|x, y| y.required.cmp(&x.required));
                    ops
                } else {
                    unreachable!()
                };

                twilight_model::application::command::Command {
                    application_id: None,
                    default_member_permissions: None,
                    description: meta.description.to_owned(),
                    description_localizations: None,
                    dm_permission: Some(true),
                    guild_id: None,
                    id: None,
                    kind: twilight_model::application::command::CommandType::ChatInput,
                    name: meta.name.to_owned(),
                    name_localizations: None,
                    nsfw: Some(meta.age_restricted),
                    options,
                    version: twilight_model::id::Id::new(1),
                    contexts: Some(if meta.guild_only {
                            vec![
                                twilight_model::application::interaction::InteractionContextType::Guild,
                            ]
                        } else {
                            vec![
                                twilight_model::application::interaction::InteractionContextType::Guild,
                                twilight_model::application::interaction::InteractionContextType::BotDm,
                                twilight_model::application::interaction::InteractionContextType::PrivateChannel
                            ]
                        }),
                    integration_types: Some(if meta.guild_only {
                            vec![
                                twilight_model::oauth::ApplicationIntegrationType::GuildInstall
                            ]
                        } else {
                            vec![
                                twilight_model::oauth::ApplicationIntegrationType::GuildInstall,
                                twilight_model::oauth::ApplicationIntegrationType::UserInstall
                            ]
                    })
                }
            }

            fn interaction_info(&self) -> crate::command::CommandGroupingInteractionInfo {
                use crate::command::arguments::ParseArgument;

                let mut command_options = Vec::new();
                #(
                  command_options.extend(#command_option_exprs);
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

            #[allow(unreachable_code)]
            async fn arg_autocomplete(
                &self,
                assyst: crate::assyst::ThreadSafeAssyst,
                arg_name: String,
                user_input: String,
                data: crate::command::autocomplete::AutocompleteData
            ) -> Result<Vec<twilight_model::application::command::CommandOptionChoice>, crate::command::ExecutionError> {
                let meta = self.metadata();

                let options: Vec<String> = match arg_name.as_str() {
                    #autocomplete_fns
                };

                let choices: Vec<twilight_model::application::command::CommandOptionChoice> = options
                    .iter()
                    .filter(|x| {
                        x.to_ascii_lowercase()
                            .starts_with(&user_input.to_ascii_lowercase())
                    })
                    .take(crate::command::autocomplete::SUGG_LIMIT)
                    .map(|x| twilight_model::application::command::CommandOptionChoice {
                        name: x.clone(),
                        name_localizations: None,
                        // FIXME: hardcoded string type
                        value: twilight_model::application::command::CommandOptionChoiceValue::String(x.clone()),
                    })
                    .collect::<Vec<twilight_model::application::command::CommandOptionChoice>>();

                Ok(choices)
            }
        }
    };

    let mut output = item.into_token_stream();
    output.extend(following);

    //panic!("{output}");

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
