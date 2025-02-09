use log::error;

mod args;
use args::*;

use lupo::errors::*;
use lupo::*;

// Rust doesn't trap a unix signal appropriately occasionally: https://github.com/rust-lang/rust/issues/46016
fn reset_signal_pipe_handler() -> Result<()> {
    #[cfg(target_family = "unix")]
    {
        use nix::sys::signal;

        unsafe {
            signal::signal(signal::Signal::SIGPIPE, signal::SigHandler::SigDfl)
                .chain_err(|| "Internal error: cannot trap signal")?;
        }
    }

    Ok(())
}
fn main() {
    reset_signal_pipe_handler().unwrap();

    if let Err(ref e) = run() {
        let mut s = e.to_string();

        for e in e.iter().skip(1) {
            s.push_str(&format!("\n\tcaused by: {}", e));
        }

        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            s.push_str(&format!("\n\tbacktrace:\n{:?}", backtrace));
        }

        error!("{}", s);

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let opts = parse_args();

    stderrlog::new()
        .module(module_path!())
        .show_level(false)
        .quiet(opts.quiet)
        .verbosity(opts.verbose + 1) // The user needs warnings
        .timestamp(opts.ts.unwrap_or(stderrlog::Timestamp::Off))
        .init()
        .unwrap();

    let home_dir = &opts.directory.unwrap();

    match opts.subcmd {
        SubCommand::Init { force } => {
            let store = Store::new(home_dir, force)?;
            println!("Data directory: {}", store.home_dir.to_string_lossy());
            Ok(())
        }
        SubCommand::Check {} => {
            let store = Store::open(home_dir)?;
            let (ct, cs) = store.check()?;
            println!("{} trades processed correctly.", ct);
            println!("{} stocks processed correctly.", cs);
            Ok(())
        }
        SubCommand::Trades { name_substring } => {
            let store = Store::open(home_dir)?;
            let trades = store.trades(name_substring)?;
            trades.iter().for_each(|t| println!("{}", t));
            Ok(())
        }
        SubCommand::Port {} => {
            let store = Store::open(home_dir)?;
            let port_lines = store.port()?;
            port_lines.iter().for_each(|l| println!("{:}", l));
            Ok(())
        }
    }
}
