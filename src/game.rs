use spin;
use crate::{print, vga_buffer, get_random_u32};
use pc_keyboard::{KeyEvent, KeyCode, KeyState};
use lazy_static::lazy_static;
use x86_64::instructions::interrupts::without_interrupts;

pub const MAX_SNAKE_LENGTH: u32 = 100;
pub const SNAKE_START_ROW: u32 = 11;
pub const SNAKE_START_COLUMN: u32 = 0;
pub const SNAKE_START_LENGTH: u32 = 3;
pub const GAME_ROWS: u32 = 23;
pub const GAME_COLUMNS: u32 = 78;

lazy_static! {
    pub static ref GAME: spin::Mutex<Game> = spin::Mutex::new(Game::new());
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
pub enum Difficulty {
    Easy = 10,    
    Medium = 5,
    Hard = 3,
}

impl Difficulty {
    fn as_u32(&self) -> u32 {
        *self as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Position(u32, u32);

#[derive(Debug, Clone, Copy)]
struct Snake {
    positions: [Position; MAX_SNAKE_LENGTH as usize],
    length: u32,
    direction: Direction,
    treat: Option<Position>,
}

impl Snake {
    pub fn new() -> Snake {
        let mut snake = Snake {
            positions: [Position(SNAKE_START_ROW, SNAKE_START_COLUMN); MAX_SNAKE_LENGTH as usize],
            length: 1,
            direction: Direction::Right,
            treat: None,
        };

        // start with initial size
        for _ in 1..SNAKE_START_LENGTH {
            snake.grow().unwrap();
        }

        snake
    }

    pub fn head(&self) -> Position {
        self.body()[(self.length as usize) - 1]
    }

    pub fn body(&self) -> &[Position] {
        &self.positions[0..(self.length as usize)]
    }

    fn next_step(&self) -> Option<Position> {
        let Position(head_row, head_column) = self.head();
        match self.direction {
            Direction::Right => {
                if head_column == (GAME_COLUMNS - 1) {
                    None
                } else {
                    Some(Position(head_row, head_column+1))
                }
            },
            Direction::Down => {
                if head_row == (GAME_ROWS - 1) {
                    None
                } else {
                    Some(Position(head_row+1, head_column))
                }
            },
            Direction::Left => {
                if head_column == 0 {
                    None
                } else {
                    Some(Position(head_row, head_column-1))
                }
            },
            Direction::Up => {
                if head_row == 0 {
                    None
                } else {
                    Some(Position(head_row - 1, head_column))
                }
            }
        }
    }

    fn grow(&mut self) -> Result<(), &str> {
        if self.is_at_maximum_length() {
            Err("Snake is at maximum length")
        } else if let Some(position) = self.next_step() {
            if !self.occupies(position) {
                self.positions[self.length as usize] = position;
                self.length += 1;
                if self.treat == Some(position) { self.treat = None };
                Ok(())
            } else {
                Err("Your snake bit itself!")
            }
        } else {
            Err("Bounds reached!")    
        }
    }

    pub fn move_snake(&mut self) -> Result<(), &str> {
        if let Some(next_head) = self.next_step() {
            if Some(next_head) == self.treat {
                self.grow()?;
                Ok(())
            } else if !self.occupies(next_head) {
                for i in 0..self.length-1 {
                    self.positions[i as usize] = self.positions[(i+1) as usize];
                }
                self.positions[(self.length-1) as usize] = next_head;
                Ok(())
            } else {
                Err("Your snake bit itself!")
            }
        } else {
            Err("Bounds reached!")
        }
    }

    pub fn turn(&mut self, new_direction: Direction) {
        match (self.direction, new_direction) {
            (Direction::Right, Direction::Left) => (),
            (Direction::Left, Direction::Right) => (),
            (Direction::Up, Direction::Down) => (),
            (Direction::Down, Direction::Up) => (),
            _ => self.direction = new_direction,
        }
    }

    pub fn occupies(&self, position: Position) -> bool {
        self.body().contains(&position)
    }

    pub fn has_treat(&self, position: Position) -> bool {
        if let Some(treat_position) = self.treat {
            treat_position == position
        } else {
            false
        }
    }

    pub fn is_at_maximum_length(&self) -> bool {
        return self.length == MAX_SNAKE_LENGTH
    }
}

pub struct Game {
    difficulty: Difficulty,
    counter: spin::Mutex<u32>,
    snake: Snake,
    game_over: bool,
    next_move: Option<Direction>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            difficulty: Difficulty::Hard,
            counter: spin::Mutex::new(0),
            snake: Snake::new(),
            game_over: false,
            next_move: None,
        }
    }

    pub fn refresh_frame(&mut self) {
        let counter = self.fetch_add_counter();
        if counter % self.difficulty.as_u32() == 0 && !self.game_over {
            self.make_treat();

            if let Some(direction) = self.next_move {
                self.snake.turn(direction);
            }

            match self.snake.move_snake() {
                Ok(()) => self.print(),
                Err(msg) => {
                    self.game_over = true;
                    Game::print_msg(msg);
                },
            }
            self.next_move = None;
        }
    }

    pub fn accept_keyboard_input(&mut self, key_event: KeyEvent) {
        if let KeyEvent { code, state: KeyState::Down } = key_event {
            self.next_move = match code {
                KeyCode::ArrowUp => Some(Direction::Up),
                KeyCode::ArrowRight => Some(Direction::Right),
                KeyCode::ArrowDown => Some(Direction::Down),
                KeyCode::ArrowLeft => Some(Direction::Left),
                _ => self.next_move, // preserve old value
            }
        }
    }

    fn make_treat(&mut self) {
        if !self.snake.is_at_maximum_length() {
            match self.snake.treat {
                Some(_) => (),
                None => {
                    loop {
                        let random_row: u32 = get_random_u32() % GAME_ROWS;
                        let random_col: u32 = get_random_u32() % GAME_COLUMNS;

                        let position = Position(random_row, random_col);

                        if !self.snake.occupies(position) {
                            self.snake.treat = Some(Position(random_row, random_col));
                            break;
                        }
                    }
                },
            }
        }
    }

    fn print(&self) {
        let snake_head = self.snake.head();
        let mut array: [[u8; 80]; 25] = [[b' '; 80]; 25];

        // draw border
        for column in 0..80 {
            array[0][column] = b'#';
            array[24][column] = b'#';
        }
        for row in 0..25 {
            array[row][0] = b'#';
            array[row][79] = b'#';
        }

        // draw inner
        for row in 0..GAME_ROWS {
            for column in 0..GAME_COLUMNS {
                let position = Position(row, column);
                let character = {
                    if position == snake_head {
                        match self.snake.direction {
                            Direction::Right => b'>',
                            Direction::Down => b'v',
                            Direction::Left => b'<',
                            Direction::Up => b'^',
                        }
                    } else if self.snake.has_treat(position) {
                        b'x'
                    } else if self.snake.occupies(position) {
                        b'+'
                    } else {
                        b' '
                    }
                };
                array[(row+1) as usize][(column+1) as usize] = character;
            }
        }

        without_interrupts(|| {
            let mut writer = vga_buffer::WRITER.lock();
            unsafe {
                writer.write_full_screen(array);
            }
        });
    }

    fn print_msg(msg: &str) {
        for _ in 0..25 {
            print!("\n");
        }
        print!("{:^80}", msg);
        for _ in 0..13 {
            print!("\n");
        }
    }

    fn fetch_add_counter(&mut self) -> u32 {
        let mut old_counter: u32 = 0;
        without_interrupts(|| {
            let mut counter = self.counter.lock();
            old_counter = *counter;
            *counter += 1;
        });

        old_counter
    }
}
