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

use app::App;
use elevator_infra::ElevatorInfra;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tui::Tui;
use tui_layout::TuiLayout;

use crate::async_event::AppOwnEvent;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let mut terminal = setup_terminal()?;
    // let r = terminal.size()?;
    // println!("Terminal top: {}, left {}, bottom {}, right {}", r.top(),r.left(),r.bottom(),r.right());
    // let layout = Layout::default()
    //     .direction(Direction::Horizontal)
    //     .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref());
    // let chunks = layout.split(terminal.size()?);
    // let mut app_state = AppLayout::new(chunks);
    
    let _io_writer = io::stderr();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;

    let tui_layout = TuiLayout::new(&terminal)?;

    let screen_0 = tui_layout.get_window_corners(0);

    println!("Screen section 0 | top-left-x {} / top-left-y {}, bottom-right-x {} / bottom_right-y {}",
            screen_0.left(),screen_0.top(),screen_0.right(),screen_0.bottom()
        );

    let screen_1 = tui_layout.get_window_corners(1);

    println!("Screen section 1 | top-left-x {} / top-left-y {}, bottom-right-x {} / bottom_right-y {}",
                screen_1.left(),screen_1.top(),screen_1.right(),screen_1.bottom()
        );


     let carriage_parameters = ElevatorInfra::new(
            tui_layout.get_window_corners(1));

    println!("each floor height {}", carriage_parameters.each_level_height);


    let mut user_input = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    stdin.read_line(&mut user_input);      


     let mut tui = Tui::new(terminal, tui_layout);
     
     // Create an application.
     let mut app = App::new(carriage_parameters, 1.0, 30.0);

    tui.init()?;

    tui.start();

    // Start the main loop.
    loop {
        // Render the user interface.
        // tui.draw(&mut app)?;

        // Handle events.
        match tui.next().await {
            Some(AppOwnEvent::Tick) => {
                app.inner_display_setup.on_tick((0,1));
            },
            Some(AppOwnEvent::Render) => {
                tui.draw(&mut app)?;
            },
            Some(AppOwnEvent::Key(key_event)) => {
                app.update_app(AppOwnEvent::Key(key_event))
                //println!("KeyEvent received.");
            },
            Some(AppOwnEvent::Mouse(_)) => {},
            None => {},
            _ => {}
            
        }

        if app.should_quit_app() {
            tui.exit()?;
            break;
        }
            
    }

    Ok(())
}