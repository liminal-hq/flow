// Build script — generates man pages from the clap CLI definition
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use clap::{CommandFactory, Parser, Subcommand};
use clap_mangen::Man;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "flo",
    version = env!("CARGO_PKG_VERSION"),
    about = "Liminal Flow — terminal working-memory sidecar",
    long_about = "Liminal Flow — terminal working-memory sidecar\n\n\
        Track what you're working on, branch into sub-tasks, and maintain\n\
        ambient context awareness — all from the terminal.\n\n\
        Run `flo` with no arguments to launch the interactive TUI."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Set or replace the current thread
    Now {
        /// The thread description
        text: String,
    },
    /// Create a branch beneath the current thread
    Branch {
        /// The branch description
        text: String,
    },
    /// Return to the parent thread
    Back,
    /// Attach a note to the current focus target
    Note {
        /// The note text
        text: String,
    },
    /// Print current thread and branches
    Where,
    /// Pause the current thread
    Pause,
    /// Mark the active focus target done
    Done,
    /// Archive the active focus target
    Archive,
    /// List active, paused, and done threads
    List {
        /// Show branches and recent notes for each thread
        #[arg(short, long)]
        all: bool,
    },
}

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_else(|| "target/man".into()));
    let man_dir = out_dir.join("man");
    fs::create_dir_all(&man_dir).expect("Failed to create man page directory");

    let cmd = Cli::command();
    let man = Man::new(cmd.clone());
    let mut buf = Vec::new();
    man.render(&mut buf).expect("Failed to render man page");
    fs::write(man_dir.join("flo.1"), buf).expect("Failed to write flo.1");

    // Generate man pages for subcommands
    for sub in cmd.get_subcommands() {
        let name = format!("flo-{}", sub.get_name());
        let man = Man::new(sub.clone());
        let mut buf = Vec::new();
        man.render(&mut buf)
            .expect("Failed to render subcommand man page");
        fs::write(man_dir.join(format!("{name}.1")), buf)
            .expect("Failed to write subcommand man page");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
