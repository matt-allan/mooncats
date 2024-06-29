use clap::{Arg, ArgMatches, Command};
use mooncats::mdbook::MoonCats;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use semver::{Version, VersionReq};
use std::io;
use std::process;

pub fn make_app() -> Command {
    Command::new("mdbook-mooncats")
        .about("A mdbook preprocessor for generating luaCATS API docs")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    env_logger::init();
    
    let matches = make_app().get_matches();

    let preprocessor = MoonCats::new();

    match matches.subcommand() {
        Some(("supports", subargs)) => handle_supports(&preprocessor, subargs),
        Some((cmd, _)) => eprintln!("unknown subcommand {}", cmd),
        None => {
            if let Err(e) = handle_preprocessing(&preprocessor) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    let book_version = Version::parse(&ctx.mdbook_version)?;
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION)?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    let renderer = sub_args
        .get_one::<String>("renderer")
        .expect("Required argument");
    let supported = pre.supports_renderer(renderer);

    if supported {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
