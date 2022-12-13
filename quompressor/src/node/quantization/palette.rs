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

pub type Color = image::Rgba<u8>;

// Trait for types that describe how to convert from an arbitrary number
// of a fixed size to four bytes of RGBA
pub trait Palette: Default {
    // The bit width of each palette color's number

    // Must be `1 <= WIDTH <= 32`, because 0 bits wouldn't really be a palette
    // and more than 32 bits would be more efficiently represented as direct RGBA
    fn width(&self) -> u8;
    // Uses an instance of the implementing type to convert a number
    // representing a palette entry into an RGBA value.
    // If `c` is outside the range of the palette, an `Err` should be returned
    fn to_rgba(&self, c: u32) -> Result<Color, ()>;
    // Returns a reference to the slice listing the colors in the palette,
    // only if that oss applicable and possible given the way the colors are stored
    fn get_slice(&self) -> Option<&[Color]>;
}

// Marker trait for `Palette` implementors that can be made from lists of
// dynamic length (`Vec`s, that is).
pub trait DynamicPalette: Palette + From<Vec<Color>> {}
