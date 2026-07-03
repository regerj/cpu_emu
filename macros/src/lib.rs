use proc_macro as pm;
use quote::quote;
use syn::{
    ItemEnum,
    parse_macro_input,
};

use crate::machine_code::{
    arg_spec,
    derive_asm_impl,
    derive_mneumonic,
    metadata,
};

mod machine_code;

#[proc_macro_derive(MachineCode)]
pub fn derive_machine_code(item: pm::TokenStream) -> pm::TokenStream {
    let item = parse_macro_input!(item as ItemEnum);
    let mneumonic = derive_mneumonic(&item);
    let metadata = metadata();
    let arg_spec = arg_spec();
    let compile_impl = derive_asm_impl(&item);

    quote! {
        #mneumonic
        #metadata
        #arg_spec
        #compile_impl
    }
    .into()
}
