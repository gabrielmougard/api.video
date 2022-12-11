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
        .arg_from_usage("<INPUT> 'Path to input file`")
		.arg_from_usage("[OUTPUT] 'Path to output file; defaults to INPUT with a modified file extension`")
        .get_matches();

    let (into, from) = (cli_matches.is_present("into"), cli_matches.is_present("from"));
    match (into, from) {
        (true, true) => exit("Only one of -i/--into and -f/--from must be present", 2),
        (true, false) => {},
        (false, true) => {},
        (false, false) => exit("One of -i/--into and -f/--from must be present", 2)
    }
}