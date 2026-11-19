use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::{Activate, Button},
};
use std::collections::VecDeque;

const MAP_WIDTH: i32 = 32;
const MAP_HEIGHT: i32 = 22;
const TILE_STEP: f32 = 26.0;
const TILE_SIZE: f32 = 23.0;
const MAP_CENTER: Vec2 = Vec2::new(-170.0, -12.0);
const PLAYER_ID: usize = 0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
        .insert_resource(WorldState::generate())
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
                handle_mouse_selection,
                handle_keyboard,
                update_tile_visuals,
                update_selection_highlight,
                update_hud,
            )
                .chain(),
        )
        .run();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Terrain {
    Water,
    Grassland,
    Plains,
    Forest,
    Desert,
    Mountain,
}

impl Terrain {
    fn name(self) -> &'static str {
        match self {
            Self::Water => "Open water",
            Self::Grassland => "Grassland",
            Self::Plains => "Plains",
            Self::Forest => "Forest",
            Self::Desert => "Desert",
            Self::Mountain => "Mountains",
        }
    }

    fn base_color(self) -> [f32; 3] {
        match self {
            Self::Water => [0.08, 0.22, 0.34],
            Self::Grassland => [0.28, 0.43, 0.27],
            Self::Plains => [0.49, 0.50, 0.31],
            Self::Forest => [0.13, 0.32, 0.25],
            Self::Desert => [0.62, 0.43, 0.25],
            Self::Mountain => [0.43, 0.45, 0.48],
        }
    }

    fn is_land(self) -> bool {
        self != Self::Water
    }

    fn expansion_value(self) -> f32 {
        match self {
            Self::Water => -100.0,
            Self::Grassland => 2.0,
            Self::Plains => 3.2,
            Self::Forest => 3.5,
            Self::Desert => 1.7,
            Self::Mountain => 3.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Improvement {
    #[default]
    None,
    Farm,
    LoggingCamp,
    Quarry,
    Market,
}

impl Improvement {
    fn name(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Farm => "Farm",
            Self::LoggingCamp => "Logging camp",
            Self::Quarry => "Quarry",
            Self::Market => "Market",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct WorldTile {
    terrain: Terrain,
    owner: Option<usize>,
    improvement: Improvement,
    city: bool,
}

impl WorldTile {
    fn new(terrain: Terrain) -> Self {
        Self {
            terrain,
            owner: None,
            improvement: Improvement::None,
            city: false,
        }
    }
}

struct Faction {
    name: String,
    color: [f32; 3],
    capital: IVec2,
    population: u32,
    food: f32,
    wood: f32,
    stone: f32,
    gold: f32,
    influence: f32,
}

impl Faction {
    fn new(name: &str, color: [f32; 3], capital: IVec2) -> Self {
        Self {
            name: name.to_owned(),
            color,
            capital,
            population: 8,
            food: 48.0,
            wood: 28.0,
            stone: 16.0,
            gold: 65.0,
            influence: 22.0,
        }
    }
}

#[derive(Resource)]
struct WorldState {
    tiles: Vec<WorldTile>,
    factions: Vec<Faction>,
    selected: IVec2,
    turn: u32,
    year: u32,
    chronicle: VecDeque<String>,
    seed: u32,
}

impl WorldState {
    fn generate() -> Self {
        let seed = 20261119;
        let mut tiles = Vec::with_capacity((MAP_WIDTH * MAP_HEIGHT) as usize);

        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let continent = smooth_noise(x as f32 * 0.105, y as f32 * 0.105, seed);
                let detail =
                    smooth_noise(x as f32 * 0.29 + 31.0, y as f32 * 0.29 - 17.0, seed + 11);
                let moisture =
                    smooth_noise(x as f32 * 0.13 - 42.0, y as f32 * 0.13 + 19.0, seed + 37);
                let elevation = continent * 0.76 + detail * 0.24;

                let terrain = if elevation < 0.34 {
                    Terrain::Water
                } else if elevation > 0.79 {
                    Terrain::Mountain
                } else if moisture < 0.25 && elevation < 0.68 {
                    Terrain::Desert
                } else if moisture > 0.68 {
                    Terrain::Forest
                } else if detail > 0.62 {
                    Terrain::Plains
                } else {
                    Terrain::Grassland
                };

                tiles.push(WorldTile::new(terrain));
            }
        }

        let starts = [
            IVec2::new(6, 11),
            IVec2::new(25, 5),
            IVec2::new(25, 17),
            IVec2::new(15, 3),
            IVec2::new(15, 19),
        ];

        let definitions = [
            ("Aurelian League", [0.26, 0.66, 0.92]),
            ("Verdant Compact", [0.34, 0.76, 0.48]),
            ("Sable Dominion", [0.88, 0.35, 0.34]),
            ("Ivory Synod", [0.88, 0.72, 0.34]),
            ("Azure Cartel", [0.66, 0.48, 0.86]),
        ];

        let mut capitals: Vec<IVec2> = Vec::new();
        let mut factions = Vec::new();

        for (id, (name, color)) in definitions.iter().enumerate() {
            let capital = nearest_land(&tiles, starts[id], &capitals);
            capitals.push(capital);
            factions.push(Faction::new(name, *color, capital));

            for dy in -1_i32..=1 {
                for dx in -1_i32..=1 {
                    if dx.abs() + dy.abs() > 1 {
                        continue;
                    }
                    let point = capital + IVec2::new(dx, dy);
                    if let Some(index) = tile_index(point) {
                        if tiles[index].terrain.is_land() && tiles[index].owner.is_none() {
                            tiles[index].owner = Some(id);
                        }
                    }
                }
            }

            if let Some(index) = tile_index(capital) {
                tiles[index].owner = Some(id);
                tiles[index].city = true;
            }
        }

        let mut world = Self {
            tiles,
            factions,
            selected: capitals[PLAYER_ID],
            turn: 1,
            year: 1200,
            chronicle: VecDeque::new(),
            seed,
        };

        world.push_log("The age of realms begins. Five powers rise across an unknown land.");
        world.push_log("Your capital is founded. Claim nearby land and build a stronger realm.");
        world
    }

    fn push_log(&mut self, message: impl Into<String>) {
        self.chronicle.push_back(message.into());
        while self.chronicle.len() > 12 {
            self.chronicle.pop_front();
        }
    }

    fn tile(&self, point: IVec2) -> Option<&WorldTile> {
        tile_index(point).and_then(|index| self.tiles.get(index))
    }

    fn owned_count(&self, faction_id: usize) -> usize {
        self.tiles
            .iter()
            .filter(|tile| tile.owner == Some(faction_id))
            .count()
    }

    fn city_count(&self, faction_id: usize) -> usize {
        self.tiles
            .iter()
            .filter(|tile| tile.owner == Some(faction_id) && tile.city)
            .count()
    }
}

#[derive(Component)]
struct TileVisual(IVec2);

#[derive(Component)]
struct SelectionHighlight;

#[derive(Component, Clone, Copy)]
enum HudLine {
    Header,
    Resources,
    Realm,
    Selected,
    Chronicle,
}

#[derive(Component, Clone, Copy)]
enum UiAction {
    EndTurn,
    Claim,
    Farm,
    LoggingCamp,
    Quarry,
    Market,
}

fn setup(mut commands: Commands, world: Res<WorldState>) {
    commands.spawn(Camera2d);

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let point = IVec2::new(x, y);
            let position = tile_position(point);

            commands.spawn((
                Sprite::from_color(
                    Color::srgb(0.035, 0.055, 0.075),
                    Vec2::splat(TILE_STEP - 1.0),
                ),
                Transform::from_translation(position),
            ));
            commands.spawn((
                Sprite::from_color(Color::srgb(0.1, 0.1, 0.1), Vec2::splat(TILE_SIZE)),
                Transform::from_translation(position + Vec3::Z),
                TileVisual(point),
            ));
        }
    }

    for faction in &world.factions {
        let position = tile_position(faction.capital);
        commands.spawn((
            Sprite::from_color(
                Color::srgb(faction.color[0], faction.color[1], faction.color[2]),
                Vec2::splat(13.0),
            ),
            Transform::from_translation(position + Vec3::new(0.0, 0.0, 3.0)),
        ));
        commands.spawn((
            Sprite::from_color(Color::srgb(0.96, 0.91, 0.72), Vec2::splat(5.0)),
            Transform::from_translation(position + Vec3::new(0.0, 0.0, 4.0)),
        ));
    }

    commands.spawn((
        Sprite::from_color(
            Color::srgba(1.0, 0.84, 0.42, 0.28),
            Vec2::splat(TILE_STEP - 1.0),
        ),
        Transform::from_translation(tile_position(world.selected) + Vec3::new(0.0, 0.0, 2.0)),
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
            BackgroundColor(Color::srgba(0.035, 0.055, 0.08, 0.97)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("E C H O E S"),
                TextFont::from_font_size(27.0),
                TextColor(Color::srgb(0.91, 0.83, 0.61)),
            ));
            parent.spawn((
                Text::new("THE LIVING WORLD  •  YEAR 1200"),
                TextFont::from_font_size(15.0),
                TextColor(Color::srgb(0.55, 0.68, 0.76)),
                HudLine::Header,
            ));
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(88),
                right: px(20),
                width: px(330),
                padding: UiRect::all(px(17)),
                flex_direction: FlexDirection::Column,
                row_gap: px(11),
                ..default()
            },
            BackgroundColor(Color::srgba(0.045, 0.065, 0.09, 0.96)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("REALM COUNCIL"),
                TextFont::from_font_size(20.0),
                TextColor(Color::srgb(0.91, 0.83, 0.61)),
            ));
            parent.spawn((
                Text::new(""),
                TextFont::from_font_size(14.0),
                TextColor(Color::srgb(0.67, 0.79, 0.82)),
                HudLine::Resources,
            ));
            parent.spawn((
                Text::new(""),
                TextFont::from_font_size(13.0),
                TextColor(Color::srgb(0.46, 0.65, 0.72)),
                HudLine::Realm,
            ));
            parent.spawn((
                Text::new("SELECTED PROVINCE"),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.91, 0.83, 0.61)),
            ));
            parent.spawn((
                Text::new(""),
                TextFont::from_font_size(13.0),
                TextColor(Color::srgb(0.74, 0.79, 0.78)),
                HudLine::Selected,
            ));

            parent.spawn((
                Text::new("ORDERS"),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.91, 0.83, 0.61)),
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(7),
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
                                height: px(40),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.22, 0.39, 0.42)),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("END TURN  [SPACE]"),
                                TextFont::from_font_size(14.0),
                                TextColor(Color::WHITE),
                            ));
                        });

                    buttons
                        .spawn((
                            Button,
                            Hovered::default(),
                            UiAction::Claim,
                            Node {
                                width: percent(100),
                                height: px(36),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.20, 0.31, 0.36)),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("CLAIM SELECTED LAND  [C]"),
                                TextFont::from_font_size(12.0),
                                TextColor(Color::WHITE),
                            ));
                        });

                    buttons
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(7),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Farm,
                                Node {
                                    width: px(140),
                                    height: px(34),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.25, 0.34, 0.24)),
                            ))
                            .with_children(|button| {
                                button.spawn((
                                    Text::new("FARM  [1]"),
                                    TextFont::from_font_size(12.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::LoggingCamp,
                                Node {
                                    width: px(140),
                                    height: px(34),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.19, 0.33, 0.27)),
                            ))
                            .with_children(|button| {
                                button.spawn((
                                    Text::new("CAMP  [2]"),
                                    TextFont::from_font_size(12.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });

                    buttons
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(7),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Quarry,
                                Node {
                                    width: px(140),
                                    height: px(34),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.30, 0.31, 0.34)),
                            ))
                            .with_children(|button| {
                                button.spawn((
                                    Text::new("QUARRY  [3]"),
                                    TextFont::from_font_size(12.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                            row.spawn((
                                Button,
                                Hovered::default(),
                                UiAction::Market,
                                Node {
                                    width: px(140),
                                    height: px(34),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.38, 0.29, 0.20)),
                            ))
                            .with_children(|button| {
                                button.spawn((
                                    Text::new("MARKET  [4]"),
                                    TextFont::from_font_size(12.0),
                                    TextColor(Color::WHITE),
                                ));
                            });
                        });
                });

            parent.spawn((
                Text::new("WORLD CHRONICLE"),
                TextFont::from_font_size(16.0),
                TextColor(Color::srgb(0.91, 0.83, 0.61)),
            ));
            parent.spawn((
                Text::new(""),
                TextFont::from_font_size(12.0),
                TextColor(Color::srgb(0.61, 0.71, 0.73)),
                HudLine::Chronicle,
            ));
        });

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(20),
            bottom: px(18),
            padding: UiRect::axes(px(14), px(8)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.035, 0.055, 0.08, 0.91)),
        Text::new("LEFT CLICK  SELECT     •     C  CLAIM     •     1–4  DEVELOP     •     SPACE  ADVANCE YEAR"),
        TextFont::from_font_size(12.0),
        TextColor(Color::srgb(0.58, 0.69, 0.73)),
    ));
}

fn handle_mouse_selection(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut world: ResMut<WorldState>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = *camera;
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    let x = ((world_position.x - MAP_CENTER.x) / TILE_STEP + (MAP_WIDTH - 1) as f32 / 2.0).round()
        as i32;
    let y = ((MAP_HEIGHT - 1) as f32 / 2.0 - (world_position.y - MAP_CENTER.y) / TILE_STEP).round()
        as i32;
    let point = IVec2::new(x, y);

    if let Some(tile) = world.tile(point).copied() {
        world.selected = point;
        let owner = tile
            .owner
            .map(|id| world.factions[id].name.clone())
            .unwrap_or_else(|| "Unclaimed land".to_owned());
        world.push_log(format!(
            "Selected {} at {}, {}.",
            tile.terrain.name(),
            x + 1,
            y + 1
        ));
        if tile.city {
            world.push_log(format!("{} is a capital city.", owner));
        }
    }
}

fn handle_keyboard(input: Res<ButtonInput<KeyCode>>, mut world: ResMut<WorldState>) {
    if input.just_pressed(KeyCode::Space) {
        advance_turn(&mut world);
    }
    if input.just_pressed(KeyCode::KeyC) {
        claim_selected(&mut world);
    }
    if input.just_pressed(KeyCode::Digit1) {
        build_selected(&mut world, Improvement::Farm);
    }
    if input.just_pressed(KeyCode::Digit2) {
        build_selected(&mut world, Improvement::LoggingCamp);
    }
    if input.just_pressed(KeyCode::Digit3) {
        build_selected(&mut world, Improvement::Quarry);
    }
    if input.just_pressed(KeyCode::Digit4) {
        build_selected(&mut world, Improvement::Market);
    }
}

fn handle_ui_activate(
    event: On<Activate>,
    mut world: ResMut<WorldState>,
    actions: Query<&UiAction>,
) {
    let Ok(action) = actions.get(event.entity) else {
        return;
    };

    match *action {
        UiAction::EndTurn => advance_turn(&mut world),
        UiAction::Claim => claim_selected(&mut world),
        UiAction::Farm => build_selected(&mut world, Improvement::Farm),
        UiAction::LoggingCamp => build_selected(&mut world, Improvement::LoggingCamp),
        UiAction::Quarry => build_selected(&mut world, Improvement::Quarry),
        UiAction::Market => build_selected(&mut world, Improvement::Market),
    }
}

fn advance_turn(world: &mut WorldState) {
    world.turn += 1;
    world.year += 1;

    for faction_id in 0..world.factions.len() {
        let mut food_production = 0.0;
        let mut wood_production = 0.0;
        let mut stone_production = 0.0;
        let mut gold_production = 0.0;

        for tile in world
            .tiles
            .iter()
            .filter(|tile| tile.owner == Some(faction_id))
        {
            let yields = tile_yields(tile);
            food_production += yields.0;
            wood_production += yields.1;
            stone_production += yields.2;
            gold_production += yields.3;
        }

        let city_count = world.city_count(faction_id) as f32;
        let faction = &mut world.factions[faction_id];

        faction.food += food_production - faction.population as f32 * 0.85;
        faction.wood += wood_production;
        faction.stone += stone_production;
        faction.gold += gold_production + 2.0 + city_count * 3.0;
        faction.influence += 4.0 + city_count;

        if faction.food >= 22.0 && world.turn % 3 == 0 {
            faction.population += 1;
            faction.food -= 5.0;
        } else if faction.food < 0.0 {
            faction.food = 0.0;
            faction.population = faction.population.saturating_sub(1).max(1);
        }
    }

    for faction_id in 1..world.factions.len() {
        let can_expand = {
            let faction = &world.factions[faction_id];
            faction.influence >= 9.0 && faction.gold >= 7.0
        };
        if can_expand {
            if let Some(point) = best_expansion(world, faction_id) {
                if let Some(index) = tile_index(point) {
                    world.tiles[index].owner = Some(faction_id);
                    let faction = &mut world.factions[faction_id];
                    faction.influence -= 9.0;
                    faction.gold -= 7.0;
                }
            }
        }
    }

    let player_tiles = world.owned_count(PLAYER_ID);
    world.push_log(format!(
        "Year {}: your realm controls {} provinces. The rival powers continue to expand.",
        world.year, player_tiles
    ));

    if world.turn % 4 == 0 {
        match (world.turn + world.seed) % 3 {
            0 => {
                world.factions[PLAYER_ID].food += 14.0;
                world.push_log("HARVEST FESTIVAL — a generous season adds 14 food.");
            }
            1 => {
                world.factions[PLAYER_ID].gold += 16.0;
                world.push_log("MERCHANT CARAVAN — traders bring 16 gold to your treasury.");
            }
            _ => {
                world.factions[PLAYER_ID].food = (world.factions[PLAYER_ID].food - 9.0).max(0.0);
                world.push_log("DRY SUMMER — the harvest suffers and 9 food is lost.");
            }
        }
    }
}

fn claim_selected(world: &mut WorldState) {
    let point = world.selected;
    let Some(index) = tile_index(point) else {
        return;
    };
    let tile = world.tiles[index];

    if !tile.terrain.is_land() {
        world.push_log("The sea cannot be claimed by a land charter.");
        return;
    }
    if tile.owner.is_some() {
        world.push_log("This province already belongs to a realm.");
        return;
    }
    if !has_adjacent_owner(world, point, PLAYER_ID) {
        world.push_log("Claiming requires a border connection to your realm.");
        return;
    }

    let faction = &mut world.factions[PLAYER_ID];
    if faction.influence < 9.0 || faction.gold < 7.0 {
        world.push_log("Not enough influence or gold to claim this province.");
        return;
    }

    faction.influence -= 9.0;
    faction.gold -= 7.0;
    world.tiles[index].owner = Some(PLAYER_ID);
    world.push_log(format!(
        "New province secured: {}. Your borders grow.",
        tile.terrain.name()
    ));
}

fn build_selected(world: &mut WorldState, improvement: Improvement) {
    let Some(index) = tile_index(world.selected) else {
        return;
    };
    let tile = world.tiles[index];

    if tile.owner != Some(PLAYER_ID) {
        world.push_log("You can only develop land owned by your realm.");
        return;
    }
    if tile.city {
        world.push_log("The capital already has its own administration.");
        return;
    }
    if tile.improvement != Improvement::None {
        world.push_log("This province already has an improvement.");
        return;
    }

    let faction = &mut world.factions[PLAYER_ID];
    let (wood_cost, stone_cost, gold_cost) = match improvement {
        Improvement::Farm => (10.0, 0.0, 2.0),
        Improvement::LoggingCamp => (8.0, 0.0, 3.0),
        Improvement::Quarry => (12.0, 4.0, 3.0),
        Improvement::Market => (10.0, 4.0, 5.0),
        Improvement::None => return,
    };

    if faction.wood < wood_cost || faction.stone < stone_cost || faction.gold < gold_cost {
        world.push_log("The treasury lacks the materials for that improvement.");
        return;
    }

    faction.wood -= wood_cost;
    faction.stone -= stone_cost;
    faction.gold -= gold_cost;
    world.tiles[index].improvement = improvement;
    world.push_log(format!(
        "{} established in the {} province.",
        improvement.name(),
        tile.terrain.name()
    ));
}

fn best_expansion(world: &WorldState, faction_id: usize) -> Option<IVec2> {
    let mut best: Option<(IVec2, f32)> = None;

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let point = IVec2::new(x, y);
            let Some(tile) = world.tile(point) else {
                continue;
            };
            if tile.owner.is_some() || !tile.terrain.is_land() {
                continue;
            }
            if !has_adjacent_owner(world, point, faction_id) {
                continue;
            }

            let variety =
                ((x * 17 + y * 31 + faction_id as i32 * 13).rem_euclid(11)) as f32 / 100.0;
            let score = tile.terrain.expansion_value() + variety;
            if best.is_none_or(|(_, best_score)| score > best_score) {
                best = Some((point, score));
            }
        }
    }

    best.map(|(point, _)| point)
}

fn has_adjacent_owner(world: &WorldState, point: IVec2, faction_id: usize) -> bool {
    [
        IVec2::new(1, 0),
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
        IVec2::new(0, -1),
    ]
    .into_iter()
    .any(|offset| {
        world
            .tile(point + offset)
            .is_some_and(|tile| tile.owner == Some(faction_id))
    })
}

fn tile_yields(tile: &WorldTile) -> (f32, f32, f32, f32) {
    let (mut food, mut wood, mut stone, mut gold) = match tile.terrain {
        Terrain::Water => (0.0, 0.0, 0.0, 0.0),
        Terrain::Grassland => (2.0, 0.0, 0.0, 0.0),
        Terrain::Plains => (2.0, 0.0, 0.0, 0.0),
        Terrain::Forest => (1.0, 2.0, 0.0, 0.0),
        Terrain::Desert => (0.0, 0.0, 0.0, 1.0),
        Terrain::Mountain => (0.0, 0.0, 2.0, 0.0),
    };

    match tile.improvement {
        Improvement::None => {}
        Improvement::Farm => food += 3.0,
        Improvement::LoggingCamp => wood += 3.0,
        Improvement::Quarry => stone += 3.0,
        Improvement::Market => gold += 3.0,
    }

    (food, wood, stone, gold)
}

fn update_tile_visuals(world: Res<WorldState>, mut visuals: Query<(&TileVisual, &mut Sprite)>) {
    if !world.is_changed() {
        return;
    }

    for (visual, mut sprite) in &mut visuals {
        if let Some(tile) = world.tile(visual.0) {
            let mut rgb = tile.terrain.base_color();
            if let Some(owner) = tile.owner {
                let faction = world.factions[owner].color;
                rgb = [
                    rgb[0] * 0.48 + faction[0] * 0.52,
                    rgb[1] * 0.48 + faction[1] * 0.52,
                    rgb[2] * 0.48 + faction[2] * 0.52,
                ];
            }
            sprite.color = Color::srgb(rgb[0], rgb[1], rgb[2]);
        }
    }
}

fn update_selection_highlight(
    world: Res<WorldState>,
    mut highlight: Single<&mut Transform, With<SelectionHighlight>>,
) {
    if world.is_changed() {
        highlight.translation = tile_position(world.selected) + Vec3::new(0.0, 0.0, 2.0);
    }
}

fn update_hud(world: Res<WorldState>, mut labels: Query<(&mut Text, &HudLine)>) {
    if !world.is_changed() {
        return;
    }

    let player = &world.factions[PLAYER_ID];
    for (mut text, line) in &mut labels {
        **text = match line {
            HudLine::Header => format!(
                "THE LIVING WORLD  •  YEAR {}  •  TURN {}",
                world.year, world.turn
            ),
            HudLine::Resources => format!(
                "FOOD       {:>5.0}\nWOOD       {:>5.0}\nSTONE      {:>5.0}\nGOLD       {:>5.0}\nINFLUENCE  {:>5.0}",
                player.food, player.wood, player.stone, player.gold, player.influence
            ),
            HudLine::Realm => format!(
                "POPULATION   {}\nPROVINCES    {}\nCITIES       {}\n\nAurelian League",
                player.population,
                world.owned_count(PLAYER_ID),
                world.city_count(PLAYER_ID)
            ),
            HudLine::Selected => {
                if let Some(tile) = world.tile(world.selected) {
                    let owner = tile
                        .owner
                        .map(|id| world.factions[id].name.as_str())
                        .unwrap_or("Unclaimed");
                    format!(
                        "{}\nOwner: {}\nImprovement: {}\nPosition: {}, {}",
                        tile.terrain.name(),
                        owner,
                        tile.improvement.name(),
                        world.selected.x + 1,
                        world.selected.y + 1
                    )
                } else {
                    "No province selected".to_owned()
                }
            }
            HudLine::Chronicle => {
                let mut output = String::new();
                for entry in world.chronicle.iter().rev().take(5).rev() {
                    output.push_str("• ");
                    output.push_str(entry);
                    output.push('\n');
                }
                output
            }
        };
    }
}

fn tile_position(point: IVec2) -> Vec3 {
    let x = MAP_CENTER.x + (point.x as f32 - (MAP_WIDTH - 1) as f32 / 2.0) * TILE_STEP;
    let y = MAP_CENTER.y + ((MAP_HEIGHT - 1) as f32 / 2.0 - point.y as f32) * TILE_STEP;
    Vec3::new(x, y, 0.0)
}

fn tile_index(point: IVec2) -> Option<usize> {
    if point.x < 0 || point.y < 0 || point.x >= MAP_WIDTH || point.y >= MAP_HEIGHT {
        None
    } else {
        Some((point.y * MAP_WIDTH + point.x) as usize)
    }
}

fn nearest_land(tiles: &[WorldTile], target: IVec2, existing: &[IVec2]) -> IVec2 {
    let mut best = target;
    let mut best_score = f32::MAX;

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let point = IVec2::new(x, y);
            let Some(index) = tile_index(point) else {
                continue;
            };
            if !tiles[index].terrain.is_land() {
                continue;
            }

            let mut score = point.distance_squared(target) as f32;
            for other in existing {
                let distance_squared = point.distance_squared(*other) as f32;
                if distance_squared < 49.0 {
                    score += (7.0 - distance_squared.sqrt()) * 100.0;
                }
            }
            if score < best_score {
                best_score = score;
                best = point;
            }
        }
    }

    best
}

fn smooth_noise(x: f32, y: f32, seed: u32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let tx = x - x.floor();
    let ty = y - y.floor();
    let sx = tx * tx * (3.0 - 2.0 * tx);
    let sy = ty * ty * (3.0 - 2.0 * ty);

    let a = hash_value(x0, y0, seed);
    let b = hash_value(x0 + 1, y0, seed);
    let c = hash_value(x0, y0 + 1, seed);
    let d = hash_value(x0 + 1, y0 + 1, seed);

    let top = a + (b - a) * sx;
    let bottom = c + (d - c) * sx;
    top + (bottom - top) * sy
}

fn hash_value(x: i32, y: i32, seed: u32) -> f32 {
    let value =
        ((x as f32 * 127.1 + y as f32 * 311.7 + seed as f32 * 74.7).sin() * 43_758.547).fract();
    value.abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_has_expected_dimensions_and_realms() {
        let world = WorldState::generate();
        assert_eq!(world.tiles.len(), (MAP_WIDTH * MAP_HEIGHT) as usize);
        assert_eq!(world.factions.len(), 5);
    }

    #[test]
    fn every_realm_starts_with_a_capital() {
        let world = WorldState::generate();
        for faction_id in 0..world.factions.len() {
            let capital = world.tile(world.factions[faction_id].capital).unwrap();
            assert!(capital.city);
            assert_eq!(capital.owner, Some(faction_id));
        }
    }

    #[test]
    fn map_contains_land_and_water() {
        let world = WorldState::generate();
        assert!(world.tiles.iter().any(|tile| tile.terrain.is_land()));
        assert!(
            world
                .tiles
                .iter()
                .any(|tile| tile.terrain == Terrain::Water)
        );
    }

    #[test]
    fn starting_realms_have_claimed_provinces() {
        let world = WorldState::generate();
        for faction_id in 0..world.factions.len() {
            assert!(world.owned_count(faction_id) >= 1);
        }
    }

    #[test]
    fn improvements_produce_their_resource() {
        let mut tile = WorldTile::new(Terrain::Grassland);
        tile.improvement = Improvement::Farm;
        assert_eq!(tile_yields(&tile).0, 5.0);

        tile.improvement = Improvement::Market;
        assert_eq!(tile_yields(&tile).3, 3.0);
    }
}
