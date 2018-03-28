use quote::ToTokens;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::iter::FromIterator;
use std::vec::Vec;
use syn::spanned::Spanned;
use syn::synom::{PResult, Synom};
use syn::{parse_error, buffer::Cursor};
use error::*;

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
    T: Spanned,
    P: Spanned,
{
    /// Creates an empty punctuated sequence.
    pub fn new() -> Pu<T, P> {
        Pu {
            inner: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Determines whether this punctuated sequence is empty, meaning it
    /// contains no syntax tree nodes or punctuation.
    pub fn is_empty(&self) -> bool {
        self.inner.len() == 0
    }

    /// Returns the number of syntax tree nodes in this punctuated sequence.
    ///
    /// This is the number of nodes of type `T`, not counting the punctuation of
    /// type `P`.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    // /// Returns an iterator over borrowed syntax tree nodes of type `&T`.
    // pub fn iter(&self) -> Iter<T> {
    //     Iter {
    //         inner: self.inner.iter(),
    //     }
    // }

    // /// Returns an iterator over mutably borrowed syntax tree nodes of type
    // /// `&mut T`.
    // pub fn iter_mut(&mut self) -> IterMut<T, P> {
    //     IterMut {
    //         inner: self.inner.iter_mut(),
    //     }
    // }

    /// Appends a syntax tree node onto the end of this punctuated sequence. The
    /// sequence must previously have a trailing punctuation.
    ///
    /// Use [`push`] instead if the punctuated sequence may or may not already
    /// have trailing punctuation.
    ///
    /// [`push`]: #method.push
    ///
    /// # Panics
    ///
    /// Panics if the sequence does not already have a trailing punctuation when
    /// this method is called.
    pub fn push(&mut self, value: T) {
        self.inner.push(value);
    }

    /// Removes the last punctuated pair from this sequence, or `None` if the
    /// sequence is empty.
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }
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

    /// Parse **one or more** syntax tree nodes with punctuation in bewteen and
    /// **no trailing** punctuation.
    /// allowing trailing punctuation.
    pub fn parse_separated_nonempty(input: Cursor) -> PResult<Self> {
        Self::parse_separated_nonempty_with(input, T::parse)
    }

    /// Parse **zero or more** syntax tree nodes with punctuation in between and
    /// **optional trailing** punctuation.
    pub fn parse_terminated(input: Cursor) -> PResult<Self> {
        Self::parse_terminated_with(input, T::parse)
    }

    /// Parse **one or more** syntax tree nodes with punctuation in between and
    /// **optional trailing** punctuation.
    pub fn parse_terminated_nonempty(input: Cursor) -> PResult<Self> {
        Self::parse_terminated_nonempty_with(input, T::parse)
    }
}

impl<T, P> Pu<T, P>
where
    T: Synom + Spanned + ToTokens,
    P: Synom + Spanned + ToTokens,
{
    /// Parse **zero or more** syntax tree nodes using the given parser with
    /// punctuation in between and **no trailing** punctuation.
    pub fn parse_separated_with(input: Cursor, parse: fn(Cursor) -> PResult<T>) -> PResult<Self> {
        Self::parse(input, parse, false)
    }

    /// Parse **one or more** syntax tree nodes using the given parser with
    /// punctuation in between and **no trailing** punctuation.
    pub fn parse_separated_nonempty_with(
        input: Cursor,
        parse: fn(Cursor) -> PResult<T>,
    ) -> PResult<Self> {
        match Self::parse(input, parse, false) {
            Ok((ref b, _)) if b.is_empty() => {
                input
                    .span()
                    .unstable()
                    .error("expected sequence of separated elements")
                    .emit();
                parse_error()
            }
            other => other,
        }
    }

    /// Parse **zero or more** syntax tree nodes using the given parser with
    /// punctuation in between and **optional trailing** punctuation.
    pub fn parse_terminated_with(input: Cursor, parse: fn(Cursor) -> PResult<T>) -> PResult<Self> {
        Self::parse(input, parse, true)
    }

    /// Parse **one or more** syntax tree nodes using the given parser with
    /// punctuation in between and **optional trailing** punctuation.
    pub fn parse_terminated_nonempty_with(
        input: Cursor,
        parse: fn(Cursor) -> PResult<T>,
    ) -> PResult<Self> {
        match Self::parse(input, parse, true) {
            Ok((ref b, _)) if b.is_empty() => parse_error(),
            other => other,
        }
    }

    fn exit_eof(mut i: Cursor, res: T, err: String) -> PResult<T> {
        if i.eof() {
            Ok((res, i))
        } else {
            exit_err(i, err)
        }
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
            Err(_) => {
                if input.eof() {
                    return Ok((res, input));
                } else {
                    return exit_err(input, String::from("parse error"));
                }
            }
            Ok((o, i)) => {
                input = i;
                res.inner.push(o);

                println!("here 1");
                loop {
                    if input.eof() {
                        println!("!! eof");
                        return Ok((res, input));
                    }
                    // get the separator first
                    if let Ok((s, i1)) = P::parse(input) {
                        println!("here, comma parsed");
                        if i1.eof() {
                            println!("no more input");
                            if terminated {
                                return Ok((res, i1));
                            } else {
                                return exit_err(input, String::from("excessive separator"));
                            }
                        }
                        // get next element
                        if let Ok((o, after)) = parse(i1) {
                            input = after;
                            res.inner.push(o);
                        } else {
                            return exit_err(i1, String::from("parse error"));
                        }
                    } else {
                        return exit_err(input, String::from("parse error"));
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Separated<T, P>
where
    T: Synom + Debug + ToTokens,
    P: Synom + Debug + ToTokens,
{
    data: Pu<T, P>,
}

/// Parse non-termintated
impl<T, P> Synom for Separated<T, P>
where
    T: Synom + Debug + ToTokens,
    P: Synom + Debug + ToTokens,
{
    named!(parse -> Self,
        map!(call!(Pu::parse_separated), |data| Separated { data })
    );
}

/// Parse with optional termination
#[derive(Debug)]
pub struct Terminated<T, P>
where
    T: Synom + Debug + ToTokens,
    P: Synom + Debug + ToTokens,
{
    data: Pu<T, P>,
}

impl<T, P> Synom for Terminated<T, P>
where
    T: Synom + Debug + ToTokens,
    P: Synom + Debug + ToTokens,
{
    named!(parse -> Self,
        map!(call!(Pu::parse_terminated), |data| Terminated { data })
    );
}

#[derive(Debug)]
pub struct SeparatedNonEmpty<T, P>
where
    T: Synom + Debug + ToTokens,
    P: Synom + Debug + ToTokens,
{
    data: Pu<T, P>,
}

impl<T, P> Synom for SeparatedNonEmpty<T, P>
where
    T: Synom + Debug + ToTokens,
    P: Synom + Debug + ToTokens,
{
    named!(parse -> Self,
        map!(call!(Pu::parse_separated_nonempty), |data| SeparatedNonEmpty { data })
    );
}

// #[derive(Debug)]
// pub struct SeparatedNonTerminatedNonEmpty<T, P>
// where
//     T: Synom + Debug + ToTokens,
//     P: Synom + Debug + ToTokens,
// {
//     data: Pu<T, P>,
// }

// impl<T, P> Synom for SeparatedNonTerminatedNonEmpty<T, P>
// where
//     T: Synom + Debug + ToTokens,
//     P: Synom + Debug + ToTokens,
// {
//     named!(parse -> Self,
//         map!(call!(Pu::parse_terminated_nonempty, |data| SeparatedNonTerminatedNonEmpty { data })
//     );
// }
