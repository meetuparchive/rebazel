extern crate notify;
#[macro_use]
extern crate error_chain;
extern crate term;

use notify::{RecommendedWatcher, Watcher, RecursiveMode};
use notify::DebouncedEvent::{Write, Remove, Rename};
use std::env;
use std::io::Write as IOWrite;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::mpsc::channel;
use std::time::Duration;

// generates Result,Error,ErrorKind types as compile time
error_chain! {
  errors {
    MissingAction
  }
  foreign_links {
    IO(::std::io::Error);
    Notify(::notify::Error);
    Term(::term::Error);
  }
}

// https://github.com/bazelbuild/bazel/blob/master/tools/defaults/BUILD
#[inline]
fn tools_default(path: &str) -> bool {
    path == "//tools/defaults:BUILD"
}

#[inline]
fn external_workspace(path: &str) -> bool {
    path.starts_with("@")
}

#[inline]
fn aliased(path: &str) -> bool {
    path.starts_with("//external")
}

#[inline]
fn clean_path(path: &str) -> String {
    path.trim_left_matches("//:")
        .trim_left_matches("//")
        .replace(":", "/")
}

#[inline]
fn watchable(path: &str) -> bool {
    !(external_workspace(path) || aliased(path) || tools_default(path))
}

#[inline]
fn buildfile(path: PathBuf) -> bool {
    path.ends_with("BUILD") || path.extension().iter().find(|ext| **ext == "bzl").is_some()
}

fn query(executable: &String, q: String) -> Result<Vec<String>> {
    let query = Command::new(executable)
        .arg("query")
        .arg(q)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&query.stdout);
    let stderr = String::from_utf8_lossy(&query.stderr);
    println!("{}", stderr);
    let lines = stdout
        .lines()
        .filter(|path| watchable(path))
        .map(clean_path)
        .collect();
    Ok(lines)
}

const SOURCES: &str = "kind('source file', deps(set({target})))";

fn sources(executable: &String, target: &String) -> Result<Vec<String>> {
    query(
        executable,
        format!("kind('source file', deps(set({target})))", target = target),
    )
}

fn builds(executable: &String, target: &String) -> Result<Vec<String>> {
    query(
        executable,
        format!("buildfiles(deps(set({target})))", target = target),
    )
}

fn exec(executable: &String, action: &String, args: Vec<String>) -> Result<Child> {
    Ok(Command::new(executable)
        .arg(action)
        .args(args)
        .spawn()?)
}

fn watch(
    executable: &String,
    targets: Vec<&String>,
    watcher: &mut RecommendedWatcher,
) -> Result<()> {
    let mut t = term::stderr().unwrap();
    for target in targets {
        for file in sources(&executable, &target)? {
            t.fg(term::color::GREEN)?;
            write!(t, "INFO: ")?;
            t.reset()?;
            writeln!(t, "watching source file: {file}", file = file)?;
            watcher.watch(file, RecursiveMode::NonRecursive)?;
        }
        for file in builds(&executable, &target)? {
            t.fg(term::color::GREEN)?;
            write!(t, "INFO: ")?;
            t.reset()?;
            writeln!(t, "watching build: {file}", file = file)?;
            watcher.watch(file, RecursiveMode::NonRecursive)?;
        }
        println!("watching {target} dependencies...", target = target)
    }
    Ok(())
}

fn run() -> Result<()> {
    let (tx, rx) = channel();
    let executable = env::var("BAZEL_EXEC").unwrap_or(String::from("bazel"));
    let delay = Duration::from_millis(
        env::var("DEBOUNCE_DELAY")
            .map(|delay| delay.parse().unwrap())
            .unwrap_or(100),
    );
    let action = env::args().nth(1).ok_or::<Error>(
        ErrorKind::MissingAction.into(),
    )?;
    let args = env::args().skip(2).collect::<Vec<_>>();
    let targets = args.iter()
                 // skip flags
                .skip_while(|arg| arg.starts_with("-"))
                .collect::<Vec<_>>();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, delay)?;
    watch(&executable, targets.clone(), &mut watcher)?;
    let mut child = exec(&executable, &action, args.clone())?;
    loop {
        match rx.recv() {
            Ok(ev) => {
                match ev {
                    Write(path) | Remove(path) | Rename(path, _) => {
                        let mut t = term::stdout().unwrap();
                        t.fg(term::color::GREEN)?;
                        write!(t, "INFO: ")?;
                        t.reset()?;
                        writeln!(t, "changed {path}", path = path.display())?;
                        let _ = child.kill();
                        if buildfile(path) {
                            // update watch sources if build defs change
                            watch(&executable, targets.clone(), &mut watcher)?
                        }
                        child = exec(&executable, &action, args.clone())?;
                        ()
                    }
                    _ => (),
                }
            }
            Err(e) => println!("error watching files: {}", e),
        }
    }
}

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        use error_chain::ChainedError; // trait which holds `display`
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "{}", e.display()).expect(errmsg);
        ::std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn is_tools_default() {
        assert!(tools_default("//tools/defaults:BUILD"))
    }

    #[test]
    fn is_external_workspace() {
        assert!(external_workspace("@foo//bar"))
    }

    #[test]
    fn is_aliasd() {
        assert!(aliased("//external/foo"))
    }
}
