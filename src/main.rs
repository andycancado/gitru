use anyhow::{anyhow, Context};
use clap::Parser;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::Display;
use std::path::PathBuf;
use std::process::Command;

mod commands;
use commands::CliCommands;

#[derive(Debug, Parser)]
#[command(name = "gitru")]
#[command(about = "A git user selection CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: CliCommands,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct GitUser {
    name: String,
    email: String,
}

impl Display for GitUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <{}>", self.name, self.email)
    }
}

fn create_connection() -> anyhow::Result<Connection> {
    let home = env::var("HOME").context("HOME environment variable not set")?;
    let db_path: PathBuf = [&home, ".config", "gitru.db"].iter().collect();
    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            name TEXT PRIMARY KEY,
            email TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS selected_user (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            name TEXT NOT NULL
        )",
        [],
    )?;

    Ok(conn)
}

fn list_users(conn: &Connection) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("SELECT name, email FROM users")?;
    let users = stmt.query_map([], |row| {
        Ok(GitUser {
            name: row.get(0)?,
            email: row.get(1)?,
        })
    })?;
    println!("Users:");
    for user in users {
        let user = user?;
        println!("- {} <{}>", user.name, user.email);
    }
    Ok(())
}

fn add_user(conn: &Connection, name: &str, email: &str) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO users (name, email) VALUES (?1, ?2)",
        [name, email],
    )?;
    println!("User added: {} <{}>", name, email);
    Ok(())
}

fn select_user(conn: &Connection, name: &str) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("SELECT email FROM users WHERE name = ?1")?;
    let mut rows = stmt.query([name])?;

    if let Some(row) = rows.next()? {
        let email: String = row.get(0)?;
        update_git_config(name, &email);

        conn.execute(
            "INSERT OR REPLACE INTO selected_user (id, name) VALUES (1, ?1)",
            [name],
        )?;

        println!("User '{}' selected and git config updated.", name);
    } else {
        println!("User '{}' not found.", name);
    }

    Ok(())
}

fn status(conn: &Connection) -> anyhow::Result<GitUser> {
    let mut stmt = conn.prepare("SELECT name FROM selected_user WHERE id = 1")?;
    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
        let name: String = row.get(0)?;
        let mut stmt = conn.prepare("SELECT email, name FROM users WHERE name = ?1")?;
        let mut user_rows = stmt.query([&name])?;

        if let Some(user_row) = user_rows.next()? {
            let user = GitUser {
                name: user_row.get::<usize, String>(0)?,
                email: user_row.get::<usize, String>(1)?,
            };
            Ok(user)
        } else {
            Err(anyhow!(
                "Error: Selected user '{}' not found in users table.",
                name
            ))
        }
    } else {
        Err(anyhow!("No user currently selected."))
    }
}

fn update_git_config(name: &str, email: &str) {
    Command::new("git")
        .args(["config", "--global", "user.name", name])
        .output()
        .expect("Failed to execute git command");

    println!("{}", email);
    Command::new("git")
        .args(["config", "--global", "user.email", email])
        .output()
        .expect("Failed to execute git command");

    println!("Git config updated for user: {} <{}>", name, email);
}

fn remove_user(_name: &str) -> anyhow::Result<bool> {
    todo!()
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let conn = create_connection()?;
    match args.command {
        CliCommands::Add { name, email } => {
            add_user(&conn, &name, &email)?;
        }
        CliCommands::Select { name } => {
            select_user(&conn, &name)?;
        }
        CliCommands::List => {
            list_users(&conn)?;
        }
        CliCommands::Status => {
            match status(&conn) {
                Ok(user) => println!("user Selected: {user}"),
                Err(e) => println!("{e}"),
            };
        }
        CliCommands::Remove { name } => {
            println!("remove {name} removed {}", remove_user(&name)?);
        }
    }
    Ok(())
}
