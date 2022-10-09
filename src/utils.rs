use ggez::{Context, GameResult};
use ggez::graphics::Image;

const CELL_SIZE: i16 = 140;
const GRID_SIZE: i16 = 8;
pub const CELL_DIMENSIONS: (i16, i16) = (CELL_SIZE, CELL_SIZE);
pub const GRID_DIMENSIONS: (i16, i16) = (GRID_SIZE, GRID_SIZE);
pub const SCREEN_DIMENSIONS: (i16, i16) = (CELL_DIMENSIONS.0 * GRID_DIMENSIONS.0, CELL_DIMENSIONS.1 * GRID_DIMENSIONS.1);
pub struct Imglib {
    pub black_pawn: Image,
    pub black_rook: Image, 
    pub black_knight: Image, 
    pub black_bishop: Image,
    pub black_queen: Image, 
    pub black_king: Image,
    pub white_pawn: Image,
    pub white_rook: Image, 
    pub white_knight: Image, 
    pub white_bishop: Image,
    pub white_queen: Image, 
    pub white_king: Image
}
impl Imglib {
    pub fn new(ctx: &mut Context) -> GameResult<Imglib> {
        Ok(
        Imglib { 
            black_pawn: Image::from_path(ctx, "/b_pawn.png", true)?, 
            black_rook: Image::from_path(ctx, "/b_rook.png", true)?,
            black_knight: Image::from_path(ctx, "/b_knight.png", true)?,
            black_bishop: Image::from_path(ctx, "/b_bishop.png", true)?, 
            black_queen: Image::from_path(ctx, "/b_queen.png", true)?, 
            black_king: Image::from_path(ctx, "/b_king.png", true)?, 
            white_pawn: Image::from_path(ctx, "/w_pawn.png", true)?, 
            white_rook: Image::from_path(ctx, "/w_rook.png", true)?, 
            white_knight: Image::from_path(ctx, "/w_knight.png", true)?,
            white_bishop: Image::from_path(ctx, "/w_bishop.png", true)?, 
            white_queen: Image::from_path(ctx, "/w_queen.png", true)?, 
            white_king: Image::from_path(ctx, "/w_king.png", true)?
        })
    }
}

#[derive(Eq, PartialEq)]
pub enum State {
    Waiting, 
    Playing
}
