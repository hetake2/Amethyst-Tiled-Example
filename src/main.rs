//#![allow(dead_code, unused_imports)]

extern crate amethyst;
extern crate tiled;

mod tiled_map;
use tiled_map::load_tmx_map;

use amethyst::prelude::*;
use amethyst::input::{InputBundle, InputHandler};
use amethyst::ecs::{Component, Entity, Join, NullStorage, Read, ReadStorage, System, WriteStorage};
use amethyst::core::transform::{Parent, Transform, GlobalTransform, TransformBundle};
use amethyst::core::nalgebra::{MatrixArray, Matrix4, Vector3};
use amethyst::assets::{AssetStorage, Loader};
use amethyst::renderer::{
    Camera, ColorMask, DepthMode, DisplayConfig, DrawFlat2D, Event, Projection, Pipeline, PngFormat, 
    Texture, Transparent, TextureHandle, TextureMetadata, RenderBundle, 
    Sprite, SpriteRender, ScreenDimensions, SpriteSheet, SpriteSheetFormat, 
    SpriteSheetHandle, Stage, VirtualKeyCode, ALPHA
};

pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

// Player and...
#[derive(Default)]
struct Player;

impl Component for Player {
    type Storage = NullStorage<Self>;
}

// ...his movement.
struct MovementSystem;

impl<'s> System<'s> for MovementSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        Read<'s, InputHandler<String, String>>
    );

    fn run(&mut self, (players, mut transforms, input): Self::SystemData) {
        let x_move = input.axis_value("entity_x").unwrap();
        let y_move = input.axis_value("entity_y").unwrap();

        for (_, transform) in (&players, &mut transforms).join() {
            transform.translate_x(x_move as f32 * 5.0);
            transform.translate_y(y_move as f32 * 5.0);
        }
    }
}

fn load_sprite_sheet(world: &World, png_path: &str, ron_path: &str) -> SpriteSheetHandle {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            png_path,
            PngFormat,
            TextureMetadata::srgb_scale(),
            (),
            &texture_storage,
        )
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        ron_path,
        SpriteSheetFormat,
        texture_handle,
        (),
        &sprite_sheet_store,
    )
}

// Creating player.
fn init_player(world: &mut World, sprite_sheet: &SpriteSheetHandle) -> Entity {
    let mut transform = Transform::default();
    // let (width, height) = {
    //     let dim = world.read_resource::<ScreenDimensions>();
    //     (dim.width(), dim.height())
    // };
    transform.set_x(0.0);
    transform.set_y(0.0);
    let sprite = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 1,
    };
    world
        .create_entity()
        .with(transform)
        .with(Player)
        .with(sprite)
        .with(Transparent)
        .build()
}

// Camera.
pub fn initialize_camera(world: &mut World, parent: Entity) {
    let mut transform = Transform::default();
    transform.set_xyz(-16.0, -16.0, 2.0);
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };
    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0, width, 0.0, height
        )))
        .with(Parent { entity: parent })
        .with(transform)
        .build();
}

// Actually the gameplay struct with data.
#[derive(Debug)]
pub struct TiledGame;

impl SimpleState for TiledGame { 
    fn on_start(&mut self, data: StateData<GameData>) {
        
        let world = data.world;

        // Loading map.
        load_tmx_map(world, 
        "resources/tiled/tilesheet.png",
        "resources/tiled/untitled.tmx",
        );
        // Loading character.
        let circle_sprite_sheet_handle = load_sprite_sheet(world, 
        "resources/sprites/example_sprite.png", 
        "resources/sprites/example_sprite.ron"
        );

        // Loading camera.
        let parent = init_player(world, &circle_sprite_sheet_handle);
        initialize_camera(world, parent);


        

    }
}

fn main() -> amethyst::Result<()> {

    amethyst::start_logger(Default::default());
    
    use amethyst::utils::application_root_dir;

    let conf_dir = format!(
        "{}/resources/display_config.ron", 
        application_root_dir()
        );

    let config = DisplayConfig::load(&conf_dir);

    // Rendering code.
    let pipe = Pipeline::build()
    .with_stage(
        Stage::with_backbuffer()
            // Background.
            .clear_target(BLACK, -1.0) 
            .with_pass(DrawFlat2D::new()
                .with_transparency(
                ColorMask::all(),
                ALPHA,
                Some(DepthMode::LessEqualWrite))), // Tells the pipeline to respect sprite z-depth.
    );

    let game_data = GameDataBuilder::default()
    .with_bundle(
        TransformBundle::new())?
    .with_bundle(
        InputBundle::<String, String>::new()
            .with_bindings_from_file("resources/input.ron")?,
    )?
    .with(MovementSystem, "movement", &[])
    .with_bundle(
        RenderBundle::new(pipe, Some(config))
            .with_sprite_sheet_processor()
            .with_sprite_visibility_sorting(&[]), // Let's us use the `Transparent` component.
    )?;

    let mut game = Application::build("./", TiledGame)?
        .build(game_data)?;

    game.run();

    Ok(())
}