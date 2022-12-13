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

pub mod palette;

use std::collections::HashMap;

type BigColor = image::Rgba<isize>;

fn abs_sub(a: u8, b: u8) -> u8 {
	(a as i16 - b as i16).abs() as u8
}

fn vec4_len_squared(a: u8, b: u8, c: u8, d: u8) -> u32 {
	(a as u32 * a as u32) +
	(b as u32 * b as u32) +
	(c as u32 * c as u32) +
	(d as u32 * d as u32)
}

fn color_distance(a: &palette::Color, b: &palette::Color) -> u32 {
	vec4_len_squared(
		abs_sub(a.0[0], b.0[0]),
		abs_sub(a.0[1], b.0[1]),
		abs_sub(a.0[2], b.0[2]),
		abs_sub(a.0[3], b.0[3]),
	)
}

fn dedup_distance(a: &palette::Color, b: &palette::Color) -> u32 {
	vec4_len_squared(
		abs_sub(a.0[0], b.0[0]),
		abs_sub(a.0[1], b.0[1]),
		abs_sub(a.0[2], b.0[2]),
		abs_sub(a.0[3], b.0[3]) / 4, // alpha is less important
	)
}

fn color_add_big(a: BigColor, b: BigColor) -> BigColor {
	image::Rgba::<isize>([
		a.0[0] + b.0[0],
		a.0[1] + b.0[1],
		a.0[2] + b.0[2],
		a.0[3] + b.0[3],
	])
}

fn color_mul(a: &palette::Color, b: &isize) -> BigColor {
	image::Rgba::<isize>([
		a.0[0] as isize * b,
		a.0[1] as isize * b,
		a.0[2] as isize * b,
		a.0[3] as isize * b,
	])
}

fn color_div(a: BigColor, b: isize) -> palette::Color {
	image::Rgba::<u8>([
		(a.0[0] / b) as u8,
		(a.0[1] / b) as u8,
		(a.0[2] / b) as u8,
		(a.0[3] / b) as u8,
	])
}

// Selects a palette of a given size and type through a process similar to
// (but not quite the same as) finding the most commonly used colors in the image.
//
// `dedup_thres` indicates the (squared) limit for how "distant" colors can be
// while still being quantized as one color
pub fn generate_palette<P: palette::DynamicPalette>(
    img: &image::RgbaImage,
    dedup_thres: u32
) -> P {
    let mut successes = HashMap::new();
    for pixel in img.pixels() {
        *successes.entry(*pixel).or_insert(0isize) += 1;
    }
    let mut similars: Vec<Vec<(palette::Color, isize)>> = Vec::new();
    for (col, count) in successes.into_iter() {
        let mut found = false;
        for comp in similars.iter_mut() {
            if dedup_distance(&comp[0].0, &col) < dedup_thres {
                comp.push((col, count));
                found = true;
                break;
            }
        }
        if !found {
            similars.push(vec![(col, count)]);
        }
    }
    let mut rank = Vec::new();
    rank.extend(similars.into_iter().map(|cat| {
        let total = cat.iter().map(|cc| cc.1).sum();
        let col = color_div(
            cat.iter()
                .map(|cc| color_mul(&cc.0, &cc.1))
                .fold(image::Rgba::<isize>([0; 4]), color_add_big),
            total
        );
        (col, total)
    }));
    rank.sort_by_key(|cc: &(palette::Color, isize)| -cc.1);
    P::from(rank.iter().map(|x| x.0).collect())
}

// Process an image given a palette so as to convert it to a "rectangle"
// of pixels each represented by a palette-color-number that most closely
// matches the original color
//
// For the efficiency of the quadtree, the image may be Gaussian-blurred
// before quantization; the extent to which this is done is controlled by `blur`
pub fn quantize_to_palette<P: palette::Palette>(
    img: &image::RgbaImage,
    palette: &P
) -> Vec<u32> {
    let palette_colors = palette.get_slice().map(|x| x.to_owned())
        .unwrap_or_else(|| (0..1 << palette.width())
            .map(|n| palette.to_rgba(n as u32).unwrap())
            .collect::<Vec<_>>());
    let mut quant_cache = HashMap::new();
    img.pixels()
        .map(|pix| {
            match quant_cache.get(pix) {
                Some(c) => *c,
                None => {
                    let c = palette_colors.iter()
                        .enumerate()
                        .map(|(ind, col)| (color_distance(pix, col), ind as u32))
                        .min().unwrap().1;
                    quant_cache.insert(pix, c);
                    c
                }
            }
        })
        .collect::<Vec<_>>()
}
