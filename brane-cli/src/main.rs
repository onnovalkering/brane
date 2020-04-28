#[macro_use]
extern crate human_panic;

use log::LevelFilter;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

type FResult<T> = Result<T, failure::Error>;

#[derive(StructOpt)]
#[structopt(name = "brane", about = "The Brane command-line interface.")]
struct CLI {
    #[structopt(short, long, help = "Enable debug mode")]
    debug: bool,
    #[structopt(subcommand)]
    command: SubCommand,
}

#[derive(StructOpt)]
enum SubCommand {
    #[structopt(name = "build", about = "Build a package")]
    Build {
        #[structopt(short, long, help = "Path to the directory to use as context", default_value = ".")]
        context: PathBuf,
        #[structopt(name = "FILE", help = "Path to the file to build, relative to the context")]
        file: PathBuf,
        #[structopt(short, long, help = "Kind of package: container, cwl, open-api, script")]
        kind: String
    },

    #[structopt(name = "list", about = "List packages")]
    List {

    },

    #[structopt(name = "login", about = "Log in to a registry")]
    Login {
        #[structopt(name = "HOST", help = "Hostname of the registry")]
        host: String,
        #[structopt(short, long, help = "Password of the account")]
        password: Option<String>,
        #[structopt(short, long, help = "Username of the account")]
        username: String,
    },

    #[structopt(name = "logout", about = "Log out from a registry")]
    Logout {
        #[structopt(name = "HOST", help = "Hostname of the registry")]
        host: String,
    },

    #[structopt(name = "pull", about = "Pull a package from a registry")]
    Pull {
        #[structopt(name = "NAME", help = "Name of the package")]
        name: String
    },

    #[structopt(name = "push", about = "Push a package to a registry")]
    Push {
        #[structopt(name = "NAME", help = "Name of the package")]
        name: String
    },

    #[structopt(name = "remove", about = "Remove one or more local packages")]
    Remove {
        #[structopt(name = "NAME", help = "Name of the package")]
        name: String
    },

    #[structopt(name = "test", about = "Test a package locally")]
    Test {
        #[structopt(name = "NAME", help = "Name of the package")]
        name: String
    },

    #[structopt(name = "search", about = "Search a registry for packages")]
    Search {
        #[structopt(name = "TERMS", help = "Terms to use as search criteria")]
        terms: Vec<String>,
    },
}

fn main() -> FResult<()> {
    let options = CLI::from_args();

    let mut logger = env_logger::builder();
    logger.format_module_path(false);

    if options.debug {
        logger.filter_level(LevelFilter::Debug).init();
    } else {
        logger.filter_level(LevelFilter::Info).init();

        setup_panic!(Metadata {
            name: "Brane CLI".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: env!("CARGO_PKG_AUTHORS").replace(":", ", ").into(),
            homepage: env!("CARGO_PKG_HOMEPAGE").into(),
        });
    }

    let deps_check = brane::check_dependencies();
    if deps_check.is_err() {
        println!("Dependency not found: Docker (version >= 19.0.0).");
        process::exit(1);
    }

    Ok(())
}
