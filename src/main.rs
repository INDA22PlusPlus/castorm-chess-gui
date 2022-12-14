use ggez::{
    event::{self, EventHandler},
    graphics::{self, Color, MeshBuilder, DrawMode},
    Context, GameResult, conf,
};
use glam::*;
use networking::C2sMessage;
use networking::S2cMessage;
use prost::{Message};
use std::{env, path, io::{Write, Read}};
use std::net::*;
mod networking;
mod utils;
use utils::*;

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

     //Set TcpStream to non blocking so that we can do networking in the update thread
    stream
        .set_nonblocking(true)
        .expect("Failed to set stream to non blocking");
    (stream, is_client)
}


struct MainState {
    pieces: Imglib,
    board: chess::board::Board, 
    highlights: Vec<chess::util::Pos>,
    selected_pos: Option<chess::util::Pos>, 
    stream: Option<TcpStream>, 
    is_client: bool, 
    state: State
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut s = MainState { 
            pieces: Imglib::new(ctx)?,
            board: chess::board::Board::new(), 
            highlights: Vec::new(), 
            selected_pos: None, 
            stream: None, 
            is_client: false, 
            state: State::Waiting
        };

        s.draw(ctx);
        Ok(s)
    }

    

    

    fn send_c2s_packet(&mut self, data: networking::C2sMessage) {
        let v = data.encode_to_vec(); //(&mut buf).expect("Couldn't encode message");
        self.stream.as_ref().unwrap().write(&v).expect("Failed to send c2s packet");
    }

    fn send_s2c_packet(&mut self, data: networking::S2cMessage) {
        let v = data.encode_to_vec(); //(&mut buf).expect("Couldn't encode message");
        self.stream.as_ref().unwrap().write(&v).expect("Failed to send c2s packet");
    }

    fn receive_c2s_packet(&mut self) -> Option<networking::C2sMessage> {
        let mut buf= [0_u8; 512];
        if self.stream.is_none() { return None }
        match self.stream.as_ref().unwrap().read(&mut buf) {
            Ok(p) => {
                println!("Recieved s2c packet {}", p);
                return Some(C2sMessage::decode(&buf[..p], ).expect("erro reading"))
            },
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => return None,
                _ => panic!("Error: {}", e)
            }
        }
    }

    fn receive_s2c_packet(&mut self) -> Option<networking::S2cMessage> {
        let mut buf= [0_u8; 512];
        if self.stream.is_none() { return None }
        match self.stream.as_ref().unwrap().read(&mut buf) {
            Ok(p) => {
                println!("Recieved s2c packet {}", p);
                return Some(S2cMessage::decode(&buf[..p], ).expect("erro reading"))
            } ,
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => return None,
                _ => panic!("Error: {}", e)
            }
        }
        
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {

        if self.is_client {
            let data = self.receive_s2c_packet();
            if data.is_some() {
                match data.unwrap().msg {
                    None => println!("Received NONE msg!!!"), 
                    Some(msg) => {
                        println!("Received SOME msg!!!!");
                        match msg {
                            networking::s2c_message::Msg::Move(m) => {
                                let p = chess::util::Pos {
                                    x: (m.from_square % 8) as i8,
                                    y: (m.from_square / 8) as i8,
                                };
                                let pos = chess::util::Pos {
                                    x: (m.to_square % 8) as i8,
                                    y: (m.to_square / 8) as i8,
                                };
                                self.board.perform_move(p, pos, None);
                                println!("RECEIVED MOVE PACKET");
                                self.draw(ctx);
                            },
                            networking::s2c_message::Msg::ConnectAck(ca) => {
                                println!("RECEIVED COnnecte Ack PACKET");
    
                            },
                            networking::s2c_message::Msg::MoveAck(ma) => {
                                println!("RECEIVED move Ack PACKET");
    
                            },
                        }
                    }
                }
            }
            
        } else {
            let data = self.receive_c2s_packet();
            if data.is_some() {
                match data.unwrap().msg {
                    None => (), 
                    Some(msg) => {
                        match msg {
                            networking::c2s_message::Msg::Move(m) => {
                                let p = chess::util::Pos {
                                    x: (m.from_square % 8) as i8,
                                    y: (m.from_square / 8) as i8,
                                };
                                let pos = chess::util::Pos {
                                    x: (m.to_square % 8) as i8,
                                    y: (m.to_square / 8) as i8,
                                };
                                self.board.perform_move(p, pos, None);
                                println!("RECEIVED MOVE PACKET");
                                self.draw(ctx);
                            },
                            networking::c2s_message::Msg::ConnectRequest(_) => {
                                println!("RECEIVED connect request PACKET");
    
                            },
                        }
                    }
                }
            }
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
                    let scale = glam::Vec2::new((CELL_DIMENSIONS.0 as f32) / (img.width() as f32), (CELL_DIMENSIONS.1 as f32) / (img.height() as f32));
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
                Vec2::new((CELL_DIMENSIONS.0 as f32 / 2.0) + (m.x as i16 * CELL_DIMENSIONS.0) as f32, (CELL_DIMENSIONS.1 as f32 / 2.0) + (m.y as i16 * CELL_DIMENSIONS.1) as f32), 
                CELL_DIMENSIONS.0 as f32 / 10.0, 
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
                    println!("SENDING MOVE PACKET CLient");
                    self.state = State::Waiting;
                    self.send_c2s_packet(data);
                } else {
                    let data = S2cMessage {
                        msg: Some(networking::s2c_message::Msg::Move(networking::Move {
                            from_square: (p.x + p.y * 8) as u32,
                            to_square: (pos.x + pos.y * 8) as u32,
                            promotion: None,
                        })),
                    };
                    println!("SENDING MOVE PACKET Server");
                    self.state = State::Waiting;
                    self.send_s2c_packet(data);
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

    fn key_down_event(
            &mut self,
            ctx: &mut Context,
            input: ggez::input::keyboard::KeyInput,
            _repeated: bool,
        ) -> Result<(), ggez::GameError> {
            if input.keycode.unwrap() == ggez::input::keyboard::KeyCode::Return {
                let (stream, is_client) = get_stream();
                self.stream = Some(stream);
                self.is_client = is_client;
            }

            self.draw(ctx)
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

    let mut state = MainState::new(&mut ctx)?;

    state.draw(&mut ctx);
    event::run(ctx, events_loop, state)
}
