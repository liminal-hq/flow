// Entrypoint for the Liminal Flow CLI and TUI
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use clap::{Parser, Subcommand};

mod cli;

#[derive(Parser)]
#[command(
    name = "flo",
    version,
    about = "Liminal Flow — terminal working-memory sidecar",
    long_about = "Liminal Flow — terminal working-memory sidecar\n\n\
        Track what you're working on, branch into sub-tasks, and maintain\n\
        ambient context awareness — all from the terminal.\n\n\
        Run `flo` with no arguments to launch the interactive TUI.\n\n\
        Examples:\n  \
          flo now \"improving the auth flow\"    # Start a thread\n  \
          flo branch \"debugging token refresh\"  # Branch off\n  \
          flo note \"try the new endpoint\"       # Attach a note\n  \
          flo where                              # See current state\n  \
          flo back                               # Return to parent\n  \
          flo list                               # Show all threads\n  \
          flo done                               # Mark the active focus done",
    after_help = "Run `flo` with no arguments to launch the TUI."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Set or replace the current thread
    #[command(long_about = "Set or replace the current thread.\n\n\
            If another thread is active, it will be paused automatically.\n\n\
            Examples:\n  \
              flo now \"improving AIDX\"\n  \
              flo now \"working on the component library\"")]
    Now {
        /// The thread description
        text: String,
    },

    /// Create a branch beneath the current thread
    #[command(long_about = "Create a branch beneath the current thread.\n\n\
            Branches track tangential sub-tasks. Use `/back` to return.\n\n\
            Examples:\n  \
              flo branch \"debugging auth\"\n  \
              flo branch \"researching API options\"")]
    Branch {
        /// The branch description
        text: String,
    },

    /// Return to the parent thread
    #[command(
        long_about = "Return to the parent thread, parking all active branches.\n\n\
            Parked branches remain visible in the thread list."
    )]
    Back,

    /// Attach a note to the current focus target
    #[command(long_about = "Attach a note to the current focus target.\n\n\
            Notes are attached to the active branch if one exists,\n\
            otherwise to the current thread.\n\n\
            Examples:\n  \
              flo note \"try the new endpoint\"\n  \
              flo note \"waiting on code review\"")]
    Note {
        /// The note text
        text: String,
    },

    /// Print current thread and branches
    #[command(
        long_about = "Print the current thread, its branches, and their statuses.\n\n\
            Shows the active thread with all branches and their current state."
    )]
    Where,

    /// Pause the current thread
    #[command(long_about = "Pause the current thread.\n\n\
            The thread remains in the list and can be resumed later\n\
            with `flo now` or via the TUI.")]
    Pause,

    /// Mark the active focus target done
    #[command(long_about = "Mark the active focus target as done.\n\n\
            If a branch is active, that branch is marked done.\n\
            Otherwise the current thread is marked done.\n\n\
            Done items remain visible until they are archived.")]
    Done,

    /// List active, paused, and done threads
    #[command(
        long_about = "List all active, paused, and done threads with their branches.\n\n\
            Use --all to include branches and notes.\n\n\
            The active thread is marked with `>`."
    )]
    List {
        /// Show branches and recent notes for each thread
        #[arg(short, long)]
        all: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "liminal_flow=info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        None => {
            // No subcommand — launch the TUI
            let conn = liminal_flow_store::open_store()?;
            liminal_flow_tui::run_tui(conn).await?;
        }
        Some(cmd) => {
            let conn = liminal_flow_store::open_store()?;
            match cmd {
                Command::Now { text } => cli::now::handle(&conn, &text)?,
                Command::Branch { text } => cli::branch::handle(&conn, &text)?,
                Command::Back => cli::back::handle(&conn)?,
                Command::Note { text } => cli::note::handle(&conn, &text)?,
                Command::Where => cli::where_cmd::handle(&conn)?,
                Command::Pause => cli::pause::handle(&conn)?,
                Command::Done => cli::done::handle(&conn)?,
                Command::List { all } => cli::list::handle(&conn, all)?,
            }
        }
    }

    Ok(())
}
