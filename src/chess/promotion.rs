use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{PieceTextureHolder,BoardEntity,BoardChange};

use super::PIECE_SCALE;
use super::board::{Piece, PieceColor, PieceType};

pub struct PromotionPlugin;

impl Plugin for PromotionPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, (promotion_choice, promotion_choose))
        .add_event::<PromotionChoiceEvent>()
        .add_event::<PromotionChosenEvent>();
    }
}

#[derive(Component)]
pub struct PromotionBackground;

#[derive(Component)]
pub struct PromotionOption{
    piece: Piece,
    from: (usize, usize),
    to: (usize, usize)
}

#[derive(Event)]
pub struct PromotionChoiceEvent{
    pub color: PieceColor,
    pub from: (usize, usize),
    pub to: (usize, usize),
}

#[derive(Event)]
pub struct PromotionChosenEvent{
    pub piece: Piece,
    pub from: (usize, usize),
    pub to: (usize, usize),
}

pub fn promotion_choice(
    mut commands: Commands,
    mut er_promotion: EventReader<PromotionChoiceEvent>,
    texture_holder: Res<PieceTextureHolder>,
    board_ent: Query<(&BoardEntity, Entity)>,
    asset_server: Res<AssetServer>,
) {
    if let Some(event) = er_promotion.iter().last() {
        let (board_ent, board_entity) = board_ent.get_single().unwrap();
        let textures = match &texture_holder.textures {
            Some(x) => x,
            None => return,
        };
        let piece_types = [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight];
        commands.entity(board_entity).with_children(|parent| {
            parent.spawn(SpriteBundle{
                transform: Transform {
                    scale: Vec3::new(1.0,1.0,1.0),
                    translation: Vec3::new(0.0,0.0,5.0),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgba(0.0,0.0,0.0,0.45),
                    ..default()
                },
                ..default()
            })
            .insert(PromotionBackground);
        });
        let circle: Handle<Image> = asset_server.load("textures/circle.png");
        for i in 0..4 {
            let piece = Piece{
                piece: piece_types[i],
                color: event.color,
            };
            commands.entity(board_ent.tiles[event.to.0][event.to.1]).with_children(|parent| {
                parent.spawn(SpriteBundle{
                    transform: Transform {
                        scale: Vec3::new(PIECE_SCALE,PIECE_SCALE,1.0),
                        translation: Vec3::new(0.0,0.0 - (i as f32),5.0),
                        ..default()
                    },
                    texture: circle.clone(),
                    ..default()
                }).insert(PromotionOption{
                    piece,
                    from: event.from,
                    to: event.to,
                }).with_children(|parent| {
                    parent.spawn(SpriteBundle{
                        transform: Transform {
                            scale: Vec3::new(1.0,1.0,1.0),
                            translation: Vec3::new(0.0,0.0,1.0),
                            ..default()
                        },
                        texture: textures.get_texture(piece),
                        ..default()
                    });
                });
            });
        }
    }
}

pub fn promotion_choose (
    mut commands: Commands,
    q_option: Query<(&PromotionOption, &GlobalTransform, &Handle<Image>, Entity)>,
    q_background: Query<Entity, With<PromotionBackground>>,
    buttons: Res<Input<MouseButton>>, 
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    assets: Res<Assets<Image>>,
    mut ew_chosen: EventWriter<PromotionChosenEvent>,
    mut ew_board: EventWriter<BoardChange>,
) {
    let window = q_windows.single();
    let (camera, camera_transform) = q_camera.single();
    let mut option_chosen = false;
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(world_position) = window.cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            for (option, gtransform, image_handle, entity) in q_option.iter() {
                let transform = gtransform.compute_transform();
                let image_dimensions = assets.get(image_handle).unwrap().size();
                let scaled_image_dimension = image_dimensions * transform.scale.truncate();
                let bounding_box = Rect::from_center_size(gtransform.translation().truncate(), scaled_image_dimension);
                
                if bounding_box.contains(world_position) {
                    ew_chosen.send(PromotionChosenEvent { piece: option.piece, from: option.from, to: option.to });
                    option_chosen = true;
                }
                commands.entity(entity).despawn_recursive();
            }
            for entity in q_background.iter(){
                commands.entity(entity).despawn();
            }
            if !option_chosen && !q_option.is_empty() {
                ew_board.send(BoardChange);
            }
        }
    }
}