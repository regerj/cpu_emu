use proc_macro2::{
    Span,
    TokenStream,
};

use quote::{
    ToTokens,
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

struct ConstIdent(&'static str);

impl ToTokens for ConstIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        Ident::new(self.0, Span::call_site()).to_tokens(tokens);
    }
}

pub struct AsmDeriver {
    item: ItemEnum,
}

impl AsmDeriver {
    pub fn new(item: ItemEnum) -> Self {
        Self { item }
    }

    pub fn derive(self) -> TokenStream {
        let mneumonic_tokens = MneumonicDeriver::new(&self.item).derive();
        let metadata_tokens = MetadataDeriver::new(&self.item).derive();
        let argspec_tokens = ArgSpecDeriver::new(&self.item).derive();
        let direct_tokens = DirectDeriver::new(&self.item).derive();

        quote! {
            #direct_tokens
            #mneumonic_tokens
            #metadata_tokens
            #argspec_tokens
        }
    }
}

struct MneumonicDeriver<'a> {
    item: &'a ItemEnum,
}

impl<'a> MneumonicDeriver<'a> {
    pub const IDENT: ConstIdent = ConstIdent("Mneumonics");

    pub fn new(item: &'a ItemEnum) -> Self {
        Self { item }
    }

    pub fn derive(self) -> TokenStream {
        let self_enum = self.derive_enum();
        let self_impl = self.derive_impl();

        quote! {
            #self_enum
            #(#self_impl)*
        }
    }

    fn derive_enum(&self) -> ItemEnum {
        let mneumonics: Vec<_> = self
            .item
            .variants
            .iter()
            .map(|variant| variant.ident.clone())
            .collect();

        let ident = Self::IDENT;

        parse_quote! {
            #[repr(u8)]
            enum #ident {
                #(#mneumonics),*
            }
        }
    }

    fn derive_impl(&self) -> Vec<ItemImpl> {
        let self_ident = Self::IDENT;
        let pat_idents: Vec<_> = self
            .item
            .variants
            .iter()
            .map(|variant| {
                let ident = variant.ident.to_string().to_uppercase();
                format_ident!("{ident}_PATTERN")
            })
            .collect();

        let num_args: Vec<_> = self
            .item
            .variants
            .iter()
            .map(|variant| variant.fields.len())
            .collect();

        let idents: Vec<_> = self
            .item
            .variants
            .iter()
            .map(|variant| variant.ident.clone())
            .collect();

        let direct_impl = parse_quote! {
            impl #self_ident {
                #(const #pat_idents: u8 = Self::#idents as u8);*;

                pub fn num_args(&self) -> usize {
                    match self {
                        #(Self::#idents => #num_args),*,
                    }
                }
            }
        };

        let try_from_impl = parse_quote! {
            impl TryFrom<u8> for #self_ident {
                type Error = anyhow::Error;
                fn try_from(value: u8) -> Result<Self, Self::Error> {
                    let ret = match value {
                        #(Self::#pat_idents => Self::#idents),*,
                        _ => anyhow::bail!("Invalid mneumonic bits"),
                    };

                    Ok(ret)
                }
            }
        };

        vec![direct_impl, try_from_impl]
    }
}

struct MetadataDeriver<'a> {
    _item: &'a ItemEnum,
}

impl<'a> MetadataDeriver<'a> {
    pub const IDENT: ConstIdent = ConstIdent("Metadata");

    pub fn new(item: &'a ItemEnum) -> Self {
        Self { _item: item }
    }

    pub fn derive(self) -> TokenStream {
        let self_struct = self.derive_struct();

        quote! {
            #self_struct
        }
    }

    fn derive_struct(&self) -> ItemStruct {
        let argspec_ident = ArgSpecDeriver::IDENT;
        let ident = Self::IDENT;
        parse_quote! {
            #[bitfield_struct::bitfield(u8)]
            struct #ident {
                #[bits(2)]
                arg0_spec: #argspec_ident,
                #[bits(2)]
                arg1_spec: #argspec_ident,
                #[bits(2)]
                arg2_spec: #argspec_ident,
                #[bits(2)]
                arg3_spec: #argspec_ident,
            }
        }
    }
}

struct ArgSpecDeriver<'a> {
    _item: &'a ItemEnum,
}

impl<'a> ArgSpecDeriver<'a> {
    pub const IDENT: ConstIdent = ConstIdent("ArgSpec");
    pub fn new(item: &'a ItemEnum) -> Self {
        Self { _item: item }
    }

    pub fn derive(self) -> TokenStream {
        let self_struct = self.derive_struct();

        quote! {
            #self_struct
        }
    }

    fn derive_struct(&self) -> ItemStruct {
        let ident = Self::IDENT;
        parse_quote! {
            #[bitfield_struct::bitfield(u8)]
            struct #ident {
                deref: bool,
                register: bool,
                #[bits(6)]
                __: u8,
            }
        }
    }
}

struct DirectDeriver<'a> {
    item: &'a ItemEnum,
}

impl<'a> DirectDeriver<'a> {
    pub fn new(item: &'a ItemEnum) -> Self {
        Self { item }
    }

    pub fn derive(self) -> TokenStream {
        let self_impl = self.derive_impl();

        quote! {
            #self_impl
        }
    }

    fn derive_impl(&self) -> ItemImpl {
        let ident = self.item.ident.clone();
        let compile_impl = self.derive_compile();
        let metadata_impl = self.derive_metadata();
        let constructor_impls = self.derive_constructors();
        let consume_impl = self.derive_consume();

        parse_quote! {
            impl #ident {
                #compile_impl
                #metadata_impl
                #consume_impl
                #(#constructor_impls)*
            }
        }
    }

    fn derive_metadata(&self) -> ImplItemFn {
        let arms: Vec<_> = self.item.variants.iter().map(derive_metadata_arm).collect();
        let meta_ident = MetadataDeriver::IDENT;
        parse_quote! {
            fn metadata(&self) -> #meta_ident {
                let mut meta = #meta_ident::new();

                match self {
                    #(#arms),*
                }

                meta
            }
        }
    }

    fn derive_constructors(&self) -> Vec<ImplItemFn> {
        let mut vec = vec![];

        for variant in &self.item.variants {
            let fn_ident = format_ident!("create_{}", variant.ident.to_string().to_lowercase());
            let var_ident = &variant.ident;

            let argi_lsb = (0..variant.fields.len()).map(|i| format_ident!("arg{i}_lsb"));
            let argi_msb = (0..variant.fields.len()).map(|i| format_ident!("arg{i}_msb"));
            let argi_val = (0..variant.fields.len()).map(|i| format_ident!("arg{i}_val"));
            let argi_spec = (0..variant.fields.len()).map(|i| format_ident!("arg{i}_spec"));
            let opi = (0..variant.fields.len())
                .map(|i| format_ident!("op{i}"))
                .collect::<Vec<_>>();

            let instatiation_arg_tokens = if variant.fields.is_empty() {
                quote! {}
            } else {
                quote! {(#(#opi),*)}
            };

            vec.push(parse_quote! {
                fn #fn_ident(bytes: &mut Iter<u8>) -> anyhow::Result<Self> {
                    let meta = Metadata::from_bits(*bytes.next().context("Invalid binary")?);

                    #(
                        let #argi_lsb = bytes.next().context("Invalid binary")?;
                        let #argi_msb = bytes.next().context("Invalid binary")?;
                        let #argi_val: u16 = (*#argi_lsb as u16) | ((*#argi_msb as u16) << 8);
                        let #opi = Operand::__create(meta.#argi_spec().register(), meta.#argi_spec().deref(), #argi_val);
                    )*

                    Ok(Self::#var_ident #instatiation_arg_tokens)
                }
            });
        }

        vec
    }

    fn derive_consume(&self) -> ImplItemFn {
        let var_idents = self.item.variants.iter().map(|var| &var.ident);
        let var_creators = self
            .item
            .variants
            .iter()
            .map(|var| format_ident!("create_{}", var.ident.to_string().to_lowercase()));
        parse_quote! {
            fn consume(bytes: &mut Iter<u8>) -> anyhow::Result<Option<Self>> {
                let Some(mneumonic_byte) = bytes.next() else {
                    return Ok(None);
                };

                let mneumonic = Mneumonics::try_from(*mneumonic_byte)?;

                match mneumonic {
                    #(Mneumonics::#var_idents => Self::#var_creators(bytes)),*
                }
                .map(Some)
            }
        }
    }

    fn derive_compile(&self) -> ImplItemFn {
        let variant_matches: Vec<_> = self
            .item
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

        let operand_serialization_arms: Vec<_> = self
            .item
            .variants
            .iter()
            .map(derive_operand_compile_arm)
            .collect();
        let variants: Vec<_> = self
            .item
            .variants
            .iter()
            .map(|variant| variant.ident.clone())
            .collect();

        let mneumonic_ident = MneumonicDeriver::IDENT;

        parse_quote! {
            pub fn compile(self) -> Vec<u8> {
                let mneumonic_bits = match self {
                    #(Self::#variant_matches => #mneumonic_ident::#variants as u8),*
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
