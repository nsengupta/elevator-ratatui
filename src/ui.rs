
use ratatui::{widgets::{Paragraph, Block, Borders, BorderType, Wrap, canvas::{Canvas, Rectangle}}, style::{Style, Color, Stylize}, layout::Alignment, text::{Text, Span, Line as TextLine}, Frame, symbols::Marker};

use crate::{app::App, elevator_infra::{ElevatorInfra, MX_FLOORS}, tui_layout::TuiLayout};

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

fn create_state_description_paragraph(heading: &str,left: f64, top: f64, right: f64, bottom: f64) -> Paragraph<'static> {
    let text = Text::from(vec![
        TextLine::from(vec![
            Span::raw("First"),
            Span::styled("line",Style::new().green().italic()),
            ".".into(),
        ]),
        TextLine::from("Second line".red()),
        "Third line".into(),
        heading.to_owned().into(),
        format!("left {}, top {}, width {}, height {}", left,top,right,bottom).into()
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

fn display_inner_structure_as_paragraph(heading: &str,inner: &ElevatorInfra) -> Paragraph<'static> {

    let main_desc = vec![
        TextLine::from(vec![
            Span::raw("First"),
            Span::styled("line",Style::new().green().italic()),
            ".".into(),
        ]),
        TextLine::from("Second line".red()),
        "Third line".into(),
        heading.to_owned().into(),
        format!("carriage_playground.x {}, carriage_playground.y {}, carriage_playground.width {}, carriage_playground.height {}",inner.carriage_playground.x,inner.carriage_playground.y,inner.carriage_playground.width,inner.carriage_playground.height).into(),
        format!("rect {}, left {}, top {}, right {}, bottom {}",0,inner.floor_as_rects[0].left(), inner.floor_as_rects[0].top(), inner.floor_as_rects[0].right(), inner.floor_as_rects[0].bottom()).into(),
        format!("rect {}, left {}, top {}, right {}, bottom {}",1,inner.floor_as_rects[1].left(), inner.floor_as_rects[1].top(), inner.floor_as_rects[1].right(), inner.floor_as_rects[1].bottom()).into(),
        format!("rect {}, left {}, top {}, right {}, bottom {}",2,inner.floor_as_rects[2].left(), inner.floor_as_rects[2].top(), inner.floor_as_rects[2].right(), inner.floor_as_rects[2].bottom()).into(),

    ];

    /* let inner_rect_desc: Vec<String> = 
        inner.floor_as_rects
        .iter()
        .enumerate()
        .map(|(index,next_rec)| {
            format!("rect {}, x {}, y {}, width {}, height {}",index,next_rec.x, next_rec.y, next_rec.width, next_rec.height).into()
        })
        .collect(); */

    let text = Text::from(main_desc);
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
    let chunks = layout.output_windows.clone();

    let elevator_transitions_window = create_state_description_paragraph(
        "Nothing",
        chunks[1].left() as f64,
        chunks[1].top()  as f64,
        chunks[1].right()  as f64,
        chunks[1].bottom()  as f64
    );


    f.render_widget(elevator_transitions_window, chunks[0]);

    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("Carriage"))
        .marker(app.inner_display_setup.marker)
        .paint(|ctx| {
            // ctx.draw(&app.inner_display_setup.carriage_shape);

            ctx.draw(&app.inner_display_setup.building_wall);
        })
        //.x_bounds([150.0, 190.0])
        //.y_bounds([0.0, 43.0])
        .x_bounds([1.0, chunks[1].width as f64  - 1.0])
        .y_bounds([1.0, chunks[1].height as f64 - 1.0])

        ;
    f.render_widget(canvas, chunks[1]);

}

/// Renders the user interface widgets.
pub fn render_working(app: &mut App,layout: &TuiLayout, f: &mut Frame) {
    let output_chunks = layout.output_windows.clone();

   /* let elevator_transitions_window = create_state_description_paragraph(
        "Line coordinates",
        app.inner_display_setup.floor_as_rects[0].left() as f64,
        app.inner_display_setup.floor_as_rects[0].top() as f64,
        (app.inner_display_setup.floor_as_rects[0].left() + app.inner_display_setup.floor_as_rects[0].width) as f64 ,
        (app.inner_display_setup.floor_as_rects[0].top() + app.inner_display_setup.floor_as_rects[0].height) as f64
    ); */

    let elevator_transitions_window = display_inner_structure_as_paragraph("Floors",&app.inner_display_setup);

    f.render_widget(elevator_transitions_window, output_chunks[0]);

    let mut text_lines: Vec<TextLine> = Vec::new();


    /* for floor in &app.inner_display_setup.level_markers {

        let next_line: TextLine = 
            format!("left {}, top {}, width {}, height {}",
                floor.x1,
                floor.y1,
                floor.x2,
                floor.y1
            )
            .into();

        text_lines.push(next_line);
    }


    let text = Text::from(text_lines);

    let paragraph = Paragraph::new(text)
    .block(Block::new()
        .title("Elevator-Panel")
        .borders(Borders::ALL))
    .style(Style::new().white().on_black())
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });


    f.render_widget(paragraph, output_chunks[0]); */

    let floors_as_rectangles: Vec<Rectangle> = 
            (0..MX_FLOORS).into_iter()
            .map(|i| {
                app.inner_display_setup.tranlate_coords_to_viewport(i as usize, (0,0))
            })
            .collect();



    let canvas = Canvas::default()
        .block(
            Block::default()
            .bg(Color::White)
            .borders(Borders::ALL)
            .title("Floors + Carriage")
            .style(Style::default().bg(Color::LightBlue).fg(Color::Gray))
        )
        .marker(Marker::HalfBlock)
        .paint(|ctx| {
            ctx.draw(&Rectangle {

                x: 1.0   as f64,
                y: 1.0    as f64,
                width: app.inner_display_setup.carriage_playground.width  as f64 - 1.0,
                height: app.inner_display_setup.carriage_playground.height as f64 - 1.0,
                color: Color::LightBlue
            });

            ctx.draw(&app.inner_display_setup.building_wall);
            /* for floor_level_marker in &app.inner_display_setup.level_markers {
                ctx.draw(floor_level_marker);
            } */

            for each_floor_as_rectangle in &floors_as_rectangles{
                ctx.draw(each_floor_as_rectangle);
            }

            ctx.draw(&app.inner_display_setup.carriage_shape);


        })
        .x_bounds([0.0, /* 46.0 */ output_chunks[1].width as f64 ])
        .y_bounds([0.0, /* 42.0 */ output_chunks[1].height as f64 ])

        ;
    f.render_widget(canvas, output_chunks[1]);

    /* let input_windows = layout.input_window.clone();

    let input_canvas = Canvas::default()
        .block(
            Block::default()
            .bg(Color::White)
            .borders(Borders::ALL)
            .title("Passenger Input")
            .style(Style::default().bg(Color::Red).fg(Color::LightYellow))
        )
        .marker(Marker::Bar);

    f.render_widget(input_canvas, input_windows[1]); */
}


