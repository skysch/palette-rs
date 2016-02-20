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
//! Provides components for interacting with the ZPL palette format.
//!
////////////////////////////////////////////////////////////////////////////////
use palette::format::Palette;
use palette::PaletteData;
use address;

use std::fmt;
use std::result;

use std::io;
use std::io::{Result, Write, Read};

// The ZPL format was built for version 2.50 build 24 of Zelda Classic, and may
// not work on versions 1.92 or older.

const ZPL_COLOR_DEPTH_SCALE: f32 = 0.25;

const ZPL_HEADER : [u8;12] = [
	0x43, 0x53, 0x45, 0x54, 
	0x04, 0x00, 0x01, 0x00, 
	0x9c, 0x0d, 0x05, 0x00
];

const ZPL_FOOTER_A : [u8;4] = [
	0x5a, 0x00, 0x00, 0x00
];

const ZPL_FOOTER_B : [u8;4] = [ // x 109
	0x00, 0x00, 0x00, 0x00
];

const ZPL_FOOTER_C : [u8;20] = [
	0x00, 0x00, 0x00, 0x36, 
	0x00, 0x00, 0x4e, 0x00, 
	0x00, 0x14, 0x00, 0x00, 
	0x36, 0x00, 0x00, 0x4e,
	0x00, 0x00, 0x14, 0x00
];

const ZPL_FOOTER_D : [u8;4] = [ // x 79
	0x00, 0x00, 0x00, 0x00
];

const ZPL_FOOTER_E : [u8;36] = [
	0x22, 0x00, 0x00, 0x66, 
	0x00, 0x00, 0x5a, 0x00, 
	0x00, 0x22, 0x00, 0x00, 
	0x86, 0x00, 0x00, 0x3c,
	0x00, 0x00, 0x22, 0x00, 
	0x00, 0x86, 0x00, 0x00, 
	0x3c, 0x00, 0x00, 0x20, 
	0x30, 0x40, 0x3f, 0x3f, 
	0x3f, 0x07, 0x07, 0x07
];

////////////////////////////////////////////////////////////////////////////////
// ZplPalette
////////////////////////////////////////////////////////////////////////////////
/// The default palette format with no special configuration.
#[derive(Debug)]
pub struct ZplPalette {
	core: PaletteData,
}

impl Palette for ZplPalette {
	fn new<S>(name: S) -> Self where S: Into<String> {
		let mut pal = ZplPalette {core: Default::default()};
		pal.core.set_label(address::Select::All, "ZplPalette 1.0.0");
		pal.core.set_name(address::Select::All, name.into());
		pal.core.page_count = 255;
		pal.core.line_count = 16;
		pal.core.column_count = 16;
		pal.core.set_initialized(address::Select::All, true);
		pal
	}

	fn write_palette<W>(&self, out_buf: &mut W) -> io::Result<()> 
		where W: io::Write
	{
		// Write header.
		try!(out_buf.write(&ZPL_HEADER)); 
		// Write all groups in sequence.

		// Write level names.

		// Write footer.
		try!(out_buf.write(&ZPL_FOOTER_A));
		for _ in 1..109 {
			try!(out_buf.write(&ZPL_FOOTER_B));
		}
		try!(out_buf.write(&ZPL_FOOTER_C));
		for _ in 1..79 {
			try!(out_buf.write(&ZPL_FOOTER_D));
		}
		try!(out_buf.write(&ZPL_FOOTER_E));
		Ok(())
	}

	fn read_palette<R>(in_buf: &R) -> io::Result<Self>
		where R: io::Read, Self: Sized
	{
		unimplemented!()
	}
}

impl fmt::Display for ZplPalette {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		write!(f, "{} {}",
			self.core.get_label(address::Select::All).unwrap_or(""),
			self.core
		)
	}
}