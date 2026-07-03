use quote::{
    format_ident,
    quote,
};
use syn::{
    Arm,
    Expr,
    Fields,
    Ident,
    ImplItemFn,
    ItemEnum,
    ItemImpl,
    ItemStruct,
    Pat,
    Stmt,
    Variant,
    parse_quote,
};

pub fn derive_mneumonic(item: &ItemEnum) -> ItemEnum {
    let mneumonics: Vec<_> = item
        .variants
        .iter()
        .map(|variant| variant.ident.clone())
        .collect();

    eprintln!("mneumonics: {mneumonics:?}");

    parse_quote! {
        #[repr(u8)]
        enum Mneumonics {
            #(#mneumonics),*
        }
    }
}

pub fn metadata() -> ItemStruct {
    parse_quote! {
        #[bitfield_struct::bitfield(u8)]
        struct Metadata {
            #[bits(2)]
            arg0_spec: ArgSpec,
            #[bits(2)]
            arg1_spec: ArgSpec,
            #[bits(2)]
            arg2_spec: ArgSpec,
            #[bits(2)]
            arg3_spec: ArgSpec,
        }
    }
}

pub fn arg_spec() -> ItemStruct {
    parse_quote! {
        #[bitfield_struct::bitfield(u8)]
        struct ArgSpec {
            deref: bool,
            register: bool,
            #[bits(6)]
            __: u8,
        }
    }
}

pub fn derive_asm_impl(item: &ItemEnum) -> ItemImpl {
    let ident = item.ident.clone();
    let compile_impl = derive_compile(item);
    let metadata_impl = derive_metadata(item);

    parse_quote! {
        impl #ident {
            #compile_impl
            #metadata_impl
        }
    }
}

fn get_pattern(var: &Variant) -> Pat {
    let variant_ident = var.ident.clone();
    match &var.fields {
        Fields::Unit => parse_quote! { Self::#variant_ident },
        Fields::Unnamed(fields) => {
            let op_idents = op_names(fields.unnamed.len());

            assert!(
                op_idents.len() <= 4,
                "More than four operands are not supported"
            );
            parse_quote! { Self::#variant_ident(#(#op_idents),*) }
        }
        _ => unreachable!(),
    }
}

fn op_names(num: usize) -> Vec<Ident> {
    (0..num).map(|num| format_ident!("op{num}")).collect()
}

fn derive_metadata_modifications(fields: &Fields) -> Vec<Stmt> {
    match fields {
        Fields::Unit => vec![],
        Fields::Unnamed(fields) => (0..fields.unnamed.len())
            .map(|num| {
                let method_name = format_ident!("set_arg{num}_spec");
                let op_name = format_ident!("op{num}");
                parse_quote! {
                    meta.#method_name(
                        ArgSpec::new()
                            .with_deref(#op_name.is_deref())
                            .with_register(#op_name.is_reg())
                    );
                }
            })
            .collect(),
        _ => unreachable!(),
    }
}

fn derive_metadata_arm(var: &Variant) -> Arm {
    let pat = get_pattern(var);
    let mods = derive_metadata_modifications(&var.fields);
    parse_quote! {
        #pat => {
            #(#mods)*
        }
    }
}

fn derive_op_bytes(fields: &Fields) -> Expr {
    match fields {
        Fields::Unit => parse_quote! { vec![] },
        Fields::Unnamed(fields) => {
            let op_idents = op_names(fields.unnamed.len());
            parse_quote! {
                [#(#op_idents.value_bytes()),*].concat()
            }
        }
        _ => unreachable!(),
    }
}

fn derive_operand_compile_arm(var: &Variant) -> Arm {
    let pat = get_pattern(var);
    let bytes = derive_op_bytes(&var.fields);

    parse_quote! {
        #pat => #bytes
    }
}

fn derive_metadata(item: &ItemEnum) -> ImplItemFn {
    let arms: Vec<_> = item.variants.iter().map(derive_metadata_arm).collect();
    parse_quote! {
        fn metadata(&self) -> Metadata {
            let mut meta = Metadata::new();

            match self {
                #(#arms),*
            }

            meta
        }
    }
}

fn derive_compile(item: &ItemEnum) -> ImplItemFn {
    let variant_matches: Vec<_> = item
        .variants
        .iter()
        .map(|variant| {
            let ident = variant.ident.clone();
            match variant.fields {
                syn::Fields::Unit => quote! {#ident },
                syn::Fields::Unnamed(..) => quote! {#ident (..) },
                _ => panic!("Struct-like enum variants not supported"),
            }
        })
        .collect();

    let operand_serialization_arms: Vec<_> = item
        .variants
        .iter()
        .map(derive_operand_compile_arm)
        .collect();
    let variants: Vec<_> = item
        .variants
        .iter()
        .map(|variant| variant.ident.clone())
        .collect();

    parse_quote! {
        pub fn compile(self) -> Vec<u8> {
            let mneumonic_bits = match self {
                #(Self::#variant_matches => Mneumonics::#variants as u8),*
            };

            let meta_bits = self.metadata().into_bits();

            let mut ret = vec![mneumonic_bits, meta_bits];

            let op_bytes = match self {
                #(#operand_serialization_arms),*
            };

            ret.extend(op_bytes);

            ret
        }
    }
}
