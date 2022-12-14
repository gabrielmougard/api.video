// Copyright 2022 gab
// 
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// 
//     http://www.apache.org/licenses/LICENSE-2.0
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use bitvec::vec::BitVec;

use super::error::*;
use super::quantization::palette::{DynamicPalette, Palette};

use std::collections::HashMap;

// A `BitVec` variant ideal for encoding and decoding quadtrees.
type QuadtreeEncodeBitVec = BitVec<bitvec::order::Msb0, u8>;

impl<P: Palette + Default> super::QuadtreeNode<P> {
    // Converts the `QuadtreeNode` into a binary data format.
	//
	// Takes the bit width of the palette and converts each node into a
	// palette color, plus an extra bit at the start to indicate containing
	// subsections; each node's number will be immediately followed by the
	// numbers for its subsections.
	//
	// Palette color numbers are bitwise big-endian.
	pub fn encode(
		&self,
		buffer: &mut QuadtreeEncodeBitVec,
		palette: &P
	) -> Result<(), EncodeError> {
		// Validate color value
		if self.color >= 1 << palette.width() {
			return Err(EncodeError::ColorOutOfRange);
		}
		// Bit to indicate subsections
		buffer.push(self.sections.is_some());
		// Color number
		for bit_ind in 0..palette.width() {
			buffer.push(self.color & (1 << (palette.width() - bit_ind - 1)) != 0);
		}
		// Recursion
		if let Some(ref sects) = self.sections {
			for section in sects.iter() {
				section.encode(buffer, palette)?;
			}
		}
		Ok(())
	}

    // Reads a `BitVec` of the sort that would be output from `.encode()`
	// and parses a quadtree from it.
	//
	// Successful return value is the index to which the parser has progressed,
	// to assist with the recursive algorithm.
	//
	// 0 should be passed for `curr_ind` by outside callers, unless they
	// know what they're doing and have a good reason otherwise.
	pub fn decode(
		&mut self,
		buffer: &QuadtreeEncodeBitVec,
		palette: &P,
		mut curr_ind: usize
	) -> Result<usize, DecodeError> {
		// Validate data quantity
		if buffer.len() - curr_ind < (palette.width()) as usize {
			return Err(DecodeError::InsufficientData);
		}
		// Extract current node
		let mut n = 0;
		for bit_ind in 0..(palette.width()) {
			n |= (buffer[curr_ind + bit_ind as usize + 1] as u32) << (palette.width() - bit_ind - 1);
		}
		self.color = n;
		// Recursion
		let should_recurse = buffer[curr_ind];
		curr_ind += 1 + palette.width() as usize;
		if should_recurse {
			self.sections = Some(Default::default());
			for sect_ind in 0..4 {
				curr_ind = self.sections.as_mut().unwrap()[sect_ind]
					.decode(buffer, palette, curr_ind)?;
			}
		}
		Ok(curr_ind)
	}

    // Encodes the quadtree and a palette into QIM data.
	pub fn to_qim(&self, palette: &P) -> Result<Vec<u8>, EncodeError> {
		let mut ret = Vec::new();
		// Header (version 1)
		ret.extend_from_slice(b"QuadIM\x01");
		let mut palette_vec = palette.get_slice()
			.map(|x| x.to_owned())
			.unwrap_or_else(|| (0..palette.width() << 1)
				.map(|n| palette.to_rgba(n as u32).unwrap())
				.collect::<Vec<_>>());
		palette_vec.resize(1 << palette.width(), image::Rgba([0; 4]));
		let palette_len = std::cmp::max((1 << palette.width()) - palette_vec.iter()
			.rev()
			.take_while(|c| **c == image::Rgba([0; 4]))
			.count(),
			(9 * (1 << palette.width()) + 15) / 16);
		let approx_len = (palette_len as f64 * 16. / (1 << palette.width()) as f64)
			.ceil() as u32 * (1 << palette.width()) / 16;
		// Length indicator
		ret.push((((approx_len * 16) / (1 << palette.width()) - 9) << 5) as u8 |
			(palette.width() - 1));
		// Palette
		for c in 0..approx_len {
			ret.extend_from_slice(&palette.to_rgba(c).unwrap().0);
		}
		// Quadtree
		let mut bit_buf = QuadtreeEncodeBitVec::new();
		self.encode(&mut bit_buf, palette)?;
		ret.extend_from_slice(bit_buf.as_slice());
		Ok(ret)
	}

	// "Trims" the tree by removing leaf nodes.
	//
	// Only leaf nodes past a depth of `depth` and with color repetition
	// will be removed.
	pub fn trim(&mut self, depth: isize) {
		if let Some(sections) = &mut self.sections {
			if depth <= 0 && sections.iter().all(|s| s.sections.is_none()) {
				// Count unique colors
				let col_f = sections.iter().fold(HashMap::new(),
					|mut m, e| { *m.entry(e.color).or_insert(0) += 1; m });
				let freq = col_f.values().collect::<Vec<_>>();
				if freq.len() == 3 || (freq.len() == 2 && **freq.iter().max().unwrap() == 3) {
					self.sections = None;
				}
			} else {
				sections.iter_mut().for_each(|s| s.trim(depth - 1));
			}
		}
	}
}

impl<'a, P: DynamicPalette + Default + std::fmt::Debug> super::QuadtreeNode<P> {
    // Derives a palette and quadtree from the data of a QIM file.
	pub fn from_qim(source: &[u8]) -> Result<(super::QuadtreeNode<P>, P), DecodeError> {
		// Verify header (version 1 is required for compatibility)
		if &source[..6] != b"QuadIM" {
			return Err(DecodeError::MissingHeader);
		}
		let pal_size = (source[7] & 0x1f) + 1;
		let pal_len = (
			((source[7] >> 5) as f64 + 9.) *
			(pal_size as f64 - 4.).exp2()
		) as u32;
		assert!(pal_len.count_ones() <= 4);
		// Extract palette
		let mut pal = vec![];
		for offset in (0..pal_len).map(|n| n as usize * 4 + 8) {
			pal.push(image::Rgba([
				source[offset],
				source[offset + 1],
				source[offset + 2],
				source[offset + 3],
			]));
		}
		pal.resize(1 << pal_size, image::Rgba([0; 4]));
		let palette = P::from(pal);
		// Decode tree
		let tree_bits = QuadtreeEncodeBitVec::from(&source[8 + 4 * pal_len as usize..]);
		let mut tree: super::QuadtreeNode<P> = Default::default();
		match source[6] {
			1 => { // Version one. Maybe other decoder in the future ?
				tree.decode(&tree_bits, &palette, 0)?;
				Ok((tree, palette))
			},
			_ => Err(DecodeError::MissingHeader)
		}
	}
}
