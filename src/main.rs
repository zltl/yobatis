// yobatis DB code generator program

extern crate clap;
use clap::{App, Arg, SubCommand};
mod gen;
mod init;

use log::{debug, error, info, trace, warn};

fn main() {
    let cli = App::new("yobatis")
        .version("0.1.0")
        .author("Liao Tonglang <liaotonglang@gmail.com>")
        .about("yobatis DB code generator")
        .subcommand(
            SubCommand::with_name("init")
                .about(
                    "Connect to mysql server, get the DDLS and create initial yobatis mapper files",
                )
                .arg(
                    Arg::with_name("host")
                        .short("h")
                        .long("host")
                        .help("MySQL host")
                        .takes_value(true)
                        .required(true)
                        .default_value("localhost"),
                )
                .arg(
                    Arg::with_name("port")
                        .short("P")
                        .long("port")
                        .help("MySQL port")
                        .takes_value(true)
                        .required(true)
                        .default_value("3306"),
                )
                .arg(
                    Arg::with_name("user")
                        .short("u")
                        .long("user")
                        .help("MySQL user")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("password")
                        .short("p")
                        .long("password")
                        .help("MySQL password")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("database")
                        .short("d")
                        .long("database")
                        .help("MySQL database")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .help("Output directory")
                        .takes_value(true)
                        .required(true)
                        .default_value("."),
                ),
        )
        .subcommand(
            SubCommand::with_name("gen")
                .about("Generate C sources code from mapper files")
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .long("input")
                        .help("mapper files dir")
                        .takes_value(true)
                        .required(true)
                        .default_value("."),
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .help("generated source code output dir")
                        .takes_value(true)
                        .required(true)
                        .default_value("."),
                ),
        );
    let matches = cli.get_matches();

    env_logger::init();
    trace!("trace");
    debug!("debug");
    info!("info");
    warn!("warn");
    error!("error");

    if let Some(matches) = matches.subcommand_matches("init") {
        let host = matches.value_of("host").unwrap_or("not kown");
        println!("Value for host: {}", host);
        let dbopt = init::info::DBOpt {
            host: String::from(matches.value_of("host").unwrap()),
            port: matches.value_of("port").unwrap().parse::<i32>().unwrap(),
            user: String::from(matches.value_of("user").unwrap()),
            password: String::from(matches.value_of("password").unwrap()),
            database: String::from(matches.value_of("database").unwrap()),
        };
        let output = matches.value_of("output").unwrap();
        let inf = init::mysql::get_info(&dbopt).unwrap();
        init::mapper::generate(&inf, &output).unwrap();
    }

    if let Some(matches) = matches.subcommand_matches("gen") {
        let input = matches.value_of("input").unwrap();
        let output = matches.value_of("output").unwrap();
        println!("Value for input: {}", input);
        println!("Value for output: {}", output);

        let mappers = gen::mapper::parse_mappers(&input).unwrap();
        gen::genc::gen_c(mappers, &output).unwrap();
    }
}
