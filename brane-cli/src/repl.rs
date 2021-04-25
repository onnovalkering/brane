use crate::docker::{self, ExecuteInfo};
use crate::{packages, registry};
use anyhow::{Context as _, Result};
use brane_bvm::values::Value;
use brane_bvm::{VmCall, VmResult, VM};
use brane_drv::grpc::{CreateSessionRequest, DriverServiceClient, ExecuteRequest};
use brane_dsl::{Compiler, CompilerOptions};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::{CompletionType, Config, Context, EditMode, Editor};
use rustyline_derive::Helper;
use serde::de::DeserializeOwned;
use specifications::common::Value as SpecValue;
use specifications::package::PackageInfo;
use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashMap;
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
    clear: bool,
    remote: Option<String>,
    attach: Option<String>,
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
        remote_repl(&mut rl, remote, attach).await?;
    } else {
        local_repl(&mut rl).await?;
    }

    rl.save_history(&history_file).unwrap();

    Ok(())
}

///
///
///
async fn remote_repl(
    rl: &mut Editor<ReplHelper>,
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
                println!("{}", response.into_inner().output);
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
async fn local_repl(rl: &mut Editor<ReplHelper>) -> Result<()> {
    let compiler_options = CompilerOptions::new();
    let package_index = registry::get_package_index().await?;

    let mut compiler = Compiler::new(compiler_options, package_index.clone());
    let mut vm = VM::new(package_index, None);

    let mut count: u32 = 1;
    loop {
        let p = format!("{}> ", count);

        rl.helper_mut().expect("No helper").colored_prompt = format!("\x1b[1;32m{}\x1b[0m", p);

        let readline = rl.readline(&p);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match compiler.compile(line) {
                    Ok(function) => {
                        vm.call(function, 1usize);

                        loop {
                            match vm.run(None) {
                                VmResult::Call(call) => {
                                    let result = make_function_call(call).await?;
                                    println!("{:?}", result);
                                    vm.result(result);
                                }
                                VmResult::Ok(value) => {
                                    println!("ok: {:?}", value);
                                    break;
                                }
                                VmResult::RuntimeError => {
                                    eprintln!("Runtime errro!");
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{:?}", e);
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
async fn make_function_call(call: VmCall) -> Result<Value> {
    let package_dir = packages::get_package_dir(&call.package, Some("latest"))?;
    let package_file = package_dir.join("package.yml");
    let package_info = PackageInfo::from_path(package_file)?;

    let image = format!("{}:{}", package_info.name, package_info.version);
    let image_file = Some(package_dir.join("image.tar"));

    let command = vec![
        String::from("--application-id"),
        String::from("test"),
        String::from("--location-id"),
        String::from("localhost"),
        String::from("--job-id"),
        String::from("1"),
        String::from("code"),
        call.function.clone(),
        base64::encode(serde_json::to_string(&call.arguments)?),
    ];

    let exec = ExecuteInfo::new(image, image_file, None, Some(command));

    let (stdout, stderr) = docker::run_and_wait(exec).await?;
    debug!("stderr: {}", stderr);
    debug!("stdout: {}", stdout);

    let output = stdout.lines().last().unwrap_or_default().to_string();
    let output: Result<SpecValue> = decode_b64(output);
    match output {
        Ok(value) => Ok(Value::from(value)),
        Err(err) => {
            println!("{:?}", err);
            Ok(Value::Unit)
        }
    }
}

///
///
///
fn decode_b64<T>(input: String) -> Result<T>
where
    T: DeserializeOwned,
{
    let input =
        base64::decode(input).with_context(|| "Decoding failed, encoded input doesn't seem to be Base64 encoded.")?;

    let input = String::from_utf8(input[..].to_vec())
        .with_context(|| "Conversion failed, decoded input doesn't seem to be UTF-8 encoded.")?;

    serde_json::from_str(&input)
        .with_context(|| "Deserialization failed, decoded input doesn't seem to be as expected.")
}
