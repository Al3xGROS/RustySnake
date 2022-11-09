extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

extern crate rand;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{Button, Key, EventLoop, ButtonEvent, ButtonState};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use std::collections::LinkedList;
use std::iter::FromIterator;

pub struct Game {
    gl: GlGraphics,
    rows: u32,
    cols: u32,
    snake: Snake,
    eaten: bool,
    square: u32,
    food: Food,
    score: u32,
}

impl Game {
    fn render(&mut self, args: &RenderArgs) {

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

        self.gl.draw(args.viewport(), |_c, gl| {
            graphics::clear(GREEN, gl);
        });

        self.snake.render(args);
        self.food.render(& mut self.gl, args, self.square);
    }

    fn update(&mut self, _args: &UpdateArgs) -> bool {
        if !self.snake.update(self.eaten, self.cols, self.rows) {
            return false;
        }

        if self.eaten {
            self.score += 1;
            self.eaten = false;
        }

        self.eaten = self.food.update(&self.snake);
        if self.eaten {
            use rand::Rng;
            use rand::thread_rng;
            // try my luck
            let mut r = thread_rng();
            loop {
                let new_x = r.gen_range(0..self.cols);
                let new_y = r.gen_range(0..self.rows);
                if !self.snake.is_collide(new_x, new_y) {
                    self.food = Food { x: new_x, y: new_y };
                    break;
                }
            }
        }

        true
    }

    fn pressed(&mut self, button: &Button) {
        let last_direction = self.snake.direction.clone();
        self.snake.direction = match button {
            &Button::Keyboard(Key::Up) if last_direction != Direction::DOWN => Direction::UP,
            &Button::Keyboard(Key::Down) if last_direction != Direction::UP => Direction::DOWN,
            &Button::Keyboard(Key::Left) if last_direction != Direction::RIGHT => Direction::LEFT,
            &Button::Keyboard(Key::Right) if last_direction != Direction::LEFT => Direction::RIGHT,
            _ => last_direction,            
        }
    } 
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}


pub struct Snake {
    gl: GlGraphics,
    snake_parts: LinkedList<SnakePiece>,
    width: u32,
    direction: Direction,
}

#[derive(Clone, Copy, PartialEq)]
pub struct SnakePiece(u32, u32);

impl Snake {
    fn render(&mut self, args: &RenderArgs) {

        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let squares: Vec<graphics::types::Rectangle> = self.snake_parts
            .iter()
            .map(|p| SnakePiece(p.0 * self.width, p.1 * self.width))
            .map(|p| graphics::rectangle::square(p.0 as f64, p.1 as f64, self.width as f64))
            .collect();

        self.gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            squares
                .into_iter()
                .for_each(|square| graphics::rectangle(RED, square, transform, gl));
        })
    }

    fn update(&mut self, eaten: bool, rows: u32, cols: u32) -> bool {
        let mut new_front: SnakePiece = (*self.snake_parts.front().expect("No front of the snake found")).clone();

        if (self.direction == Direction::UP && new_front.1 == 0)
            || (self.direction == Direction::LEFT && new_front.0 == 0)
            || (self.direction == Direction::DOWN && new_front.1 == rows - 1)
            || (self.direction == Direction::RIGHT && new_front.0 == cols - 1)
        {
            return false;
        }

        match self.direction {
            Direction::UP => new_front.1 -= 1,
            Direction::DOWN => new_front.1 += 1,
            Direction::LEFT => new_front.0 -= 1,
            Direction::RIGHT => new_front.0 += 1,
        }

        if !eaten {
            self.snake_parts.pop_back();
        }

        if self.is_collide(new_front.0, new_front.1) {
            return false;
        }

        self.snake_parts.push_front(new_front);
        true
    }

    fn is_collide(&mut self, x: u32, y: u32) -> bool {
        self.snake_parts.iter().any(|p| x == p.0 && y == p.1)
    }
}


pub struct Food {
    x: u32,
    y: u32,
}

impl Food {
    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs, width: u32) {

        const BLACK: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

        let x = self.x * width;
        let y = self.y * width;

        let square = graphics::rectangle::square(x as f64, y as f64, width as f64);

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            graphics::rectangle(BLACK, square, transform, gl)
        });
    }

    // true if the snake ate the food at this update
    fn update(&mut self, s: &Snake) -> bool {
        let front = s.snake_parts.front().unwrap();
        if front.0 == self.x && front.1 == self.y {
            true
        } else {
            false
        }
    }
}



fn main() {
    let opengl = OpenGL::V3_2;

    const ROWS: u32 = 20;
    const COLS: u32 = 20;
    const SQUARE: u32 = 20;

    let width = COLS * SQUARE;
    let height = ROWS * SQUARE;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("RustySnake", [width, height])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut game = Game {
        gl: GlGraphics::new(opengl),
        rows: ROWS,
        cols: COLS,
        snake: Snake { 
            gl: GlGraphics::new(opengl), 
            snake_parts: LinkedList::from_iter((vec![SnakePiece(COLS / 2, ROWS / 2)]).into_iter()), 
            width: SQUARE, 
            direction: Direction::DOWN},
        eaten: false,
        square: SQUARE,
        food: Food {x: 1, y: 1},
        score: 0,
    };

    let mut events = Events::new(EventSettings::new()).ups(10);
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(u) = e.update_args() {
            if !game.update(&u) {
                break;
            }
        }

        if let Some(k) = e.button_args() {
            if k.state == ButtonState::Press {
                game.pressed(&k.button);
            }
        }
    }
    println!("Congratulations, your score was: {}", game.score);
}
