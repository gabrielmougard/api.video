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

extern crate image;
use image::error::ImageError;

use node::error::AnalyzeError;
use node::error::DrawError;
use pyo3::types::PyBool;


mod node;

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};

use node::*;

use quantization::palette::{DynamicPaletteView};

use pyo3::prelude::*;
use pyo3::types::PyLong;
use pyo3::types::PyFloat;
use pyo3::exceptions::PyRuntimeError;

#[derive(Debug)]
pub struct ImageLoadGenericError; // ImageError type not recognized

#[derive(Debug)]
pub struct ImageLoadDecodingError; // custom msg when ImageError::Decoding

#[derive(Debug)]
pub struct ImageLoadLimitsError; // custom msg when ImageError::Limits

#[derive(Debug)]
pub struct ImageLoadIOError; // custom msg when ImageError::IoError
     

impl fmt::Display for ImageLoadDecodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid image data")
    }
}

impl Error for ImageLoadDecodingError {}


impl fmt::Display for ImageLoadLimitsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Computation limits exceeded")
    }
}

impl Error for ImageLoadLimitsError {}

impl fmt::Display for ImageLoadIOError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "File not found or could not be read")
    }
}

impl Error for ImageLoadIOError {}

impl fmt::Display for ImageLoadGenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occurred while loading the image")
    }
}

impl Error for ImageLoadGenericError {}

#[derive(Debug)]
pub struct QIMSerializationError;

impl Error for QIMSerializationError {}

impl fmt::Display for QIMSerializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occurred while serializing to QIM format")
    }
}

#[derive(Debug)]
pub struct QIMFileOpenOutputError;

impl Error for QIMFileOpenOutputError {}

impl fmt::Display for QIMFileOpenOutputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not open output QIM file")
    }
}

#[derive(Debug)]
pub struct QIMFileOpenInputError;

impl Error for QIMFileOpenInputError {}

impl fmt::Display for QIMFileOpenInputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not open input QIM file")
    }
}

#[derive(Debug)]
pub struct QIMFileWriteError;

impl Error for QIMFileWriteError {}

impl fmt::Display for QIMFileWriteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not write data to QIM file")
    }
}

pub struct TreeWithPalette {
	tree: node::QuadtreeNode<DynamicPaletteView>,
	palette: DynamicPaletteView
}

/// Lib
// DONE
pub fn generate_quadtree(
	path: &str,
	dedup: u32,
	blur: f32,
	sensitivity: usize,
	trim: usize
) -> Result<TreeWithPalette, Box<dyn Error + 'static>> {
	let source = match image::open(path) {
		Ok(i) => i,
		Err(e) => {
			match e {
				ImageError::Decoding(_) => {
					return Err(ImageLoadDecodingError.into());
				},
				ImageError::Limits(_) => {
					return Err(ImageLoadLimitsError.into());
				},
				ImageError::IoError(_) => {
					return Err(ImageLoadIOError.into());
				},
				_ => {
					return Err(ImageLoadGenericError.into());
				}
			};
		}
	}.into_rgba8();

	let palette = quantization::generate_palette::
		<quantization::palette::DynamicPaletteView>(&source, dedup);

	let mut tree: QuadtreeNode<_> = Default::default();
	
	match tree.from_image(&source, &palette, sensitivity, blur, true) {
		Ok(()) => (),
		// TODO: Add support for non-square/non-power-of-two images
		Err(e) => {
			match e {
				AnalyzeError::NonPowerOfTwo => {
					return Err(AnalyzeError::NonPowerOfTwo.into());
				},
				AnalyzeError::NonSquare => {
					return Err(AnalyzeError::NonSquare.into());
				}
			}	
		}
	}
	for _ in 0..trim {
		tree.trim(6);
	}
	Ok(TreeWithPalette{tree, palette})
} 

pub fn generate_img(
	width: u32,
	tree: QuadtreeNode<DynamicPaletteView>,
	palette: DynamicPaletteView,
	output: &str
) -> Result<String, Box<dyn Error + 'static>>{
	let mut output_buf = image::RgbaImage::new(width, width);
	match tree.to_image(&mut output_buf, &palette, None, None, true) {
		Ok(_) => {
			match output_buf.save(output) {
				Ok(_) => Ok(output.to_string()),
				Err(e) => Err(e.into())
			}
		},
		Err(e) => {
			match e {
				DrawError::NonSquare => {
					Err(DrawError::NonSquare.into())
				},
				DrawError::NonPowerOfTwo => {
					Err(DrawError::NonPowerOfTwo.into())
				},
				DrawError::ColorOutOfRange => {
					Err(DrawError::ColorOutOfRange.into())
				}
			}
		}
	}




}

pub fn im2qim(
	input: &str,
	output: &str,
	dedup: u32,
	blur: f32,
	sensitivity: usize,
	trim: usize
) -> Result<String, Box<dyn Error + 'static>> {
	match generate_quadtree(input, dedup, blur, sensitivity, trim) {
		Ok(tree_with_palette) => {
			// the only error that can occur here is a color in the quadtree out
			// of range of the palette, but since the quadtree is generated
			// programmatically from an image, that should not happen.
			// If it does happen, there is a bug in the program to be fixed.
			match tree_with_palette.tree.to_qim(&tree_with_palette.palette) {
				Ok(qim_stream) => {
					let out_fh = File::create(output);
					match out_fh {
						Ok(mut f) => {
							match f.write_all(&qim_stream) {
								Ok(_) => Ok(output.to_string()),
								Err(_) => {
									return Err(QIMFileWriteError.into());
								}
							}
						},
						Err(_) => {
							return Err(QIMFileOpenOutputError.into());
						}
					}
				},
				Err(_) => {
					Err(QIMSerializationError.into())
				}
			}
		},
		Err(e) => {
			Err(e)
		} 
	}
}

pub fn qim2im(
	input: &str,
	output: &str,
	width: u32
) -> Result<String, Box<dyn Error + 'static>> {
	let mut source_data = Vec::new();
	match File::open(input) {
		Ok(mut f) => {
			match f.read_to_end(&mut source_data) {
				Ok(_) => {
					match QuadtreeNode::from_qim(&source_data) {
							Ok((t, p)) => {
								match generate_img(width, t, p, output) {
									Ok(_) => Ok(output.to_string()),
									Err(e) => Err(e.into())
								}
							},
							Err(e) => Err(e.into())
					}
				},
				Err(_) => Err(QIMFileOpenInputError.into())
			}
		},
		Err(_) => Err(QIMFileOpenInputError.into())
	}
}

/// Python FFIs
#[pyfunction]
fn compress(
	input: String,
	output: String,
	dedup_: Option<&PyLong>,
	blur_: Option<&PyFloat>,
	sensitivity_: Option<&PyLong>,
	trim_: Option<&PyLong>,
	width_: Option<&PyLong>,
	to_qim_: Option<&PyBool>,
	from_qim_: Option<&PyBool> 
) -> PyResult<String> {
	// TODO: Instead of PyResult<String>,
	// Consider PyResult<PyCompressionResult>.. `PyCompressionResult` being a custom python class  
	let (dedup, blur, sensitivity, trim, width, from_qim, to_qim) = (
		match dedup_ {
			None => 256,
			Some(d) => {
				let native_t = d.extract()?;
				native_t
			}
		} as u32,
		match blur_ {
			None => 1.0,
			Some(b) => {
				let native_t = b.extract()?;
				native_t
			}
		} as f32,
		match sensitivity_ {
			None => 16128, // formula as below with s = 63
			Some(s) => {
				let native_t: i32 = s.extract()?;				
				(16384 * native_t) / (native_t + 1)
			}
		} as usize,
		match trim_ {
			None => 0,
			Some(t) => {
				let native_t = t.extract()?;
				native_t
			}
		} as usize,
		match width_ {
			None => 512,
			Some(w) => {
				let native_t = w.extract()?;
				native_t
			}
		} as u32,
		match from_qim_ {
			Some(q) => {
				let native_t = q.extract()?;
				native_t
			},
			None => false
		},
		match to_qim_ {
			Some(q) => {
				let native_t = q.extract()?;
				native_t
			},
			None => false
		}
	);

	if from_qim && to_qim {
		return Err(PyRuntimeError::new_err("can not enable `from_qim` and `to_qim` together as they are mutually exclusive parameters"));
	}

	if from_qim && !to_qim {
		// `input` is QIM format and `output` is PNG format.
		// Read QIM file data, recover precomputed quadtree and palette object (not costly) and
		// write PNG image.
		let in_file = input.clone();
		let out_file = output.clone();

		if (input.ends_with(".qim") || input.ends_with(".QIM")) && (output.ends_with(".png") || output.ends_with(".PNG")) {
			match qim2im(input.as_str(), output.as_str(), width) {
				Ok(o) => {
					return Ok(o);
				},
				Err(e) => {
					return Err(PyRuntimeError::new_err(e.to_string()));
				}
			};
		}
		
		return Err(
			PyRuntimeError::new_err(
				format!(
					"input={}, output={} :  Wrong file extensions for config : from_qim={}, to_qim={}",
					in_file.as_str(), out_file.as_str(), from_qim, to_qim
				)
			)
		);
	}

	if !from_qim && to_qim {
		// `input` is PNG format and `output` is QIM format.
		// Generate quadtree and palette from input, serialize them into QIM format and write
		// it on disk for later. Can be interesting to have a DB of precompressed artifacts,
		// when we do `offline` computation (in a distributed queuing system).
		let in_file = input.clone();
		let out_file = output.clone();

		if (input.ends_with(".png") || input.ends_with(".PNG")) && (output.ends_with(".qim") || output.ends_with(".QIM")) {
			match im2qim(input.as_str(), output.as_str(), dedup, blur, sensitivity, trim) {
				Ok(o) => {
					return Ok(o)
				},
				Err(e) => {
					return Err(PyRuntimeError::new_err(e.to_string()));
				}
			};
		}
		return Err(
			PyRuntimeError::new_err(
				format!(
					"input={}, output={} :  Wrong file extensions for config : from_qim={}, to_qim={}",
					in_file.as_str(), out_file.as_str(), from_qim, to_qim
				)
			)
		);
	}

	// Else, default case :
	// `input` is PNG format and `output` is PNG format.
	// Generate quadtree and palette from input, keep them in mem and write PNG image out of it. 
	match generate_quadtree(input.as_str(), dedup, blur, sensitivity, trim) {
		Ok(tree_with_palette) => {
			match generate_img(width, tree_with_palette.tree, tree_with_palette.palette, output.as_str()) {
				Ok(o) => {
					return Ok(o)
				},
				Err(e) => {
					return Err(PyRuntimeError::new_err(e.to_string()));
				}
			}
		},
		Err(e) => {
			Err(PyRuntimeError::new_err(e.to_string()))
		} 
	}
}

#[pymodule]
fn quompressor(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
	m.add_function(wrap_pyfunction!(compress, m)?)?;
	Ok(())
}
