use bevy::prelude::*;

const A: i32 = 10;
const B: i32 = 10;
const C: i32 = 1;
const NEIGH_LEN: usize  = 9;
const NEIGH: [(i32,i32,i32);NEIGH_LEN] = [(-1,-1,0),(-1,1,0),(1,-1,0),(1,1,0),(1,0,0),(-1,0,0),(0,1,0),(0,-1,0),(0,0,0)];

#[derive(Clone,Copy,PartialEq)]
enum CellVal{
    Dead,
    Alive,
    None,
}

#[derive(Clone,Copy,PartialEq)]
struct CellContent{
    Current: CellVal,
    Next: CellVal,
}

#[derive(Resource)]
struct Grid{
    content: [CellContent;(A as usize)*(B as usize)*(C as usize)],
}
#[derive(Component)]
struct Cell{
    x:i32,
    y:i32,
    z:i32,
}

#[derive(Resource)]
struct PeriodTimer(Timer);

fn get_cell(x:i32, y:i32, z:i32) -> Option<i32>{
    if x<0 || x>=A || y<0 || y>=B || z<0 || z>=C {
        return None;
    }
    return Some(z*A*B+y*A+x);
}

fn from_cell(i:i32) -> (i32,i32,i32){
    return (i%A,(i/A)%B,i/(A*B))  
}

fn neighborhood(grid: &Grid, x:i32, y:i32, z:i32, buffer: &mut [CellVal;NEIGH_LEN]){
    for (i,(nx,ny,nz)) in NEIGH.iter().enumerate(){
        buffer[i] = match get_cell(x+nx,y+ny,z+nz) {
            Some(coor) => grid.content[coor as usize].Current,
            None => CellVal::None,
        }
    }
}

fn growth_function(neighborhood: &[CellVal;NEIGH_LEN]) -> CellVal{
    let mut s = 0;
    for c in neighborhood{
        if *c == CellVal::Alive{
            s+=1;
        }
    }
    if match neighborhood[8]{
        CellVal::Dead => s==3,
        CellVal::Alive => s==3 || s==4,
        CellVal::None => panic!("Erreur, la case courante n'existe pas."),
    }{
        return CellVal::Alive;
    }
    return CellVal::Dead;
}

fn change_state(mut grid: ResMut<Grid>,
        time: Res<Time>, mut timer: ResMut<PeriodTimer>){
    if timer.0.tick(time.delta()).just_finished() {

    let mut buffer = [CellVal::None;NEIGH_LEN];

        for i in 0..(A*B*C-1){
            let (x,y,z) = from_cell(i);
            let i = i as usize;
            neighborhood(&grid,x,y,z,&mut buffer);
            grid.content[i].Next = growth_function(&buffer);
        }
        for i in 0..(A*B*C-1){
            let i = i as usize;
            grid.content[i].Current = grid.content[i].Next;
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update_view)
        .add_systems(Update, change_state)
        .insert_resource(PeriodTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut grid = Grid {
        content: [{Current: CellVal::Dead,
                   Next: CellVal::Dead} ;(A*B*C)as usize],
    };
    grid.content[23].Current = CellVal::Alive; 
    grid.content[24].Current = CellVal::Alive;
    grid.content[25].Current = CellVal::Alive; 
    grid.content[23].Next = CellVal::NewAlive; 
    grid.content[24].Next = CellVal::NewAlive;
    grid.content[25].Next = CellVal::NewAlive;   
    commands.insert_resource(grid);


    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 50.0).looking_at(Vec3::new(0.5, 0.5, -0.5), Vec3::Y),
    ));

    let cube = meshes.add(Cuboid::new(0.5, 0.5, 0.5));

    let mut hsla = Hsla::hsl(0.0, 1.0, 0.5);
    for i in 0..(A*B*C) {
            let (x,y,z) = from_cell(i);
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(materials.add(StandardMaterial{base_color: Color::srgba(0.2, 0.7, 0.1, 0.0),alpha_mode: AlphaMode::Mask(0.5),..default()})),
                Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32)),
                Cell {x:x,
                      y:y,
                      z:z}));
    }
}

fn update_view(
    query: Query<(  &MeshMaterial3d<StandardMaterial>,&Cell)>,
    grid: ResMut<Grid>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    for (material_handle,cell) in &query {
            if let Some(material) = materials.get_mut(material_handle){
                let c = match get_cell(cell.x,cell.y,cell.z) {
                    Some(coor) => grid.content[coor as usize].Current,
                    None => panic!("Les coordonnées en entrée ne correpondent pas à une couleur"),
                };
                material.base_color.set_alpha(match c{
                                                        CellVal::Dead => 0.0,
                                                        CellVal::Alive => 1.0,
                                                        _ => panic!("Erreur: mauvaise valeur de case renseignée")})
                
            }
    }
}
