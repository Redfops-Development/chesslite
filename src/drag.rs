use bevy::{prelude::*, window::PrimaryWindow};

pub struct DragPlugin;

impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update,(startdrag,dragging,drop))
        .add_event::<Dropped>();
    }
}

#[derive(Event)]
pub struct Dropped{
    pub dropped: Entity,
    pub recieved: Option<Entity>,
}

#[derive(Component)]
pub struct Draggable;

#[derive(Component)]
pub struct Dragging;

#[derive(Component)]
pub struct Reciever;

fn startdrag(
    mut commands: Commands,
    q_draggable: Query<(&GlobalTransform,&Handle<Image>,Entity), With<Draggable>>, 
    dragging: Query<&Dragging>, 
    buttons: Res<Input<MouseButton>>, 
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    assets: Res<Assets<Image>>,
){
    let window = q_windows.single();
    let (camera, camera_transform) = q_camera.single();
    if buttons.just_pressed(MouseButton::Left) && dragging.is_empty() {
        if let Some(world_position) = window.cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            for (gtransform, image_handle, entity) in q_draggable.iter() {
                let transform = gtransform.compute_transform();
                let image_dimensions = assets.get(image_handle).unwrap().size();
                let scaled_image_dimension = image_dimensions * transform.scale.truncate();
                let bounding_box = Rect::from_center_size(gtransform.translation().truncate(), scaled_image_dimension);
                
                if bounding_box.contains(world_position) {
                    commands.entity(entity).insert(Dragging);
                    break;
                }
                
            }
        }
    }
}

fn dragging(
    q_parent:Query<&GlobalTransform>,
    mut q_dragging:Query<(&Parent,&mut Transform),With<Dragging>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
){
    let window = q_windows.single();
    let (camera, camera_transform) = q_camera.single();
    for (parent, mut transform) in q_dragging.iter_mut() {
        let gtransform = q_parent.get(parent.get()).unwrap();
        if let Some(world_position) = window.cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            let mut mat = gtransform.compute_matrix();
            mat = mat.inverse();

            let local_point4 = mat.mul_vec4(Vec4::new(world_position.x,world_position.y,0.0,1.0));
            let local_point = Vec3::new(local_point4.x,local_point4.y, 4.0);

            transform.translation = local_point;
        }
    }
}

fn drop(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>, 
    q_recievers: Query<(&GlobalTransform, Entity),With<Reciever>>,
    q_dragging: Query<Entity, With<Dragging>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut ew_dropped: EventWriter<Dropped>,
){
    if buttons.pressed(MouseButton::Left){
        return
    }
    let window = q_windows.single();
    let (camera, camera_transform) = q_camera.single();
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        for (gtransform, entity) in q_recievers.iter() {
            let transform = gtransform.compute_transform();
            let bounding_box = Rect::from_center_size(gtransform.translation().truncate(), transform.scale.truncate());
            if bounding_box.contains(world_position) {
                for dragging in q_dragging.iter(){
                    ew_dropped.send(Dropped{dropped:dragging, recieved: Some(entity)});
                    commands.entity(dragging).remove::<Dragging>();
                }
                return;
            }
        }
        for dragging in q_dragging.iter(){
            ew_dropped.send(Dropped{dropped:dragging, recieved: None});
            commands.entity(dragging).remove::<Dragging>();
        }
    }
}