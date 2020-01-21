use hal::{
    gpio::{*, gpiob::*},
    time::Instant,
};

use heapless::{consts::*, Vec};
use max7219::{*, connectors::*};
use wyhash::wyrng;
use crate::joystick::*;

#[derive(Debug, PartialEq, Clone, Copy)]
struct Point {
    x: u8,
    y: u8
}

struct Snake {
    body: Vec<Point, U64>, // TODO: Make size of snake (and map) configurable
    direction: Direction,
}

impl Snake {
    pub fn new(start_point: Point, start_direction: Direction) -> Self {
        let mut body = Vec::new();
        body.push(start_point).expect("Unable to add element to empty vec");

        Snake {
            body: body,
            direction: start_direction
        }
    }

    pub fn slither(&mut self, new_direction: Option<Direction>, ate_fruit: bool) {
        // Update the snake's direction if supplied
        if let Some(dir) = new_direction {
            if let Some(dir) = Snake::direction_conversion(dir) {
                // Don't let the snake turn 180 in on itself.
                if !dir.opposite(&self.direction) {
                    self.direction = dir;
                }
            }
        }

        // Given the current heading, we want to add a segment to the front of the snake
        let next_head = self.next_head(self.direction, self.head());
        self.body.reverse();
        if !self.is_full() {
            self.body.push(next_head).expect("Snake has grown too long");
        }
        self.body.reverse();

        if !ate_fruit {
            self.body.pop(); // Remove segment from tail of the snake
        }
    }

    pub fn to_array(&self) -> [u8; 8] {
        let mut world = [0, 0, 0, 0, 0, 0, 0, 0];
        for &p in self.body.iter() {
            world[p.y as usize] = world[p.y as usize] | (1 << p.x) as u8;
        }
        world
    }

    pub fn is_full(&self) -> bool {
        self.body.len() == self.body.capacity()
    }

    pub fn len(&self) -> usize {
        self.body.len()
    }

    pub fn head(&self) -> Point {
        self.body[0]
    }

    pub fn collided_with_tail(&self) -> bool {
        let head = self.head();
        for &body in self.body.iter().skip(1) {
            if body == head {
                return true
            }
        }
        false
    }

    fn next_head(&self, direction: Direction, current_head: Point) -> Point {
        let mut next = current_head;
        match direction {
            Direction::North => { next.y = Snake::bounded_subtract_one(current_head.y.into(), 8) as u8 },
            Direction::East  => { next.x = Snake::bounded_subtract_one(current_head.x.into(), 8) as u8 },
            Direction::South => { next.y = Snake::bounded_add_one(current_head.y.into(), 8) as u8 },
            Direction::West  => { next.x = Snake::bounded_add_one(current_head.x.into(), 8) as u8 },
            _ => panic!("Unhandled direction: {:?}", direction)
        }

        next
    }

    fn bounded_add_one(val: u32, bound: u32) -> u32 {
        (val + 1) % bound
    }

    fn bounded_subtract_one(val: u32, bound: u32) -> u32 {
        if val == 0 {
            bound - 1
        } else {
            val - 1
        }
    }

    fn direction_conversion(direction: Direction) -> Option<Direction> {
        // Keep processing as simple as possible by ignoring some points of the compass
        match direction {
            Direction::NorthWest | Direction::SouthEast | Direction::NorthEast | Direction::SouthWest => None,
            _ => Some(direction)
        }
    }
}

pub type DisplayConnector = PinConnector<PB8<Output<PushPull>>, PB9<Output<PushPull>>, PB10<Output<PushPull>>>;
pub struct Game {
    snake: Snake,
    fruit: Point,
    seed: Instant,
    pub display: MAX7219<DisplayConnector>,
    pub joystick: Joystick
}


impl Game {
    pub fn new(seed: Instant, mut display: MAX7219<DisplayConnector>, joystick: Joystick) -> Self {
        display.power_on().expect("Unable to turn on display");
        let mut game = Game {
            seed: seed,
            snake: Snake::new(Game::random_point(seed), Direction::West),
            fruit: Game::random_point(seed),
            display: display,
            joystick: joystick
        };
        game.render();
        game
    }

    pub fn tick(&mut self) -> Option<usize> {
        let ate_fruit: bool = self.snake.head() == self.fruit;
        self.snake.slither(self.joystick.direction().expect("Unable to get joystick direction"), ate_fruit);

        if self.snake.collided_with_tail() || self.snake.is_full() {
            self.render();
            return Some(self.snake.len())
        }

        if ate_fruit {
            self.fruit = Game::random_point(self.seed);
        }

        self.render();
        None
    }

    pub fn render(&mut self) {
        let mut world = self.snake.to_array();
        world[self.fruit.y as usize] = world[self.fruit.y as usize] | (1 << self.fruit.x) as u8;
        self.display.write_raw(0, &world).expect("Unable to render snake on display");
    }

    pub fn reset(&mut self){
        self.snake = Snake::new(Game::random_point(self.seed), Direction::West);
        self.fruit = Game::random_point(self.seed);
    }

    fn random_point(seed: Instant) -> Point {
        Point { x: wyrng(&mut (seed.elapsed() as u64)) as u8 % 8, y: wyrng(&mut (seed.elapsed() as u64)) as u8 % 8 }
    }
}
