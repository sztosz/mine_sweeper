use bevy::prelude::*;
use bevy::render::camera::Camera;
use rand::Rng;
// const WIDTH: f32 = 40.0;
// const HEIGHT: f32 = 40.0;
const BOARD_SIZE: usize = 10;

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "Mine Sweeper".to_string(),
            width: 600,
            height: 600,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_default_plugins()
        .init_resource::<State>()
        .add_resource(create_board())
        .add_resource(Mouse {
            pos: Vec2::new(0.0, 0.0),
            world_pos: Vec2::new(0.0, 0.0),
            clicked: false,
        })
        .add_startup_system(setup.system())
        .add_system(mouse_movement_updating_system.system())
        .add_system(click_system.system())
        .run();
}

struct Discovered(bool);
struct Interactable(bool);

struct X(usize);
struct Y(usize);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    board: Res<Board>,
) {
    commands.spawn(Camera2dComponents::default());

    asset_server.load_asset_folder("assets/sprite").unwrap();
    let tile_handle = asset_server.get_handle("assets/sprite/tile.png").unwrap();
    let material = materials.add(ColorMaterial {
        texture: Some(tile_handle),
        color: Color::default(),
    });
    let field_spacing = 1.0;
    let field_size = Vec2::new(40.0, 40.0);
    let field_width = BOARD_SIZE as f32 * (field_size.x() + field_spacing) - field_spacing;
    let field_height = BOARD_SIZE as f32 * (field_size.y() + field_spacing) - field_spacing;
    let field_offset = Vec3::new(
        -(field_width - field_size.x()) / 2.0,
        -(field_height - field_size.x()) / 2.0,
        0.0,
    );
    for (pos, tile) in board.revealed.iter().enumerate() {
        let x_pos = (pos / BOARD_SIZE) as f32 * (field_size.y() + field_spacing);
        let y_pos = (pos % BOARD_SIZE) as f32 * (field_size.y() + field_spacing);
        let x = pos / BOARD_SIZE;
        let y = pos % BOARD_SIZE;
        let translation = Translation(Vec3::new(x_pos, y_pos, 0.0) + field_offset);
        commands
            .spawn(SpriteComponents {
                material,
                sprite: Sprite { size: field_size },
                translation,
                ..Default::default()
            })
            .with(Discovered(false))
            .with(Interactable(true))
            .with(X(x))
            .with(Y(y));
    }
}

#[derive(Default)]
struct State {
    cursor_moved_event_reader: EventReader<CursorMoved>,
}
struct Mouse {
    pos: Vec2,
    world_pos: Vec2,
    clicked: bool,
}

fn mouse_movement_updating_system(
    mut state: ResMut<State>,
    mut mouse: ResMut<Mouse>,
    windows: Res<Windows>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut query: Query<(&Camera, &Translation, &Scale)>,
) {
    if let Some(cursor_moved) = state.cursor_moved_event_reader.latest(&cursor_moved_events) {
        mouse.pos = cursor_moved.position;
    }
    if let Some(window) = windows.get_primary() {
        for (_camera_2d, translation, scale) in &mut query.iter() {
            let cursor_x =
                translation.0.x() + (mouse.pos[0] - (window.width as f32 * 0.5)) * scale.0;
            let cursor_y =
                translation.0.y() + (mouse.pos[1] - (window.height as f32 * 0.5)) * scale.0;
            mouse.world_pos = [cursor_x, cursor_y].into();
        }
    }
    mouse.clicked = mouse_button_input.just_pressed(MouseButton::Left);
}

fn click_system(
    asset_server: Res<AssetServer>,
    mouse: ResMut<Mouse>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut tile_query: Query<(
        &Interactable,
        &Translation,
        &Sprite,
        &mut Handle<ColorMaterial>,
    )>,
) {
    if mouse.clicked {
        for (_, translation, sprite, color_material) in &mut tile_query.iter() {
            let position = translation.truncate();
            let extents = sprite.size / 2.0;
            let min = position - extents;
            let max = position + extents;
            if (min.x()..max.x()).contains(&mouse.world_pos.x())
                && (min.y()..max.y()).contains(&mouse.world_pos.y())
            {
                let mine_handle = asset_server.get_handle("assets/sprite/mine.png").unwrap();
                if let Some(color_material) = materials.get_mut(&color_material) {
                    color_material.texture = Some(mine_handle);
                }
            }
        }
    }
}

#[derive(Default, Debug)]
struct Board {
    mines: Vec<bool>,
    revealed: Vec<bool>,
    nearby_mines: Vec<usize>,
}

fn create_board() -> Board {
    let mut rng = rand::thread_rng();
    let mut mines = vec![false; BOARD_SIZE * BOARD_SIZE];
    let mut nearby_mines = vec![0; BOARD_SIZE * BOARD_SIZE];
    let mut numbers: Vec<usize> = Vec::new();
    let mut generated_count = 0;
    loop {
        let r = rng.gen_range(0, BOARD_SIZE * BOARD_SIZE);
        if !numbers.contains(&r) {
            numbers.push(r);
            if let Some(elem) = mines.get_mut(r) {
                *elem = true;
            }
            if let Some(elem) = nearby_mines.get_mut(r) {
                *elem += 1;
            }

            generated_count += 1;
        }
        if generated_count >= BOARD_SIZE {
            break;
        }
    }
    let revealed = vec![false; BOARD_SIZE * BOARD_SIZE];
    Board {
        mines,
        revealed,
        nearby_mines,
    }
}
