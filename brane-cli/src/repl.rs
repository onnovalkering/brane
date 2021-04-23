use anyhow::Result;
use brane_bvm::{VM, InterpretResult};
use std::borrow::Cow::{self, Borrowed, Owned};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::{CompletionType, Config, Context, EditMode, Editor};
use rustyline_derive::Helper;
use std::path::PathBuf;
use std::fs;
use crate::registry;
use brane_dsl::{Compiler, CompilerOptions};

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
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
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
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    ///
    ///
    ///
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    ///
    ///
    ///
    fn highlight_char(&self, line: &str, pos: usize) -> bool {
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

pub async fn start(clear: bool) -> Result<()> {
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

    let compiler_options = CompilerOptions::new();
    let package_index = registry::get_package_index().await?;

    let mut compiler = Compiler::new(compiler_options, package_index.clone());
    let mut vm = VM::new(package_index);

    let mut count = 1;
    loop {
        let p = format!("{}> ", count);

        rl.helper_mut().expect("No helper").colored_prompt = format!("\x1b[1;32m{}\x1b[0m", p);

        let readline = rl.readline(&p);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match compiler.compile(line) {
                    Ok(function) => {
                        if let InterpretResult::InterpretOk(value) = vm.run(Some(function)) {
                            println!("{:?}", value);
                        }
                    },
                    Err(e) => { eprintln!("{:?}", e); }
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

    rl.save_history(&history_file).unwrap();

    Ok(())
}

///
///
///
fn get_history_file() -> PathBuf {
    appdirs::user_data_dir(Some("brane"), None, false)
        .expect("Couldn't determine Brane data directory.")
        .join("repl_history.txt")
}
