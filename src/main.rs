use bevy::prelude::*;
use bevy::render::color::HexColorError;
use bevy_egui::{EguiPlugin, EguiContexts, egui};
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes::Circle;
use std::fs::File;
use std::io::{BufReader, Read};
use svg::node::element::path::{Command, Data};
use svg::node::element::tag::Circle;
use svg::node::element::tag::Path;
use wasm_bindgen::prelude::*;

use svg::parser::Event;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(ShapePlugin)
        .add_systems(Startup, setup_system)
        .add_systems(Update, ui_example_system)
        // .add_systems(Update, update)
        .run();
}

#[derive(Component)]
struct Name(String);

#[derive(Component)]
struct RoadSign;

#[derive(Component)]
struct RoadSignLimit;

#[derive(Component)]
struct RoadSignTempBackground;

#[derive(Component)]
struct RoadSignCommonBackground;

#[derive(Component)]
struct RoadSignInspector {
    limit: u32,
    is_temp: bool
}

impl Default for RoadSignInspector {
    fn default() -> Self {
        Self {
            limit: 90,
            is_temp: false
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

fn ui_example_system(
    mut contexts: EguiContexts,
    mut inspector: Local<RoadSignInspector>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut child_query: Query<Entity, With<RoadSign>>,
    mut transform_query: Query<&mut Transform, With<RoadSign>>,
    mut text_query: Query<&mut Text, With<RoadSignLimit>>,
    mut temp_back_query: Query<&mut Fill, With<RoadSignTempBackground>>
) {
    egui::Window::new("Sign props").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui: &mut egui::Ui| {
            ui.label("Ограничение");
            if ui.add(egui::widgets::DragValue::new(&mut inspector.limit).speed(1.).clamp_range(5.0..=110.)).changed() {
                updateSignLimit(text_query, inspector.limit);
            }
        });
        ui.horizontal(|ui: &mut egui::Ui| {
            if ui.add(egui::widgets::Checkbox::new(&mut inspector.is_temp, "Временный")).changed() {
                updateSignTempChange(temp_back_query, inspector.is_temp);
            }
        });
    });
}

fn create_road_sign(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut inspector: &Local<RoadSignInspector>,
    query: Query<Entity, With<RoadSign>>,
) {
    let t: &str = "
    <svg width=\"300\" height=\"300\" viewBox=\"0 0 300 300\" fill=\"none\" xmlns=\"http://www.w3.org/2000/svg\">
    <path d=\"M300 150C300 232.843 232.843 300 150 300C67.1573 300 0 232.843 0 150C0 67.1572 67.1573 0 150 0C232.843 0 300 67.1572 300 150Z\" fill=\"black\"/>
    <path d=\"M297 150C297 231.186 231.186 297 150 297C68.8141 297 3 231.186 3 150C3 68.8141 68.8141 3 150 3C231.186 3 297 68.8141 297 150Z\" fill=\"white\"/>
    <circle cx=\"150\" cy=\"150\" r=\"142\" fill=\"#FF0000\"/>
    <circle cx=\"150\" cy=\"150\" r=\"119\" fill=\"white\" tempMarker=\"true\"/>
    <textgenerator value=\"50\"/>
    </svg>
    ";
    let font = asset_server.load("fonts/russian_roads_font.otf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 180.0,
        color: Color::BLACK,
    };

    commands
        .spawn((
            Name("Sign".to_owned()),
            RoadSign,
            SpatialBundle {
                transform: Transform {
                    translation: Vec3::new(0., 0., 0.),
                    scale: Vec3::new(1., 1., 1.),
                    ..Default::default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            let svg_doc_size = Vec2::new(300., 300.);
            let s: svg::Parser<'_> = svg::read(&t).unwrap();
            for event in s {
                match event {
                    Event::Tag(Path, _, attributes) => {
                        let d: &svg::node::Value = attributes.get("d").unwrap();
                        let fill: std::option::Option<&svg::node::Value> = attributes.get("fill");
                        let mut color: Result<bevy::prelude::Color, HexColorError> =
                            Color::hex("000000");
                        if !fill.is_none() {
                            if fill.unwrap().to_string() == "white" {
                                color = Color::hex("FFFFFF");
                            } else if fill.unwrap().to_string() == "green" {
                                color = Color::hex("00FF00");
                            } else if fill.unwrap().to_string() == "red" {
                                color = Color::hex("FF0000");
                            } else if fill.unwrap().to_string() == "purple" {
                                color = Color::hex("BF40BF");
                            } else if fill.unwrap().to_string() == "blue" {
                                color = Color::hex("0000FF");
                            }
                        }
                        parent.spawn((
                            ShapeBundle {
                                path: GeometryBuilder::build_as(&shapes::SvgPathShape {
                                    svg_path_string: d.to_string(),
                                    svg_doc_size_in_px: svg_doc_size.to_owned(),
                                }),
                                ..default()
                            },
                            Fill::color(color.ok().unwrap()),
                        ));
                    }
                    Event::Tag(Circle, _, attributes) => {
                        let cx: &svg::node::Value = attributes.get("cx").unwrap();
                        let cy: &svg::node::Value = attributes.get("cy").unwrap();
                        let r: &svg::node::Value = attributes.get("r").unwrap();
                        let fill: std::option::Option<&svg::node::Value> = attributes.get("fill");
                        let tempMarker = attributes.get("tempMarker");
                        let mut color: Result<bevy::prelude::Color, HexColorError> =
                            Color::hex("000000");
                        if !fill.is_none() {
                            if fill.unwrap().to_string() == "white" {
                                color = Color::hex("FFFFFF");
                            } else if fill.unwrap().to_string() == "green" {
                                color = Color::hex("00FF00");
                            } else if fill.unwrap().to_string() == "red" {
                                color = Color::hex("FF0000");
                            } else if fill.unwrap().to_string() == "purple" {
                                color = Color::hex("BF40BF");
                            } else if fill.unwrap().to_string() == "blue" {
                                color = Color::hex("0000FF");
                            } else {
                                color = Color::hex(fill.unwrap().to_string());
                            }
                        }
                        if tempMarker.is_some() {
                            parent.spawn((
                                ShapeBundle {
                                    path: GeometryBuilder::build_as(&shapes::Circle {
                                        radius: r.to_string().parse::<f32>().unwrap(),
                                        center: Vec2 {
                                            x: 0.,
                                            y: 0.
                                        },
                                    }),
                                    ..default()
                                },
                                Fill::color(color.ok().unwrap()),
                                RoadSignTempBackground
                            ));
                        } else {
                            parent.spawn((
                                ShapeBundle {
                                    path: GeometryBuilder::build_as(&shapes::Circle {
                                        radius: r.to_string().parse::<f32>().unwrap(),
                                        center: Vec2 {
                                            x: 0.,
                                            y: 0.
                                        },
                                    }),
                                    ..default()
                                },
                                Fill::color(color.ok().unwrap()),
                            ));
                        }
                    }
                    Event::Tag("textgenerator", _, attributes) => {
                        let value = attributes.get("value").unwrap();
                        parent.spawn((Text2dBundle {
                            text: Text::from_section(inspector.limit.to_string(), text_style.clone())
                                .with_alignment(TextAlignment::Right),
                            transform: Transform {
                                translation: Vec3::new(0., 0., 1.),
                                scale: Vec3::new(1., 1., 0.),
                                ..Default::default()
                            },
                            ..default()
                        }, RoadSignLimit));
                    }
                    _ => {}
                }
            }
        });
}

fn setup_system(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut inspector: Local<RoadSignInspector>,
    query: Query<Entity, With<RoadSign>>,
) {
    commands.spawn(Camera2dBundle::default());
    create_road_sign(commands, asset_server, &mut inspector, query)
}

fn updateSignLimit(
    mut query: Query<&mut Text, With<RoadSignLimit>>,
    val: u32
) {
    for mut text in &mut query {
        text.sections[0].value = val.to_string();
        console_log!("{:?} {}", text.sections[0].value, val.to_string());
    }
}

fn updateSignTempChange(
    mut query: Query<&mut Fill, With<RoadSignTempBackground>>,
    is_temp: bool
) {
    for mut fill in &mut query {
        if is_temp {
            fill.color = Color::hex("FFFF00").unwrap();
        } else {
            fill.color = Color::hex("FFFFFF").unwrap();
        }
    }
}
