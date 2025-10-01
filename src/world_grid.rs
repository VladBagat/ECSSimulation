use bevy::prelude::{Resource, Vec2};

#[derive(Clone, Copy, Debug, Default)]
pub struct Tile {
    pub terrain_type: u8,
}

#[derive(Clone, Debug, Default, Resource)]
pub struct WorldGrid {
    tiles: Vec<Tile>,
    width: u32,
    height: u32,
}

impl WorldGrid {
    pub fn new(height: u32, width: u32) -> WorldGrid {
        Self {
            tiles: vec![Tile::default(); (height * width) as usize],
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

    fn vec2_to_index(&self, coords: Vec2) -> Option<usize> {
        let x = coords.x.floor() as i32;
        let y = coords.y.floor() as i32;

        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }

        Some((y as u32 * self.width + x as u32) as usize)
    }
}
