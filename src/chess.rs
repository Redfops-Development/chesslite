pub const BLACK_TILE:Color = Color::Rgba{red: 0.462745, green: 0.588235, blue:0.337254, alpha:1.0};
pub const WHITE_TILE:Color = Color::Rgba{red: 0.933333, green: 0.933333, blue:0.823529, alpha:1.0};
pub const PIECE_SCALE:f32 = 0.0078125;

pub mod board;

pub mod promotion;

pub mod arrow;

use bevy::prelude::*;

use bevy_dragndrop::*;

use crate::chess::promotion::{PromotionChoiceEvent,PromotionChosenEvent,PromotionPlugin};

use crate::chess::board::{Board,Piece,PieceColor,PieceType,GameOverState};
use crate::chess::arrow::*;

use bevy_prototype_lyon::prelude::*;



pub struct ChessPluginClient;

impl Plugin for ChessPluginClient {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(DragPlugin)
        .add_plugins(PromotionPlugin)
        .add_plugins(ShapePlugin)
        .insert_resource(Board::new())
        .insert_resource(PieceTextureHolder{textures: None})
        .insert_resource(Msaa::Sample4)
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
        handle_arrow_dragged(&mut commands, event, &q_arrow, &parents);
    }
}


fn on_hovered (
    mut commands: Commands,
    mut er_hovered: EventReader<HoveredChange>,
    transforms: Query<&Transform>,
    q_tiles: Query<&TileEntity>,
    mut q_arrow: Query<(&mut ArrowDraggable, Entity)>,
    q_board: Query<Entity, With<BoardEntity>>,
) {
    if er_hovered.is_empty() {return;}
    let board = q_board.single();
    for event in er_hovered.read() {
        handle_arrow_hovered(&mut commands, event, &transforms, &q_tiles, &mut q_arrow, board);
    }
}
