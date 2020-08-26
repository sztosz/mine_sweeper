use bevy::prelude::*;
use bevy::render::camera::Camera;

// const WIDTH: f32 = 40.0;
// const HEIGHT: f32 = 40.0;

fn main() {
    App::build()
        .add_default_plugins()
        .init_resource::<State>()
        .add_startup_system(setup.system())
        .add_resource(Mouse {
            pos: Vec2::new(0.0, 0.0),
            world_pos: Vec2::new(0.0, 0.0),
            clicked: false,
        })
        .add_system(mouse_movement_updating_system.system())
        .add_system(click_system.system())
        .run();
}

struct MineField;

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dComponents::default());

    let field_rows = 10;
    let field_columns = 10;
    let field_spacing = 1.0;
    let field_size = Vec2::new(40.0, 40.0);
    let field_width = field_columns as f32 * (field_size.x() + field_spacing) - field_spacing;
    let field_height = field_columns as f32 * (field_size.y() + field_spacing) - field_spacing;
    let field_offset = Vec3::new(
        -(field_width - field_size.x()) / 2.0,
        -(field_height - field_size.x()) / 2.0,
        0.0,
    );
    for row in 0..field_rows {
        let y_position = row as f32 * (field_size.y() + field_spacing);
        for column in 0..field_columns {
            let field_position = Vec3::new(
                column as f32 * (field_size.x() + field_spacing),
                y_position,
                0.0,
            ) + field_offset;

            commands
                .spawn(SpriteComponents {
                    material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
                    sprite: Sprite { size: field_size },
                    translation: Translation(field_position),
                    ..Default::default()
                })
                .with(MineField);
        }
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
    mouse: ResMut<Mouse>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut minefield_query: Query<(
        &MineField,
        &Translation,
        &Sprite,
        &mut Handle<ColorMaterial>,
    )>,
) {
    if mouse.clicked {
        for (mf, translation, sprite, color) in &mut minefield_query.iter() {
            let position = translation.truncate();
            let extents = sprite.size / 2.0;
            let min = position - extents;
            let max = position + extents;
            if (min.x()..max.x()).contains(&mouse.world_pos.x())
                && (min.y()..max.y()).contains(&mouse.world_pos.y())
            {
                if let Some(color_material) = materials.get_mut(&color) {
                    color_material.color = Color::rgb(1.0, 1.0, 1.0);
                }
            }
        }
    }
}
