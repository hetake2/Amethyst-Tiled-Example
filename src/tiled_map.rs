extern crate amethyst;
extern crate tiled;

use tiled::parse;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use amethyst::prelude::*;
use amethyst::core::transform::{Transform, GlobalTransform};
use amethyst::core::nalgebra::Vector3;
use amethyst::assets::{AssetStorage, Loader};
use amethyst::renderer::{
    PngFormat, Texture, TextureCoordinates, TextureHandle, 
    TextureMetadata, Sprite, SpriteRender, ScreenDimensions, SpriteSheet
};


pub fn load_tmx_map<N,S>(world: &mut World, map_texture: N, map_tmx: S)
    where
        N: Into<String>,
        S: std::convert::AsRef<std::ffi::OsStr>
{ 

    // Get the game window screen height.
    let screen_height = {
        let dim = world.read_resource::<ScreenDimensions>();
        dim.height()
    };

    fn load_texture<N>(name: N, world: &World) -> TextureHandle
        where 
            N: Into<String>,
    {
    let res_loader = world.read_resource::<Loader>();
    let texture_storage = world.read_resource::<AssetStorage<Texture>>();
    res_loader.load(
        name,
        PngFormat,
        TextureMetadata::srgb_scale(),
        (),
        &texture_storage,
    )
    }

    // Load tiled map.
    //let world = data.world;
    //let world_texture_handle = load_texture("resources/tiled/tilesheet.png", &world);
    let world_texture_handle = load_texture(map_texture, &world);
    let tmx_map_file = File::open(&Path::new(&map_tmx)).unwrap();
    let reader = BufReader::new(tmx_map_file);
    let tmx_map = parse(reader).unwrap();
    if let Some(map_tileset) = tmx_map.get_tileset_by_gid(1) {
       
        let tile_width = map_tileset.tile_width as i32;
        let tile_height = map_tileset.tile_height as i32;
        let tileset_width = &map_tileset.images[0].width;
        let tileset_height = &map_tileset.images[0].height;

        let tileset_sprite_columns = tileset_width / tile_width as i32;
        let tileset_sprite_offset_colums = 1.0 / tileset_sprite_columns as f32;

        let tileset_sprite_rows = tileset_height / tile_height as i32;
        let tileset_sprite_offset_rows = 1.0 / tileset_sprite_rows as f32;
                
        // A place to store the tile sprites in
        let mut tile_sprites: Vec<Sprite> = Vec::new();

        // The x-axis needs to be reversed for TextureCoordinates
        for x in (0..tileset_sprite_rows).rev() {
            for y in 0..tileset_sprite_columns {
                        
                        // Coordinates of the 32x32 tile sprite inside the whole
                        // tileset image, `terrainTiles_default.png` in this case
                        // Important: TextureCoordinates Y axis goes from BOTTOM (0.0) to TOP (1.0)
                let tex_coords = TextureCoordinates {
                left: y as f32 * tileset_sprite_offset_colums,
                right: (y + 1) as f32 * tileset_sprite_offset_colums,
                bottom: x as f32 * tileset_sprite_offset_rows,
                top: (x + 1) as f32 * tileset_sprite_offset_rows
                };

                let sprite = Sprite {
                width: tile_width as f32,
                height: tile_height as f32,
                offsets: [0.0, 32.0],
                tex_coords
                };

                tile_sprites.push(sprite);
            }
        }
    
            // A sheet of sprites.. so all the tile sprites
            let sprite_sheet = SpriteSheet {
                texture: world_texture_handle,
                sprites: tile_sprites,
            };

            // Insert the sprite sheet, which consists of all the tile sprites,
            // into world resources for later use
            let sprite_sheet_handle = {
                let loader = world.read_resource::<Loader>();
                let sprite_sheet_storage = world.read_resource::<AssetStorage<SpriteSheet>>();

                loader.load_from_data(sprite_sheet, (), &sprite_sheet_storage)
            };

            // Now that all the tile sprites/textures are loaded in
            // we can start drawing the tiles for our viewing pleasure
            let layer: &tiled::Layer = &tmx_map.layers[0];

            // Loop the row first and then the individual tiles on that row
            // and then switch to the next row
            // y = row number
            // x = column number
            for (y, row) in layer.tiles.iter().enumerate().clone() {
                for (x, &tile) in row.iter().enumerate() {
                    // Do nothing with empty tiles
                    if tile == 0 {
                        continue;
                    }

                    // Tile ids start from 1 but tileset sprites start from 0
                    let tile = tile - 1;

                    // Sprite for the tile
                    let tile_sprite = SpriteRender {
                        sprite_sheet: sprite_sheet_handle.clone(),
                        sprite_number: tile as usize,
                    };

                    // Where we should draw the tile?
                    let mut tile_transform = Transform::default();
                    let x_coord = x * tile_width as usize;
                    // Bottom Left is 0,0 so we flip it to Top Left with the
                    // ScreenDimensions.height since tiled coordinates start from top
                    let y_coord = (screen_height) - (y as f32 * tile_height as f32);
                
                    *tile_transform.translation_mut() = Vector3::new(
                        x_coord as f32,
                        y_coord as f32,
                        -1.0
                    );

                    // Create the tile entity
                    world
                        .create_entity()
                            .with(GlobalTransform::default())
                            .with(tile_transform)
                            .with(tile_sprite)
                        .build();
                }
            
            }
    }
}