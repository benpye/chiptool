use anyhow::Result;
use proc_macro2::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;

use crate::ir::*;
use crate::util;

use super::sorted;

pub fn render(_opts: &super::Options, _ir: &IR, e: &Enum, path: &str) -> Result<TokenStream> {
    let span = Span::call_site();

    let ty = match e.bit_size {
        1..=8 => quote!(u8),
        9..=16 => quote!(u16),
        17..=32 => quote!(u32),
        33..=64 => quote!(u64),
        _ => panic!("Invalid bit_size {}", e.bit_size),
    };

    let (_, name) = super::split_path(path);
    let name = Ident::new(name, span);
    let doc = util::doc(&e.description);
    let mask = util::hex(1u64.wrapping_shl(e.bit_size).wrapping_sub(1));

    let mut out = TokenStream::new();
    let mut items = TokenStream::new();

    for f in sorted(&e.variants, |f| (f.value, f.name.clone())) {
        let name = Ident::new(&f.name, span);
        let value = util::hex(f.value);
        let doc = util::doc(&f.description);
        items.extend(quote!(
            #doc
            pub const #name: Self = Self(#value);
        ));
    }

    out.extend(quote! {
        #doc
        #[repr(transparent)]
        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
        pub struct #name (pub #ty);

        impl #name {
            #items
        }

        impl #name {
            pub const fn from_bits(val: #ty) -> #name {
                Self(val & #mask)
            }

            pub const fn to_bits(self) -> #ty {
                self.0
            }
        }
    });

    out.extend(quote! {
        impl From<#ty> for #name {
            #[inline(always)]
            fn from(val: #ty) -> #name {
                #name::from_bits(val)
            }
        }

        impl From<#name> for #ty {
            #[inline(always)]
            fn from(val: #name) -> #ty {
                #name::to_bits(val)
            }
        }
    });

    Ok(out)
}
