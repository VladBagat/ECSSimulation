use bevy::prelude::{Resource, Vec2};

#[derive(Clone, Copy, Debug, Default)]
pub struct Tile {
    pub terrain_type: u8,
}

#[derive(Clone, Debug, Default, Resource)]
pub struct WorldGrid {
    tiles: Vec<Tile>,
    scale: u16,
    width: u32,
    height: u32,
}

impl WorldGrid {
    pub fn new(height: u32, width: u32, scale: u16) -> WorldGrid {
        Self {
            tiles: vec![Tile::default(); (height * width) as usize],
            scale,
            width,
            height,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn scale(&self) -> u32 {
        self.scale as u32
    }

    pub fn world_to_grid(&self, coords: Vec2) -> Vec2 {
        let total_width = (self.width * self.scale as u32) as f32;
        let total_height = (self.height * self.scale as u32) as f32;
        let world_centre = Vec2::new(total_width / 2.0, total_height / 2.0);
        
        let scale = self.scale as f32;
        let grid_x = (coords.x + world_centre.x) / scale;
        let grid_y = (coords.y + world_centre.y) / scale;
        
        Vec2::new(grid_x.floor(), grid_y.floor())
    }

    pub fn grid_to_world(&self, coords: Vec2, building_size: Vec2) -> Vec2 {
        let total_width = (self.width * self.scale as u32) as f32;
        let total_height = (self.height * self.scale as u32) as f32;
        let world_centre = Vec2::new(total_width / 2.0, total_height / 2.0);
        let cell_offset = self.scale as f32 / 2.0;
        let even_x = (building_size.x.round() as i32) % 2 == 0;
        let even_y = (building_size.y.round() as i32) % 2 == 0;
        let center_offset = Vec2::new(if even_x { 0.5 } else { 0.0 }, if even_y { 0.5 } else { 0.0 });

        Vec2::new(
            ((coords.x + center_offset.x) * self.scale as f32) + cell_offset - world_centre.x,
            ((coords.y + center_offset.y) * self.scale as f32) + cell_offset - world_centre.y)
    }

    pub fn modify_rectangle(&mut self, origin: Vec2, size: Vec2) {
        let width = size.x.round() as i32;
        let height = size.y.round() as i32;
        if width <= 0 || height <= 0 {
            return;
        }

        let cx = origin.x.floor() as i32;
        let cy = origin.y.floor() as i32;
        let w = width;
        let h = height;

        let start_x = cx - (w - 1) / 2;
        let end_x = start_x + w - 1;
        let start_y = cy - (h - 1) / 2;
        let end_y = start_y + h - 1;

        for y in start_y..=end_y {
            for x in start_x..=end_x {
                let coords = Vec2::new(x as f32, y as f32);
                if let Some(idx) = self.vec2_to_index(coords) {
                    if let Some(tile) = self.tiles.get_mut(idx) {
                        tile.terrain_type = 1;
                    }
                }
            }
        }
    }

    fn vec2_to_index(&self, coords: Vec2) -> Option<usize> {
        let x = coords.x.floor() as i32;
        let y = coords.y.floor() as i32;

        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }

        Some((y as u32 * self.width + x as u32) as usize)
    }
}
