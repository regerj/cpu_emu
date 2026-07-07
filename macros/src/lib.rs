use proc_macro as pm;
use syn::{
    ItemEnum,
    parse_macro_input,
};

use crate::machine_code::AsmDeriver;

mod machine_code;

#[proc_macro_derive(MachineCode)]
pub fn derive_machine_code(item: pm::TokenStream) -> pm::TokenStream {
    let item = parse_macro_input!(item as ItemEnum);
    let deriver = AsmDeriver::new(item);

    deriver.derive().into()
}
