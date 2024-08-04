use macroquad::math::Vec2;

#[derive(Eq, Hash, PartialEq)]
pub struct Cell {
    /// Row number (0..).
    pub i: usize,
    /// Column number (0..).
    pub j: usize,
}

#[derive(Default)]
pub struct Cells {
    pub rows:        usize,
    pub columns:     usize,
    pub cell_width:  f32,
    pub cell_height: f32,
}

impl Cells {
    #[inline(always)]
    pub fn get_cell_by_pos(&self, pos: &Vec2) -> Cell {
        Cell {
            i: (pos.y / self.cell_height) as usize,
            j: (pos.x / self.cell_width) as usize,
        }
    }
}
