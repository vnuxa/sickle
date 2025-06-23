use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::Text;
use iced::advanced::Widget;
// use iced::Renderer;
use iced::advanced::Renderer;
use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::border::Radius;
use iced::keyboard;
use iced::keyboard::key::Named;
use iced::keyboard::Key;
use iced::Background;
use iced::Border;
use iced::Color;
use iced::Element;
use iced::Event;
use iced::Font;
use iced::Length;
use iced::Point;
use iced::Rectangle;
use iced::Shadow;
use iced::Size;
use iced::Theme;
use iced::advanced::renderer;
use iced::Vector;
use iced::advanced::graphics::core;
use iced::advanced::text::Renderer as _;


use crate::Config;
use crate::Messages;


pub struct Timeline<Message> {
    pub duration: f32,
    pub cursor_position: f32,
    pub start: f32,
    pub end: f32,
    pub pressed_start:  bool,
    pub pressed_end:  bool,
    pub pressed_anywhere:  bool,
    pub config: Config,

    pub update_start: Box<dyn Fn(f32) -> Message>,
    pub update_end: Box<dyn Fn(f32) -> Message>,

    pub toggle_start: Box<dyn Fn(bool) -> Message>,
    pub toggle_end: Box<dyn Fn(bool) -> Message>,

    pub set_time: Box<dyn Fn(f32) -> Message>,
    pub update_anywhere: Box<dyn Fn(bool) -> Message>,

    pub play_pause: Box<dyn Fn() -> Message>,
    pub restart: Box<dyn Fn() -> Message>,
    pub mouse: f32,
    pub mouse_content: String,
    pub mouse_move: Box<dyn Fn(f32) -> Message>,
}

pub fn hex_to_rgb(hex: &str) -> Color {

    let r = u8::from_str_radix(&hex[1..3], 16).unwrap();
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap();
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap();


    Color::from_rgb8(r, g, b)
}
pub fn hex_to_rgba(hex: &str, alpha: f32) -> Color {

    let r = u8::from_str_radix(&hex[1..3], 16).unwrap();
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap();
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap();


    Color::from_rgba8(r, g, b, alpha)
}

impl<Message, Theme, Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer<Font = iced::Font>> Widget<Message, Theme, Renderer> for Timeline<Message>{
    fn draw(
            &self,
            tree: &iced::advanced::widget::Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &iced::advanced::renderer::Style,
            layout: iced::advanced::Layout<'_>,
            cursor: iced::advanced::mouse::Cursor,
            viewport: &iced::Rectangle,
        ) {

        // println!("position is {:?}", layout.position());
        // println!("viewport size is {:?}", viewport);
        let mut view_position = layout.position();
        let view_size = layout.bounds();
        let border = renderer.fill_quad(renderer::Quad {
            bounds: Rectangle::new(view_position, Size { width: view_size.width, height: 60.0 }),
            border: Border {
                color: hex_to_rgb(&self.config.main_color),
                width: 2.5,
                radius: Radius::new(0.0),
            },
            shadow: Shadow {
                color: Color::from_rgb(0.0, 0.0, 0.0),
                offset: Vector::ZERO,
                blur_radius: 0.0,
            }
        }, Background::Color(hex_to_rgb(&self.config.background_color)));

        let start_portion = view_size.width / (self.duration / self.start);
        let end_portion = view_size.width / (self.duration / self.end);
        // println!("end portion: {:?} and the other thing {:?}, whole portion: {:?}", end_portion, view_size.x / (self.duration / self.end), view_size);
        let mut handle_start_position = view_position;
        handle_start_position.x = view_position.x + start_portion;
        let mut handle_end_position = view_position;
        handle_end_position.x = view_position.x + end_portion - 7.0;

        let timeline = renderer.fill_quad(renderer::Quad {
            bounds: Rectangle::new(handle_start_position, Size { width: end_portion - start_portion, height: 60.0 }),
            border: Border::default(),
            shadow: Shadow::default()
        },
            // Background::Color(Color::from_rgba8(255, 255, 255, 0.25))
            Background::Color(
                hex_to_rgba(&self.config.timeline_color, 0.15)
                // Color::from_rgba8(130, 159, 98, 0.15)

            )


        );

        let handle_start = renderer.fill_quad(renderer::Quad {
            bounds: Rectangle::new(handle_start_position, Size { width: 7.0, height: 60.0 }),
            border: Border::default()
                .color(Color::from_rgb(0.0, 0.0, 0.0))
                .width(1.0),
            shadow: Shadow::default()

        },
            hex_to_rgb(&self.config.timeline_color)
            // Background::Color(Color::from_rgba8(130, 159, 98, 1.0))

        );

        let handle_end = renderer.fill_quad(renderer::Quad {
            bounds: Rectangle::new(handle_end_position, Size { width: 7.0, height: 60.0 }),
            border: Border::default()
                .color(Color::from_rgb(0.0, 0.0, 0.0))
                .width(1.0),
            shadow: Shadow::default()

        },
            hex_to_rgb(&self.config.timeline_color),
            // Background::Color(Color::from_rgba8(130, 159, 98, 1.0))

        );

        let mut cursor_position = view_position;
        cursor_position.x = view_position.x +  view_size.width / (self.duration / self.cursor_position);
        let cursor_thing = renderer.fill_quad(renderer::Quad {
            bounds: Rectangle::new(cursor_position, Size { width: 3.0, height: 60.0 }),
            border: Border::default(),
                // .color(Color::from_rgb(0.0, 0.0, 0.0))
                // .width(1.0),
            shadow: Shadow::default()

        },
            hex_to_rgba(&self.config.main_color, 0.75)
            // Background::Color(Color::from_rgba8(178, 135, 161, 0.75))

        );

        if cursor.is_over(view_size) {
            let mut mouse_position = view_position;
            mouse_position.x = view_position.x +  view_size.width * (self.mouse) - 60.0;
            mouse_position.y -= 40.0;

            let bounds = Rectangle::new(mouse_position, Size { width: 120.0, height: 35.0 });
            renderer.with_layer(bounds, |renderer| {

                let cursor_thing = renderer.fill_quad(renderer::Quad {
                    bounds: Rectangle::new(mouse_position, Size { width: 120.0, height: 35.0 }),
                    border: Border::default().rounded(5.0),
                    // .color(Color::from_rgb(0.0, 0.0, 0.0))
                    // .width(1.0),
                    shadow: Shadow::default()

                },
                    Background::Color(
                        hex_to_rgba(&self.config.hover_background, 0.95)
                    )
                );

                let text = renderer.fill_text(
                    Text {
                        wrapping: core::text::Wrapping::None,
                        shaping: core::text::Shaping::Basic,
                        horizontal_alignment: Horizontal::Center,
                        vertical_alignment: Vertical::Center,
                        font: Font::default(),
                        size: iced::Pixels(15.0),
                        line_height: core::text::LineHeight::Absolute(iced::Pixels(10.0)),
                        bounds: Size { width: 120.0, height: 20.0 },
                        content: self.mouse_content.clone(),
                    },
                    Point { x: mouse_position.x + 60.0, y: mouse_position.y + 15.0 },
                    // Color::from_rgba8(178, 135, 161, 1.0),
                    hex_to_rgb(&self.config.main_color),
                    bounds,

                );
            }
            );

        }



    }
    fn size(&self) -> iced::Size<Length> {
        iced::Size { width: Length::Fill, height: Length::Fixed(60.0) }
    }
    fn layout(
            &self,
            tree: &mut iced::advanced::widget::Tree,
            renderer: &Renderer,
            limits: &iced::advanced::layout::Limits,
        ) -> iced::advanced::layout::Node {
        // limits.width(width).hei
        let limits = limits.width(Length::Fill).height(Length::Fixed(60.0)).min_height(60.0);
        let size = Size::new(limits.max().width, limits.min().height);
        layout::Node::new(limits.resolve(Length::Fill, Length::Fill, size))


    }

    fn mouse_interaction(
            &self,
            state: &iced::advanced::widget::Tree,
            layout: layout::Layout<'_>,
            cursor: iced::advanced::mouse::Cursor,
            viewport: &Rectangle,
            _renderer: &Renderer,
        ) -> iced::advanced::mouse::Interaction {
        let mut view_position = layout.position();
        let view_size = layout.bounds();

        if self.pressed_start || self.pressed_end {
            return mouse::Interaction::Grabbing
        }
        if self.pressed_anywhere {

            return mouse::Interaction::ResizingHorizontally
        }

        let start_portion = view_size.width / (self.duration / self.start);
        let end_portion = view_size.width / (self.duration / self.end);

        let mut handle_start_position = view_position.x;
        handle_start_position += start_portion;

        let mut handle_end_position = view_position.x;
        handle_end_position += end_portion - 7.0;

        if cursor.is_over(Rectangle {
            x: handle_start_position - 11.0,
            y: view_position.y,
            width: 18.0,
            height: 60.0
        }) {
            return mouse::Interaction::Grab;
        } else if cursor.is_over(Rectangle {
            x: handle_end_position - 11.0,
            y: view_position.y,
            width: 18.0,
            height: 60.0
        }) {
            return mouse::Interaction::Grab;
            // println!("clicked end");

        }
        mouse::Interaction::None
        // if  {
        //
        // }
    }

    fn on_event(
            &mut self,
            _state: &mut iced::advanced::widget::Tree,
            event: Event,
            layout: layout::Layout<'_>,
            cursor: mouse::Cursor,
            _renderer: &Renderer,
            _clipboard: &mut dyn iced::advanced::Clipboard,
            shell: &mut iced::advanced::Shell<'_, Message>,
            _viewport: &Rectangle,
        ) -> core::event::Status {

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                text,
                ..
            }) => {
                match key {
                    Key::Named(named) => match named {
                        Named::Space => {
                            shell.publish((self.play_pause)());
                            return core::event::Status::Captured;
                        }
                        Named::ArrowLeft => {
                            if self.pressed_end {
                                let mut new_position = self.end - (self.duration * 0.001);
                                if new_position < 0.0 {
                                    new_position = 0.0;
                                }
                                shell.publish((self.update_end)(new_position));
                                return core::event::Status::Captured;
                            }
                            if self.pressed_start {
                                let mut new_position = self.start - (self.duration * 0.001);
                                if new_position < 0.0 {
                                    new_position = 0.0;
                                }
                                // if new_position > self.duration {
                                //     new_position = self.duration;
                                // }
                                shell.publish((self.update_start)(new_position));
                                return core::event::Status::Captured;
                            }
                            let mut new_position = self.cursor_position - (self.duration * 0.001);
                            if new_position < 0.0 {
                                new_position = 0.0;
                            }
                            shell.publish((self.set_time)(new_position));
                            return core::event::Status::Captured;
                        }
                        Named::ArrowRight => {
                            if self.pressed_end {
                                let mut new_position = self.end + (self.duration * 0.001);
                                if new_position > self.duration {
                                    new_position = self.duration;
                                }
                                shell.publish((self.update_end)(new_position));
                                return core::event::Status::Captured;
                            }
                            if self.pressed_start {
                                let mut new_position = self.start + (self.duration * 0.001);
                                if new_position > self.duration {
                                    new_position = self.duration;
                                }
                                shell.publish((self.update_start)(new_position));
                                return core::event::Status::Captured;
                            }
                            let mut new_position = self.cursor_position + (self.duration * 0.001);
                            if new_position > self.duration {
                                new_position = self.duration;
                            }
                            shell.publish((self.set_time)(new_position));
                            return core::event::Status::Captured;
                        }
                        _ => core::event::Status::Ignored,
                    }
                    Key::Character(char) => {

                        println!("char is {:?}", char);
                        if char == "r" && modifiers.control() {
                            println!("restart te thing!");
                            shell.publish((self.restart)());
                            return core::event::Status::Captured;

                        }
                        core::event::Status::Ignored
                    }
                    _ => core::event::Status::Ignored
                }
            }

            Event::Mouse(mouse::Event::CursorMoved { .. }) => {

                if self.pressed_start {
                    let view_position = layout.position();
                    let view_size = layout.bounds();

                    let Some(position) = cursor.position() else { return core::event::Status::Ignored };
                    // println!("cursor position: {:?} view size {:?}", position, view_size);
                    let mut x_position = position.x - view_position.x;
                    // if position.x > (view_size.width + view_position.x - 7.0) {
                    //     x_position = view_size.width - 7.0;
                    // }
                    if position.x > (view_size.x + (view_size.width / (self.duration / self.end)) - 20.0) {
                        x_position = (view_size.width / (self.duration / self.end)) - 20.0;
                    } else if position.x < view_position.x {
                        x_position = 0.0;
                    }
                    let mut new_position = x_position / (view_size.width ) ;
                    // if new_position < 0.0 {
                    //     new_position = 0.0
                    // } else if new_position > 1.0 {
                    //     new_position = 1.0
                    // }
                    // println!("new position: {:?}", new_position);
                    // println!("new start: {:?}", self.duration * new_position);
                    // self.start = self.duration * new_position;
                    // let message = (self.update_start)(self.duration * new_position);
                    // shell.publish(message);
                    shell.publish((self.update_start)(self.duration * new_position));
                    return core::event::Status::Captured;
                }
                if self.pressed_end {
                    let view_position = layout.position();
                    let view_size = layout.bounds();

                    let Some(position) = cursor.position() else { return core::event::Status::Ignored };
                    let mut x_position = position.x - view_position.x;
                    if position.x > (view_size.width + view_position.x) {
                        x_position = view_size.width ;
                    } else if position.x < (view_size.x + (view_size.width / (self.duration / self.start)) + 18.0) {
                        x_position =  (view_size.width / (self.duration / self.start)) + 18.0;
                    }
                    let mut new_position = x_position / (view_size.width ) ;
                    // if new_position < 0.0 {
                    //     new_position = 0.0
                    // } else if new_position > 1.0 {
                    //     new_position = 1.0
                    // }
                    // println!("new position: {:?}", new_position);
                    // println!("new start: {:?}", self.duration * new_position);
                    // self.end= self.duration * new_position;
                    shell.publish((self.update_end)(self.duration * new_position));
                    return core::event::Status::Captured;
                }
                if self.pressed_anywhere {

                    // println!("pressed anywehre is {:?}", self.pressed_anywhere);
                    let view_position = layout.position();
                    let view_size = layout.bounds();

                    let Some(position) = cursor.position() else { return core::event::Status::Ignored };
                    let mut x_position = position.x - view_position.x;
                    if position.x > (view_size.width + view_position.x) {
                        x_position = view_size.width ;
                    } else if position.x < view_position.x {
                        x_position = 0.0;
                    }

                    let mut new_position = x_position / (view_size.width ) ;
                    shell.publish((self.set_time)(self.duration * new_position));
                    return core::event::Status::Captured;
                }


                let view_size = layout.bounds();
                if cursor.is_over(view_size) {
                    let view_position = layout.position();
                    let Some(position) = cursor.position() else { return core::event::Status::Ignored };

                    let mut x_position = position.x - view_position.x;
                    if position.x > (view_size.width + view_position.x) {
                        x_position = view_size.width ;
                    } else if position.x < view_position.x {
                        x_position = 0.0;
                    }

                    let mut new_position = x_position / (view_size.width ) ;
                    shell.publish((self.mouse_move)(new_position));
                    return core::event::Status::Captured;
                }

                core::event::Status::Ignored
            }

            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {

                let bounds = layout.bounds();
                if cursor.is_over(bounds) {
                    // println!("got cursor drag thingy");
                    let mut view_position = layout.position();
                    let view_size = layout.bounds();

                    let start_portion = view_size.width / (self.duration / self.start);
                    let end_portion = view_size.width / (self.duration / self.end);

                    let mut handle_start_position = view_position.x;
                    handle_start_position += start_portion;

                    let mut handle_end_position = view_position.x;
                    handle_end_position += end_portion - 7.0;

                    if cursor.is_over(Rectangle {
                        x: handle_start_position - 11.0,
                        y: view_position.y,
                        width: 18.0,
                        height: 60.0
                    }) {
                        // println!("itss over the other thingy!");
                        shell.publish((self.toggle_start)(true));
                        return core::event::Status::Captured;
                        // let Some(position) = cursor.position() else { return core::event::Status::Ignored };
                        // let mut new_position = view_size.x / position.x;
                        // if new_position < 0.0 {
                        //     new_position = 0.0
                        // } else if new_position > 1.0 {
                        //     new_position = 1.0
                        // }
                        // println!("new position: {:?}", new_position);
                        // println!("new start: {:?}", self.duration * new_position);
                        // self.start = self.duration * new_position
                    } else if cursor.is_over(Rectangle {
                        x: handle_end_position - 11.0,
                        y: view_position.y,
                        width: 18.0,
                        height: 60.0
                    }) {
                        // println!("clicked end");
                        shell.publish((self.toggle_end)(true));
                        return core::event::Status::Captured;

                    } else {
                        let Some(position) = cursor.position() else { return core::event::Status::Ignored };
                        let mut x_position = position.x - view_position.x;
                        if position.x > (view_size.width + view_position.x) {
                            x_position = view_size.width ;
                        } else if position.x < view_position.x {
                            x_position = 0.0;
                        }

                        let mut new_position = x_position / (view_size.width ) ;
                        shell.publish((self.set_time)(self.duration * new_position));
                        shell.publish((self.update_anywhere)(true));
                        return core::event::Status::Captured;
                    }
                }
                core::event::Status::Ignored
            },
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if self.pressed_start || self.pressed_end || self.pressed_anywhere {
                    if self.pressed_anywhere {

                        shell.publish((self.update_anywhere)(false));
                    }
                    if self.pressed_start {
                        self.pressed_start = false;
                        shell.publish((self.toggle_start)(false));
                    }
                    if self.pressed_end {
                        self.pressed_end = false;
                        shell.publish((self.toggle_end)(false));
                    }

                    return core::event::Status::Captured;
                }
                core::event::Status::Ignored
            }
            _ => core::event::Status::Ignored
        }
    }
}

impl<'a, Message> From<Timeline<Message>> for Element<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    fn from(terminal_box: Timeline<Message>) -> Self {
        Self::new(terminal_box)
    }
}
