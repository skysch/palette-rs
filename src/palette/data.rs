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
//! Defines a structured PaletteData object for storing common data for palette
//! formats.
//!
////////////////////////////////////////////////////////////////////////////////
use super::element::{Slot, ColorElement};
use super::metadata::Metadata;
use super::error::{Error, Result};
use color::Color;
use address::{Address, Group, 
	PageCount, LineCount, ColumnCount, 
	PAGE_MAX, LINE_MAX, COLUMN_MAX
};

use std::rc::Rc;
use std::collections::BTreeMap;
use std::fmt;
use std::result;
use std::mem;

/// Default function for prepare_new_page and prepare_new_line triggers.
#[allow(unused_variables)]
#[inline]
fn no_op(data: &mut PaletteData) {}

////////////////////////////////////////////////////////////////////////////////
// PaletteData
////////////////////////////////////////////////////////////////////////////////
/// Encapsulates a single palette.
pub struct PaletteData {
	/// A map assigning addresses to palette slots.
	pub slotmap: BTreeMap<Address, Rc<Slot>>,
	/// Provided metadata for various parts of the palette.
	pub metadata: BTreeMap<Group, Metadata>,
	/// The internal address cursor that is used to track the next available 
	/// address.
	pub address_cursor: Address,
	/// The number of pages in the palette.
	pub page_count: PageCount,
	/// The default number of lines in each page.
	pub default_line_count: LineCount,
	/// The default number of columns in each line.
	pub default_column_count: ColumnCount,
	/// Called before an element is added to a new page in the palette. The 
	/// expectation is that this will add the appropriate meta data to the 
	/// palette. This will be called before the prepare_new_line function is 
	/// called.
	pub prepare_new_page: fn(&mut PaletteData),
	/// Called before an element is added to a new line in the palette. The 
	/// expectation is that this will add the appropriate meta data to the 
	/// palette.
	pub prepare_new_line: fn(&mut PaletteData),
}


impl PaletteData {
	/// Returns the number of colors in the PaletteData.
	///
	/// # Example
	/// ```rust
	/// use rampeditor::palette::PaletteData;
	/// use rampeditor::Color;
	////
	/// let mut dat: PaletteData = Default::default();
	/// assert_eq!(dat.len(), 0);
	///
	/// dat.add_color(Color(1, 2, 3));
	/// assert_eq!(dat.len(), 1);
	/// ```
	#[inline]
	pub fn len(&self) -> usize {
		self.slotmap.len()
	}

	/// Adds a new color to the palette in the nearest valid location after 
	/// the selection cursor and returns its address. Returns an error if the 
	/// palette is full.
	///
	/// # Example
	/// ```rust
	/// use rampeditor::palette::PaletteData;
	/// use rampeditor::Color;
	/// 
	/// let mut dat: PaletteData = Default::default();
	/// dat.add_color(Color(255, 0, 0));
	/// dat.add_color(Color(0, 255, 0));
	/// dat.add_color(Color(0, 0, 255));
	///
	/// assert_eq!(dat.len(), 3);
	/// ```
	///
	/// # Errors
	/// ```rust
	/// # use rampeditor::palette::PaletteData;
	/// # use rampeditor::Color;
	/// let mut dat: PaletteData = Default::default();
	/// dat.page_count = 1;
	/// dat.default_line_count = 1;
	/// dat.default_column_count = 1;
	/// dat.add_color(Color(0, 0, 0));
	/// let result = dat.add_color(Color(0, 0, 0)); // fails...
	/// assert!(result.is_err()); 
	/// ```
	#[inline]
	pub fn add_color(
		&mut self, 
		new_color: Color) 
		-> Result<Address> 
	{
		self.add_element(ColorElement::ZerothOrder {color: new_color})
	}

	/// Returns the color located at the given address, or None if the address 
	/// is invalid or empty.
	///
	/// # Examples
	/// ```rust
	///  use rampeditor::palette::PaletteData;
	///  use rampeditor::{Address, Color};
	/// 
	/// let mut dat: PaletteData = Default::default();
	/// dat.add_color(Color(255, 0, 0));
	/// dat.add_color(Color(0, 255, 0));
	/// dat.add_color(Color(0, 0, 255));
	///
	/// let red = dat.get_color(Address::new(0, 0, 0)).unwrap();
	/// let blue = dat.get_color(Address::new(0, 0, 1)).unwrap();
	/// let green = dat.get_color(Address::new(0, 0, 2)).unwrap();
	///
	/// assert_eq!(red, Color(255, 0, 0));
	/// assert_eq!(blue, Color(0, 255, 0));
	/// assert_eq!(green, Color(0, 0, 255));
	/// ```
	///
	/// Empty slots are empty:
	/// ```rust
	/// # use rampeditor::palette::PaletteData;
	/// # use rampeditor::{Address, Color};
	/// let dat: PaletteData = Default::default();
	/// assert!(dat.get_color(Address::new(0, 2, 4)).is_none())
	/// ```
	#[inline]
	pub fn get_color(&self, address: Address) -> Option<Color> {
		self.slotmap.get(&address).and_then(|slot| slot.get_color())
	}

	/// Sets the color at the located address. Returns the old color if it 
	/// succeeds, or none if there was no color at the location. Returns an 
	/// error if the address is invalid, or if the element at the address is a
	/// derived color value.
	pub fn set_color(
		&mut self, 
		address: Address,
		new_color: Color) 
		-> Result<Option<Color>>
	{
		if self.check_address(address) {
			self.prepare_address(address);
			let new_element = ColorElement::ZerothOrder {color: new_color};
			if self.slotmap.contains_key(&address) {
				if let Some(slot) = self.slotmap.get(&address) {
					if slot.get_order() != 0 {
						return Err(Error::CannotSetDerivedColor)
					}
					let old_element = &mut*slot.borrow_mut();
					let old = mem::replace(old_element, new_element);
					return Ok(old.get_color())
				} 
			}
		} 
		Err(Error::InvalidAddress)
	}


	/// Adds a new element to the palette in the nearest valid location after 
	/// the group cursor and returns its address. Returns an error if the 
	/// palette is full.
	#[inline]
	pub fn add_element(
		&mut self, 
		new_element: ColorElement) 
		-> Result<Address> 
	{
		self.add_slot(Slot::new(new_element))
	}

	/// Sets the element at the located address. Returns the old element if it 
	/// succeeds, or none if there was no element at the location. Returns an 
	/// error if the address is invalid.
	pub fn set_element(
		&mut self, 
		address: Address, 
		new_element: ColorElement) 
		-> Result<Option<ColorElement>> 
	{
		if self.check_address(address) {
			self.prepare_address(address);
			if self.slotmap.contains_key(&address) {
				if let Some(slot) = self.slotmap.get(&address) {
					let old_element = &mut*slot.borrow_mut();
					let old = mem::replace(old_element, new_element);
					return Ok(Some(old));
				}
			}
			self.slotmap.insert(address, Rc::new(Slot::new(new_element)));
			return Ok(None)
		}
		Err(Error::InvalidAddress)
	}

	/// Adds a new slot to the palette in the nearest valid location after the 
	/// group cursor and returns its address. Returns an error if the 
	/// palette is full.
	#[inline]
	pub fn add_slot(&mut self, new_slot: Slot) -> Result<Address> {
		let address = try!(self.next_free_address_advance_cursor());
		self.slotmap.insert(address, Rc::new(new_slot));
		self.prepare_address(address);
		Ok(address)
	}

	/// Returns the label associated with the given group, or
	/// None if it has no label.
	pub fn get_label(&self, group: Group) -> Option<&str> {
		self.metadata
			.get(&group)
			.and_then(|ref slotmap| slotmap.format_label.as_ref())
			.map(|label| &label[..])
	}

	/// Sets the label for the given group.
	pub fn set_label<S>(
		&mut self, 
		group: Group, 
		format_label: S) 
		where S: Into<String> 
	{
		self.metadata
			.entry(group)
			.or_insert(Default::default())
			.format_label = Some(format_label.into());
	}

	/// Returns the name associated with the given group, or None if it has
	/// no name.
	pub fn get_name(&self, group: Group) -> Option<&str> {
		self.metadata
			.get(&group)
			.and_then(|ref data| data.name.as_ref())
			.map(|name| &name[..])
	}

	/// Sets the name for the given group.
	pub fn set_name<S>(&mut self, group: Group, name: S) 
		where S: Into<String> 
	{
		self.metadata
			.entry(group)
			.or_insert(Default::default())
			.name = Some(name.into());
	}

	/// Returns whether the format's prepare function has been called for the 
	/// given group.
	fn is_initialized(&self, group: Group) -> bool {
		self.metadata
			.get(&group)
			.map_or(false, |ref slotmap| slotmap.initialized)
	}

	/// Sets the format preparation flag for the group.
	pub fn set_initialized(&mut self, group: Group, value: bool) {
		self.metadata
			.entry(group)
			.or_insert(Default::default())
			.initialized = value;
	}

	/// Returns the next available address after the cursor, and also advances
	/// the cursor to the next (wrapped) address. Returns an error and fails to 
	/// advance the cursor if there are no free addresses.
	#[inline]
	fn next_free_address_advance_cursor(&mut self) -> Result<Address> {
		let address = try!(self.next_free_address());
		// Update the cursor.
		self.address_cursor = address.wrapped_next(
			self.page_count,
			self.default_line_count, 
			self.default_column_count
		);
		Ok(address)
	}

	/// Returns the next available address after the cursor. Returns an error if
	/// there are no free addresses.
	#[inline]
	fn next_free_address(&self) -> Result<Address> {
		if self.len() >= (self.page_count as usize * 
			self.default_line_count as usize * self.default_column_count as usize)
		{
			return Err(Error::MaxSlotLimitExceeded);
		}

		let mut address = self.address_cursor;
		while self.slotmap.get(&address).and_then(|s| s.get_color()).is_some() {
			address = address.wrapped_next(
				self.page_count,
				self.default_line_count, 
				self.default_column_count
			);
		}
		Ok(address)
	}

	/// Returns whether the give address lies within the bounds defined by the 
	/// wrapping and max page settings for the palette.
	#[inline]
	fn check_address(&self, address: Address) -> bool {
		address.page < self.page_count &&
		address.line < self.default_line_count &&
		address.column < self.default_column_count
	}

	/// Checks if the groups containing the given addresses have been 
	/// initialized by the Palette format yet, and if not, initializes them.
	#[inline]
	fn prepare_address(&mut self, address: Address) {
		if !self.is_initialized(address.page_group()) {
			(self.prepare_new_page)(self);
			self.set_initialized(address.page_group(), true);
		}
		if !self.is_initialized(address.line_group()) {
			(self.prepare_new_line)(self);
			self.set_initialized(address.line_group(), true);
		}

	}
}


impl fmt::Debug for PaletteData {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		write!(f, "PaletteData {{ \
			slotmap: {:#?}, \
			metadata: {:#?}, \
			address_cursor: {:#?}, \
			page_count: {:#?}, \
			default_line_count: {:#?}, \
			default_column_count: {:#?} }}",
			self.slotmap,
			self.metadata,
			self.address_cursor,
			self.page_count,
			self.default_line_count,
			self.default_column_count
		)
	}
}


impl fmt::Display for PaletteData {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		if let Some(data) = self.metadata.get(&Group::All) {
			try!(write!(f, " {}\n", data));
		}
		try!(write!(f, 
			" [{} pages] [wrap {}:{}] [cursor {}]",
			self.page_count,
			self.default_line_count,
			self.default_column_count,
			self.address_cursor
		));
		

		try!(write!(f, "\n\tAddress   Color    Order  Name\n"));
		for (&address, ref slot) in self.slotmap.iter() {
			try!(write!(f, "\t{:X}  {:X}  {:<5}  ",
				address,
				slot.borrow().get_color().unwrap_or(Color(0,0,0)),
				slot.borrow().get_order()
			));
		}
		Ok(())
	}
}


impl Default for PaletteData {
	fn default() -> Self {
		PaletteData {
			slotmap: BTreeMap::new(),
			metadata: BTreeMap::new(),
			address_cursor: Default::default(),
			page_count: PAGE_MAX,
			default_line_count: LINE_MAX,
			default_column_count: COLUMN_MAX,
			prepare_new_page: no_op,
			prepare_new_line: no_op,
		}
	}
}