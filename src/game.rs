// #![no_std]

// use core::str;
// use core::fmt::{self, Write};
use hal::{
    prelude::*,
    gpio::{*, gpiob::*}
};

use heapless::{consts::*, Vec};
use crate::joystick::Direction;


use max7219::*;
use max7219::connectors::*;
type CONNECTOR = PinConnector<PB8<Output<PushPull>>, PB9<Output<PushPull>>, PB10<Output<PushPull>>>;


struct Snake {
    body: Vec<(u8, u8), U64>, // TODO: Make size of snake (and map) configurable
    direction: Direction,
}

impl Snake {
    pub fn new() -> Self {
        let mut body = Vec::new();
        body.push((4, 4)).expect("Unable to add element to empty vec"); // TODO: Randomise the snake's head
        let direction = Direction::North;   // TODO: Randomise snake direction

        Snake {
            body: body,
            direction: direction
        }
    }

    pub fn slither(&mut self, new_direction: Option<Direction>, eat: bool) {
        // Update the snake's direction if supplied
        if let Some(dir) = new_direction {
            self.direction = dir;
        }

        // Given the current heading, we want to add a segment to the front of the snake
        let head = self.next_head();
        self.body.reverse();
        if !self.is_full() {
            self.body.push(head).expect("Snake has grown too long");
        }
        self.body.reverse();

        if !eat {
            self.body.pop(); // Remove segment from tail of the snake
        }
    }

    pub fn to_array(&self) -> [u8; 8] {
        let mut a = [0, 0, 0, 0, 0, 0, 0, 0];

        for &(x, y) in self.body.iter() {
            a[y as usize] = a[y as usize] | (1 << x) as u8;
        }

        a
    }

    pub fn is_full(&self) -> bool {
        self.body.len() == self.body.capacity()
    }

    pub fn head(&self) -> (u8, u8) {
        self.body[0]
    }

    pub fn collided_with_tail(&self) -> bool {
        let head = self.body[0];

        for &body in self.body.iter().skip(1) {
            if body == head {
                return true
            }
        }

        false
    }

    fn next_head(&self) -> (u8, u8) {
        let mut next = self.body[0];
        match self.direction {
            Direction::North | Direction::NorthWest => { next.1 = Snake::bounded_subtract_one(self.body[0].1.into(), 8) as u8 },
            // Direction::NorthEast   => { next.1 = Snake::bounded_subtract_one(x_y.1, 8); next.0 = Snake::bounded_subtract_one(x_y.0, 8) },
            Direction::East | Direction::NorthEast  => { next.0 = Snake::bounded_subtract_one(self.body[0].0.into(), 8) as u8 },
            // Direction::SouthEast   => { next.1 = Snake::bounded_add_one(x_y.1, 8); next.0 = Snake::bounded_subtract_one(x_y.0, 8) },
            Direction::South | Direction::SouthEast => { next.1 = Snake::bounded_add_one(self.body[0].1.into(), 8) as u8 },
            // Direction::SouthWest   => { next.0 = Snake::bounded_add_one(x_y.0, 8); next.1 = Snake::bounded_add_one(x_y.1, 8); },
            Direction::West | Direction::SouthWest  => { next.0 = Snake::bounded_add_one(self.body[0].0.into(), 8) as u8 },
            // Direction::NorthWest   => { next.0 = Snake::bounded_add_one(x_y.0, 8); next.1 = Snake::bounded_subtract_one(x_y.1, 8); },
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
}



pub struct Game {
    snake: Snake,
    fruit: (u8, u8),
    pub display: MAX7219<CONNECTOR>
}


impl Game {
    pub fn new(mut display: MAX7219<CONNECTOR>) -> Self {
        display.power_on().expect("Unable to turn on display");
        let mut game = Game {
            snake: Snake::new(),
            fruit: (0, 1), // TODO: Randomise fruit location
            display: display,
        };
        game.render();
        game
    }

    pub fn tick(&mut self, direction: Option<Direction>) -> bool {
        let eat: bool = self.snake.head() == self.fruit;
        self.snake.slither(direction, eat);
        if self.snake.collided_with_tail() {
            self.display.power_off().expect("Unable to turn off display");
            return false // Turn into enum
        }
        self.render();
        true
    }

    pub fn render(&mut self) {
        let mut world = self.snake.to_array();
        world[self.fruit.1 as usize] = world[self.fruit.1 as usize] | (1 << self.fruit.0);
        self.display.write_raw(0, &world).expect("Unable to render snake on display");
    }
}
