use lazy_static::lazy_static;
use nix::sys::signal::Signal::SIGTERM;
use rand::seq::SliceRandom;
use rand::Rng;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::thread;
use structopt::StructOpt;

#[derive(Debug, PartialEq, StructOpt)]
struct Opt {
    #[structopt(short, long, default_value = "1")]
    delay: u64,

    #[structopt(short, long)]
    log: Option<String>,

    #[structopt(short, long, help = "Keep going when command fails")]
    keep_going: bool,

    #[structopt(subcommand)]
    cmd: Subcommands,
}
#[derive(Debug, PartialEq, StructOpt)]
#[structopt(name = "command")]
enum Subcommands {
    #[structopt(external_subcommand)]
    Command(Vec<String>),
}
enum CommandResult {
    LaunchFailure(std::io::Error),
    Interrupted,
    Code(i32),
}

lazy_static! {
    static ref SPARKLES: usize = rand::thread_rng().gen();
}

macro_rules! sparkle_fmt {
    ($msg:expr) => {
        format!("{} {} {}", sparkles(), $msg, sparkles())
    };
    ($fmt:expr, $($arg:tt)*) => {
        format!("{} {} {}", sparkles(), format!($fmt, $($arg)*), sparkles())
    };
}

macro_rules! sparkle {
    ($out:expr, $msg:expr) => {
        if let Err(e) = writeln!($out, "{}", sparkle_fmt!($msg)) {
            esparkle!("failed to write: {}", e);
        }
    };
    ($out:expr, $fmt:expr, $($arg:tt)*) => {
        if let Err(e) = writeln!($out, "{}", sparkle_fmt!($fmt, $($arg)*)) {
            esparkle!("failed to write {}", e);
        }
    };
}

macro_rules! esparkle {
    ($msg:expr) => {
        eprintln!("{}", sparkle_fmt!($msg))
    };
    ($fmt:expr, $($arg:tt)*) => {
        eprintln!("{}", sparkle_fmt!($fmt, $($arg)*))
    };
}

fn sparkles() -> String {
    let sparkly = vec!['🟈', '✹', '*', '✧', '⭑', '✯', '✵', '✦', '☆', '✰', '꙳'];
    let amount = (*SPARKLES % (sparkly.len() - 1)) + 1;
    sparkly
        .choose_multiple(&mut rand::thread_rng(), amount)
        .collect::<String>()
}

//
// Read output from file as it updates, like the `tail -f` command in Linux,
// prints to stdout.
//
fn tail_f_output(output: std::fs::File) {
    let metadata = match output.metadata() {
        Ok(data) => data,
        Err(e) => {
            esparkle!("failure: {}", e);
            std::process::exit(1);
        }
    };
    let mut pos = metadata.len();
    let mut reader = std::io::BufReader::new(output);
    if let Err(e) = reader.seek(SeekFrom::Start(pos)) {
        esparkle!("failure: {}", e);
        std::process::exit(1);
    }

    loop {
        let mut line = String::new();
        if let Ok(len) = reader.read_line(&mut line) {
            if len > 0 {
                pos += len as u64;
                print!("{}", line);
            }
        }

        if let Err(e) = reader.seek(SeekFrom::Start(pos)) {
            esparkle!("failure: {}", e);
            std::process::exit(1);
        }
    }
}

//
// We want to output both in the terminal and to a file, similar to:
//   $ command |& tee -a file
//
// This function makes that happen by opening a file if a filename
// is supplied, and otherwise use a tempfile.
//
// Then we spawn a function that will read the updates from that
// file and prints to stdout.
//
fn setup_output(filename: Option<String>) -> std::fs::File {
    let logfile = match filename {
        Some(ref filename) => OpenOptions::new()
            .truncate(true)
            .read(true)
            .write(true)
            .create(true)
            .open(filename),
        None => tempfile::tempfile(),
    };

    let output = match logfile {
        Ok(file) => file,
        Err(e) => {
            esparkle!("failed to open file for logging: {}", e);
            std::process::exit(1);
        }
    };

    let output_clone = match output.try_clone() {
        Ok(output) => output,
        Err(e) => {
            esparkle!("failure: {}", e);
            std::process::exit(1);
        }
    };

    thread::spawn(move || {
        tail_f_output(output_clone);
    });

    output
}

//
// Run a command and wait for it to complete, returns status in form of a
// CommandResult struct. The stdout and stderr from the child process
// will be directed to the output specified in arguments.
//
fn run_command(cmd: &[String], output: &std::fs::File) -> CommandResult {
    let stdout = match output.try_clone() {
        Ok(file) => file,
        Err(e) => return CommandResult::LaunchFailure(e),
    };

    let stderr = match output.try_clone() {
        Ok(file) => file,
        Err(e) => return CommandResult::LaunchFailure(e),
    };

    match std::process::Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdout(stdout)
        .stderr(stderr)
        .status()
    {
        Ok(status) => match status.code() {
            Some(n) => CommandResult::Code(n),
            None => CommandResult::Interrupted,
        },
        Err(e) => CommandResult::LaunchFailure(e),
    }
}

fn main() {
    let opt = Opt::from_args();
    let Subcommands::Command(cmd) = opt.cmd;
    let mut iteration = 0;
    let mut failures = 0;

    let mut output = setup_output(opt.log);

    ctrlc::set_handler(move || {
        if nix::sys::signal::killpg(nix::unistd::getpgrp(), SIGTERM).is_err() {
            esparkle!("failed to terminate process");
        }

        if nix::sys::wait::wait().is_err() {
            std::process::exit(1);
        };

        std::process::exit(0);
    })
    .expect("error setting ctrl-c handler");

    loop {
        let start = std::time::Instant::now();
        match run_command(&cmd, &output) {
            CommandResult::Code(n) => {
                if n != 0 {
                    sparkle!(output, "Command exited with code: {}", n);
                    failures += 1;
                    if !opt.keep_going {
                        break;
                    }
                }
            }
            CommandResult::LaunchFailure(e) => {
                esparkle!("Failed to launch command: {}", e);
                break;
            }
            CommandResult::Interrupted => {
                sparkle!(output, "Command was interrupted!");
                break;
            }
        };

        iteration += 1;

        std::thread::sleep(std::time::Duration::from_millis(500));

        if let Err(e) = writeln!(output,) {
            esparkle!("failure: {}", e);
            std::process::exit(1);
        }

        let iter_fmt = if opt.keep_going {
            format!("Iterations: {}, failures: {}", iteration, failures)
        } else {
            format!("Iterations: {}", iteration)
        };
        sparkle!(output, "{}, elapsed time: {:?}", iter_fmt, start.elapsed());

        std::thread::sleep(std::time::Duration::from_secs(opt.delay));
    }
}
