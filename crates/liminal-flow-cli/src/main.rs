// Entrypoint for the Liminal Flow CLI and TUI
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: MIT

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "flo", version, about = "Liminal Flow — terminal working-memory sidecar")]
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

    /// Mark the current thread done
    Done,

    /// List active threads
    List,
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
        Some(_cmd) => {
            // CLI commands — will be wired in Phase 3
            println!("CLI commands coming soon.");
        }
    }

    Ok(())
}
