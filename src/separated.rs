use quote::ToTokens;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::vec::Vec;
use syn::spanned::Spanned;
use syn::synom::{PResult, Synom};
use syn::{parse_error, buffer::Cursor};

#[derive(Debug)]
pub struct Pu<T, P>
where
    T: Spanned,
    P: Spanned,
{
    inner: Vec<T>,
    _marker: PhantomData<P>,
}

impl<T, P> Pu<T, P>
where
    T: Synom + Spanned + ToTokens,
    P: Synom + Spanned + ToTokens,
{
    /// Parse **zero or more** syntax tree nodes with punctuation in between and
    /// **no trailing** punctuation.
    pub fn parse_separated(input: Cursor) -> PResult<Self> {
        Self::parse_separated_with(input, T::parse)
    }
}

impl<T, P> Pu<T, P>
where
    T: Synom + Spanned + ToTokens,
    P: Synom + Spanned + ToTokens,
{
    /// Parse **zero or more** syntax tree nodes using the given parser with
    /// punctuation in between and **no trailing** punctuation.
    pub fn parse_separated_with(
        input: Cursor,
        parse: fn(Cursor) -> PResult<T>,
    ) -> PResult<Self> {
        Self::parse(input, parse, false)
    }

    fn parse(
        mut input: Cursor,
        parse: fn(Cursor) -> PResult<T>,
        terminated: bool,
    ) -> PResult<Self> {
        let mut res = Pu {
            inner: Vec::new(),
            _marker: PhantomData,
        };

        // get the first element
        match parse(input) {
            Err(_) => Ok((res, input)),
            Ok((o, i)) => {
                if i == input {
                    o.span().unstable().error("parse error").emit();
                    return parse_error();
                }
                input = i;
                res.inner.push(o);

                println!("here 1");
                // get the separator first
                while let Ok((s, i2)) = P::parse(input) {
                    if i2 == input {
                        s.span().unstable().error("parse error").emit();
                        return parse_error();
                    }
                    println!("here 2");
                    // get the element next
                    if let Ok((o3, i3)) = parse(i2) {
                        if i3 == i2 {
                            return parse_error();
                        }
                        res.inner.push(o3);
                        input = i3;
                    } else {
                        break;
                    }
                }
                println!("here 3");
                if terminated {
                    if let Ok((_s, after)) = P::parse(input) {
                        input = after;
                        return Ok((res, input));
                    } else {
                        return parse_error();
                    }
                } else {
                    return Ok((res, input));
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Pun<T, P>
where
    T: Synom + Debug + ToTokens,
    P: Synom + Debug + ToTokens,
{
    data: Pu<T, P>,
}

impl<T, P> Synom for Pun<T, P>
where
    T: Synom + Debug + ToTokens,
    P: Synom + Debug + ToTokens,
{
    named!(parse -> Self,
        map!(call!(Pu::parse_separated), |data| Pun { data })
    );
}
