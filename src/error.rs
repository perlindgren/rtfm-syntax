//! Errors

error_chain!();

// use quote::ToTokens;
// use std::fmt::Debug;
// use std::marker::PhantomData;
// use std::iter::FromIterator;
// use std::vec::Vec;
use syn::spanned::Spanned;
use syn::synom::{PResult, Synom};
use syn::{parse_error, buffer::Cursor};

pub fn exit_err<T>(mut i: Cursor, err: String) -> PResult<T> {
    i.span().unstable().error(err).emit();
    return parse_error::<T>();
}
