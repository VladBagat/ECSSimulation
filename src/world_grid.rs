#[derive(Clone, Copy, Debug, Default)]
pub struct Tile {
    pub terrain_type: u8,
}

#[derive(Clone, Debug, Default)]
pub struct WorldGrid { 
    tiles: Vec<Tile>,
    width: u32
}

impl WorldGrid {
    fn new(height:u32, width:u32) -> WorldGrid{
        Self {
            tiles: vec![Tile::default(); (height * width) as usize],
            width: width
        }
    }
}