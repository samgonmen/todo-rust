use clap::{Parser, Subcommand};
use colored::Colorize;
use sqlx::{
    Pool, Sqlite,
    prelude::FromRow,
    sqlite::{SqliteConnectOptions, SqlitePool, SqliteQueryResult},
};

#[derive(Debug, FromRow)]
pub struct TaskResponse {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub done: bool,
}

#[derive(Parser, Debug)]
#[command(version, about = "Simple task manager for terminal cli", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a new task to the list, shows tasks after adding
    Add {
        /// Title for the task
        title: String,
        /// Optional description for the task
        description: Option<String>,
    },
    /// Show all tasks in the list
    List,
    /// Remove a task from the list by its index, shows tasks after removing
    Remove {
        /// Index from the list, use "list" command to see the index of each task
        index: i64,
    },
    /// Mark a task as done or undone by its index, shows tasks after marking
    Toggle {
        /// Index from the list, use "list" command to see the index of each task
        index: i64,
    },
}

#[derive(Debug)]
struct NewTask {
    pub title: String,
    pub description: Option<String>,
}

#[tokio::main]
async fn main() {
    let pool = match connect().await {
        Ok(pool) => {
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("Migrations unsucessfull");
            pool
        }
        Err(e) => panic!("Database connection error: {:?}", e),
    };

    let cli = Cli::parse();
    match cli.command {
        Commands::Add { title, description } => {
            add_task(&pool, NewTask { title, description })
                .await
                .expect("Error adding task: ");
            println!("Task added!");
            list_tasks(&pool).await.expect("Error retrieving tasks: ");
        }
        Commands::List => {
            list_tasks(&pool).await.expect("Error retrieving tasks: ");
        }
        Commands::Remove { index } => {
            remove_task(&pool, index)
                .await
                .expect("Error removing task: ");
            println!("Task removed!");
            list_tasks(&pool).await.expect("Error retrieving tasks: ");
        }
        Commands::Toggle { index } => {
            done_or_undone(&pool, index)
                .await
                .expect("Error marking task as done/undone: ");
            println!("Task marked as done/undone!");
            list_tasks(&pool).await.expect("Error retrieving tasks: ");
        }
    }
}

async fn connect() -> Result<Pool<Sqlite>, sqlx::Error> {
    let options = SqliteConnectOptions::new()
        .filename("tasks.db")
        .create_if_missing(true);
    SqlitePool::connect_with(options).await
}

async fn get_tasks(pool: &Pool<Sqlite>) -> Result<Vec<TaskResponse>, sqlx::Error> {
    sqlx::query_as::<_, TaskResponse>("SELECT * FROM tasks")
        .fetch_all(pool)
        .await
}

async fn list_tasks(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let tasks = get_tasks(pool).await?;
    let mut current_index: i64 = 1;

    if tasks.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }

    for task in tasks {
        print!("{}", format!("{}.", current_index).bright_black());
        println!(
            " {} {} ",
            if task.done {
                "●".green()
            } else {
                "○".red()
            },
            if task.done {
                task.title.strikethrough().bright_black()
            } else {
                task.title.white()
            },
        );
        if let Some(text) = task.description
            && !task.done
        {
            println!("\t{}", text);
        }
        current_index += 1;
    }
    Ok(())
}
async fn add_task(pool: &Pool<Sqlite>, task: NewTask) -> Result<SqliteQueryResult, sqlx::Error> {
    sqlx::query("INSERT INTO tasks (title, description) VALUES (?, ?)")
        .bind(&task.title)
        .bind(&task.description)
        .execute(pool)
        .await
}

async fn remove_task(pool: &Pool<Sqlite>, index: i64) -> Result<SqliteQueryResult, sqlx::Error> {
    let tasks = get_tasks(pool).await?;

    if index <= 0 || index > tasks.len() as i64 {
        return Err(sqlx::Error::RowNotFound);
    }

    sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(&tasks[(index - 1) as usize].id) // Exemplo: remover a tarefa com id 1
        .execute(pool)
        .await
}

async fn done_or_undone(pool: &Pool<Sqlite>, index: i64) -> Result<SqliteQueryResult, sqlx::Error> {
    let tasks = get_tasks(pool).await?;
    if index <= 0 || index > tasks.len() as i64 {
        return Err(sqlx::Error::RowNotFound);
    }

    sqlx::query("UPDATE tasks SET done = NOT done WHERE id = ?")
        .bind(&tasks[(index - 1) as usize].id) // Exemplo: marcar a tarefa com id 1 como feita ou desfeita
        .execute(pool)
        .await
}
