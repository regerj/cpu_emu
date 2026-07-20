use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemEnum;

pub struct DisplayRegDeriver {
    item: ItemEnum,
}

impl DisplayRegDeriver {
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
            impl Display for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let s = match self {
                        #(Self::#variants => #str_variants,)*
                    };

                    write!(f, "{s}")
                }
            }
        }
    }
}
