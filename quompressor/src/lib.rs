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

pub mod node;

pub use node::*;

use quantization::palette::{Palette};

use std::collections::HashMap;

impl<P: Palette + Default> node::QuadtreeNode<P> {

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