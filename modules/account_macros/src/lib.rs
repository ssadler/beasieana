

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{braced, parse_macro_input, Ident, Lifetime, Token};


struct MacroCalls {
    name: Ident,
    lifetime: Lifetime,
    calls: Vec<proc_macro2::TokenStream>,
}

impl Parse for MacroCalls {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let lifetime: Lifetime = input.parse()?;
        input.parse::<Token![,]>()?;
        let content;
        braced!(content in input);
        let calls = content
            .parse_terminated(proc_macro2::TokenStream::parse, Token![,])?
            .into_iter()
            .collect();
        Ok(MacroCalls { name, lifetime, calls })
    }
}

#[proc_macro]
pub fn my_accounts(input: TokenStream) -> TokenStream {
    let MacroCalls { name, lifetime, calls } = parse_macro_input!(input as MacroCalls);
    
    let expanded = quote! {
        #[derive(Accounts)]
        #[instruction(token: Pubkey)]
        pub struct #name<#lifetime> {
            #(#calls)*
        }
    };
    
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn global(input: TokenStream) -> TokenStream {
    let lifetime = parse_macro_input!(input as Lifetime);
    TokenStream::from(quote! {
        #[account(init_if_needed, payer = payer, space = 4096, seeds = [b"grid"], bump)]
        pub global: Account<#lifetime, Global>,
    })
}

#[proc_macro]
pub fn payer(_input: TokenStream) -> TokenStream {
    TokenStream::from(quote! {
        #[account(mut)]
        pub payer: Signer<'info>
    })
}

#[proc_macro]
pub fn system_program(_input: TokenStream) -> TokenStream {
    TokenStream::from(quote! {
        pub system_program: Program<'info, System>
    })
}
