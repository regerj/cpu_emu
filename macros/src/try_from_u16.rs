use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote,
};
use syn::ItemEnum;

pub struct TryFromU16Deriver {
    item: ItemEnum,
}

impl TryFromU16Deriver {
    pub fn new(item: ItemEnum) -> Self {
        Self { item }
    }

    pub fn derive(self) -> TokenStream {
        let ident = self.item.ident;
        let variants: Vec<_> = self
            .item
            .variants
            .iter()
            .map(|variant| variant.ident.clone())
            .collect();
        let pats: Vec<_> = variants
            .iter()
            .map(|ident| format_ident!("{}_PAT", ident.to_string().to_uppercase()))
            .collect();
        quote! {
            impl TryFrom<u16> for #ident {
                type Error = anyhow::Error;
                fn try_from(value: u16) -> Result<Self, Self::Error> {
                    #(const #pats: u16 = #ident::#variants as u16;)*

                    Ok(match value {
                        #(#pats => Self::#variants,)*
                        _ => bail!("Invalid interpretation of integer to register"),
                    })
                }
            }
        }
    }
}
