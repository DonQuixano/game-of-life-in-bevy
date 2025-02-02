//! Shows how to render simple primitive shapes with a single color.
//!
//! You can toggle wireframes with the space bar except on wasm. Wasm does not support
//! `POLYGON_MODE_LINE` on the gpu.
//!
use bevy::{
    asset::processor::InitializeError, color, ecs::query, gizmos::grid, input::mouse, prelude::*,
    render::mesh, text::cosmic_text::ttf_parser::Width,
};
use std::collections::HashSet;
use std::u8;
use std::{any::Any, io};
#[cfg(not(target_arch = "wasm32"))]
use std::{
    cell::{self, Cell},
    sync::{self, Arc},
    vec,
};

const WIDTH: usize = 100;
const HEIGHT: usize = 70;

const GRIDSIZE: usize = WIDTH * HEIGHT;

#[derive(Resource)]
struct CellGrid {
    arr: Vec<u8>,
}

impl Default for CellGrid {
    fn default() -> Self {
        CellGrid {
            arr: vec![0; GRIDSIZE],
        }
    }
}

#[derive(Resource)]
pub struct Pixels {
    grid: Vec<Handle<Mesh>>,
}

impl Default for Pixels {
    fn default() -> Self {
        return Pixels { grid: vec![] };
    }
}
#[derive(Resource)]
pub struct Entities {
    grid: Vec<Entity>,
}

impl Default for Entities {
    fn default() -> Self {
        return Entities { grid: vec![] };
    }
}

#[derive(Resource)]
pub struct Matts {
    grid: Vec<AssetId<ColorMaterial>>,
}

impl Default for Matts {
    fn default() -> Self {
        return Matts { grid: vec![] };
    }
}

#[derive(Resource)]
pub struct InititalRun {
    isfirst: bool,
}

impl Default for InititalRun {
    fn default() -> Self {
        return InititalRun { isfirst: true };
    }
}

#[derive(Resource)]
pub struct MouseXY {
    x: f32,
    y: f32,
    paused: bool,
}

impl Default for MouseXY {
    fn default() -> Self {
        return MouseXY {
            x: 0.0,
            y: 0.0,
            paused: false,
        };
    }
}
#[derive(Resource, Debug)]

pub struct Rulestring {
    survive: HashSet<u8>,
    birth: HashSet<u8>,
    decay_states: u8,
}
//3456/278/6
impl Default for Rulestring {
    fn default() -> Self {
        return Rulestring {
            survive: HashSet::from([2u8, 3u8]),
            birth: HashSet::from([3u8]),
            decay_states: 0,
        };
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins));

    app.init_resource::<Pixels>();
    app.init_resource::<MouseXY>();
    app.init_resource::<CellGrid>();
    app.init_resource::<Entities>();
    app.init_resource::<Matts>();
    app.init_resource::<InititalRun>();
    app.init_resource::<Rulestring>();
    app.add_systems(Startup, setup);
    app.add_systems(Update, mouse_button_input);
    app.add_systems(Update, keyboard_input);

    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, game_loop);
    app.add_systems(Update, my_cursor_system);
    app.run();
}

const X_EXTENT: f32 = 900.;

fn modulo(a: i32, b: i32) -> i32 {
    return ((a % b) + b) % b;
}

fn index_map(x: i32, y: i32, w: i32, h: i32) -> usize {
    let outval: i32 = modulo(x, w) + (modulo(y, h) * w);
    return outval as usize;
}

fn index_map_rev(n: i32, w: i32) -> (i32, i32) {
    return ((n % w), n / w);
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut Pixels: ResMut<Pixels>,
    mut Entities: ResMut<Entities>,
    mut Matts: ResMut<Matts>,
    mut CellGrid: ResMut<CellGrid>,
    mut InititalRun: ResMut<InititalRun>,
    mut window: Single<&mut Window>,
) {
    window
        .resolution
        .set((WIDTH as f32) * 10.0, (HEIGHT as f32) * 10.0);
    let mut initial_vec: Vec<u8> = vec![0; GRIDSIZE];

    CellGrid.arr[index_map(25, 25, WIDTH as i32, HEIGHT as i32)] = 1;
    CellGrid.arr[index_map(26, 25, WIDTH as i32, HEIGHT as i32)] = 1;
    CellGrid.arr[index_map(27, 25, WIDTH as i32, HEIGHT as i32)] = 1;

    commands.spawn(Camera2d);
    //meshes.add(Rectangle::new(10.0, 10.0)),

    // let mut shapes: Vec<Handle<Mesh>> = vec![];

    for i in 0..GRIDSIZE {
        Pixels.grid.push(meshes.add(Rectangle::new(10.0, 10.0)));
    }

    //Pixels = shapes.clone();

    let num_shapes = Pixels.grid.len();

    for (i, shape) in Pixels.grid.iter().enumerate() {
        // Distribute colors evenly across the rainbow.
        let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);
        let f_width: f32 = WIDTH as f32;
        let f_height: f32 = HEIGHT as f32;
        let f_i: f32 = i as f32;

        let mut matt = materials.add(Color::rgb(0.0, 0.0, 0.0));
        let matt_id = matt.id();
        let pixel: Entity = commands
            .spawn((
                Mesh2d(shape.clone()),
                MeshMaterial2d(matt),
                Transform::from_xyz(
                    ((f_i % f_width) - (f_width + 1.0) / 2.0) * 10.0,
                    ((f_i / f_width) - (f_height + 1.0) / 2.0) * 10.0,
                    0.0,
                ),
            ))
            .id();
        Entities.grid.push(pixel);
        Matts.grid.push(matt_id)
    }
}

fn game_loop(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut Pixels: ResMut<Pixels>,
    mut CellGrid: ResMut<CellGrid>,
    mut Entities: ResMut<Entities>,
    mut Matts: ResMut<Matts>,
    mut InititalRun: ResMut<InititalRun>,
    query: Query<bevy::prelude::Entity>,
    mousexy: ResMut<MouseXY>,
    rulestr: ResMut<Rulestring>,
) {
    let mut change: bool = false;
    let mut tmp_vec: Vec<u8> = vec![];
    let scangrid: Vec<u8> = CellGrid.arr.clone();
    {
        for (lin, cvalue) in CellGrid.arr.iter_mut().enumerate() {
            if *cvalue > 1 {
                tmp_vec.push(if *cvalue != 2 { *cvalue - 1 } else { 0 });
                loop {
                    if let Some(material) = materials.get_mut(Matts.grid[lin]) {
                        let cvalinto: f32 = f32::from(*cvalue);
                        let statedecinto: f32 = f32::from(rulestr.decay_states);
                        material.color = Color::linear_rgb(0.0, (cvalinto / statedecinto), 0.0);
                        break;
                    }
                }

                continue;
            }

            if !mousexy.paused {
                let mut n_count: u8 = 0;
                let width32: i32 = WIDTH as i32;
                let hieght32: i32 = HEIGHT as i32;
                let (cell_x, cell_y): (i32, i32) = index_map_rev(lin as i32, (WIDTH as i32));
                n_count += if scangrid
                    [index_map(cell_x + 1, cell_y + 1, WIDTH as i32, HEIGHT as i32)]
                    == 1
                {
                    1
                } else {
                    0
                };
                n_count += if scangrid
                    [index_map(cell_x - 1, cell_y - 1, WIDTH as i32, HEIGHT as i32)]
                    == 1
                {
                    1
                } else {
                    0
                };
                n_count += if scangrid
                    [index_map(cell_x - 1, cell_y + 1, WIDTH as i32, HEIGHT as i32)]
                    == 1
                {
                    1
                } else {
                    0
                };
                n_count += if scangrid
                    [index_map(cell_x + 1, cell_y - 1, WIDTH as i32, HEIGHT as i32)]
                    == 1
                {
                    1
                } else {
                    0
                };

                n_count +=
                    if scangrid[index_map(cell_x + 1, cell_y, WIDTH as i32, HEIGHT as i32)] == 1 {
                        1
                    } else {
                        0
                    };
                n_count +=
                    if scangrid[index_map(cell_x - 1, cell_y, WIDTH as i32, HEIGHT as i32)] == 1 {
                        1
                    } else {
                        0
                    };
                n_count +=
                    if scangrid[index_map(cell_x, cell_y + 1, WIDTH as i32, HEIGHT as i32)] == 1 {
                        1
                    } else {
                        0
                    };
                n_count +=
                    if scangrid[index_map(cell_x, cell_y - 1, WIDTH as i32, HEIGHT as i32)] == 1 {
                        1
                    } else {
                        0
                    };

                change = true;
                if *cvalue == 1u8 {
                    if rulestr.survive.contains(&n_count) {
                        tmp_vec.push(1);
                    } else {
                        change = true;
                        tmp_vec.push(rulestr.decay_states as u8);
                    }
                } else {
                    if rulestr.birth.contains(&n_count) {
                        change = true;
                        tmp_vec.push(1);
                    } else {
                        tmp_vec.push(0);
                    }
                }
            } else {
                tmp_vec.push(*cvalue);
            }

            let mut NewColor: Color = Color::hsl(312.0, 100.0, 50.0);
            let mut OldColor: Color = Color::hsl(0.0, 0.0, 0.0);
            let mut dummy: u8 = 0;
            if change == true && *cvalue <= 1 {
                loop {
                    if let Some(material) = materials.get_mut(Matts.grid[lin]) {
                        material.color = if *cvalue == dummy { OldColor } else { NewColor };
                        break;
                    }
                }
            }
        }
    }
    /*for (i, pixel)  in Matts.grid.iter_mut().enumerate() {
            {

                let mut NewColor: Color = Color::hsl(312.0, 100.0, 50.0);
                let mut OldColor: Color = Color::hsl(0.0, 0.0, 0.0);
                //let res= query.get(*pixel);
                //let matter = materials.get_mut(*pixel);
                let mut fixpixel = false;
                if let Some(material) = materials.get_mut(*pixel){
                    fixpixel = if (material.color != NewColor && CellGrid.arr[i] == 1) || (material.color != OldColor && CellGrid.arr[i] == 0) {true} else {false};
                }
    //fixpixel
                //println!("{:?}", fixpixel);
                if fixpixel {
                loop {
                if let Some(material) = materials.get_mut(*pixel) {
                    material.color = if CellGrid.arr[i] == 0 {OldColor} else {NewColor};
                    break;
                }}}

                //materials.get_mut();
            }
        }*/
    InititalRun.isfirst = false;
    CellGrid.arr = tmp_vec.clone();
}

fn my_cursor_system(
    windows: Query<&Window>,
    mut coords: ResMut<MouseXY>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();

    if let Some(world_position) = window.cursor_position().and_then(|cursor| {
        let err = camera.viewport_to_world_2d(camera_transform, cursor);
        match err {
            Ok(pos) => Some(pos),
            Err(_) => None,
        }
    }) {
        //eprintln!("World coords: {}/{}", world_position.x, world_position.y);
        (coords.x, coords.y) = (world_position.x, world_position.y)
    }
}

fn mouse_button_input(
    buttons: Res<ButtonInput<MouseButton>>,
    mousexy: ResMut<MouseXY>,
    mut grid: ResMut<CellGrid>,
    mut Matts: ResMut<Matts>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if buttons.just_pressed(MouseButton::Left) && mousexy.paused {
        //grid[index_map(mousexy, y, WIDTH as i32, h as i32)]
        let w_f32: f32 = (WIDTH as f32) * 10.0;
        let h_f32: f32 = (HEIGHT as f32) * 10.0;

        let x: i32 = ((mousexy.x + w_f32 / 2.0) / 10.0).ceil() as i32;
        let y: i32 = ((mousexy.y + h_f32 / 2.0) / 10.0).ceil() as i32;

        let the_index = index_map(x, y, WIDTH as i32, WIDTH as i32) as usize;

        //println!("{:?}/{:?}", ((mousexy.x + w_f32/2.0) / 10.0).floor(),  ((mousexy.y + h_f32/2.0) / 10.0).floor());
        if grid.arr[the_index] == 1 {
            grid.arr[the_index] = 0;
        } else {
            grid.arr[the_index] = 1;
        }
        let dummy = 0u8;

        let mut NewColor: Color = Color::hsl(312.0, 100.0, 50.0);
        let mut OldColor: Color = Color::hsl(0.0, 0.0, 0.0);
        loop {
            if let Some(material) = materials.get_mut(Matts.grid[the_index]) {
                material.color = if grid.arr[the_index] == dummy {
                    OldColor
                } else {
                    NewColor
                };
                break;
            }
        }
    }
    if buttons.just_released(MouseButton::Left) {
        // Left Button was released
    }
    if buttons.pressed(MouseButton::Right) {
        // Right Button is being held down
    }
    // we can check multiple at once with `.any_*`
    if buttons.any_just_pressed([MouseButton::Left, MouseButton::Middle]) {
        // Either the left or the middle (wheel) button was just pressed
    }
}

fn spawn_stdin_channel() -> String {
    let mut input = String::new();

    _ = io::stdin().read_line(&mut input);

    return input;
}

fn nullify(t: bool) {}

fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut mousexy: ResMut<MouseXY>,
    mut InititalRun: ResMut<InititalRun>,
    mut Rulestring: ResMut<Rulestring>,
) {
    if keys.just_pressed(KeyCode::Space) {
        mousexy.paused = !mousexy.paused;
        //InititalRun.isfirst = true;
        println!("{:?}", Rulestring);
    }

    if keys.just_pressed(KeyCode::Slash) {
        println!("enter rulestring seperated by slashes: example: 3/23/0/");
        println!("birth/survive/corpse-lifetime. \n example: 2/23/0/");

        let mut NUMERALS: HashSet<char> =
            HashSet::from(['0', '1', '2', '3', '4', '5', '7', '8', '9']);

        let cmdIn = spawn_stdin_channel();
        let mut state: u8 = 0;
        Rulestring.birth.clear();
        Rulestring.survive.clear();
        let mut setDecayPlace: String = String::from("");
        for i in cmdIn.chars() {
            if i == '/' {
                state += 1;
                continue;
            }
            //let radix: u32 = 10;
            let num: u8 = i.to_digit(10).unwrap_or(0).try_into().unwrap();
            //let num = num0.unwrap_or(0) as u8;
            //let mut u8: numCounter = "";
            match state {
                0 => nullify(Rulestring.birth.insert(num)),
                1 => nullify(Rulestring.survive.insert(num)),
                2 => {
                    setDecayPlace.push(i);
                    println!("{:?}", num)
                }
                _ => break,
            }
        }
        if setDecayPlace.len() > 0 {
            println!("{:?}", setDecayPlace.trim());
            let tempwork = setDecayPlace.trim().parse::<u8>().unwrap();
            Rulestring.decay_states = tempwork;
        }
    }

    if keys.just_released(KeyCode::ControlLeft) {
        // Left Ctrl was released
    }
    if keys.pressed(KeyCode::KeyW) {
        // W is being held down
    }
    // we can check multiple at once with `.any_*`
    if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        // Either the left or right shift are being held down
    }
    if keys.any_just_pressed([KeyCode::Delete, KeyCode::Backspace]) {
        // Either delete or backspace was just pressed
    }
}

/*fn spawn_stdin_channel() -> String {
    let mut buf = String::new();
    io::stdin()::read_line(&mut buf).unwrap();
    buf
} */
