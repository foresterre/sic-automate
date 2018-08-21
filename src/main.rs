#![feature(extern_prelude)]

extern crate arrayvec;
extern crate clap;
extern crate image;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use clap::{App, Arg};

use std::path::Path;
use std::process;

mod conversion;
mod help;
mod operations;

const SIC_LICENSE: &str = include_str!("../LICENSE");
const DEP_LICENSES: &str = include_str!("../DEP");

const HELP_OPERATIONS_AVAILABLE: &str = include_str!("../docs/cli_help_script.txt");

fn main() {
    let matches = App::new("Simple Image Converter")
        .version("0.6.0")
        .author("Martijn Gribnau <garm@ilumeo.com>")
        .about("Converts an image from one format to another.\n\n\
                Supported input formats are described BMP, GIF, ICO, JPEG, PNG, PPM (limitations may apply). \n\n\
                The image conversion is actually done by the awesome 'image' crate [1]. \n\
                Sic itself is a small command line frontend which supports a small part of the \
                conversion operations supported by the 'image' library. \n\n\
                [1] image crate by PistonDevelopers: https://github.com/PistonDevelopers/image \n\n\
                ")
        .arg(Arg::with_name("forced_output_format")
            .short("f")
            .long("force-format")
            .value_name("FORMAT")
            .help("Output formats supported: JPEG, PNG, GIF, ICO, PPM")
            .takes_value(true))
        .arg(Arg::with_name("license")
            .long("license")
            .help("Displays the license of the `sic` software.")
            .takes_value(false))
        .arg(Arg::with_name("dep-licenses")
            .long("dep-licenses")
            .help("Displays the licenses of the dependencies on which this software relies.")
            .takes_value(false))
        .arg(Arg::with_name("script-user-manual")
            .long("script-manual")
            .help("Displays help text for each supported script operation.")
            .takes_value(true))
        .arg(Arg::with_name("script")
            .long("script")
            .help(HELP_OPERATIONS_AVAILABLE)
            .value_name("SCRIPT")
            .takes_value(true))
        .arg(Arg::with_name("input_file")
            .help("Sets the input file")
            .value_name("INPUT_FILE")
            .required_unless_one(&["license", "dep-licenses", "script-user-manual"])
            .index(1))
        .arg(Arg::with_name("output_file")
            .help("Sets the output file")
            .value_name("OUTPUT_FILE")
            .required_unless_one(&["license", "dep-licenses", "script-user-manual"])
            .index(2))
        .get_matches();

    match (
        matches.is_present("license"),
        matches.is_present("dep-licenses"),
        matches.is_present("script-user-manual"),
    ) {
        (true, true, _) => {
            println!(
                "Simple Image Converter license:\n{} \n\n{}",
                SIC_LICENSE, DEP_LICENSES
            );
            process::exit(0);
        }
        (true, _, _) => {
            println!("{}", SIC_LICENSE);
            process::exit(0);
        }
        (_, true, _) => {
            println!("{}", DEP_LICENSES);
            process::exit(0);
        }
        (false, false, true) => {
            if let Some(topic) = matches.value_of("script-user-manual") {
                let help_text = help::UserManual::help_text(topic);

                match help_text {
                    Ok(text) => {
                        println!("Help page for script command: {}\n\n{}", topic, text);
                        process::exit(0);
                    }
                    Err(err) => {
                        println!("Unable to display help: {}", err);
                        process::exit(94);
                    }
                }
            }
        }
        _ => {}
    }

    // Can be unwrap because these values are required arguments.
    let input = matches.value_of("input_file").unwrap();
    let output = matches.value_of("output_file").unwrap();
    println!("Provided input file path: {}", input);
    println!("Provided output file path: {}", output);

    let image_buffer: Result<image::DynamicImage, String> =
        image::open(&Path::new(input)).map_err(|err| err.to_string());

    // perform image operations
    let operated_buffer = match matches.value_of("script") {
        Some(script) => {
            println!("Preparing to apply image operations: `{}`", script);
            image_buffer.map_err(|err| err.to_string()).and_then(|img| {
                println!("Applying image operations.");
                operations::parse_and_apply_script(img, script)
            })
        }
        None => image_buffer,
    };

    // encode
    let forced_format = matches.value_of("forced_output_format");
    let encode_buffer: Result<(), String> = operated_buffer
        .map_err(|err| err.to_string())
        .and_then(|img| {
            forced_format.map_or_else(
                || conversion::convert_image_unforced(&img, output),
                |format| conversion::convert_image_forced(&img, output, format),
            )
        });

    match encode_buffer {
        Ok(_) => println!("Operations complete."),
        Err(err) => println!("Operations ended with an Error: {}", err),
    }
}
