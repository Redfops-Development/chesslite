pub const BLACK_TILE:Color = Color::Rgba{red: 0.462745, green: 0.588235, blue:0.337254, alpha:1.0};
pub const WHITE_TILE:Color = Color::Rgba{red: 0.933333, green: 0.933333, blue:0.823529, alpha:1.0};
pub const PIECE_SCALE:f32 = 0.0078125;

pub mod board;

pub mod promotion;

use bevy::prelude::*;

use bevy_dragndrop::*;

use crate::chess::promotion::{PromotionChoiceEvent,PromotionChosenEvent,PromotionPlugin};

use crate::chess::board::{Board,Piece,PieceColor,PieceType,GameOverState};

#[derive(Component)]
pub struct ArrowDraggable{
    parent: Entity,
    drawn: Option<Entity>,
    color: Option<Color>,
}

#[derive(Component)]
pub struct ArrowDrawn{
    from: Entity,
}

pub struct ChessPluginClient;

impl Plugin for ChessPluginClient {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(DragPlugin)
        .add_plugins(PromotionPlugin)
        .insert_resource(Board::new())
        .insert_resource(PieceTextureHolder{textures: None})
        .add_event::<BoardChange>()
        .add_systems(Startup, setup_client)
        .add_systems(Update, (updateboardstate,on_dropped,on_dragged,draw_arrows.after(on_hovered),on_hovered,clear_arrows.after(on_dropped)));
    }
}

fn setup_client(mut commands: Commands, asset_server: Res<AssetServer>, mut texture_holder: ResMut<PieceTextureHolder>, mut ev_board: EventWriter<BoardChange>){
    texture_holder.textures = Some(PieceTextures { 
        black_king: asset_server.load("textures/pieces/black/king.png"),
        black_queen: asset_server.load("textures/pieces/black/queen.png"), 
        black_rook: asset_server.load("textures/pieces/black/rook.png"), 
        black_bishop: asset_server.load("textures/pieces/black/bishop.png"), 
        black_knight: asset_server.load("textures/pieces/black/knight.png"), 
        black_pawn: asset_server.load("textures/pieces/black/pawn.png"), 
        white_king: asset_server.load("textures/pieces/white/king.png"), 
        white_queen: asset_server.load("textures/pieces/white/queen.png"), 
        white_rook: asset_server.load("textures/pieces/white/rook.png"), 
        white_bishop: asset_server.load("textures/pieces/white/bishop.png"), 
        white_knight: asset_server.load("textures/pieces/white/knight.png"), 
        white_pawn: asset_server.load("textures/pieces/white/pawn.png") 
    });


    commands.spawn(Camera2dBundle::default());

    let mut tiles: Vec<[Entity;8]> = Vec::new();
    commands
    .spawn(SpriteBundle{
        transform: Transform {
            scale: Vec3::new(100.0,100.0,1.0),
            translation: Vec3::new(0.0,0.0,0.0),
            ..default()
        },
        ..default()
    })
    .with_children(|parent| {
        for y in 0..8 {
            let mut file: Vec<Entity> = Vec::new();
            for x in 0..8 {
                let tile_num = y + x;
                let col = if tile_num % 2 == 0 {BLACK_TILE} else {WHITE_TILE};
                let x_coord: f32 = (x as f32 * 0.125) - 0.4375;
                let y_coord: f32 = (y as f32 * 0.125) - 0.4375;
                let child = parent.spawn(SpriteBundle{
                    transform: Transform {
                        scale: Vec3::new(0.125,0.125,1.0),
                        translation: Vec3::new(x_coord,y_coord,1.0),
                        ..default()
                    },
                    sprite: Sprite{
                        color: col,
                        ..default()
                    },
                    ..default()
                })
                .insert(TileEntity{index_x: y, index_y: x})
                .insert(Receiver)
                .id();
                file.push(child);
            }
            tiles.push(file.try_into().expect("Trying to convert vec to array"));
        }

        let font = asset_server.load("fonts/FiraSans-Bold.ttf");
        let text_style = TextStyle {
            font: font.clone(),
            font_size: 60.0,
            color: Color::WHITE,
        };
        let text_alignment = TextAlignment::Center;

        for x in 0..8 {
            let x_coord = (x as f32 * 0.125) - 0.4375;
            let y_coord = -0.55;
            parent.spawn(Text2dBundle {
                text: Text::from_section((97u8 + x) as char, text_style.clone()).with_alignment(text_alignment),
                transform: Transform { 
                    translation: Vec3::new(x_coord, y_coord,3.0),
                    scale: Vec3::new(0.001,0.001,1.0),
                    ..default()
                },
                ..default()
            }
            );
            parent.spawn(Text2dBundle {
                text: Text::from_section((49u8 + x) as char, text_style.clone()).with_alignment(text_alignment),
                transform: Transform { 
                    translation: Vec3::new(y_coord, x_coord,3.0),
                    scale: Vec3::new(0.001,0.001,1.0),
                    ..default()
                },
                ..default()
            }
            );
        }
    })
    .insert(BoardEntity{tiles: tiles.try_into().expect("Should initialize with correct size")});

    ev_board.send(BoardChange);
}


#[derive(Event)]
pub struct BoardChange;

#[derive(Component)]
pub struct BoardEntity{
    tiles: [[Entity;8];8],
}

#[derive(Component)]
pub struct TileEntity{
    index_x: usize,
    index_y: usize,
}

#[derive(Component)]
pub struct PieceEntity{
    piece: Piece,
}


#[derive(Resource)]
pub struct PieceTextureHolder{
    textures: Option<PieceTextures>,
}

pub struct PieceTextures{
    black_king: Handle<Image>,
    black_queen: Handle<Image>,
    black_rook: Handle<Image>,
    black_bishop: Handle<Image>,
    black_knight: Handle<Image>,
    black_pawn: Handle<Image>,
    white_king: Handle<Image>,
    white_queen: Handle<Image>,
    white_rook: Handle<Image>,
    white_bishop: Handle<Image>,
    white_knight: Handle<Image>,
    white_pawn: Handle<Image>,
}

impl PieceTextures{
    pub fn get_texture(&self, piece: Piece) -> Handle<Image>{
        match piece.color{
            PieceColor::Black => {
                match piece.piece {
                    PieceType::King => self.black_king.clone(),
                    PieceType::Queen => self.black_queen.clone(),
                    PieceType::Rook => self.black_rook.clone(),
                    PieceType::Bishop => self.black_bishop.clone(),
                    PieceType::Knight => self.black_knight.clone(),
                    PieceType::Pawn => self.black_pawn.clone(),
                }
            },
            PieceColor::White => {
                match piece.piece {
                    PieceType::King => self.white_king.clone(),
                    PieceType::Queen => self.white_queen.clone(),
                    PieceType::Rook => self.white_rook.clone(),
                    PieceType::Bishop => self.white_bishop.clone(),
                    PieceType::Knight => self.white_knight.clone(),
                    PieceType::Pawn => self.white_pawn.clone(),
                }
            }
        }
    }
}

fn updateboardstate(
    mut commands: Commands, 
    board: Res<Board>, 
    board_ents: Query<&BoardEntity>, 
    texture_holder: Res<PieceTextureHolder>, 
    tile_query: Query<&Children,With<TileEntity>>, 
    mut er_board: EventReader<BoardChange>,
    asset_server: Res<AssetServer>
){
    let check_image: Handle<Image> = asset_server.load("textures/check.png");
    for _ in er_board.read() {
        let board_ent = match board_ents.iter().last() {
            Some(x) => x,
            None => return,
        };
        println!("Gamestate: {:?}", board.is_gameover());
        for x in 0..8 {
            for y in 0..8 {
                let tile_ent = board_ent.tiles[x][y];
                match tile_query.get(tile_ent) {
                    Ok(children) => {
                        for &child in children.iter(){
                            commands.entity(tile_ent).remove_children(&[child]);
                            commands.entity(child).despawn();
                        }
                    },
                    Err(_) => (),
                };
                
                let texture = match board.tiles[x][y] {
                    Some(piece) => {
                        match &texture_holder.textures {
                            Some(textures) => textures.get_texture(piece),
                            None => continue
                        }
                    }
                    None => {
                        commands.entity(tile_ent).with_children(|parent| {
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
                        continue;
                    }
                };
                commands.entity(tile_ent).with_children(|parent| {
                    parent.spawn(SpriteBundle{
                        transform: Transform {
                            scale: Vec3::new(PIECE_SCALE,PIECE_SCALE,1.0),
                            translation: Vec3::new(0.0,0.0,2.0),
                            ..default()
                        },
                        texture,
                        ..default()
                    })
                    .insert(Draggable {
                        required: InputFlags::LeftClick,
                        disallowed: InputFlags::RightClick
                            | InputFlags::MiddleClick,
                        minimum_held: Some(0.05),
                    })
                    .insert(PieceEntity{ piece:board.tiles[x][y].unwrap()});

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
        for color in [PieceColor::White,PieceColor::Black] {
            if board.is_check(color) {
                let king = board.king_coords(color);
                let tile_ent = board_ent.tiles[king.0][king.1];
                commands.entity(tile_ent).with_children(|parent| {
                    parent.spawn(SpriteBundle{
                        transform: Transform {
                            scale: Vec3::new(PIECE_SCALE,PIECE_SCALE,1.0),
                            translation: Vec3::new(0.0,0.0,1.0),
                            ..default()
                        },
                        texture:check_image.clone(),
                        ..default()
                    });
                });
            }
        }
    }
}

fn on_dropped(
    mut commands: Commands,
    mut ew_board: EventWriter<BoardChange>,
    mut er_drop: EventReader<Dropped>,
    mut board: ResMut<Board>,
    piece_ents: Query<(&PieceEntity,&Parent)>,
    mut transforms: Query<&mut Transform, With<Draggable>>,
    tile_ents: Query<&TileEntity>,
    mut ew_promotion: EventWriter<PromotionChoiceEvent>,
    mut er_promotion: EventReader<PromotionChosenEvent>,
    q_arrow: Query<&ArrowDraggable>,
) {
    let mut events = 0;
    for event in er_drop.read() {
        if handle_arrow_dropped(&mut commands, event, &mut transforms, &q_arrow) {continue;}
        if let Some(received) = event.received {
            if handle_piece_dropped(event, received, &mut board, &piece_ents,  &tile_ents, &mut ew_promotion) {events += 1;}

            

        } else {
            let mut transform = transforms.get_mut(event.dropped).unwrap();
            transform.translation = Vec3::new(0.0,0.0,2.0);
        }
    }
    for event in er_promotion.read() {
        board.make_move(event.from,event.to,Some(event.piece));

        events += 1;
    }
    if events > 0 {
        ew_board.send(BoardChange);
    }
}

fn handle_piece_dropped(
    event: &Dropped,
    received: Entity,
    board: &mut ResMut<Board>,
    piece_ents: &Query<(&PieceEntity,&Parent)>,
    tile_ents: &Query<&TileEntity>,
    ew_promotion: &mut EventWriter<PromotionChoiceEvent>,
) -> bool {
    let tile_ent = tile_ents.get(received).unwrap();
    let Ok((piece_ent, parent)) = piece_ents.get(event.dropped) else {
        return false;
    };
    let parent_tile = tile_ents.get(parent.get()).unwrap();
    /*if(board.is_legal_move((parent_tile.index_x,parent_tile.index_y),(tile_ent.index_x,tile_ent.index_y))) {
        board.tiles[parent_tile.index_x][parent_tile.index_y] = None;
        board.tiles[tile_ent.index_x][tile_ent.index_y] = Some(piece_ent.piece);
    }*/
    if board.can_promote((parent_tile.index_x,parent_tile.index_y), (tile_ent.index_x,tile_ent.index_y)) {
        ew_promotion.send(PromotionChoiceEvent { color: piece_ent.piece.color, from: (parent_tile.index_x,parent_tile.index_y), to: (tile_ent.index_x,tile_ent.index_y) })
    } else {
        board.make_move((parent_tile.index_x,parent_tile.index_y), (tile_ent.index_x,tile_ent.index_y), None);
        
        return true;
    }
    return false;
}

fn handle_arrow_dropped(
    commands: &mut Commands,
    event: &Dropped,
    transforms: &mut Query<&mut Transform, With<Draggable>>,
    q_arrow: &Query<&ArrowDraggable>,
) -> bool {
    if let Ok(arrow) = q_arrow.get(event.dropped) {
        if let Some(received) = event.received {
            commands.entity(event.dropped).remove_parent();
            commands.entity(received).add_child(event.dropped);
            let mut transform = transforms.get_mut(event.dropped).unwrap();
            transform.translation = Vec3::new(0.0, 0.0, 2.0);
            commands.entity(event.dropped).remove::<Draggable>();
            return true; 
        }
        commands.entity(arrow.parent).remove_children(&[event.dropped]);
        commands.entity(event.dropped).despawn();
        return true;
    }
    return false;
}

fn on_dragged(
    mut commands: Commands,
    mut er_drag: EventReader<Dragged>,
    mut q_draggable: Query<&mut Transform, With<Draggable>>,
    q_arrow: Query<&ArrowDraggable>,
    parents: Query<&Parent>,
) {
    for event in er_drag.read() {
        let mut transform = q_draggable.get_mut(event.dragged).unwrap();
        transform.translation.z = 15.0;
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
}

fn draw_arrows(
    mut commands: Commands,
    transforms: Query<&Transform>,
    mut q_arrow: Query<(&mut ArrowDraggable, &Parent, Entity)>,
    asset_server: Res<AssetServer>,
    q_board: Query<Entity, With<BoardEntity>>,
) {
    if q_arrow.is_empty() {return;}
    let board = q_board.single();
    let rod: Handle<Image> = asset_server.load("textures/rod.png");
    let triangle: Handle<Image> = asset_server.load("textures/triangle.png");
    for (mut arrow,parent, entity) in q_arrow.iter_mut() {
        let parent = parent.get();
        if parent != arrow.parent {
            if let None = arrow.drawn {
                let p1 = transforms.get(arrow.parent).unwrap().translation.xy();
                let p2 = transforms.get(parent).unwrap().translation.xy();
                arrow.drawn = Some(draw_arrow(&mut commands, p1, p2, rod.clone(), triangle.clone(), board, entity, arrow.color));
            }
        }
    }
}

fn draw_arrow(
    commands: &mut Commands,
    p1: Vec2,
    p2: Vec2,
    rod: Handle<Image>,
    triangle: Handle<Image>,
    board: Entity,
    arrow: Entity,
    color: Option<Color>,
) -> Entity {
    println!("Drawing arrow!");
    let avg = (p1 + p2) / 2.0;
    let dif = p2 - p1;
    let dist = (dif.x * dif.x + dif.y * dif.y).sqrt();
    let angle = dif.y.atan2(dif.x);
    let mut ent = board;
    let color = color.unwrap_or(Color::rgba(0.0,1.0,0.0,0.7));
    commands.entity(board).with_children(|parent|{
        let mut p = parent.spawn((SpriteBundle{
            transform: Transform {
                translation: Vec3::new(avg.x,avg.y,10.0),
                rotation: Quat::from_rotation_z(angle),
                scale: Vec3::new(dist / 128.0, 1.0 / 512.0 ,1.0),
            },
            sprite: Sprite {
                color,
                ..default()
            },
            texture: rod,
            ..default()
        }, ArrowDrawn{from: arrow}));
        p.with_children(|parent| {
            parent.spawn(SpriteBundle{
                transform: Transform {
                    translation: Vec3::new(69.0, 0.0, 1.0),
                    scale: Vec3::new(0.25/dist,1.0,1.0),
                    ..default()
                },
                sprite: Sprite {
                    color,
                    ..default()
                },
                texture: triangle,
                ..default()
            });
        });
        ent = p.id();
    });
    return ent;
}

fn on_hovered (
    mut commands: Commands,
    mut er_hovered: EventReader<HoveredChange>,
    transforms: Query<&Transform>,
    mut q_arrow: Query<(&mut ArrowDraggable, Entity)>,
    asset_server: Res<AssetServer>,
    q_board: Query<Entity, With<BoardEntity>>,
) {
    if er_hovered.is_empty() {return;}
    let board = q_board.single();
    let rod: Handle<Image> = asset_server.load("textures/rod.png");
    let triangle: Handle<Image> = asset_server.load("textures/triangle.png");
    for event in er_hovered.read() {
        if let Ok((mut arrow, ent)) = q_arrow.get_mut(event.hovered) {
            arrow.color = Some(color_from_inputs(event.inputs));
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
                    let p2 = transforms.get(over).unwrap().translation.xy();
                    arrow.drawn = Some(draw_arrow(&mut commands, p1, p2, rod.clone(), triangle.clone(), board, ent, arrow.color));
                }
            }
        }
    }
}

fn clear_arrows(
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