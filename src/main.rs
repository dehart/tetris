#![allow(non_snake_case)]

extern crate minifb;

use minifb::{Key, ScaleMode, Window, WindowOptions};
use rand::Rng;


fn main() {
   
	let mut tetris = Tetris {
		w: 12, h: 18, field: vec![0; 216], currentRotate: 0
	};

	tetris.start();
}

const nScreenWidth: usize = 16;
const nScreenHeight: usize = 22;

const BLACK: u32 = 0x00_00_00;
const GREEN: u32 = 0x00_ff_00;
const WHITE: u32 = 0xff_ff_ff;
const GREY: u32 = 0xaa_aa_aa;

struct Tetromino {
	color: u32,
	shape: u16
}

const tetrominos: [Tetromino; 7] = [
	Tetromino {color: 0x0B_AA_88, shape: 0b0010_0010_0010_0010}, // I
	Tetromino {color: 0x00_CC_99, shape: 0b0100_0100_0110_0000}, // L
	Tetromino {color: 0x00_DD_AA, shape: 0b0010_0010_0110_0000}, // J
	Tetromino {color: 0x00_EE_BB, shape: 0b0000_0110_0110_0000}, // O
	Tetromino {color: 0x00_00_CC, shape: 0b0100_0110_0010_0000}, // S
	Tetromino {color: 0x00_00_DD, shape: 0b0010_0110_0100_0000}, // Z
	Tetromino {color: 0x00_00_EE, shape: 0b0010_0110_0010_0000}, // T
];

struct Tetris {
	field: Vec<u32>,
	w: usize,
	h: usize,
	currentRotate: u8
}

impl Tetris {

	fn start(&mut self) {

		//self.field = vec![0; 216];
		// Create Screen Buffer
		let mut screen: Vec<u32> = vec![0; nScreenWidth * nScreenHeight];
		let mut window = Window::new(
			"Tetris - Press ESC to exit",
			nScreenWidth,
			nScreenHeight,
			WindowOptions {
				resize: false,
				scale_mode: ScaleMode::UpperLeft,
				scale: minifb::Scale::X32,
				..WindowOptions::default()
			},
		)
		.expect("Unable to create window");
	
		// Limit to max ~60 fps update rate
		window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
	
		for x in 0..self.w {// Board Boundary
			for y in 0..self.h {
				self.field[(y * self.w + x) as usize] = if x == 0 || x == self.w - 1 || y == self.h - 1 { GREY } else { BLACK };
			}
		}
	
		// Game logic
		let mut currentPiece: &Tetromino = &tetrominos[0];
		let mut nCurrentX: i32 = (self.w / 2) as i32;
		let mut nCurrentY = 0;
		let mut speed = 20;
		let mut speedCount = 0;
		let mut forceDown;
		let mut rotateHold = true;
		let mut rotate;
		let mut pieceCount = 0;
		let mut score = 0;
		let mut lines: Vec<usize> = vec![];
		let mut isGameOver = false;
	
		while window.is_open() && !window.is_key_down(Key::Escape) && !isGameOver { // Main Loop
	
			// Timing =======================
			std::thread::sleep(std::time::Duration::from_millis(50));  // Small Step = 1 Game Tick
			speedCount+=1;
			forceDown = speedCount == speed;
	
			// Input ========================
			rotate = false;
			// Handle player movement
			window.get_keys().map(|keys| {
				for t in keys {
					 match t {
						  Key::Space => rotate = true, // rotate
						  Key::Left => nCurrentX -= if self.does_piece_fit(currentPiece, nCurrentX - 1, nCurrentY) {1}else{0}, // left
						  Key::Down => nCurrentY += if self.does_piece_fit(currentPiece, nCurrentX as i32, nCurrentY + 1) {1}else{0}, // down
						  Key::Right => nCurrentX += if self.does_piece_fit(currentPiece, nCurrentX as i32 + 1, nCurrentY) {1}else{0},
						  _ => (),
					 }
				}  
			});
			
			// Game Logic ===================
	
			// Rotate, but latch to stop wild spinning
			if rotate {
				if rotateHold {
					self.currentRotate+=1;
					if !self.does_piece_fit(currentPiece, nCurrentX, nCurrentY) {
						self.currentRotate-=1;
					}

				}
				rotateHold = false;
			} else {
				rotateHold = true;
			}
	
			if forceDown {
				// Update difficulty every 50 pieces
				speedCount = 0;
				pieceCount+=1;
				if pieceCount % 50 == 0 {
					if speed >= 10 { speed-=1 };
				}
			
	
				// Test if piece can be moved down
				if self.does_piece_fit(&currentPiece, nCurrentX, nCurrentY + 1) {
					nCurrentY+=1; // It can, so do it!
				} else {

					// It can't! Lock the piece in place
					self.pos(currentPiece, |px,py| self.field.to_owned()[((nCurrentY + py) * self.w as i32 + (nCurrentX + px)) as usize] = WHITE);
					for px in 0..4_i32 {
						for py in 0..4_i32 {
							if currentPiece.shape & (1 << self.rotate(px, py)) != 0 {
								self.field[((nCurrentY + py) * self.w as i32 + (nCurrentX + px)) as usize] = currentPiece.color;
							}
						}
					}
	
					// Check for lines
					for py in 0..4_i32 {
						if nCurrentY + py  < self.h as i32 - 1 // not dead
						{
							let mut bLine = true;
							for px in 1..(self.w - 1) {
								bLine &= (self.field[((nCurrentY + py) * self.w as i32 + px as i32) as usize]) != BLACK;
							}
							if bLine
							{
								// Remove Line, maek green first
								for px in 1..(self.w - 1) {
									self.field[((nCurrentY + py) * self.w as i32 + px as i32) as usize] = GREEN;
								}
								lines.push((nCurrentY + py).try_into().unwrap());
	
							}						
						}
					}
	
					score += 25;
					if !lines.is_empty() { score += (1 << lines.len()) * 100; }
	
					// Pick New Piece
					nCurrentX = (self.w / 2) as i32;
					nCurrentY = 0;
					currentPiece = &tetrominos[rand::thread_rng().gen_range(0..7)];
					self.currentRotate = 0;
	
					// If piece does not fit straight away, game over!
					isGameOver = !self.does_piece_fit(currentPiece, nCurrentX, nCurrentY);
				}
			}
	
			// Display ======================
	
			// Draw Field
			for x in 0..self.w {
				for y in 0..self.h {
					let i = ((y + 2)*nScreenWidth + (x + 2)) as usize;
					screen[i] = self.field[y*self.w + x];
				}
			}
	
			// Draw Current Piece
			//self.pos(currentPiece, |px,py| screen.to_owned()[((nCurrentY + py + 2)*nScreenWidth as i32 + (nCurrentX as i32 + px + 2)) as usize] = currentPiece.color);
			for px in 0..4 {
				for py in 0..4 {
					//let n = self.rotate(px, py);
						if currentPiece.shape & (1 << self.rotate(px, py)) != 0 {
						//if tetromino[nCurrentPiece].chars().nth(self.rotate(px, py, nCurrentRotation) as usize).unwrap() != '.' {
							screen[((nCurrentY + py + 2)*nScreenWidth as i32 + (nCurrentX as i32 + px + 2)) as usize] = currentPiece.color;
						}
				}
			}
	
			// Draw score
			// Animate Line Completion
			if !lines.is_empty()
			{
				// Display Frame (cheekily to draw lines)
				window.update_with_buffer(&screen, nScreenWidth, nScreenHeight).unwrap();
				std::thread::sleep(std::time::Duration::from_millis(400)); // Delay a bit
	
				for v in lines.iter() {
					for px in 1..(self.w - 1) {
						for py in (1..*v+1).rev() {
							self.field[py * self.w + px] = self.field[(py - 1) * self.w + px];
						}
						self.field[px] = 0;
					}
				}
	
				lines.clear();
			}
	
			// Display Frame
			window.update_with_buffer(&screen, nScreenWidth, nScreenHeight).unwrap();
		}

		println!("Game Over!! Score: {}", score);
	}
	
	fn does_piece_fit(&self, piece: &Tetromino, nPosX: i32, nPosY: i32) -> bool 
	{
		// All Field cells >0 are occupied
		for px in 0..4_i32 {
			for py in 0..4_i32 {
				// Get index into piece
				let pi = self.rotate(px, py);
	
				// Get index into field
				let fi = (nPosY + py) * self.w as i32 + (nPosX + px);
	
				// Check that test is in bounds. Note out of bounds does
				// not necessarily mean a fail, as the long vertical piece
				// can have cells that lie outside the boundary, so we'll
				// just ignore them
				if nPosX + px >= 0 && nPosX + px < self.w as i32
				{
					if nPosY + py >= 0 && nPosY + py < self.h as i32
					{
						// In Bounds so do collision check
						if piece.shape & (1 << pi) != 0 && self.field[fi as usize] != BLACK {
							return false; // fail on first hit
						}
					}
				}
			}
		}
		return true;
	}

	fn pos<F>(&self, piece: &Tetromino, func: F) where F: Fn(i32, i32) {
		for px in 0..4 {
			for py in 0..4 {
				let n = self.rotate(px, py);
				if piece.shape & (1 << n) != 0 {
					func(px, py);
				}
			}
		}
	}

	fn rotate(&self, px: i32, py: i32) -> u32
	{
		(match self.currentRotate % 4
		{
			0 => py * 4 + px,		    // 0 degrees	
			1 => 12 + py - (px * 4),    // 90 degrees
			2 => 15 - (py * 4) - px,	// 180 degrees
			3 => 3 - py + (px * 4),	    // 270 degrees					
			_ => py * 4 + px,		    // 0 degrees	
		}) as u32
	}
}