mod tui;
mod event;
mod handler;
mod app;
mod app_layout;
mod machinery;
mod tui_layout;
mod ui;


use std::error::Error;
use std::io;
use std::thread::sleep;
use std::time::Duration;
use app::App;
use app_layout::CarriageParameters;
use event::EventHandler;
use handler::handle_key_events;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tui::Tui;
use event::Event;
use tui_layout::TuiLayout;




fn main() -> Result<(), Box<dyn Error>> {
    // let mut terminal = setup_terminal()?;
    // let r = terminal.size()?;
    // println!("Terminal top: {}, left {}, bottom {}, right {}", r.top(),r.left(),r.bottom(),r.right());
    // let layout = Layout::default()
    //     .direction(Direction::Horizontal)
    //     .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref());
    // let chunks = layout.split(terminal.size()?);
    // let mut app_state = AppLayout::new(chunks);

   

    

     // Initialize the terminal user interface.
     let backend = CrosstermBackend::new(io::stderr());
     let terminal = Terminal::new(backend)?;
     let events = EventHandler::new(250);
     let tui_layout = TuiLayout::new(&terminal)?;
     let carriage_parameters = CarriageParameters::new(
            tui_layout.get_window_corners(1));

     let mut tui = Tui::new(terminal, events, tui_layout);
     
     // Create an application.
     let mut app = App::new(carriage_parameters);


     tui.init()?;

    // Start the main loop.
    loop {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => {
                app.inner_display_setup.on_tick((0,1))
            },
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }


    sleep(Duration::from_secs(10));

    Ok(())
}



/* 
fn ui_carriage<B: Backend, F: Fn(&mut Context)>(f: &mut Frame<B>, app_state: &mut AppLayout<F>, chunk_index: i16) {
    let chunks = app_state.display_windows.clone();

    let canvas = app_state.carriage_canvas
        .paint(|ctx| {
            ctx.draw(&app_state.carriage);
        })

        ;
    f.render_widget(canvas, chunks[chunk_index as usize]);
}

fn ui_elevator_control<B: Backend>(f: &mut Frame<B>, app: &AppLayout) {
    let chunks = app.display_windows.clone();

    let elevator_transitions_window =
        Paragraph::new(app.tell_me_more());

    f.render_widget(elevator_transitions_window, chunks[0]);

    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("Carriage"))
        .marker(app.marker)
        .paint(|ctx| {
            ctx.draw(&app.carriage);
        })
        //.x_bounds([150.0, 190.0])
        //.y_bounds([0.0, 43.0])
        .x_bounds([1.0, 50.0])
        .y_bounds([1.0, 50.0])

        ;
    f.render_widget(canvas, chunks[1]);
}
 */




/* fn create_chunks<B: Backend>(f: &mut Frame<B>) -> Rc<[Rect]> {

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    chunks

} */
