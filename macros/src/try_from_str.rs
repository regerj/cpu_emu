use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemEnum;

pub struct TryFromStrDeriver {
    item: ItemEnum,
}

impl TryFromStrDeriver {
    pub fn new(item: ItemEnum) -> Self {
        Self { item }
    }

    pub fn derive(self) -> TokenStream {
        let ident = self.item.ident.clone();
        let variants: Vec<_> = self
            .item
            .variants
            .iter()
            .map(|variant| variant.ident.clone())
            .collect();
        let str_variants: Vec<_> = variants
            .iter()
            .map(|ident| ident.to_string().to_lowercase())
            .collect();
        quote! {
            impl TryFrom<&str> for #ident {
                type Error = anyhow::Error;
                fn try_from(value: &str) -> Result<Self, Self::Error> {
                    match value {
                        #(#str_variants => Ok(Self::#variants),)*
                        _ => bail!("Invalid interpretation of string to register"),
                    }
                }
            }
        }
    }
}
