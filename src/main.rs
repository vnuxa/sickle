use std::io::Read;
use std::str::FromStr;
use std::{env::home_dir, path::PathBuf, string, time::Duration};

use std::os::unix::fs::MetadataExt;
use essi_ffmpeg::FFmpeg;
use iced::{widget::{button, Column, Container, Row, Svg}, window::Settings, Alignment, Application, Background, Border, Color, ContentFit, Font, Length, Padding, Shadow, Task};
use iced_video_player::{Position, Video, VideoPlayer};
use iced::widget;
use rfd::FileDialog;
use timeline::{hex_to_rgb, hex_to_rgba, Timeline};
use toml::Table;
use std::fs::{self, read_to_string, File};

use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_app::prelude::*;

mod timeline;

use clap::Parser;
/// Simple video trimmer that automatically compresses a video if its above 10mb size
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The file you want to edit
    file: Option<String>
}


#[derive(Clone)]
pub struct Config {
    main_color: String,
    background_color: String,
    timeline_color: String,
    hover_background: String,

}

impl Default for Config {
    fn default() -> Self {
        Self {
            main_color: "#B287A1".to_string(),
            background_color: "#111111".to_string(),
            timeline_color: "#829f62".to_string(),
            hover_background: "#0E0E0E".to_string(),
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let mut config = Config::default();
    let mut config_path = home_dir().unwrap();
    config_path.push(".config/");
    config_path.push("sickle/");
    if config_path.exists() {
        config_path.push("config.toml");
        let mut file = File::open(config_path).expect("Expected config.toml in ~/.config/sickle/");
        let mut string_buffer = String::new();
        file.read_to_string(&mut string_buffer).unwrap();
        let mut toml = Table::from_str(&string_buffer).unwrap();
        if let Some(color) = toml.get("main_color") {
            config.main_color = color.as_str().unwrap().to_string();
        }
        if let Some(color) = toml.get("background_color") {
            config.background_color = color.as_str().unwrap().to_string();
        }
        if let Some(color) = toml.get("timeline_color") {
            config.timeline_color = color.as_str().unwrap().to_string();
        }
        if let Some(color) = toml.get("hover_background") {
            config.hover_background = color.as_str().unwrap().to_string();
        }
    }

    let mut file = None;
    if let Some(cli_file) = cli.file {
        let mut path = PathBuf::from("/");

        for text in cli_file.split("/") {
            if text == "~" {
                path.push(home_dir().unwrap().to_str().unwrap());
            } else {
                path.push(text);
            }
        }

        file = Some(path);
    } else {
        file = Some(FileDialog::new()
                    .pick_file().unwrap());
    }
    let mut settings = Settings::default();

    settings.decorations = false;

    let mut app_settings = iced::Settings::default();

    app_settings.id = Some("sickle".to_string());

    // let appearance =
    iced::application("sickle", update, view)
        .window(settings)
        .settings(app_settings)
        .style(|state, theme| {
            iced::application::Appearance {
                background_color: hex_to_rgb(&state.config.background_color),
                text_color: Color::from_rgb(1.0, 1.0, 1.0)
            }
        })
        .default_font(Font::with_name(string_to_static_str("EPSON 正楷書体Ｍ".to_string())))
        .run_with(|| {
            // let mut state = App::default();
            // let old_file = home_dir()
            //         .unwrap()
            //         .join("Videos/clips/spin_big.mp4")
            //         .canonicalize()
            //         .unwrap();

            let old_file = file.unwrap();
            let uri = &url::Url::from_file_path(&old_file).unwrap();
            let video = {
                gst::init().unwrap();

                let pipeline = format!("playbin uri=\"{}\" text-sink=\"appsink name=iced_text sync=true drop=true\" video-sink=\"videoscale ! videoconvert ! appsink name=iced_video drop=true caps=video/x-raw,format=NV12,pixel-aspect-ratio=1/1,width=1280,height=720\"", uri.as_str());
                let pipeline = gst::parse::launch(pipeline.as_ref()).unwrap()
                    .downcast::<gst::Pipeline>()
                    .map_err(|_| iced_video_player::Error::Cast).unwrap();

                let video_sink: gst::Element = pipeline.property("video-sink");
                let pad = video_sink.pads().first().cloned().unwrap();
                let pad = pad.dynamic_cast::<gst::GhostPad>().unwrap();
                let bin = pad
                    .parent_element()
                    .unwrap()
                    .downcast::<gst::Bin>()
                    .unwrap();
                let video_sink = bin.by_name("iced_video").unwrap();
                let video_sink = video_sink.downcast::<gst_app::AppSink>().unwrap();

                let text_sink: gst::Element = pipeline.property("text-sink");
                let text_sink = text_sink.downcast::<gst_app::AppSink>().unwrap();

                Video::from_gst_pipeline(pipeline, video_sink, Some(text_sink)).unwrap()
            };
            // let pipeline = format!("uridecodebin uri=\"{}\" ! videoconvert ! videoscale ! appsink name=iced_video caps=video/x-raw,format=RGBA,pixel-aspect-ratio=1/1,width=720,height=1280", uri.as_str());
            // let video = Video::from_pipeline(pipeline, None);
            // let video = Video::new().unwrap();
            let state = App {

                video_length: video.duration().as_secs_f32(),
                cursor_position: 0.0,
                mouse_position: 0.0,
                mouse_content: String::new(),
                start: 0.0,
                end:  video.duration().as_secs_f32() / 2.0,
                pressed_start: false,
                pressed_end: false,
                pressed_anywhere: false,
                video_time: time::Duration::seconds_f32(video.duration().as_secs_f32()),
                video,
                old_file,
                config,
            };
            (state, Task::none())
        });
}

struct App {
    video: Video,
    old_file: PathBuf,
    mouse_position: f32,
    mouse_content: String,
    cursor_position: f32,
    video_length: f32,
    video_time: time::Duration,
    config: Config,

    start: f32,
    end: f32,

    pressed_start: bool,
    pressed_end: bool,

    pressed_anywhere: bool,

}

#[derive(Debug, Clone)]
enum Messages {
    NewFrame,
    PlayPause,
    PressedStart(bool),
    PressedEnd(bool),
    Pressed(bool),
    UpdateStart(f32),
    UpdateEnd(f32),
    SetTime(f32),
    MouseMove(f32),
    RestartStream,
    Export
}


fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}


impl Default for App {
    fn default() -> Self {
        let video = Video::new(
            &url::Url::from_file_path(
                home_dir()
                    .unwrap()
                    .join("Videos/clips/spin_big.mp4")
                    .canonicalize()
                    .unwrap(),
            ).unwrap()
        ).unwrap();

        Self {

            video_length: video.duration().as_secs_f32(),
            cursor_position: 0.0,
            mouse_position: 0.0,
            mouse_content: String::new(),
            start: 0.0,
            old_file: PathBuf::new(),
            end:  video.duration().as_secs_f32() / 2.0,
            pressed_start: false,
            pressed_end: false,
            pressed_anywhere: false,
            video_time: time::Duration::seconds_f32(video.duration().as_secs_f32()),
            config: Config::default(),
            video,

        }
    }
}


fn view(app: &App) -> iced::Element<Messages> {
    let time = time::Duration::seconds_f32(app.cursor_position);
    Column::new()
        .push(
            Container::new(
                VideoPlayer::new(&app.video)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(ContentFit::Contain)
                    .on_new_frame(Messages::NewFrame),

            )
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .height(Length::Fill)

        )
        .push(
            Row::new()
                .push(
                    button::Button::new(
                        Svg::from_path( if app.video.paused() {
                            "data/play-symbolic.svg"
                        } else {
                            "data/pause-symbolic.svg"
                        })
                            .width(Length::Fixed(50.0))
                            .height(Length::Fixed(50.0))
                            // .content_fit(ContentFit::Cover)
                            .style(|state, theme| {
                                widget::svg::Style {
                                    color: Some(hex_to_rgba(&app.config.main_color, 0.25)),
                                }
                            })
                    )
                        .style(|state, theme| {
                            widget::button::Style {
                                background: Some(Background::Color(hex_to_rgba(&app.config.main_color, 0.15))),
                                text_color: hex_to_rgba(&app.config.main_color, 0.75),
                                border: Border::default().rounded(10.0),
                                shadow: Shadow::default(),
                            }
                        })
                        .height(Length::Fixed(40.0))
                        .width(Length::Fixed(40.0))
                        .on_press(Messages::PlayPause),
                )
                    .push(
                        widget::text(format!(
                            "{:02}:{:02}.{:03.0} / {:02}:{:02}.{:03.0}",
                            time.whole_minutes(),
                            time.whole_seconds() - time.whole_minutes() * 60,
                            (time.as_seconds_f32() - time.whole_seconds() as f32) * 1000.0,
                            app.video_time.whole_minutes(),
                            app.video_time.whole_seconds() - app.video_time.whole_minutes() * 60,
                            // (app.video_time.as_seconds_f32() - app.video_time.whole_seconds() as f32),
                            (app.video_time.as_seconds_f32() - app.video_time.whole_seconds() as f32) * 1000.0,
                        ))
                        .color(hex_to_rgb(&app.config.main_color))


                    )

                    // button("pause").on_press(Messages::PlayPause))
                .push(
                    Timeline {
                        config: app.config.clone(),
                        duration: app.video_length,
                        mouse: app.mouse_position,
                        mouse_content: app.mouse_content.clone(),
                        mouse_move: Box::new(|position| Messages::MouseMove(position)),
                        start: app.start,
                        end: app.end,
                        update_start: Box::new(|position| Messages::UpdateStart(position)),
                        update_end: Box::new(|position| Messages::UpdateEnd(position)),
                        update_anywhere: Box::new(|position| Messages::Pressed(position)),
                        pressed_start: app.pressed_start,
                        pressed_end: app.pressed_end,
                        toggle_start: Box::new(|position| Messages::PressedStart(position)),
                        toggle_end: Box::new(|position| Messages::PressedEnd(position)),
                        set_time: Box::new(|position| Messages::SetTime(position)),
                        cursor_position: app.cursor_position,
                        pressed_anywhere: app.pressed_anywhere,
                        play_pause: Box::new(|| Messages::PlayPause),

                        restart: Box::new(|| Messages::RestartStream),
                    }
                )
                .push(
                    button::Button::new(
                        Svg::from_path("data/scissors-symbolic.svg")
                            .width(Length::Fixed(50.0))
                            .height(Length::Fixed(50.0))
                            // .width(Length::Fixed(20.0))
                            // .content_fit(ContentFit::Fill)
                            // .content_fit(ContentFit::Cover)
                            .style(|state, theme| {
                                widget::svg::Style {
                                    color: Some(hex_to_rgba(&app.config.main_color, 0.25)),
                                }
                            })
                    )
                        .style(|state, theme| {
                            widget::button::Style {
                                background: Some(Background::Color(hex_to_rgba(&app.config.main_color, 0.15))),
                                text_color: hex_to_rgba(&app.config.main_color, 0.15),
                                border: Border::default().rounded(10.0),
                                shadow: Shadow::default(),
                            }
                        })
                        .height(Length::Fixed(40.0))
                        .width(Length::Fixed(40.0))
                        .on_press(Messages::Export),
                )
                .width(Length::Fill)
                .spacing(6.0)
                .align_y(Alignment::Center)
                .padding(Padding::new(0.0).left(5.0).right(5.0))

        )
            .spacing(2.5)
        .into()

        // widget::slider(0..self.video_length, , on_change)
    // );


}

fn update(app: &mut App, message: Messages)  {
    match message {
        Messages::NewFrame => {
            let position = app.video.position();
            app.cursor_position = position.as_secs_f32();
        },
        Messages::PlayPause => {
            app.video.set_paused(!app.video.paused());

        }
        Messages::UpdateStart(position) => {
            app.mouse_position = position * app.video_length;
            let time = time::Duration::seconds_f32(app.video_length * position);
            app.mouse_content = format!(
                "{:02}:{:02}.{:03.0}",
                time.whole_minutes(),
                time.whole_seconds() - time.whole_minutes() * 60,
                (time.as_seconds_f32() - time.whole_seconds() as f32) * 1000.0,
            );
            app.start = position;
            app.video.seek(Position::Time(Duration::from_secs_f32((position * 1000.0).round() / 1000.0)), false).unwrap();
            app.cursor_position = position;

        }
        Messages::UpdateEnd(position) => {
            app.mouse_position = position * app.video_length;
            let time = time::Duration::seconds_f32(app.video_length * position);
            app.mouse_content = format!(
                "{:02}:{:02}.{:03.0}",
                time.whole_minutes(),
                time.whole_seconds() - time.whole_minutes() * 60,
                (time.as_seconds_f32() - time.whole_seconds() as f32) * 1000.0,
            );

            app.end = position;
            app.video.seek(Position::Time(Duration::from_secs_f32((position * 1000.0).round() / 1000.0)), false).unwrap();
            // app.video.seek(Position::Time(Duration::from_secs_f32(position)), true);
            app.cursor_position = position;

        }
        Messages::PressedStart(value) => {
            app.pressed_start = value;
        }
        Messages::PressedEnd(value) => {
            app.pressed_end = value;
        }
        Messages::Pressed(value) => {
            app.pressed_anywhere = value;
        }
        Messages::SetTime(value) => {
            let time = time::Duration::seconds_f32(app.video_length * value);
            app.mouse_content = format!(
                "{:02}:{:02}.{:03.0}",
                time.whole_minutes(),
                time.whole_seconds() - time.whole_minutes() * 60,
                (time.as_seconds_f32() - time.whole_seconds() as f32) * 1000.0,
            );
            app.mouse_position = value * app.video_length;

            app.video.seek(Position::Time(Duration::from_secs_f32((value * 1000.0).round() / 1000.0)), false).unwrap();
            app.cursor_position = value;

        }
        Messages::MouseMove(value) => {
            app.mouse_position = value;
            let time = time::Duration::seconds_f32(app.video_length * value);
            app.mouse_content = format!(
                "{:02}:{:02}.{:03.0}",
                time.whole_minutes(),
                time.whole_seconds() - time.whole_minutes() * 60,
                (time.as_seconds_f32() - time.whole_seconds() as f32) * 1000.0,
            );
        }
        Messages::RestartStream => {
        }
        Messages::Export => {
            app.video.set_paused(true);
            // let _ = FFmpeg::auto_download();
            // if let Some((handle, mut progress)) = FFmpeg::auto_download() {
            //     handle.unwrap().unwrap();
            // } else {
            //     println!("FFmpeg is downloaded, using existing installation");
            // }
            let file = FileDialog::new()
                .set_file_name(app.old_file.file_name().unwrap().to_str().unwrap())
                .set_directory(app.old_file.parent().unwrap())
                .save_file();

            // println!("the thign: {:?}", unsafe { make_static_str(&app.start.to_string()) });
            if let Some(file) = file {
                println!("got file path {:?}", file);
                println!("start is {:?}",
                        string_to_static_str(app.start.to_string())
);
                let mut ffmpeg = FFmpeg::new()
                    .stderr(std::process::Stdio::inherit())
                    .input_with_file(app.old_file.clone()).done()
                    .args([
                        "-ss",
                        string_to_static_str(app.start.to_string())
                    ])
                    .args([
                        "-t",
                        string_to_static_str((app.end - app.start).to_string())
                    ]);

                let size = app.old_file.metadata().unwrap().size();
                // if old file is already bigger than 8 mb, try using some compression techniques
                if size > 10000000 {
                    let video_bitrate = ((1450.0) / ((app.end - app.start) / 60.0)) * 0.93;
                    let audio_bitrate = video_bitrate * 0.1;
                    // let audio_bitrate = (( 318000.0 / ( 1.0 + std::f32::consts::E.powf(-0.0000014 * video_bitrate * 60.0) ) ) - 154000.0) / 2.0;
                    println!("VIDEO BITRATE SHOULD BE {:?}", video_bitrate);
                    println!("AUDIO BTIRATE SHOULD BE {:?} which would make video bitrate: {:?}", audio_bitrate, video_bitrate - audio_bitrate);
                    // app.old_file.metadata().unwrap().audi

                    let mut ffmpeg_2 = FFmpeg::new()
                        .stderr(std::process::Stdio::inherit())
                        .input_with_file(app.old_file.clone()).done()
                        .args([
                            "-ss",
                            string_to_static_str(app.start.to_string())
                        ])
                        .args([
                            "-t",
                            string_to_static_str((app.end - app.start).to_string())
                        ])
                        .arg("-an")
                        // .output_as_file(file.clone())
                        .args([
                            "-c:v",
                            "libx264"
                        ])
                        .args([
                            "-r",
                            "30"
                        ])
                        .args([
                            "-b:v",
                            string_to_static_str(format!("{:.0}k", video_bitrate - audio_bitrate))

                        ])
                        .args([
                            "-preset",
                            "veryslow"
                        ])
                        .args([
                            "-r",
                            "30"
                        ])
                        .args([
                            "-pass",
                            "1"
                        ])
                        // .args([
                        //     "-passlogfile",
                        //     "/tmp/mydummy"
                        // ])
                        .args([
                            "-f",
                            "rawvideo"
                        ])
                        .inspect_args(|args| {
                            dbg!(args);
                        });
                    ;
                    // ff
                    // println!("builder is {:?} ", ffmpeg_2.inspect_args(f));

                        ffmpeg_2.start().unwrap().wait().unwrap();
                    println!("now first done!");
                    ffmpeg = ffmpeg
                        .args([
                            "-c:a",
                            "aac"
                        ])
                        .args([
                            "-aac_coder",
                            "twoloop"
                        ])
                        .args([
                            "-b:a",
                            string_to_static_str(format!("{:.0}k", audio_bitrate))
                        ])
                        .args([
                            "-c:v",
                            "libx264"
                        ])
                        .args([
                            "-b:v",
                            string_to_static_str(format!("{:.0}k", video_bitrate - audio_bitrate))
                        ])
                        .args([
                            "-preset",
                            "veryslow"
                        ])
                        .args([
                            "-pass",
                            "1"
                        ]);
                        // .args([
                        //     "-passlogfile",
                        //     "/tmp/mydummy"
                        // ]);

                }

                ffmpeg = ffmpeg.output_as_file(file.clone()).done();
                println!("file size is {:?}", app.old_file.metadata().unwrap().size());
            // println!("file is of size {:?}", ::new(source).into_iter().map(|item| item.metadata().unwrap().len()));
                // if  {
                //
                // }


                ffmpeg.start().unwrap().wait().unwrap();
            }
        }
    }
}
