pub const ARROW_THICKNESS:f32 = 1.25;
pub const ARROW_TRILEN:f32 = 3.33;

use std::f32::consts::PI;

use bevy::prelude::*;

use bevy_dragndrop::*;
use bevy_prototype_lyon::shapes::Circle;

use bevy_prototype_lyon::prelude::*;

use super::{TileEntity, BoardEntity};

#[derive(Component)]
pub struct ArrowDraggable{
    pub parent: Entity,
    pub drawn: Option<Entity>,
    pub color: Option<Color>,
}

#[derive(Component)]
pub struct ArrowDrawn{
    from: Entity,
}

pub fn handle_arrow_dropped(
    commands: &mut Commands,
    event: &Dropped,
    transforms: &mut Query<&mut Transform, With<Draggable>>,
    q_arrow: &Query<&ArrowDraggable>,
) -> bool {
    if let Ok(_arrow) = q_arrow.get(event.dropped) {
        if let Some(received) = event.received {
            commands.entity(event.dropped).remove_parent();
            commands.entity(received).add_child(event.dropped);
            let mut transform = transforms.get_mut(event.dropped).unwrap();
            transform.translation = Vec3::new(0.0, 0.0, 2.0);
            commands.entity(event.dropped).remove::<Draggable>();
            return true; 
        }
        commands.entity(event.dropped).remove::<Draggable>();
        //commands.entity(arrow.parent).remove_children(&[event.dropped]);
        //commands.entity(event.dropped).despawn();
        return true;
    }
    return false;
}


pub fn handle_arrow_dragged(
    commands: &mut Commands,
    event: &Dragged,
    q_arrow: &Query<&ArrowDraggable>,
    parents: &Query<&Parent>,
) {
    if let Ok(_) = q_arrow.get(event.dragged) {
        let parent = parents.get(event.dragged).unwrap().get();
        commands.entity(parent).with_children(|parent| {
            parent.spawn((SpriteBundle{
                transform: Transform {
                    translation: Vec3::new(0.0,0.0,1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: Color::rgba(0.0,0.0,0.0,0.0),
                    ..default()
                },
                ..default()
            }, Draggable {
                required: InputFlags::RightClick,
                disallowed: InputFlags::MiddleClick | InputFlags::LeftClick,
                minimum_held: Some(0.05)
            }, ArrowDraggable{parent: parent.parent_entity(), drawn: None, color: None}));
        });
    }
}

pub fn draw_arrows(
    mut commands: Commands,
    transforms: Query<&Transform>,
    mut q_arrow: Query<(&mut ArrowDraggable, &Parent, Entity)>,
    q_tiles: Query<&TileEntity>,
    q_draggable: Query<&Draggable>,
    q_board: Query<Entity, With<BoardEntity>>,
) {
    if q_arrow.is_empty() {return;}
    let board = q_board.single();
    for (mut arrow,parent, entity) in q_arrow.iter_mut() {
        let parent = parent.get();
        if let None = arrow.drawn {
            if parent != arrow.parent {
                let p1 = transforms.get(arrow.parent).unwrap().translation.xy();
                let p1_tile = q_tiles.get(arrow.parent).unwrap();
                let p2 = transforms.get(parent).unwrap().translation.xy();
                let p2_tile = q_tiles.get(parent).unwrap();

                let offset = p1_tile.index_x + (p1_tile.index_y << 3) + (p2_tile.index_x << 6) + (p2_tile.index_y << 9);
                let offset_float = (offset as f32) / ((1 << 11) as f32);
                println!("{}",offset_float);

                arrow.drawn = Some(draw_arrow(&mut commands, p1, p2, board, entity, arrow.color, offset_float));
            } else if let Err(_) = q_draggable.get(entity) {
                let p1 = transforms.get(arrow.parent).unwrap().translation.xy();
                arrow.drawn = Some(draw_circle(&mut commands, p1, board, entity, arrow.color));
            }
        }
    }
}

fn draw_circle(
    commands: &mut Commands,
    p1: Vec2,
    board: Entity,
    arrow: Entity,
    color: Option<Color>,
) -> Entity {
    let color = color.unwrap_or(Color::rgba(0.0,1.0,0.0,0.7));

    let mut ent = board;

    let circle = Circle {radius: 29.0, center: Vec2::ZERO};
    

    commands.entity(board).with_children(|parent| {
        let p = parent.spawn((ShapeBundle {
            path: GeometryBuilder::build_as(&circle),
            spatial: SpatialBundle{
                transform: Transform {
                    translation: Vec3::new(p1.x,p1.y,7.),
                    scale: Vec3::new(0.001953125, 0.001953125, 1.),
                    ..default()
                },
                ..default()
            },
            ..default()
        },
        Stroke::new(color,5.5),
        ArrowDrawn {from: arrow}));
        ent = p.id();
    });

    return ent;
}

fn draw_arrow(
    commands: &mut Commands,
    p1: Vec2,
    p2: Vec2,
    board: Entity,
    arrow: Entity,
    color: Option<Color>,
    z_offset: f32,
) -> Entity {
    let dif = p2 - p1;
    let dist = (dif.x * dif.x + dif.y * dif.y).sqrt() * 100.;
    let angle = dif.y.atan2(dif.x);
    let mut ent = board;
    let color = color.unwrap_or(Color::rgba(0.0,1.0,0.0,0.7));

    let mut path_builder = PathBuilder::new();
    path_builder.move_to(Vec2::new(0.,0.));
    path_builder.cubic_bezier_to(Vec2::new(ARROW_THICKNESS, 0.), Vec2::new(ARROW_THICKNESS, ARROW_THICKNESS), Vec2::new(ARROW_THICKNESS,ARROW_THICKNESS));
    //path_builder.arc(Vec2::new(0.,ARROW_THICKNESS), Vec2::new(2.*ARROW_THICKNESS, 2.*ARROW_THICKNESS), PI/2., 0.);
    path_builder.line_to(Vec2::new(ARROW_THICKNESS, dist - ARROW_TRILEN));
    path_builder.line_to(Vec2::new(2.*ARROW_THICKNESS, dist - ARROW_TRILEN));
    path_builder.line_to(Vec2::new(0.,dist));
    path_builder.line_to(Vec2::new(-2.*ARROW_THICKNESS, dist - ARROW_TRILEN));
    path_builder.line_to(Vec2::new(-ARROW_THICKNESS, dist - ARROW_TRILEN));
    path_builder.line_to(Vec2::new(-ARROW_THICKNESS, ARROW_THICKNESS));
    path_builder.cubic_bezier_to(Vec2::new(-ARROW_THICKNESS, ARROW_THICKNESS), Vec2::new(-ARROW_THICKNESS, 0.), Vec2::new(0.,0.));
    path_builder.close();
    let path = path_builder.build();

    //let z_offset = ;

    commands.entity(board).with_children(|parent|{
        let p = parent.spawn(( ShapeBundle{
            path,
            spatial: SpatialBundle {
                transform: Transform {
                    translation: Vec3::new(p1.x,p1.y,5. + z_offset),
                    rotation: Quat::from_rotation_z(angle - PI/2.),
                    scale: Vec3::new(0.01,0.01,1.)
                },
                ..default()
            },
            ..default()
        },
        Fill::color(color),
        ArrowDrawn {
            from: arrow,
        })
    );
        ent = p.id();
    });

    //commands.entity(arrow).despawn();
    return ent;
}



pub fn handle_arrow_hovered(mut commands: &mut Commands,
    event: &HoveredChange,
    transforms: &Query<&Transform>,
    q_tiles: &Query<&TileEntity>,
    q_arrow: &mut Query<(&mut ArrowDraggable, Entity)>,
    board: Entity,
) {
    if let Ok((mut arrow, ent)) = q_arrow.get_mut(event.hovered) {
        if let None = arrow.color {
            arrow.color = Some(color_from_inputs(event.inputs));
        }
        if let Some(drawn) = arrow.drawn {
            if let Some(mut drawn_ent) = commands.get_entity(drawn) {
                drawn_ent.remove_parent();
                drawn_ent.despawn_recursive();
            }
            
            arrow.drawn = None;
        }
        if let Some(over) = event.receiver {
            if over != arrow.parent {
                let p1 = transforms.get(arrow.parent).unwrap().translation.xy();
                let p1_tile = q_tiles.get(arrow.parent).unwrap();
                let p2 = transforms.get(over).unwrap().translation.xy();
                let p2_tile = q_tiles.get(over).unwrap();

                let offset = p1_tile.index_x + (p1_tile.index_y << 3) + (p2_tile.index_x << 6) + (p2_tile.index_y << 9);
                let offset_float = (offset as f32) / ((1 << 11) as f32);

                arrow.drawn = Some(draw_arrow(&mut commands, p1, p2, board, ent, arrow.color, offset_float));
            }
        }
    }
}

pub fn clear_arrows(
    mut commands: Commands,
    button: Res<Input<MouseButton>>,
    mut q_arrow: Query<(&mut ArrowDraggable, Entity, &Parent)>,
    q_drawn: Query<(&ArrowDrawn, Entity)>,
) {
    if button.just_pressed(MouseButton::Left) && !button.pressed(MouseButton::Right) {
        for (arrow, entity, parent) in q_arrow.iter_mut() {
            if let Some(drawn) = arrow.drawn {
                commands.entity(drawn).remove_parent();
                commands.entity(drawn).despawn_recursive();
                commands.entity(entity).remove_parent().despawn();
                continue;
            }
            let parent = parent.get();
            if parent != arrow.parent {
                commands.entity(entity).remove_parent().despawn();
            }
        }
    }
    for (drawn, ent) in q_drawn.iter() {
        if commands.get_entity(drawn.from).is_none() {
            commands.entity(ent).despawn_recursive();
        }
    }
}

fn color_from_inputs(inputs: InputFlags) -> Color {
    if !(inputs.intersects(InputFlags::Modifiers)) {Color::rgba(0.0,1.0,0.0,0.85)}
    else if inputs.contains(InputFlags::Shift | InputFlags::Alt) {Color::rgba(1.0,1.0,0.0,0.85)}
    else if inputs.contains(InputFlags::Shift) {Color::rgba(1.0,0.0,0.0,0.85)}
    else if inputs.contains(InputFlags::Alt) {Color::rgba(0.0,0.0,1.0,0.85)}
    else {Color::rgba(1.0,0.0,0.0,0.7)}
}