use ratatui::{widgets::{Paragraph, Block, Borders, BorderType, Wrap, canvas::Canvas}, style::{Style, Color, Stylize}, layout::Alignment, text::{Text, Line, Span}, backend::Backend, Frame};

use crate::{app::App, app_layout::CarriageParameters, tui_layout::TuiLayout};

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame<'_>) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    frame.render_widget(
        Paragraph::new(format!(
            "This is a tui template.\n\
                Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
                Press left and right to increment and decrement the counter respectively.\n\
                Counter: {}",
            1 // app.counter
        ))
        .block(
            Block::default()
                .title("Template")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Cyan).bg(Color::Black))
        .alignment(Alignment::Center),
        frame.size(),
    )
}

fn create_state_description_paragraph() -> Paragraph<'static> {
    let text = Text::from(vec![
        Line::from(vec![
            Span::raw("First"),
            Span::styled("line",Style::new().green().italic()),
            ".".into(),
        ]),
        Line::from("Second line".red()),
        "Third line".into(),
    ]);
    let paragraph = Paragraph::new(text)
        .block(Block::new()
            .title("Elevator-Panel")
            .borders(Borders::ALL))
        .style(Style::new().white().on_black())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    paragraph
}

pub fn render_modified(app: &mut App,layout: &TuiLayout, f: &mut Frame) {
    let chunks = layout.display_windows.clone();

    let elevator_transitions_window = create_state_description_paragraph();

    f.render_widget(elevator_transitions_window, chunks[0]);

    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("Carriage"))
        .marker(app.inner_display_setup.marker)
        .paint(|ctx| {
            ctx.draw(&app.inner_display_setup.carriage_shape);
        })
        //.x_bounds([150.0, 190.0])
        //.y_bounds([0.0, 43.0])
        .x_bounds([1.0, chunks[1].width as f64  - 1.0])
        .y_bounds([1.0, chunks[1].height as f64 - 1.0])

        ;
    f.render_widget(canvas, chunks[1]);
}

pub fn render_working(app: &mut App,layout: &TuiLayout, f: &mut Frame) {
    let chunks = layout.display_windows.clone();

    let elevator_transitions_window = create_state_description_paragraph();

    f.render_widget(elevator_transitions_window, chunks[0]);

    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("Carriage"))
        .marker(app.inner_display_setup.marker)
        .paint(|ctx| {
            ctx.draw(&app.inner_display_setup.carriage_shape);
        })
        //.x_bounds([150.0, 190.0])
        //.y_bounds([0.0, 43.0])
        .x_bounds([1.0, chunks[1].width as f64  - 1.0])
        .y_bounds([1.0, chunks[1].height as f64 - 1.0])

        ;
    f.render_widget(canvas, chunks[1]);
}


