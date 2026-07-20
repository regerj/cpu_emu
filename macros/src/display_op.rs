use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Arm,
    Ident,
    ItemEnum,
    parse_quote,
};

pub struct DisplayOpDeriver {
    item: ItemEnum,
}

impl DisplayOpDeriver {
    pub fn new(item: ItemEnum) -> Self {
        Self { item }
    }

    pub fn derive(self) -> TokenStream {
        let ident = self.item.ident.clone();
        let operands: Vec<Ident> = vec![
            parse_quote!(op0),
            parse_quote!(op1),
            parse_quote!(op2),
            parse_quote!(op3),
        ];

        let variant_arms: Vec<Arm> = self
            .item
            .variants
            .iter()
            .map(|variant| {
                let ident = variant.ident.clone();
                let n = variant.fields.len();
                let ops: Vec<_> = operands.iter().take(n).collect();
                let s_ident = ident.to_string().to_lowercase();

                let s = match n {
                    #[allow(clippy::useless_format)]
                    0 => format!("{s_ident}"),
                    1 => format!("{s_ident} {{op0}}"),
                    2 => format!("{s_ident} {{op0}},{{op1}}"),
                    3 => format!("{s_ident} {{op0}},{{op1}},{{op2}}"),
                    4 => format!("{s_ident} {{op0}},{{op1}},{{op2}},{{op3}}"),
                    _ => panic!("More than 4 arugments not supported"),
                };

                if n == 0 {
                    parse_quote! {
                        Self::#ident => format!(#s)
                    }
                } else {
                    parse_quote! {
                        Self::#ident(#(#ops),*) => format!(#s)
                    }
                }
            })
            .collect();

        quote! {
            impl Display for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let instruction = match self {
                        #(#variant_arms,)*
                    };
                    write!(f, "{instruction}")
                }
            }
        }
    }
}
