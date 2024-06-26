mod tui;
mod app;
mod elevator_infra;
mod tui_layout;
mod ui;
mod conversation;
mod elevator_installation;
mod app_own_event;


use std::error::Error;
use std::io;
use std::path::PathBuf;


use app::App;
use elevator_infra::ElevatorVisualInfra;
use log::info;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
use tui_layout::TuiLayout;
use ui::DisplayManager;


#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn Error>> {


    initialize_logging()?;

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;

    let tui_layout = TuiLayout::new(&terminal)?;

    tui_layout.log_window_corners(); // For easier debugging.

    let floor_and_carriage_screen_segment = 
          ElevatorVisualInfra::new(
            tui_layout.motion_window[tui_layout.motion_window_index as usize]);

    let mut user_input = String::new();
    println!("\nAll set. Press any key to start.\n");
    let stdin = io::stdin(); // We get `Stdin` here.
    stdin.read_line(&mut user_input).unwrap();      

     // Create an application. This is what holds everything together and runs the elevator.
     let mut  app = App::new(
                floor_and_carriage_screen_segment, 
                1.0, 
                30.0,
                terminal,
                tui_layout,
                DisplayManager::new())
              .await;

    app.init()?;

    app.start()?;

    app.run().await.unwrap();


    Ok(())
}

pub fn initialize_logging() -> Result<(), Box<dyn Error>> {
    let directory = PathBuf::from("./");
    std::fs::create_dir_all(directory.clone())?;
    let log_path = directory.join("elevator.log");
    let log_file = std::fs::File::create(log_path)?;
   /*  std::env::set_var(
      "RUST_LOG",
      std::env::var("RUST_LOG")
        .or_else(|_| std::env::var(LOG_ENV.clone()))
        .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME"))),
    ); */
    let file_subscriber = tracing_subscriber::fmt::layer()
      .with_file(true)
      .with_line_number(true)
      .with_writer(log_file)
      .with_target(false)
      .with_ansi(false)
      .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());
    
    tracing_subscriber::registry()
    .with(file_subscriber)
    // .with(ErrorLayer::default())
    .init();
    
    Ok(())
  }
  