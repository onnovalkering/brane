#[macro_use]
extern crate human_panic;

use anyhow::Result;
use brane_cli::{build_cwl, build_dsl, build_ecu, build_oas, packages, registry, repl, run, test};
use dotenv::dotenv;
use git2::Repository;
use log::LevelFilter;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;
use tempfile::tempdir;

#[derive(StructOpt)]
#[structopt(name = "brane", about = "The Brane command-line interface.")]
struct CLI {
    #[structopt(short, long, help = "Enable debug mode")]
    debug: bool,
    #[structopt(short, long, help = "Skip dependencies check")]
    skip_check: bool,
    #[structopt(subcommand)]
    sub_command: SubCommand,
}

#[derive(StructOpt)]
enum SubCommand {
    #[structopt(name = "build", about = "Build a package")]
    Build {
        #[structopt(short, long, help = "Path to the directory to use as context", default_value = ".")]
        context: PathBuf,
        #[structopt(name = "FILE", help = "Path to the file to build, relative to the context")]
        file: PathBuf,
        #[structopt(short, long, help = "Kind of package: cwl, dsl, ecu or oas")]
        kind: Option<String>,
        #[structopt(short, long, help = "Path to the init binary to use (override Brane's binary)")]
        init: Option<PathBuf>,
    },

    #[structopt(name = "import", about = "Import a package")]
    Import {
        #[structopt(name = "REPO", help = "Name of the GitHub repository containt the package")]
        repo: String,
        #[structopt(short, long, help = "Path to the directory to use as context", default_value = ".")]
        context: PathBuf,
        #[structopt(short, long, help = "Path to the file to build, relative to the context")]
        file: Option<PathBuf>,
        #[structopt(short, long, help = "Kind of package: cwl, dsl, ecu or oas")]
        kind: Option<String>,
        #[structopt(short, long, help = "Path to the init binary to use (override Brane's binary)")]
        init: Option<PathBuf>,
    },

    #[structopt(name = "inspect", about = "Inspect a package")]
    Inspect {
        #[structopt(name = "NAME", help = "Name of the package")]
        name: String,
        #[structopt(name = "VERSION", help = "Version of the package", default_value = "latest")]
        version: String,
    },

    #[structopt(name = "list", about = "List packages")]
    List {},

    #[structopt(name = "load", about = "Load a package locally")]
    Load {
        #[structopt(name = "NAME", help = "Name of the package")]
        name: String,
        #[structopt(short, long, help = "Version of the package")]
        version: Option<String>,
    },

    #[structopt(name = "login", about = "Log in to a registry")]
    Login {
        #[structopt(name = "HOST", help = "Hostname of the registry")]
        host: String,
        #[structopt(short, long, help = "Username of the account")]
        username: String,
    },

    #[structopt(name = "logout", about = "Log out from a registry")]
    Logout {},

    #[structopt(name = "pull", about = "Pull a package from a registry")]
    Pull {
        #[structopt(name = "NAME", help = "Name of the package")]
        name: String,
        #[structopt(name = "VERSION", help = "Version of the package")]
        version: Option<String>,
    },

    #[structopt(name = "push", about = "Push a package to a registry")]
    Push {
        #[structopt(name = "NAME", help = "Name of the package")]
        name: String,
        #[structopt(name = "VERSION", help = "Version of the package")]
        version: String,
    },

    #[structopt(name = "remove", about = "Remove a local package")]
    Remove {
        #[structopt(name = "NAME", help = "Name of the package")]
        name: String,
        #[structopt(short, long, help = "Version of the package")]
        version: Option<String>,
        #[structopt(short, long, help = "Don't ask for confirmation")]
        force: bool,
    },

    #[structopt(name = "repl", about = "Start an interactive DSL session")]
    Repl {
        #[structopt(short, long, help = "Clear history before session")]
        clear: bool,
    },

    #[structopt(name = "run", about = "Run a DSL script locally")]
    Run {
        #[structopt(name = "FILE", help = "Path to the file to run")]
        file: PathBuf,
        #[structopt(short, long, name = "SECRETS", help = "File containing secrets")]
        secrets: Option<PathBuf>,
    },

    #[structopt(name = "test", about = "Test a package locally")]
    Test {
        #[structopt(name = "NAME", help = "Name of the package")]
        name: String,
        #[structopt(short, long, help = "Version of the package")]
        version: Option<String>,
    },

    #[structopt(name = "search", about = "Search a registry for packages")]
    Search {
        #[structopt(name = "TERM", help = "Term to use as search criteria")]
        term: String,
    },

    #[structopt(name = "unpublish", about = "Remove a package from a registry")]
    Unpublish {
        #[structopt(name = "NAME", help = "Name of the package")]
        name: String,
        #[structopt(name = "VERSION", help = "Version of the package")]
        version: String,
        #[structopt(short, long, help = "Don't ask for confirmation")]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
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

    if !options.skip_check {
        let deps_check = brane_cli::check_dependencies();
        if deps_check.is_err() {
            println!("Dependency not found: Docker (version >= 19.0.0).");
            process::exit(1);
        }
    }

    match run(options).await {
        Ok(_) => process::exit(0),
        Err(error) => {
            println!("{:?}", error); // Anyhow
            process::exit(1);
        }
    }
}

///
///
///
async fn run(options: CLI) -> Result<()> {
    use SubCommand::*;
    match options.sub_command {
        Build {
            context,
            file,
            kind,
            init,
        } => {
            let kind = if let Some(kind) = kind {
                kind.to_lowercase()
            } else {
                brane_cli::determine_kind(&context, &file)?
            };

            match kind.as_str() {
                "cwl" => build_cwl::handle(context, file, init)?,
                "dsl" => build_dsl::handle(context, file).await?,
                "ecu" => build_ecu::handle(context, file, init)?,
                "oas" => build_oas::handle(context, file, init)?,
                _ => println!("Unsupported package kind: {}", kind),
            }
        }
        Import {
            repo,
            context,
            file,
            kind,
            init,
        } => {
            let url = format!("https://github.com/{}", repo);
            let dir = tempdir()?;

            if let Err(e) = Repository::clone(&url, &dir) {
                panic!("Failed to clone: {}", e);
            };

            let context = dir.path().join(context);

            let file = if let Some(file) = file {
                file
            } else {
                brane_cli::determine_file(&context)?
            };

            let kind = if let Some(kind) = kind {
                kind.to_lowercase()
            } else {
                brane_cli::determine_kind(&context, &file)?
            };

            match kind.as_str() {
                "cwl" => build_cwl::handle(context, file, init)?,
                "dsl" => build_dsl::handle(context, file).await?,
                "ecu" => build_ecu::handle(context, file, init)?,
                "oas" => build_oas::handle(context, file, init)?,
                _ => println!("Unsupported package kind: {}", kind),
            }
        }

        Inspect { name, version } => {
            packages::inspect(name, version)?;
        }
        List {} => {
            packages::list()?;
        }
        Load { name, version } => {
            packages::load(name, version).await?;
        }
        Login { host, username } => {
            registry::login(host, username)?;
        }
        Logout {} => {
            registry::logout()?;
        }
        Pull { name, version } => {
            registry::pull(name, version).await?;
        }
        Push { name, version } => {
            registry::push(name, version).await?;
        }
        Remove { name, version, force } => {
            packages::remove(name, version, force).await?;
        }
        Repl { clear } => {
            repl::start(clear).await?;
        }
        Run { file, secrets } => {
            run::handle(file, secrets).await?;
        }
        Test { name, version } => {
            test::handle(name, version).await?;
        }
        Search { term } => {
            registry::search(term).await?;
        }
        Unpublish { name, version, force } => {
            registry::unpublish(name, version, force).await?;
        }
    }

    Ok(())
}
