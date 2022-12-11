

fn main() {
    let cli_matches = clap::App::new("quompressor")
        .version("0.1.0")
        .author("gabrielmougard")
        .about("Converts to and from a quadtree-based image compression format (QIM)")
        .arg_from_usage("-i, --input 'Convert the input file from PNG or JFIF to QIM'")
        .get_matches();
}