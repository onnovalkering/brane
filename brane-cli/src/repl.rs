use crate::docker::DockerExecutor;
use crate::packages;
use anyhow::Result;
use brane_bvm::vm::{Vm, VmOptions};
use brane_drv::grpc::{CreateSessionRequest, DriverServiceClient, ExecuteRequest};
use brane_dsl::{Compiler, CompilerOptions, Lang};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::{CompletionType, Config, Context, EditMode, Editor};
use rustyline_derive::Helper;
use std::borrow::Cow::{self, Borrowed, Owned};
use std::fs;
use std::path::PathBuf;

#[derive(Helper)]
struct ReplHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl Completer for ReplHelper {
    type Candidate = Pair;

    ///
    ///
    ///
    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for ReplHelper {
    type Hint = String;

    ///
    ///
    ///
    fn hint(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Option<String> {
        self.hinter
            .hint(line, pos, ctx)
            .map(|h| h.lines().next().map(|l| l.to_string()))
            .flatten()
    }
}

impl Highlighter for ReplHelper {
    ///
    ///
    ///
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    ///
    ///
    ///
    fn highlight_hint<'h>(
        &self,
        hint: &'h str,
    ) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    ///
    ///
    ///
    fn highlight<'l>(
        &self,
        line: &'l str,
        pos: usize,
    ) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    ///
    ///
    ///
    fn highlight_char(
        &self,
        line: &str,
        pos: usize,
    ) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for ReplHelper {
    ///
    ///
    ///
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    ///
    ///
    ///
    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

///
///
///
fn get_history_file() -> PathBuf {
    appdirs::user_data_dir(Some("brane"), None, false)
        .expect("Couldn't determine Brane data directory.")
        .join("repl_history.txt")
}

///
///
///
pub async fn start(
    bakery: bool,
    clear: bool,
    remote: Option<String>,
    attach: Option<String>,
    data: Option<PathBuf>,
) -> Result<()> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::Circular)
        .edit_mode(EditMode::Emacs)
        .output_stream(OutputStreamType::Stdout)
        .build();

    let repl_helper = ReplHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
        colored_prompt: "".to_owned(),
        validator: MatchingBracketValidator::new(),
    };

    let history_file = get_history_file();
    if clear && history_file.exists() {
        fs::remove_file(&history_file)?;
    }

    let mut rl = Editor::with_config(config);
    rl.set_helper(Some(repl_helper));
    rl.load_history(&history_file).ok();

    println!("Welcome to the Brane REPL, press Ctrl+D to exit.\n");

    if let Some(remote) = remote {
        remote_repl(&mut rl, bakery, remote, attach).await?;
    } else {
        local_repl(&mut rl, bakery, data).await?;
    }

    rl.save_history(&history_file).unwrap();

    Ok(())
}

///
///
///
async fn remote_repl(
    rl: &mut Editor<ReplHelper>,
    _bakery: bool,
    remote: String,
    attach: Option<String>,
) -> Result<()> {
    let mut client = DriverServiceClient::connect(remote).await?;
    let session = if let Some(attach) = attach {
        attach.clone()
    } else {
        let request = CreateSessionRequest {};
        let reply = client.create_session(request).await?;

        reply.into_inner().uuid.clone()
    };

    let mut count: u32 = 1;
    loop {
        let p = format!("{}> ", count);

        rl.helper_mut().expect("No helper").colored_prompt = format!("\x1b[1;32m{}\x1b[0m", p);

        let readline = rl.readline(&p);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                let request = ExecuteRequest {
                    uuid: session.clone(),
                    input: line.clone(),
                };

                let response = client.execute(request).await?;
                let mut stream = response.into_inner();

                #[allow(irrefutable_let_patterns)]
                while let message = stream.message().await {
                    match message {
                        Ok(Some(reply)) => {
                            if !reply.bytecode.is_empty() {
                                debug!("\n{}", reply.bytecode);
                            }
                            if !reply.output.is_empty() {
                                println!("{}", reply.output);
                            }

                            if reply.close {
                                break;
                            }
                        }
                        Err(status) => {
                            eprintln!("\n{}", status.message());
                            break;
                        }
                        Ok(None) => {
                            break;
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Keyboard interrupt not supported. Press Ctrl+D to exit.");
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }

        count += 1;
    }

    Ok(())
}

///
///
///
async fn local_repl(
    rl: &mut Editor<ReplHelper>,
    bakery: bool,
    data: Option<PathBuf>,
) -> Result<()> {
    let compiler_options = if bakery {
        CompilerOptions::new(Lang::Bakery)
    } else {
        CompilerOptions::new(Lang::BraneScript)
    };

    let package_index = packages::get_package_index()?;
    let mut compiler = Compiler::new(compiler_options, package_index.clone());

    let executor = DockerExecutor::new(data);
    let options = VmOptions {
        clear_after_main: true,
        ..Default::default()
    };
    let mut vm = Vm::new_with(executor, Some(package_index), Some(options));

    let mut count: u32 = 1;
    loop {
        let p = format!("{}> ", count);

        rl.helper_mut().expect("No helper").colored_prompt = format!("\x1b[1;32m{}\x1b[0m", p);

        let readline = rl.readline(&p);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match compiler.compile(line) {
                    Ok(function) => vm.main(function).await,
                    Err(error) => eprintln!("{:?}", error),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Keyboard interrupt not supported. Press Ctrl+D to exit.");
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }

        count += 1;
    }

    Ok(())
}
