mod tui;
mod async_event;
mod handler;
mod app;
mod elevator_infra;
mod machinery;
mod tui_layout;
mod ui;


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
use tui::Tui;
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


     let mut tui = Tui::new(terminal, tui_layout, display_manager);
     
     // Create an application.
     let mut app = App::new(carriage_parameters, 1.0, 30.0);

    tui.init()?;

    tui.start()?;

    let floor_locations = app.inner_display_setup.get_carriage_displacement_map_per_floor((0,0));

    let current_left_top_y_init = floor_locations[0].0; // Ground floor
    let mut current_left_top_y = current_left_top_y_init;

    let mut up_events_log: Vec<String> = Vec::new();

    // Start the main loop.
    loop {
    
        // Handle events.
        match tui.next().await {
            Some(AppOwnEvent::Tick) => {

                if let Some(floor) = app.inner_display_setup.is_any_passenger_waiting() {
                    // let log_message = format!("Tick received, current {}, target {}",current_left_top_y,floor_locations[floor as usize].0).to_owned();
                    info!("First log message here");
                    up_events_log.push(format!("Tick received, current {}, target {}",current_left_top_y,floor_locations[floor as usize].0));
                    if current_left_top_y < floor_locations[floor as usize].0 {
                        current_left_top_y += 1.0;
                        app.inner_display_setup.on_tick((0,1));
                        tui.ui.move_carriage_up((0.0,1.0));
                        up_events_log.push(format!("On tick, current_top_left_y {}, dest {}", current_left_top_y, floor_locations[floor as usize].0));
                    }
                }
                else {
                    up_events_log.push(format!("Tick received, no passenger waiting"));
                }
               
            },
            Some(AppOwnEvent::Render) => {
                tui.draw(&mut app)?;
            },
            Some(AppOwnEvent::Key(key_event)) => {
                app.update_app(AppOwnEvent::Key(key_event))
                
            },
            e@ Some(AppOwnEvent::Mouse(m)) => {
                app.update_app(e.unwrap())
            },
            None => {},
            _ => {}
            
        }

        if app.should_quit_app() {
            tui.exit()?;
            break;
        }
            
    }

    app.save_to_file();

    println!("current left top at init {}", current_left_top_y_init);

    up_events_log.iter().for_each(|s| println!("{}",s));

    floor_locations.iter().enumerate().for_each(|l| {
        println!("floor_index {}, floor_left_bottm_y {:?} floor_left_top_y {:?}", l.0 as u16, l.1.1, l.1.0);
    });

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
  