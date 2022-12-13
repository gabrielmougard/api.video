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

use super::quantize::palette::{Color, Palette};

impl<P: Palette + Default> super::QuadtreeNode<P> {
    // Analyzes a traditional image into a quadtree, "rounding" pixel colors
	// to the nearest entries in the palette.
	//
	// See documentation on `mount` for the meaning of `sensitivity`.
	//
	// `blur` is the amount of Gaussian blur to apply to the image before
	// quadtreeifying (to remove noise).
	//
	// `gradient` indicates whether or not to generate the quadtree in a way
	// such that the resultant restored image will be of higher quality
	// (in theory) if `gradient` is passed as `true` to `to_image`.
    pub fn from_image(
        &mut self,
        img: &image::RgbaImage,
        palette: &P,
        sensitivity: usize,
        blur: f32,
        gradient: bool
    ) -> Result<(), AnalyzeError> {
        // Validate image size
        if img.width() != img.height() {
            return Err(AnalyzeError::NonSquare);
        }
        if !img.width().is_power_of_two() {
			return Err(AnalyzeError::NonPowerOfTwo);
		}
        let img_tr = if blur == 0. { img.to_owned() } else { image::imageops::blur(img, blur) };
        let palettified = super::quantize::quantize_to_palette(
            &img_tr,
            palette
        );
        match self.mount(&palettified, palette, None, None, sensitivity, gradient) {
            Ok(_) => (),
            Err(_) => unreachable!("error in mounting")
        }
        Ok(())
    }
}