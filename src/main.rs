use ggez::{
    event::{self, EventHandler},
    graphics::{self, Color, MeshBuilder, Image, DrawMode},
    Context, GameResult, conf,
};
use std::sync::Mutex;
use glam::*;
use networking::C2sMessage;
use networking::S2cMessage;
use prost::Message;
use std::{env, path, io::{Write, Read}};
use std::net::*;
mod networking;
use std::thread;
const CELL_SIZE: i16 = 140;
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
    selected_pos: Option<chess::util::Pos>, 
    stream: Option<TcpStream>, 
    is_client: bool
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut s = MainState { 
            pieces: Imglib::new(ctx)?,
            board: chess::board::Board::new(), 
            highlights: Vec::new(), 
            selected_pos: None, 
            stream: None, 
            is_client: false
        };

        //s.draw(ctx);

        let (stream, is_client) = Self::get_stream();
        s.stream = Some(stream);
        s.is_client = is_client;

        Ok(s)
    }

    

    fn get_stream() -> (TcpStream, bool) {
        // A stream and a boolean indicating wether or not the program is a host or a client
        let (stream, is_client) = {
            let mut args = std::env::args();
            // Skip path to program
            let _ = args.next();

            // Get first argument after path to program
            let host_or_client = args
                .next()
                .expect("Expected arguments: host or client 'ip'");

            match host_or_client.as_str() {
                // If the program is running as host we listen on port 8080 until we get a
                // connection then we return the stream.
                "host" => {
                    let listener = TcpListener::bind("127.0.0.1:1337").unwrap();
                    (listener.incoming().next().unwrap().unwrap(), false)
                }
                // If the program is running as a client we connect to the specified IP address and
                // return the stream.
                "client" => {
                    let ip = args.next().expect("Expected ip address after client");
                    let stream = TcpStream::connect(ip).expect("Failed to connect to host");
                    (stream, true)
                }
                // Only --host and --client are valid arguments
                _ => panic!("Unknown command: {}", host_or_client),
            }
        };

        // Set TcpStream to non blocking so that we can do networking in the update thread
        //stream
        //    .set_nonblocking(fa)
        //    .expect("Failed to set stream to non blocking");
        (stream, is_client)
    }

    fn send_c2s_packet(&mut self, data: networking::C2sMessage) {
        let mut buf: Vec<u8> = Vec::new();
        data.encode(&mut buf).expect("Couldn't encode message");
        self.stream.as_ref().unwrap().write(&buf).expect("Failed to send c2s packet");
    }

    fn send_s2c_packet(&mut self, data: networking::S2cMessage) {
        let mut buf: Vec<u8> = Vec::new();
        data.encode(&mut buf).expect("Couldn't encode message");
        self.stream.as_ref().unwrap().write(&buf).expect("Failed to send c2s packet");
    }

    fn receive_c2s_packet(&mut self) -> networking::C2sMessage {
        let mut buf: Vec<u8> = Vec::new();
        self.stream.as_ref().unwrap().read(&mut buf).expect("Could not read data");
        C2sMessage::decode(&buf[..]).unwrap()
    }

    fn receive_s2c_packet(&mut self) -> networking::S2cMessage {
        let mut buf: Vec<u8> = Vec::new();
        self.stream.as_ref().unwrap().read(&mut buf).expect("Could not read data");
        S2cMessage::decode(&buf[..]).unwrap()
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {

        if !self.is_client {
            let data = self.receive_c2s_packet();
            let msg = data.msg.unwrap();
            println!("RECEIVED PACKET");
        }
        self.draw(ctx);

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
                let res = self.board.perform_move(p, pos, None);
                match res {
                    Ok(r) => println!("OK"), 
                    Err(e) => println!("Err")
                }

                
                
                if self.is_client {
                    let data = C2sMessage {
                        msg: Some(networking::c2s_message::Msg::Move(networking::Move {
                            from_square: (p.x + p.y * 8) as u32,
                            to_square: (pos.x + pos.y * 8) as u32,
                            promotion: None,
                        })),
                    };
                    self.send_c2s_packet(data);
                } else {
                    let data = S2cMessage {
                        msg: Some(networking::s2c_message::Msg::Move(networking::Move {
                            from_square: (p.x + p.y * 8) as u32,
                            to_square: (pos.x + pos.y * 8) as u32,
                            promotion: None,
                        })),
                    };
                }

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
