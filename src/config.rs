use clap::{App, Arg};

pub struct Config {
    pub print_all_glyphs: bool,
    pub debug: bool,
    pub input_string: String,
    pub font_path: String,
    pub outline_thickness: i32,
}

impl Config {
    pub fn from_args() -> Self {
        let matches = App::new("Glyph Renderer")
            .version("1.0")
            .author("Henry Goodman <henrygoodman114@gmail.com>")
            .arg(
                Arg::new("print-all-glyphs")
                    .short('p')
                    .long("print-all-glyphs")
                    .takes_value(false)
                    .help("Draw all available glyphs"),
            )
            .arg(
                Arg::new("debug")
                    .short('d')
                    .long("debug")
                    .takes_value(false)
                    .help("Enable debug visuals"),
            )
            .arg(
                Arg::new("font")
                    .short('f')
                    .long("font")
                    .takes_value(true)
                    .help("Path to the font file")
                    .default_value("fonts/JetBrainsMono-Bold.ttf"),
            )
            .arg(
                Arg::new("input")
                    .help("The input string to render")
                    .index(1),
            )
            .get_matches();

        let print_all_glyphs = matches.is_present("print-all-glyphs");
        let debug = matches.is_present("debug");
        let input_string = matches.value_of("input").unwrap_or("Hello, World!").to_string();
        let font_path = matches.value_of("font").unwrap_or("fonts/JetBrainsMono-Bold.ttf").to_string();

        Config {
            print_all_glyphs,
            debug,
            input_string,
            font_path,
            outline_thickness: 2,
        }
    }
}
