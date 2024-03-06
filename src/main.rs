mod tui;
mod async_event;
mod app;
mod elevator_infra;
mod tui_layout;
mod ui;
mod conversation;


use std::error::Error;
use std::io;
use std::path::PathBuf;


use app::App;
use elevator_infra::ElevatorInfra;
use log::info;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
use tui_layout::TuiLayout;
use ui::DisplayManager;

use crate::async_event::AppOwnEvent;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {


    initialize_logging()?;

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;

    let tui_layout = TuiLayout::new(&terminal)?;

    let screen_0 = tui_layout.get_window_corners(0);

    println!("Screen section 0 | top-left-x {} / top-left-y {}, bottom-right-x {} / bottom_right-y {}",
            screen_0.left(),screen_0.top(),screen_0.right(),screen_0.bottom()
        );

    let screen_1 = tui_layout.get_window_corners(1);

    println!("Screen section 1 | top-left-x {} / top-left-y {}, width {} / height {}",
                screen_1.left(),screen_1.top(),screen_1.width,screen_1.height
        );


     let carriage_parameters = ElevatorInfra::new(
            tui_layout.get_window_corners(1));

    println!("each floor height {}", carriage_parameters.each_floor_height);

    let display_manager = DisplayManager::new(
            (0.0,0.0), /* origin: */
            carriage_parameters.carriage_playground.width,  /* display_area_width: */   
            carriage_parameters.carriage_playground.height, /* display_areaa_height: */ 
            carriage_parameters.each_floor_height as f64    /* each_floor_height: */    
        );


    let mut user_input = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    stdin.read_line(&mut user_input);      


    // let mut tui = Tui::new(terminal, tui_layout, display_manager);
     
     // Create an application.
     let mut app = App::new(carriage_parameters, 1.0, 30.0,terminal,tui_layout,display_manager);

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
  