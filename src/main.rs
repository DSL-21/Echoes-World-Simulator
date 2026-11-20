mod persistence;
mod sim;

use std::collections::HashSet;

use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{Activate, Button},
};
use sim::{
    GameState, PLAYER_ID,
    diplomacy::Treaty,
    economy::Building,
    market::Commodity,
    military::{Army, UnitClass},
    world::{Coord, Terrain},
};

const TILE_STEP: f32 = 26.0;
const TILE_SIZE: f32 = 23.0;
const MAP_CENTER: Vec2 = Vec2::new(-145.0, -8.0);
const SAVE_PATH: &str = "echoes_save.json";

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.018, 0.027, 0.043)))
        .insert_resource(Session(GameState::new(20261120)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ECHOES — Living World Simulator".to_owned(),
                resolution: (1600, 900).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_observer(handle_ui_activate)
        .add_systems(
            Update,
            (
                handle_camera,
                handle_mouse_selection,
                handle_keyboard,
                update_tile_visuals,
                update_city_markers,
                update_army_markers,
                update_selection_highlight,
                update_hud,
            )
                .chain(),
        )
        .run();
}

#[derive(Resource)]
struct Session(GameState);

#[derive(Component)]
struct TileVisual(Coord);

#[derive(Component)]
struct CityVisual(Coord);

#[derive(Component)]
struct ArmyVisual(u64);

#[derive(Component)]
struct SelectionHighlight;

#[derive(Component, Clone, Copy)]
enum HudLine {
    Header,
    Resources,
    Market,
    Realm,
    Selected,
    Politics,
    Chronicle,
}

#[derive(Component, Clone, Copy)]
enum UiAction {
    EndTurn,
    Claim,
    FoundCity,
    Farm,
    LumberMill,
    Quarry,
    Market,
    IronMine,
    Library,
    Barracks,
    Aqueduct,
    Recruit,
    RecruitCavalry,
    RecruitSiege,
    Research,
    Trade,
    War,
    Peace,
    MoveArmy,
    Save,
    Load,
    Reform,
    BuyFood,
    SellFood,
    BuyTimber,
    SellTimber,
    BuyStone,
    SellStone,
    BuyIron,
    SellIron,
    TaxLower,
    TaxRaise,
}

fn setup(mut commands: Commands, session: Res<Session>) {
    commands.spawn(Camera2d);

    for coord in session.0.world.coords() {
        let position = tile_position(coord);
        commands.spawn((
            Sprite::from_color(
                Color::srgb(0.028, 0.046, 0.065),
                Vec2::splat(TILE_STEP - 1.0),
            ),
            Transform::from_translation(position),
        ));
        commands.spawn((
            Sprite::from_color(Color::srgb(0.1, 0.1, 0.1), Vec2::splat(TILE_SIZE)),
            Transform::from_translation(position + Vec3::Z),
            TileVisual(coord),
        ));
        commands.spawn((
            Sprite::from_color(Color::srgb(0.97, 0.86, 0.62), Vec2::splat(10.0)),
            Transform::from_translation(position + Vec3::new(0.0, 0.0, 4.0)),
            CityVisual(coord),
            Visibility::Hidden,
        ));
    }

    commands.spawn((
        Sprite::from_color(
            Color::srgba(1.0, 0.83, 0.43, 0.26),
            Vec2::splat(TILE_STEP - 1.0),
        ),
        Transform::from_translation(tile_position(session.0.selected) + Vec3::new(0.0, 0.0, 2.0)),
        SelectionHighlight,
    ));

    spawn_hud(&mut commands);
}

fn spawn_hud(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(0),
                left: px(0),
                width: percent(100),
                height: px(68),
                padding: UiRect::axes(px(24), px(10)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.043, 0.065, 0.98)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("E C H O E S"),
                TextFont::from_font_size(27.0),
                TextColor(Color::srgb(0.91, 0.83, 0.61)),
            ));
            parent.spawn((
                Text::new("THE LIVING WORLD"),
                TextFont::from_font_size(15.0),
                TextColor(Color::srgb(0.55, 0.69, 0.77)),
                HudLine::Header,
            ));
            parent.spawn((
                Text::new("WASD PAN  •  +/- ZOOM  •  SPACE ADVANCE"),
                TextFont::from_font_size(11.0),
                TextColor(Color::srgb(0.42, 0.56, 0.65)),
            ));
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(82),
                right: px(16),
                width: px(360),
                max_height: percent(88),
                padding: UiRect::all(px(15)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.035, 0.055, 0.078, 0.97)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("REALM COUNCIL"),
                TextFont::from_font_size(19.0),
                TextColor(Color::srgb(0.91, 0.83, 0.61)),
            ));
            parent.spawn((
                Text::new(""),
                TextFont::from_font_size(12.5),
                TextColor(Color::srgb(0.70, 0.80, 0.82)),
                HudLine::Resources,
            ));
            parent.spawn((
                Text::new("WORLD MARKET"),
                TextFont::from_font_size(13.0),
                TextColor(Color::srgb(0.91, 0.83, 0.61)),
            ));
            parent.spawn((
                Text::new(""),
                TextFont::from_font_size(11.0),
                TextColor(Color::srgb(0.63, 0.75, 0.77)),
                HudLine::Market,
            ));
            parent.spawn((
                Text::new("TRADE DESK  •  10 UNITS PER ORDER"),
                TextFont::from_font_size(11.5),
                TextColor(Color::srgb(0.91, 0.83, 0.61)),
            ));
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(6),
                ..default()
            }).with_children(|row| {
                row.spawn((
                    Button,
                    Hovered::default(),
                    UiAction::BuyFood,
                    Node {
                        width: px(160),
                        height: px(28),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.17, 0.31, 0.36)),
                )).with_children(|b| {
                    b.spawn((Text::new("BUY FOOD"), TextFont::from_font_size(10.0), TextColor(Color::WHITE)));
                });
                row.spawn((
                    Button,
                    Hovered::default(),
                    UiAction::SellFood,
                    Node {
                        width: px(160),
                        height: px(28),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.21, 0.36, 0.29)),
                )).with_children(|b| {
                    b.spawn((Text::new("SELL FOOD"), TextFont::from_font_size(10.0), TextColor(Color::WHITE)));
                });
            });
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(6),
                ..default()
            }).with_children(|row| {
                row.spawn((
                    Button,
                    Hovered::default(),
                    UiAction::BuyTimber,
                    Node {
                        width: px(160),
                        height: px(28),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.17, 0.31, 0.36)),
                )).with_children(|b| {
                    b.spawn((Text::new("BUY TIMBER"), TextFont::from_font_size(10.0), TextColor(Color::WHITE)));
                });
                row.spawn((
                    Button,
                    Hovered::default(),
                    UiAction::SellTimber,
                    Node {
                        width: px(160),
                        height: px(28),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.21, 0.36, 0.29)),
                )).with_children(|b| {
                    b.spawn((Text::new("SELL TIMBER"), TextFont::from_font_size(10.0), TextColor(Color::WHITE)));
                });
            });
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(6),
                ..default()
            }).with_children(|row| {
                row.spawn((
                    Button,
                    Hovered::default(),
                    UiAction::BuyStone,
                    Node {
                        width: px(160),
                        height: px(28),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.29, 0.34)),
                )).with_children(|b| {
                    b.spawn((Text::new("BUY STONE"), TextFont::from_font_size(10.0), TextColor(Color::WHITE)));
                });
                row.spawn((
                    Button,
                    Hovered::default(),
                    UiAction::SellStone,
                    Node {
                        width: px(160),
                        height: px(28),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.34, 0.32, 0.25)),
                )).with_children(|b| {
                    b.spawn((Text::new("SELL STONE"), TextFont::from_font_size(10.0), TextColor(Color::WHITE)));
                });
            });
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(6),
                ..default()
            }).with_children(|row| {
                row.spawn((
                    Button,
                    Hovered::default(),
                    UiAction::BuyIron,
                    Node {
                        width: px(160),
                        height: px(28),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.25, 0.29, 0.34)),
                )).with_children(|b| {
                    b.spawn((Text::new("BUY IRON"), TextFont::from_font_size(10.0), TextColor(Color::WHITE)));
                });
                row.spawn((
                    Button,
                    Hovered::default(),
                    UiAction::SellIron,
                    Node {
                        width: px(160),
                        height: px(28),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.34, 0.32, 0.25)),
                )).with_children(|b| {
                    b.spawn((Text::new("SELL IRON"), TextFont::from_font_size(10.0), TextColor(Color::WHITE)));
                });
            });
            parent.spawn((
                Text::new(""),
                TextFont::from_font_size(12.0),
                TextColor(Color::srgb(0.50, 0.68, 0.74)),
                HudLine::Realm,
            ));
            parent.spawn((
                Text::new("SELECTED PROVINCE"),
                TextFont::from_font_size(14.0),
                TextColor(Color::srgb(0.91, 0.83, 0.61)),
            ));
            parent.spawn((
                Text::new(""),
                TextFont::from_font_size(11.5),
                TextColor(Color::srgb(0.76, 0.81, 0.79)),
                HudLine::Selected,
            ));
            parent.spawn((
                Text::new("DIPLOMACY"),
                TextFont::from_font_size(14.0),
                TextColor(Color::srgb(0.91, 0.83, 0.61)),
            ));
            parent.spawn((
                Text::new(""),
                TextFont::from_font_size(11.5),
                TextColor(Color::srgb(0.65, 0.75, 0.79)),
                HudLine::Politics,
            ));
            parent.spawn((
                Text::new("ORDERS"),
                TextFont::from_font_size(14.0),
                TextColor(Color::srgb(0.91, 0.83, 0.61)),
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(5),
                    ..default()
                })
                .with_children(|buttons| {
                    buttons
                        .spawn((
                            Button,
                            Hovered::default(),
                            UiAction::EndTurn,
                            Node {
                                width: percent(100),
                                height: px(34),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.22, 0.39, 0.42)),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("ADVANCE YEAR  [SPACE]"),
                                TextFont::from_font_size(12.0),
                                TextColor(Color::WHITE),
                            ));
                        });

                    buttons
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Claim,
                                Node {
                                    width: px(160),
                                    height: px(30),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.20, 0.32, 0.37)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("CLAIM  [C]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::FoundCity,
                                Node {
                                    width: px(160),
                                    height: px(30),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.32, 0.34, 0.23)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("FOUND CITY  [B]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });

                    buttons
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Farm,
                                Node {
                                    width: px(160),
                                    height: px(30),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.25, 0.35, 0.25)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("FARM  [1]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::LumberMill,
                                Node {
                                    width: px(160),
                                    height: px(30),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.18, 0.33, 0.27)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("LUMBER MILL  [2]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });

                    buttons
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Quarry,
                                Node {
                                    width: px(160),
                                    height: px(30),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.30, 0.32, 0.36)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("QUARRY  [3]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Market,
                                Node {
                                    width: px(160),
                                    height: px(30),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.39, 0.30, 0.21)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("MARKET  [4]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });

                    buttons
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::IronMine,
                                Node { width: px(160), height: px(30), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                                BackgroundColor(Color::srgb(0.28, 0.30, 0.35)),
                            ))
                            .with_children(|b| { b.spawn((Text::new("IRON MINE  [5]"), TextFont::from_font_size(11.0), TextColor(Color::WHITE))); });
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Library,
                                Node { width: px(160), height: px(30), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                                BackgroundColor(Color::srgb(0.30, 0.25, 0.43)),
                            ))
                            .with_children(|b| { b.spawn((Text::new("LIBRARY  [6]"), TextFont::from_font_size(11.0), TextColor(Color::WHITE))); });
                        });

                    buttons
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Barracks,
                                Node { width: px(160), height: px(30), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                                BackgroundColor(Color::srgb(0.42, 0.22, 0.22)),
                            ))
                            .with_children(|b| { b.spawn((Text::new("BARRACKS  [7]"), TextFont::from_font_size(11.0), TextColor(Color::WHITE))); });
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Aqueduct,
                                Node { width: px(160), height: px(30), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                                BackgroundColor(Color::srgb(0.21, 0.32, 0.40)),
                            ))
                            .with_children(|b| { b.spawn((Text::new("AQUEDUCT  [8]"), TextFont::from_font_size(11.0), TextColor(Color::WHITE))); });
                        });

                    buttons
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Recruit,
                                Node {
                                    width: px(160),
                                    height: px(30),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.39, 0.22, 0.22)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("RECRUIT  [G]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Research,
                                Node {
                                    width: px(160),
                                    height: px(30),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.28, 0.25, 0.42)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("RESEARCH  [R]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });

                    buttons
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Reform,
                                Node {
                                    width: px(160),
                                    height: px(30),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.31, 0.29, 0.44)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("NATIONAL REFORM  [L]"),
                                    TextFont::from_font_size(10.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });

                    buttons
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::TaxLower,
                                Node {
                                    width: px(160),
                                    height: px(30),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.20, 0.32, 0.35)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("LOWER TAX  ["),
                                    TextFont::from_font_size(10.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::TaxRaise,
                                Node {
                                    width: px(160),
                                    height: px(30),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.42, 0.26, 0.20)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("RAISE TAX  ]"),
                                    TextFont::from_font_size(10.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });

                    buttons
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Trade,
                                Node {
                                    width: px(160),
                                    height: px(30),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.18, 0.35, 0.35)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("TRADE  [T]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::War,
                                Node {
                                    width: px(160),
                                    height: px(30),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.43, 0.19, 0.20)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("DECLARE WAR  [X]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });

                    buttons
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Peace,
                                Node {
                                    width: px(160),
                                    height: px(30),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.22, 0.35, 0.32)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("OFFER PEACE  [P]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::MoveArmy,
                                Node {
                                    width: px(160),
                                    height: px(30),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.34, 0.28, 0.20)),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new("MOVE ARMY  [M]"),
                                    TextFont::from_font_size(11.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });

                    buttons
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::RecruitCavalry,
                                Node { width: px(160), height: px(30), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                                BackgroundColor(Color::srgb(0.38, 0.28, 0.19)),
                            ))
                            .with_children(|b| { b.spawn((Text::new("CAVALRY  [V]"), TextFont::from_font_size(11.0), TextColor(Color::WHITE))); });
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::RecruitSiege,
                                Node { width: px(160), height: px(30), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                                BackgroundColor(Color::srgb(0.35, 0.30, 0.29)),
                            ))
                            .with_children(|b| { b.spawn((Text::new("SIEGE  [J]"), TextFont::from_font_size(11.0), TextColor(Color::WHITE))); });
                        });

                    buttons
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(6),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Save,
                                Node { width: px(160), height: px(30), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                                BackgroundColor(Color::srgb(0.19, 0.34, 0.31)),
                            ))
                            .with_children(|b| { b.spawn((Text::new("SAVE CAMPAIGN  [F5]"), TextFont::from_font_size(10.0), TextColor(Color::WHITE))); });
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Load,
                                Node { width: px(160), height: px(30), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                                BackgroundColor(Color::srgb(0.25, 0.29, 0.38)),
                            ))
                            .with_children(|b| { b.spawn((Text::new("LOAD CAMPAIGN  [F9]"), TextFont::from_font_size(10.0), TextColor(Color::WHITE))); });
                        });
                });

            parent.spawn((
                Text::new("WORLD CHRONICLE"),
                TextFont::from_font_size(14.0),
                TextColor(Color::srgb(0.91, 0.83, 0.61)),
            ));
            parent.spawn((
                Text::new(""),
                TextFont::from_font_size(10.5),
                TextColor(Color::srgb(0.61, 0.72, 0.75)),
                HudLine::Chronicle,
            ));
            parent.spawn((
                Text::new("F5 SAVE  •  F9 LOAD  •  T TRADE  •  X WAR  •  P PEACE"),
                TextFont::from_font_size(9.5),
                TextColor(Color::srgb(0.39, 0.52, 0.61)),
            ));
        });

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(18),
            bottom: px(16),
            padding: UiRect::axes(px(13), px(8)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.025, 0.043, 0.065, 0.92)),
        Text::new(
            "LEFT CLICK SELECT  •  WASD PAN  •  +/- ZOOM  •  1–8 DEVELOP  •  B CITY  •  G/V/J ARMY  •  L REFORM",
        ),
        TextFont::from_font_size(10.5),
        TextColor(Color::srgb(0.57, 0.68, 0.72)),
    ));
}

fn handle_camera(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    let mut direction = Vec2::ZERO;
    if input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if direction.length_squared() > 0.0 {
        let zoom = camera.scale.x.max(0.35);
        let movement = direction.normalize() * (620.0 * zoom * time.delta_secs());
        camera.translation.x += movement.x;
        camera.translation.y += movement.y;
    }

    let mut zoom = camera.scale.x;
    if input.just_pressed(KeyCode::Equal) || input.just_pressed(KeyCode::NumpadAdd) {
        zoom *= 0.88;
    }
    if input.just_pressed(KeyCode::Minus) || input.just_pressed(KeyCode::NumpadSubtract) {
        zoom *= 1.14;
    }
    zoom = zoom.clamp(0.45, 2.2);
    camera.scale = Vec3::splat(zoom);
}

fn handle_mouse_selection(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut session: ResMut<Session>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    if cursor.x > window.width() - 385.0 || cursor.y < 70.0 {
        return;
    }
    let (camera, camera_transform) = *camera;
    let Ok(position) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        return;
    };
    let width = session.0.world.width;
    let height = session.0.world.height;
    let x = ((position.x - MAP_CENTER.x) / TILE_STEP + (width - 1) as f32 / 2.0).round() as i32;
    let y = ((height - 1) as f32 / 2.0 - (position.y - MAP_CENTER.y) / TILE_STEP).round() as i32;
    session.0.select(Coord::new(x, y));
}

fn handle_keyboard(input: Res<ButtonInput<KeyCode>>, mut session: ResMut<Session>) {
    if input.just_pressed(KeyCode::F9) {
        match persistence::load_game(SAVE_PATH) {
            Ok(loaded) => session.0 = loaded,
            Err(error) => session.0.add_log(format!("LOAD ERROR — {error}")),
        }
        return;
    }
    let game = &mut session.0;
    if input.just_pressed(KeyCode::F5) {
        match persistence::save_game(game, SAVE_PATH) {
            Ok(()) => game.add_log("Campaign saved to echoes_save.json."),
            Err(error) => game.add_log(format!("SAVE ERROR — {error}")),
        }
        return;
    }
    if game.is_ended() {
        return;
    }
    if input.just_pressed(KeyCode::Space) {
        game.advance_turn();
    }
    if input.just_pressed(KeyCode::KeyC) {
        game.claim_selected();
    }
    if input.just_pressed(KeyCode::Digit1) {
        game.build_selected(Building::Farm);
    }
    if input.just_pressed(KeyCode::Digit2) {
        game.build_selected(Building::LumberMill);
    }
    if input.just_pressed(KeyCode::Digit3) {
        game.build_selected(Building::Quarry);
    }
    if input.just_pressed(KeyCode::Digit4) {
        game.build_selected(Building::Market);
    }
    if input.just_pressed(KeyCode::Digit5) {
        game.build_selected(Building::IronMine);
    }
    if input.just_pressed(KeyCode::Digit6) {
        game.build_selected(Building::Library);
    }
    if input.just_pressed(KeyCode::Digit7) {
        game.build_selected(Building::Barracks);
    }
    if input.just_pressed(KeyCode::Digit8) {
        game.build_selected(Building::Aqueduct);
    }
    if input.just_pressed(KeyCode::KeyB) {
        game.found_city();
    }
    if input.just_pressed(KeyCode::KeyG) {
        game.recruit_army(UnitClass::Infantry);
    }
    if input.just_pressed(KeyCode::KeyV) {
        game.recruit_army(UnitClass::Cavalry);
    }
    if input.just_pressed(KeyCode::KeyJ) {
        game.recruit_army(UnitClass::Siege);
    }
    if input.just_pressed(KeyCode::KeyR) {
        game.start_next_research();
    }
    if input.just_pressed(KeyCode::KeyL) {
        game.cycle_player_policy();
    }
    if input.just_pressed(KeyCode::BracketLeft) {
        game.adjust_tax_rate(-0.02);
    }
    if input.just_pressed(KeyCode::BracketRight) {
        game.adjust_tax_rate(0.02);
    }
    if input.just_pressed(KeyCode::KeyM) {
        game.move_nearest_army_to_selected();
    }
    if input.just_pressed(KeyCode::KeyT) {
        if let Some(target) = selected_foreign_owner(game) {
            game.offer_trade_agreement(target);
        } else {
            game.add_log("Select a province belonging to another realm before negotiating.");
        }
    }
    if input.just_pressed(KeyCode::KeyX) {
        if let Some(target) = selected_foreign_owner(game) {
            game.declare_war(target);
        } else {
            game.add_log("Select a foreign province to identify a diplomatic target.");
        }
    }
    if input.just_pressed(KeyCode::KeyP) {
        if let Some(target) = selected_foreign_owner(game) {
            game.offer_peace(target);
        } else {
            game.add_log("Select a foreign province to negotiate peace.");
        }
    }
}

fn selected_foreign_owner(game: &GameState) -> Option<usize> {
    game.world
        .get(game.selected)
        .and_then(|tile| tile.owner)
        .filter(|owner| *owner != PLAYER_ID)
}

fn handle_ui_activate(
    event: On<Activate>,
    mut session: ResMut<Session>,
    actions: Query<&UiAction>,
) {
    let Ok(action) = actions.get(event.entity) else {
        return;
    };
    if matches!(*action, UiAction::Load) {
        match persistence::load_game(SAVE_PATH) {
            Ok(loaded) => session.0 = loaded,
            Err(error) => session.0.add_log(format!("LOAD ERROR — {error}")),
        }
        return;
    }
    let game = &mut session.0;
    if game.is_ended() && !matches!(*action, UiAction::Save) {
        return;
    }
    match *action {
        UiAction::EndTurn => game.advance_turn(),
        UiAction::Claim => {
            game.claim_selected();
        }
        UiAction::FoundCity => {
            game.found_city();
        }
        UiAction::Farm => {
            game.build_selected(Building::Farm);
        }
        UiAction::LumberMill => {
            game.build_selected(Building::LumberMill);
        }
        UiAction::Quarry => {
            game.build_selected(Building::Quarry);
        }
        UiAction::Market => {
            game.build_selected(Building::Market);
        }
        UiAction::IronMine => {
            game.build_selected(Building::IronMine);
        }
        UiAction::Library => {
            game.build_selected(Building::Library);
        }
        UiAction::Barracks => {
            game.build_selected(Building::Barracks);
        }
        UiAction::Aqueduct => {
            game.build_selected(Building::Aqueduct);
        }
        UiAction::Recruit => {
            game.recruit_army(UnitClass::Infantry);
        }
        UiAction::RecruitCavalry => {
            game.recruit_army(UnitClass::Cavalry);
        }
        UiAction::RecruitSiege => {
            game.recruit_army(UnitClass::Siege);
        }
        UiAction::Research => {
            game.start_next_research();
        }
        UiAction::Trade => {
            if let Some(target) = selected_foreign_owner(game) {
                game.offer_trade_agreement(target);
            } else {
                game.add_log("Select a foreign province before proposing a trade agreement.");
            }
        }
        UiAction::War => {
            if let Some(target) = selected_foreign_owner(game) {
                game.declare_war(target);
            } else {
                game.add_log("Select a foreign province before declaring war.");
            }
        }
        UiAction::Peace => {
            if let Some(target) = selected_foreign_owner(game) {
                game.offer_peace(target);
            } else {
                game.add_log("Select a foreign province before offering peace.");
            }
        }
        UiAction::MoveArmy => {
            game.move_nearest_army_to_selected();
        }
        UiAction::Save => {
            match persistence::save_game(game, SAVE_PATH) {
                Ok(()) => game.add_log("Campaign saved to echoes_save.json."),
                Err(error) => game.add_log(format!("SAVE ERROR — {error}")),
            }
        }
        UiAction::Load => {}
        UiAction::Reform => {
            game.cycle_player_policy();
        }
        UiAction::BuyFood => {
            game.buy_commodity(Commodity::Food, 10.0);
        }
        UiAction::SellFood => {
            game.sell_commodity(Commodity::Food, 10.0);
        }
        UiAction::BuyTimber => {
            game.buy_commodity(Commodity::Timber, 10.0);
        }
        UiAction::SellTimber => {
            game.sell_commodity(Commodity::Timber, 10.0);
        }
        UiAction::BuyStone => {
            game.buy_commodity(Commodity::Stone, 10.0);
        }
        UiAction::SellStone => {
            game.sell_commodity(Commodity::Stone, 10.0);
        }
        UiAction::BuyIron => {
            game.buy_commodity(Commodity::Iron, 10.0);
        }
        UiAction::SellIron => {
            game.sell_commodity(Commodity::Iron, 10.0);
        }
        UiAction::TaxLower => {
            game.adjust_tax_rate(-0.02);
        }
        UiAction::TaxRaise => {
            game.adjust_tax_rate(0.02);
        }
    }
}

fn update_tile_visuals(session: Res<Session>, mut visuals: Query<(&TileVisual, &mut Sprite)>) {
    if !session.is_changed() {
        return;
    }
    for (visual, mut sprite) in &mut visuals {
        if let Some(tile) = session.0.world.get(visual.0) {
            let mut rgb = tile.terrain.color();
            if let Some(owner) = tile.owner {
                if let Some(faction) = session.0.factions.get(owner) {
                    rgb = [
                        rgb[0] * 0.50 + faction.color[0] * 0.50,
                        rgb[1] * 0.50 + faction.color[1] * 0.50,
                        rgb[2] * 0.50 + faction.color[2] * 0.50,
                    ];
                }
            }
            if tile.buildings.iter().any(|b| *b == Building::Farm) {
                rgb[1] = (rgb[1] + 0.06).min(1.0);
            }
            sprite.color = Color::srgb(rgb[0], rgb[1], rgb[2]);
        }
    }
}

fn update_city_markers(
    session: Res<Session>,
    mut markers: Query<(&CityVisual, &mut Transform, &mut Visibility)>,
) {
    if !session.is_changed() {
        return;
    }
    for (marker, mut transform, mut visibility) in &mut markers {
        if let Some(tile) = session.0.world.get(marker.0) {
            if tile.city.is_some() {
                transform.translation = tile_position(marker.0) + Vec3::new(0.0, 0.0, 4.0);
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

fn update_army_markers(
    mut commands: Commands,
    session: Res<Session>,
    mut markers: Query<(&ArmyVisual, &mut Transform, &mut Visibility, &mut Sprite)>,
) {
    if !session.is_changed() {
        return;
    }
    let existing: HashSet<u64> = markers
        .iter_mut()
        .map(|(marker, _, _, _)| marker.0)
        .collect();
    for army in &session.0.armies {
        if !existing.contains(&army.id) {
            let color = session.0.factions[army.owner].color;
            commands.spawn((
                Sprite::from_color(Color::srgb(color[0], color[1], color[2]), Vec2::splat(8.0)),
                Transform::from_translation(
                    tile_position(army.position) + Vec3::new(7.0, -7.0, 5.0),
                ),
                ArmyVisual(army.id),
                Visibility::Visible,
            ));
        }
    }
    for (marker, mut transform, mut visibility, mut sprite) in &mut markers {
        if let Some(army) = session.0.armies.iter().find(|army| army.id == marker.0) {
            transform.translation = tile_position(army.position) + Vec3::new(7.0, -7.0, 5.0);
            let color = session.0.factions[army.owner].color;
            sprite.color = Color::srgb(color[0], color[1], color[2]);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

fn update_selection_highlight(
    session: Res<Session>,
    mut highlight: Single<&mut Transform, With<SelectionHighlight>>,
) {
    if session.is_changed() {
        highlight.translation = tile_position(session.0.selected) + Vec3::new(0.0, 0.0, 2.0);
    }
}

fn update_hud(session: Res<Session>, mut labels: Query<(&mut Text, &HudLine)>) {
    if !session.is_changed() {
        return;
    }
    let game = &session.0;
    let player = &game.factions[PLAYER_ID];
    let target = selected_foreign_owner(game);
    for (mut text, line) in &mut labels {
        **text = match line {
            HudLine::Header => {
                if let Some(outcome) = &game.outcome {
                    format!(
                        "CAMPAIGN VICTORY — {} achieves {} in year {}",
                        game.faction_name(Some(outcome.winner_id)),
                        outcome.path.name(),
                        outcome.year
                    )
                } else if game.defeat_reason.is_some() {
                    format!("CAMPAIGN DEFEAT — THE CAPITAL HAS FALLEN — YEAR {}", game.year)
                } else {
                    format!("THE LIVING WORLD  •  YEAR {}  •  TURN {}", game.year, game.turn)
                }
            },
            HudLine::Resources => format!(
                "FOOD {:>5.0}   TIMBER {:>5.0}   STONE {:>5.0}\nIRON {:>5.0}   GOLD {:>5.0}   SCIENCE {:>5.0}   INFLUENCE {:>5.0}",
                player.resources.food,
                player.resources.timber,
                player.resources.stone,
                player.resources.iron,
                player.resources.gold,
                player.resources.science,
                player.resources.influence
            ),
            HudLine::Market => format!(
                "FOOD {:.2}  •  TIMBER {:.2}\nSTONE {:.2}  •  IRON {:.2}",
                game.market.food_price,
                game.market.timber_price,
                game.market.stone_price,
                game.market.iron_price
            ),
            HudLine::Realm => format!(
                "POP {}  •  PROVINCES {}  •  CITIES {}  •  ARMIES {}\n{}  •  {}\nTAX {:.0}%  •  STABILITY {:.0}%  •  UNREST {:.0}%  •  LEGITIMACY {:.0}%\nPRESTIGE {:.0}  •  WAR EXHAUSTION {:.0}%",
                player.population,
                game.owned_count(PLAYER_ID),
                game.city_count(PLAYER_ID),
                game.army_count(PLAYER_ID),
                player.government.name(),
                player.policy.name(),
                player.tax_rate * 100.0,
                player.stability,
                player.unrest,
                player.legitimacy,
                player.prestige,
                player.war_exhaustion
            ),
            HudLine::Selected => {
                if let Some(tile) = game.world.get(game.selected) {
                    let city = tile.city.as_deref().unwrap_or("No city");
                    let buildings = if tile.buildings.is_empty() {
                        "None".to_owned()
                    } else {
                        tile.buildings
                            .iter()
                            .map(|b| b.name())
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    format!(
                        "{}  •  {}, {}\nOwner: {}\nSettlement: {}  •  Population: {}\nBuildings: {}",
                        tile.terrain.name(),
                        game.selected.x + 1,
                        game.selected.y + 1,
                        game.faction_name(tile.owner),
                        city,
                        tile.population,
                        buildings
                    )
                } else {
                    "No province selected".to_owned()
                }
            }
            HudLine::Politics => {
                if let Some(target) = target {
                    let relation = &game.relations[PLAYER_ID][target];
                    format!(
                        "Target: {}\nStatus: {}  •  Opinion: {}\n[T] trade  [X] war  [P] peace",
                        game.factions[target].name,
                        relation.treaty.label(),
                        relation.opinion
                    )
                } else {
                    "Select a foreign province to inspect relations and negotiate.".to_owned()
                }
            }
            HudLine::Chronicle => {
                let mut output = String::new();
                for entry in game.chronicle.iter().rev().take(5).rev() {
                    output.push_str("• ");
                    output.push_str(entry);
                    output.push('\n');
                }
                output
            }
        };
    }
}

fn tile_position(coord: Coord) -> Vec3 {
    let width = sim::WORLD_WIDTH;
    let height = sim::WORLD_HEIGHT;
    let x = MAP_CENTER.x + (coord.x as f32 - (width - 1) as f32 / 2.0) * TILE_STEP;
    let y = MAP_CENTER.y + ((height - 1) as f32 / 2.0 - coord.y as f32) * TILE_STEP;
    Vec3::new(x, y, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_view_transform_matches_simulation_dimensions() {
        let game = GameState::new(19);
        assert_eq!(game.world.width, sim::WORLD_WIDTH);
        assert_eq!(game.world.height, sim::WORLD_HEIGHT);
        assert!(game.world.get(Coord::new(0, 0)).is_some());
    }

    #[test]
    fn every_terrain_has_a_visible_color() {
        let colors = [
            Terrain::Ocean.color(),
            Terrain::Coast.color(),
            Terrain::Grassland.color(),
            Terrain::Plains.color(),
            Terrain::Forest.color(),
            Terrain::Desert.color(),
            Terrain::Mountain.color(),
            Terrain::Tundra.color(),
        ];
        assert!(
            colors
                .iter()
                .all(|rgb| rgb.iter().all(|channel| (0.0..=1.0).contains(channel)))
        );
    }
}
