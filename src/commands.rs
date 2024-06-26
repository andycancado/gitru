use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum CliCommands {
    /// list configured users
    #[command()]
    List,
    /// show current user selected
    #[command()]
    Status,
    /// Remove user
    #[command(arg_required_else_help = true)]
    Remove {
        #[arg(required = true)]
        name: String,
    },
    /// add new user
    #[command(arg_required_else_help = true)]
    Add {
        #[arg(required = true)]
        name: String,
        #[arg(required = true)]
        email: String,
    },
    /// select user
    #[command(arg_required_else_help = true)]
    Select {
        #[arg(required = true)]
        name: String,
    },
}
