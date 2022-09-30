use ggez::{
    event,
    graphics::{self, Color, MeshBuilder, Image, DrawMode},
    Context, GameResult, conf,
};
use glam::*;
use std::{env, path};
const CELL_SIZE: i16 = 210;
const GRID_SIZE: i16 = 8;
const CELL_DIMENSIONS: (i16, i16) = (CELL_SIZE, CELL_SIZE);
const GRID_DIMENSIONS: (i16, i16) = (GRID_SIZE, GRID_SIZE);
const SCREEN_DIMENSIONS: (i16, i16) = (CELL_DIMENSIONS.0 * GRID_DIMENSIONS.0, CELL_DIMENSIONS.1 * GRID_DIMENSIONS.1);
struct Imglib {
    black_pawn: Image,
    black_rook: Image, 
    black_knight: Image, 
    black_bishop: Image,
    black_queen: Image, 
    black_king: Image,
    white_pawn: Image,
    white_rook: Image, 
    white_knight: Image, 
    white_bishop: Image,
    white_queen: Image, 
    white_king: Image
}
impl Imglib {
    fn new(ctx: &mut Context) -> GameResult<Imglib> {
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
struct MainState {
    pieces: Imglib,
    board: chess::board::Board, 
    highlights: Vec<chess::util::Pos>,
    selected_pos: Option<chess::util::Pos>
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState { 
            pieces: Imglib::new(ctx)?,
            board: chess::board::Board::new(), 
            highlights: Vec::new(), 
            selected_pos: None
         };

         Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            graphics::CanvasLoadOp::Clear([0.1, 0.2, 0.3, 1.0].into()),
        );

        // draw the grid
        let mut mb = MeshBuilder::new();
        for row in 0..GRID_DIMENSIONS.0 {
            for column in 0..GRID_DIMENSIONS.1 {
                let mut color: Color = Color::from_rgb(118,150,86);
                if (row + column) % 2 == 0 { color = Color::from_rgb(238,238,210)}
                mb.rectangle(
                    DrawMode::fill(), 
                    graphics::Rect { x: (row * CELL_DIMENSIONS.0) as f32, y: (column * CELL_DIMENSIONS.1) as f32, w: CELL_DIMENSIONS.0 as f32, h: CELL_DIMENSIONS.1 as f32 }, 
                    color).expect("Error in building mesh");
                
            }
        }
        canvas.draw(&graphics::Mesh::from_data(ctx, mb.build()), graphics::DrawParam::new().image_scale(false));
        
        //draw selected piece
        if let Some(h) = self.selected_pos {
            let mut mb = MeshBuilder::new();
            mb.rectangle(
                DrawMode::fill(), 
                graphics::Rect { x: (h.x as i16 * CELL_DIMENSIONS.0) as f32, y: (h.y as i16 * CELL_DIMENSIONS.1) as f32, w: CELL_DIMENSIONS.0 as f32, h: CELL_DIMENSIONS.1 as f32 }, 
                Color::from_rgb(0, 85, 71)).expect("Error in building mesh");
            canvas.draw(&graphics::Mesh::from_data(ctx, mb.build()), graphics::DrawParam::new().image_scale(false));
    }
        
        // draw the pieces
        for row in 0..GRID_DIMENSIONS.0 {
            for column in 0..GRID_DIMENSIONS.1 {
                if let Some(p) = &self.board.board[column as usize][row as usize] {
                    let t = p.get_type();
                    let c = p.get_color();

                    let mut img = &self.pieces.white_pawn;

                    if c == chess::util::Color::White {
                        if t == chess::piece::PieceType::Rook {
                            img = &self.pieces.white_rook;
                        } else if t == chess::piece::PieceType::Knight {
                            img = &self.pieces.white_knight;
                        } else if t == chess::piece::PieceType::Bishop {
                            img = &self.pieces.white_bishop;
                        } else if t == chess::piece::PieceType::Queen {
                            img = &self.pieces.white_queen;
                        } else if t == chess::piece::PieceType::King {
                            img = &self.pieces.white_king;
                        }
                    } else {
                        if t == chess::piece::PieceType::Rook {
                            img = &self.pieces.black_rook;
                        } else if t == chess::piece::PieceType::Knight {
                            img = &self.pieces.black_knight;
                        } else if t == chess::piece::PieceType::Bishop {
                            img = &self.pieces.black_bishop;
                        } else if t == chess::piece::PieceType::Queen {
                            img = &self.pieces.black_queen;
                        } else if t == chess::piece::PieceType::King {
                            img = &self.pieces.black_king;
                        } else if t == chess::piece::PieceType::Pawn {
                            img = &self.pieces.black_pawn;
                        }
                    }

                    let dst = glam::Vec2::new((row * CELL_DIMENSIONS.0) as f32, (column * CELL_DIMENSIONS.1) as f32);
                    let scale = glam::Vec2::new((CELL_SIZE as f32) / (img.width() as f32), (CELL_SIZE as f32) / (img.height() as f32));
                    canvas.draw(img, 
                    graphics::DrawParam::new().dest(dst).scale(scale));
                }
            }
        }
        
        //draw highlights
        let mut mb = MeshBuilder::new();
        for m in &self.highlights {
            //mb.rectangle(
            //    DrawMode::fill(), 
            //    graphics::Rect { x: (m.x as i16 * CELL_DIMENSIONS.0) as f32, y: (m.y as i16 * CELL_DIMENSIONS.1) as f32, w: CELL_DIMENSIONS.0 as f32, h: CELL_DIMENSIONS.1 as f32 }, 
            //    Color::CYAN).expect("Error in building mesh");
            mb.circle(
                DrawMode::fill(), 
                Vec2::new((CELL_SIZE as f32 / 2.0) + (m.x as i16 * CELL_DIMENSIONS.0) as f32, (CELL_SIZE as f32 / 2.0) + (m.y as i16 * CELL_DIMENSIONS.1) as f32), 
                CELL_SIZE as f32 / 10.0, 
                0.1, 
                Color::from_rgb(94, 74, 130)).expect("Error in building mesh");
        }
        canvas.draw(&graphics::Mesh::from_data(ctx, mb.build()), graphics::DrawParam::new().image_scale(false));

        
        canvas.finish(ctx);
        Ok(())
    }

    fn mouse_button_down_event(
            &mut self,
            _ctx: &mut Context,
            _button: event::MouseButton,
            _x: f32,
            _y: f32,
    ) -> Result<(), ggez::GameError> {
        println!("Clicked: {}, {}", _x, _y);
        let pos = chess::util::Pos {
            x: (_x / CELL_DIMENSIONS.0 as f32) as i8,
            y: (_y / CELL_DIMENSIONS.0 as f32) as i8,
        };
        if let Some(p) = self.selected_pos {
            println!("here");
            if self.highlights.contains(&pos) {
                println!("Moving to pos");
                self.board.perform_move(p, pos, None);
                self.highlights = Vec::new();
                self.selected_pos = None;
                return self.draw(_ctx);
            }
        }
        self.highlights = self.board.get_possible_moves_at_square(pos);
        self.selected_pos = None;
        println!("{}", self.board.print(None));
        if let Some(p) = &self.board.board[pos.y as usize][pos.x as usize] {
            if p.get_color() == self.board.turn {
                self.selected_pos = Some(pos);
            }
        }

        self.draw(_ctx)
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("drawing", "ggez").add_resource_path(resource_dir);

    let (mut ctx, events_loop) = cb
    .window_mode(conf::WindowMode::default().dimensions(SCREEN_DIMENSIONS.0 as f32, SCREEN_DIMENSIONS.1 as f32))
    .build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, events_loop, state)
}