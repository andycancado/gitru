use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

#[derive(Clone, Serialize, Deserialize, Debug)]
struct GitUser {
    name: String,
    email: String,
}

const STORAGE_FILE: &str = "gitru_users.json";

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "gitru")]
#[command(about = "A git user selection CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// initialize gitru
    #[command(arg_required_else_help = true)]
    Init,
    /// list configured users
    #[command(arg_required_else_help = true)]
    List,
    /// show current user selected
    #[command(arg_required_else_help = true)]
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

fn read_users() -> anyhow::Result<Vec<GitUser>> {
    // if !Path::new(STORAGE_FILE).exists() {
    //     return Vec::new();
    // }
    // TODO: concatenate storage file path witn $HOME/.config/gitru if exit or create folder
    let mut file = File::open(STORAGE_FILE).context("Unable to open file")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    let res = serde_json::from_str(&contents).unwrap_or_else(|_| Vec::new());
    Ok(res)
}

fn write_users(users: &[GitUser]) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(users).expect("Unable to serialize users");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(STORAGE_FILE)
        .context("Unable to open file for writing")?;
    file.write_all(json.as_bytes())
        .context("Unable to write to file")?;
    Ok(())
}

fn list_users() -> anyhow::Result<()> {
    let users = read_users()?;
    if users.is_empty() {
        println!("No users found.");
    } else {
        println!("Users:");
        for user in users {
            println!("- {} <{}>", user.name, user.email);
        }
    }
    Ok(())
}

fn add_user(name: &str, email: &str) -> anyhow::Result<GitUser> {
    let mut users = read_users()?;
    if users.iter().any(|u| u.name == name) {
        return Err(anyhow!("User '{}' already exists.", name));
    }
    let new_user = GitUser {
        name: name.to_string(),
        email: email.to_string(),
    };
    users.push(new_user.clone()); // TODO
    write_users(&users);
    println!("User added: {} <{}>", name, email);
    Ok(new_user)
}

fn select_user(name: &str) -> anyhow::Result<()> {
    let users = read_users()?;
    match users.iter().find(|u| u.name == name) {
        Some(user) => {
            update_git_config(&user.name, &user.email);
        }
        None => println!("User '{}' not found.", name),
    }
    Ok(())
}

fn update_git_config(name: &str, email: &str) {
    Command::new("git")
        .args(&["config", "--global", "user.name", name])
        .output()
        .expect("Failed to execute git command");

    Command::new("git")
        .args(&["config", "--global", "user.email", email])
        .output()
        .expect("Failed to execute git command");

    println!("Git config updated for user: {} <{}>", name, email);
}

fn init(name: &str) -> anyhow::Result<()> {
    todo!()
}

fn remove_user(name: &str) -> anyhow::Result<()> {
    todo!()
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Init => {
            println!("Init configuration");
        }
        Commands::Add { name, email } => {
            println!("add {name} {email}")
        }
        Commands::Select { name } => {
            println!("select {name}")
        }
        Commands::List => {
            println!("list users")
        }
        Commands::Status => {
            println!("Status");
        }
        Commands::Remove { name } => {
            println!("remove {name}");
        }
    }
}
