use image::error::ImageError;

use quompressor::QuadtreeNode;
use quompressor::quantization;
use quompressor::error::DrawError;

use std::fs::File;
use std::io::{Read, Write};

fn exit(msg: &str, code: i32) -> ! {
    eprintln!("{}", msg);
    std::process::exit(code);
}

fn main() {
    let cli_matches = clap::App::new("quompressor")
        .version("0.1.0")
        .author("gabrielmougard")
        .about("Converts to and from a QIM image compression format")
        .arg_from_usage("-i, --into 'Convert the input file from PNG or JFIF to QIM'")
        .arg_from_usage("-f, --from 'Convert the input file from QIM to PNG'")
        .arg_from_usage("-d, --dedup=[N] 'Color distance threshold for palette deduplication (--into only); defaults to 256'")
		.arg_from_usage("-b, --blur=[N] 'Amount of precompression blur (--into only); defaults to 1'")
		.arg_from_usage("-s, --sensitivity=[N] 'Noise sensitivity as a fraction S/(S+1) (--into only); defaults to 63/64'")
		.arg_from_usage("-t, --trim=[N] 'Number of times to trim output (--into only); defaults to 0'")
		.arg_from_usage("-w, --width=[N] 'Output image width (and, for now, also height) (--from only); must be a power of two; defaults to 512'")
        .arg_from_usage("<INPUT> 'Path to input file`")
		.arg_from_usage("[OUTPUT] 'Path to output file; defaults to INPUT with a modified file extension`")
        .get_matches();

    let (into, from) = (cli_matches.is_present("into"), cli_matches.is_present("from"));
    match (into, from) {
        (true, true) => exit("Only one of -i/--into and -f/--from must be present", 2),
        (true, false) => {
            let path = cli_matches.value_of("INPUT").unwrap();
            let source = match image::open(path) {
                Ok(i) => i,
                Err(e) => {
                    let (msg, code) = match e {
                        ImageError::Decoding(_) => ("Invalid image data", 4),
                        ImageError::Limits(_) => ("Computation limits exceeded", 5),
                        ImageError::IoError(_) => ("File not found or could not be read", 3),
                        _ => ("An error occurred", 10)
                    };
                    exit(msg, code);
                }
            }.into_rgba8();
            let (dedup, blur, sensitivity, trim) = (
				match cli_matches.value_of("dedup").unwrap_or("256").parse() {
					Ok(n) => n,
					Err(_) => exit("Non-numeric value for dedup", 2)
				},
				match cli_matches.value_of("blur").unwrap_or("1").parse() {
					Ok(n) => n,
					Err(_) => exit("Non-numeric value for blur", 2)
				},
				match cli_matches.value_of("sensitivity").unwrap_or("63").parse::<usize>() {
					Ok(n) => (16384 * n) / (n + 1),
					Err(_) => exit("Non-numeric value for sensitivity", 2)
				},
				match cli_matches.value_of("trim").unwrap_or("0").parse::<usize>() {
					Ok(n) => n,
					Err(_) => exit("Non-numeric value for trim", 2)
				}
			);

            let palette = quantization::generate_palette::
				<quantization::palette::DynamicPaletteView>(&source, dedup);
			eprintln!("{} colors in generated palette", palette.colors.len());
			let mut tree: QuadtreeNode<_> = Default::default();
			// TODO: Allow runtime configuration of gradient mode
			match tree.from_image(&source, &palette, sensitivity, blur, true) {
				Ok(()) => (),
				// TODO: Add support for non-square/non-power-of-two images
				Err(_) => exit("Input image has invalid dimensions", 4)
			}
			for _ in 0..trim {
				// TODO: Allow runtime configuration of trim depth
				// And perhaps improve trim with a sensitivity parameter?
				tree.trim(6);
			}
            // `.expect()` is valid here, because the only error that can occur here
			// is a color in the quadtree out of range of the palette, but since the
			// quadtree is generated programmatically from an image, that should not
			// happen. If it does happen, there is a bug in the program to be fixed.
			let qim_data = tree.to_qim(&palette).expect("failure to serialize to QIM");
			let mut out_fh = match File::create(cli_matches.value_of("OUTPUT")
				.unwrap_or(&(path.rsplitn(2, '.').last().unwrap().to_string() + ".qim"))) {
				Ok(f) => f,
				Err(_) => exit("Could not open output file", 3)
			};
			match out_fh.write_all(&qim_data) {
				Ok(_) => (),
				Err(_) => exit("Could not write to output file", 3)
			}
        },
        (false, true) => {
            let input_path = cli_matches.value_of("INPUT").unwrap();
			let mut source_data = Vec::new();
			let mut source_fh = match File::open(input_path) {
				Ok(f) => f,
				Err(_) => exit("File not found or could not be read", 3)
			};
			match source_fh.read_to_end(&mut source_data) {
				Ok(_) => (),
				Err(_) => exit("Could not read from input file", 3)
			}
			let (tree, palette): (_, quantization::palette::DynamicPaletteView) =
				match QuadtreeNode::from_qim(&source_data) {
				Ok((t, p)) => (t, p),
				Err(_) => exit("Invalid image data", 4)
			};
			let width = match cli_matches.value_of("width").unwrap_or("512").parse() {
				Ok(n) => n,
				Err(_) => exit("Non-numeric value for width", 2)
			};
            let mut output = image::RgbaImage::new(width, width);
			// TODO: Allow runtime configuration of gradient mode
			match tree.to_image(&mut output, &palette, None, None, true) {
				Ok(_) => (),
				Err(e) => {
					let (msg, code) = match e {
						DrawError::NonSquare |
						DrawError::NonPowerOfTwo => ("Invalid output dimensions", 2),
						DrawError::ColorOutOfRange => ("Invalid image data", 4)
					};
					exit(msg, code)
				}
			}
			match output.save(cli_matches.value_of("OUTPUT")
				.unwrap_or(&(input_path.rsplitn(2, '.').last().unwrap().to_string() + ".png"))) {
				Ok(_) => (),
				Err(_) => exit("Could not save output", 3)
			}
        },
        (false, false) => exit("One of -i/--into and -f/--from must be present", 2)
    }
}