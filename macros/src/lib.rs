use proc_macro as pm;
use syn::{
    ItemEnum,
    parse_macro_input,
};

use crate::{
    display_op::DisplayOpDeriver,
    display_reg::DisplayRegDeriver,
    machine_code::AsmDeriver,
    try_from_str::TryFromStrDeriver,
    try_from_u16::TryFromU16Deriver,
};

mod display_op;
mod display_reg;
mod machine_code;
mod try_from_str;
mod try_from_u16;

#[proc_macro_derive(MachineCode)]
pub fn derive_machine_code(item: pm::TokenStream) -> pm::TokenStream {
    let item = parse_macro_input!(item as ItemEnum);
    let deriver = AsmDeriver::new(item);

    deriver.derive().into()
}

#[proc_macro_derive(TryFromU16)]
pub fn derive_try_from_u16(item: pm::TokenStream) -> pm::TokenStream {
    let item = parse_macro_input!(item as ItemEnum);
    let deriver = TryFromU16Deriver::new(item);

    deriver.derive().into()
}

#[proc_macro_derive(TryFromStr)]
pub fn derive_try_from_str(item: pm::TokenStream) -> pm::TokenStream {
    let item = parse_macro_input!(item as ItemEnum);
    let deriver = TryFromStrDeriver::new(item);

    deriver.derive().into()
}

#[proc_macro_derive(DisplayReg)]
pub fn derive_display_reg(item: pm::TokenStream) -> pm::TokenStream {
    let item = parse_macro_input!(item as ItemEnum);
    let deriver = DisplayRegDeriver::new(item);

    deriver.derive().into()
}

#[proc_macro_derive(DisplayOp)]
pub fn derive_display_op(item: pm::TokenStream) -> pm::TokenStream {
    let item = parse_macro_input!(item as ItemEnum);
    let deriver = DisplayOpDeriver::new(item);

    deriver.derive().into()
}
