// The MIT License (MIT)
// 
// Copyright (c) 2016 Skylor R. Schermer
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in 
// all copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
////////////////////////////////////////////////////////////////////////////////
//!
//! Defines results for error-producing `Palette` operations.
//!
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use address::{
	Address,
	Reference,
};

// Standard imports.
use std::fmt;
use std::result;
use std::error;

/// An alias for `std::result::Result<T, palette::Error>`.
pub type Result<T> = result::Result<T, Error>;

////////////////////////////////////////////////////////////////////////////////
// Error
////////////////////////////////////////////////////////////////////////////////
/// Encapsulates errors associated with mutating palette operations.
#[derive(Debug)]
pub enum Error {
	/// Attempted to add a color to the palette, but the palette contains the 
	/// maximum number of slots already.
	MaxCellLimitExceeded,

	/// Attempted to set a color to a non-zeroth-order slot.
	CannotSetDerivedColor,
	
	/// An `Address` was provided that lies outside of the range defined for the 
	/// palette.
	InvalidAddress(Address),
	
	/// A `Reference` was generated for an invalid address.
	InvalidReference(Reference),
	
	/// A `Reference` component was invalid.
	InvalidReferenceComponent,

	/// A `Reference` component was requested from an unresolved address.
	UnresolvedReferenceComponent,
	
	/// An empty address was provided for an operation that requires a color.
	EmptyAddress(Address),
	
	/// An element could not be created because the address was occupied.
	AddressInUse(Address),
}


impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		match *self {
			Error::EmptyAddress(address)
				=> write!(f, "{}: {}", 
					error::Error::description(self), 
					address
				),

			_	=> write!(f, "{}", error::Error::description(self))
		}
	}
}


impl error::Error for Error {
	fn description(&self) -> &str {
		match *self {
			Error::MaxCellLimitExceeded
				=> "maximum number of color slots for palette exceeded",

			Error::CannotSetDerivedColor
				=> "cannot assign color to a location containing a derived \
				color value",

			Error::InvalidAddress(..)
				=> "address lies outside of allowed range",

			Error::InvalidReference(..)
				=> "cell reference lies outside of allowed range",

			Error::InvalidReferenceComponent
				=> "reference component overflow or underflow.",

			Error::UnresolvedReferenceComponent
				=> "reference component did not resolve completely.",

			Error::EmptyAddress(..)
				=> "empty address provided to an operation requiring a color",

			Error::AddressInUse(..)
				=> "the address is in use",
		}
	}
}
