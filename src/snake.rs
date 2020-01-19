// #![no_std]

// use core::str;
// use core::fmt::{self, Write};
use heapless::{consts::*, Vec, spsc::Queue};

struct Snake {
    body: Vec<(u8, u8), U64>, // TODO: Slight hack... we know how big our display is
    direction: u8,
}


pub struct Game {
    snake: Snake,
    fruit: (u8, u8),
}


impl Game {
    pub fn new() -> Self {
        let mut vec = Vec::new();
        vec.push((0, 0)).unwrap(); // TODO: Randomise starting location

        Game {
            snake: Snake { body: vec, direction: 1 }, // TODO: Randomise direction
            fruit: (0, 1),                 // TODO: Randomise fruit location
        }
    }

    pub fn tick(&mut self) {
        // The head of the snake is the first element in this array.
        // We pop the last segment from the tail and then add it to the front.
        self.snake.body.pop();
        let mut head = self.snake.body[0];
        //TODO: Update head with direction
        head.0 += 1;

        self.snake.body.reverse();
        self.snake.body.push(head).unwrap(); // TODO: don't ignore
        self.snake.body.reverse();
    }
}
