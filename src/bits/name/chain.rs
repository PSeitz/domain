//! A chain of domain names.
//!
//! This is a private module. Its public types are re-exported by the parent
//! crate.

use std::{fmt, iter};
use bytes::BufMut;
use ::bits::compose::{Compose, Compress, Compressor};
use ::bits::parse::ShortBuf;
use super::label::Label;
use super::traits::{ToLabelIter, ToRelativeDname, ToDname};
use super::uncertain::UncertainDname;


//------------ Chain ---------------------------------------------------------

/// Two domain names chained together.
///
/// This type is the result of calling the `chain` method on
/// [`RelativeDname`], [`UncertainDname`], or on [`Chain`] itself.
///
/// The chain can be both an absolute or relative domain name—and implements
/// the respective traits [`ToDname`] or [`ToRelativeDname`]—, depending on
/// whether the second name is absolute or relative.
///
/// A chain on an uncertain name is special in that the second name is only
/// used if the uncertain name is relative.
///
/// [`RelativeDname`]: struct.RelativeDname.html#method.chain
/// [`Chain`]: #method.chain
/// [`ToDname`]: trait.ToDname.html
/// [`ToRelativeDname`]: trait.ToRelativeDname.html
/// [`UncertainDname`]: struct.UncertainDname.html#method.chain
pub struct Chain<L, R> {
    /// The first domain name.
    left: L,

    /// The second domain name.
    right: R,
}

impl<L: Compose, R: Compose> Chain<L, R> {
    /// Creates a new chain from a first and second name.
    pub(super) fn new(left: L, right: R) -> Result<Self, LongChainError> {
        if left.compose_len() + right.compose_len() > 255 {
            Err(LongChainError)
        }
        else {
            Ok(Chain { left, right })
        }
    }
}

impl<L: ToRelativeDname, R: Compose> Chain<L, R> {
    /// Extends the chain with another domain name.
    ///
    /// While the method accepts anything [`Compose`] as the second element of
    /// the chain, the resulting `Chain` will only implement [`ToDname`] or
    /// [`ToRelativeDname`] if if also implements [`ToDname`] or
    /// [`ToRelativeDname`], respectively.
    ///
    /// The method will fail with an error if the chained name is longer than
    /// 255 bytes.
    ///
    /// [`Compose`]: ../compose/trait.Compose.html
    /// [`ToDname`]: trait.ToDname.html
    /// [`ToRelativeDname`]: trait.ToRelativeDname.html
    pub fn chain<N: Compose>(self, other: N)
                                -> Result<Chain<Self, N>, LongChainError> {
        Chain::new(self, other)
    }
}

impl<L, R> Chain<L, R> {
    /// Unwraps the chain into its two constituent components.
    pub fn unwrap(self) -> (L, R) {
        (self.left, self.right)
    }
}

impl<'a, L: ToRelativeDname, R: for<'r> ToLabelIter<'r>> ToLabelIter<'a>
            for Chain<L, R> {
    type LabelIter = ChainIter<'a, L, R>;

    fn iter_labels(&'a self) -> Self::LabelIter {
        ChainIter(self.left.iter_labels().chain(self.right.iter_labels()))
    }
}

impl<L: Compose, R: Compose> Compose for Chain<L, R> {
    fn compose_len(&self) -> usize {
        self.left.compose_len() + self.right.compose_len()
    }

    fn compose<B: BufMut>(&self, buf: &mut B) {
        self.left.compose(buf);
        self.right.compose(buf)
    }
}

impl<L: ToRelativeDname, R: ToDname> Compress for Chain<L, R> {
    fn compress(&self, buf: &mut Compressor) -> Result<(), ShortBuf> {
        buf.compress_name(self)
    }
}

impl<L: ToRelativeDname, R: ToRelativeDname> ToRelativeDname for Chain<L, R> {
}

impl<L: ToRelativeDname, R: ToDname> ToDname for Chain<L, R> {
}

impl<L: fmt::Display, R: fmt::Display> fmt::Display for Chain<L, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.left, self.right)
    }
}

impl<'a, R: ToDname> ToLabelIter<'a> for Chain<UncertainDname, R> {
    type LabelIter = ChainIter<'a, UncertainDname, R>;

    fn iter_labels(&'a self) -> Self::LabelIter {
        unimplemented!()
    }
}

impl<R: ToDname> Compress for Chain<UncertainDname, R> {
    fn compress(&self, buf: &mut Compressor) -> Result<(), ShortBuf> {
        if let UncertainDname::Absolute(ref name) = self.left {
            buf.compress_name(name)
        }
        else {
            // XXX Test this!
            buf.compress_name(self)
        }
    }
}

impl<R: ToDname> ToDname for Chain<UncertainDname, R> { }


//------------ ChainIter -----------------------------------------------------

/// The label iterator for chained domain names.
#[derive(Clone, Debug)]
pub struct ChainIter<'a, L: ToLabelIter<'a>, R: ToLabelIter<'a>>(
    iter::Chain<L::LabelIter, R::LabelIter>
);

impl<'a, L, R> Iterator for ChainIter<'a, L, R>
        where L: ToLabelIter<'a>, R: ToLabelIter<'a> {
    type Item = &'a Label;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, L, R> DoubleEndedIterator for ChainIter<'a, L, R>
        where L: ToLabelIter<'a>, R: ToLabelIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}


//------------ LongChainError ------------------------------------------------

/// Chaining domain names would exceed the size limit.
#[derive(Clone, Copy, Debug, Eq, Fail, PartialEq)]
#[fail(display="long domain name")]
pub struct LongChainError;



