use clap::{Parser, Subcommand};

mod commands;
mod config;
mod git;
mod index;
mod note;
mod parser;
mod scorer;

#[derive(Parser, Debug)]
#[command(name = "cw", version, about = "Code reading assistance tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize .codewatch/ directory
    Init,
    /// Scan all files and update score index
    Scan,
    /// Show score details and notes for a file
    Show {
        /// File path (relative to project root or current directory)
        file: String,
    },
    /// Open notes for a file in your editor
    Note {
        /// File path (relative to project root or current directory)
        file: String,
    },
    /// List all indexed files by metric
    List {
        /// Sort order (score, recent)
        #[arg(long, value_enum, default_value_t = commands::list::SortOrder::Score)]
        sort: commands::list::SortOrder,

        /// Only show files that have a note
        #[arg(long)]
        noted: bool,

        /// Limit the number of files shown
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Show top files by score
    Top {
        /// Number of files to show
        #[arg(long, default_value_t = 10)]
        n: usize,
    },
    /// Generate code reading report
    Report {
        /// Number of files to show
        #[arg(long, default_value_t = 10)]
        n: usize,

        /// Filter by directory prefix
        #[arg(long)]
        prefix: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let res = match cli.command {
        Commands::Init => commands::init::run(),
        Commands::Scan => commands::scan::run(),
        Commands::Show { file } => commands::show::run(&file),
        Commands::Note { file } => commands::note::run(&file),
        Commands::List { sort, noted, limit } => commands::list::run(&sort, noted, limit),
        Commands::Top { n } => commands::top::run(n),
        Commands::Report { n, prefix } => commands::report::run(n, prefix),
    };

    if let Err(e) = res {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
