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

// Reason why an image couldn't be turned into a quadtree.
#[derive(Debug)]
pub enum AnalyzeError {
	// The image buffer's dimensions are not equal; the image is not a square.
	NonSquare,
	// The image buffer's dimensions are not powers of two.
	NonPowerOfTwo,
}

// Reason why a quadtree couldn't be encoded.
#[derive(Debug)]
pub enum EncodeError {
// A color specified in the quadtree is outside the range of the palette.
	ColorOutOfRange,
}

// Reason why a quadtree encoding couldn't be decoded.
#[derive(Debug)]
pub enum DecodeError {
	// A node number was exepcted but not found.
	InsufficientData,
	// There was no valid QIM file header.
	MissingHeader,
	// `GenericPalette` could not stored a palette of the necessary size.
	PaletteTooLarge,
}

// Reason why an "image" of palette colors couldn't be made into a quadtree.
#[derive(Debug)]
pub enum MountError {
	// The size of the "image" buffer is not a power of 4.
	InvalidSize,
	// A pixel has a color outside the extent of the palette.
	ColorOutOfRange,
}

// Reason why a quadtree couldn't be rendered to an image buffer.
#[derive(Debug)]
pub enum DrawError {
	// The image buffer's dimensions are not equal; the image is not a square.
	NonSquare,
	// The image buffer's dimensions are not powers of two.
	NonPowerOfTwo,
	// A color specified in the quadtree is outside the range of the palette.
	ColorOutOfRange,
}
