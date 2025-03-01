extern crate termion;

use std::io::{stdout, Read, Write};
use termion::async_stdin;
use termion::raw::IntoRawMode;

const TARGET_FPS: u8 = 60;
const FRAME_DURATION: std::time::Duration =
    std::time::Duration::from_millis(1000 / TARGET_FPS as u64);
const MOVE_DURATION: std::time::Duration = std::time::Duration::from_millis(150);

trait Render {
    fn render(
        &self,
        screen: &mut termion::raw::RawTerminal<std::io::Stdout>,
    ) -> Result<(), std::io::Error>;
}

#[derive(PartialEq)]
enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, PartialEq)]
struct Coordinate(u16, u16);

struct Food {
    location: Coordinate,
}

struct Player {
    move_direction: MoveDirection,
    segments: std::collections::VecDeque<Coordinate>,
}

impl Player {
    fn new() -> Self {
        let mut player = Player {
            move_direction: MoveDirection::Right,
            segments: std::collections::VecDeque::new(),
        };

        for i in 1..5 {
            player.segments.push_front(Coordinate(i, 1));
        }

        player
    }

    fn change_direction(&mut self, new_direction: MoveDirection) {
        if match new_direction {
            MoveDirection::Up => self.move_direction == MoveDirection::Down,
            MoveDirection::Down => self.move_direction == MoveDirection::Up,
            MoveDirection::Left => self.move_direction == MoveDirection::Right,
            MoveDirection::Right => self.move_direction == MoveDirection::Left,
        } {
            return;
        }

        self.move_direction = new_direction;
    }

    fn check_collisions(&self, coord: &Coordinate, (screen_w, screen_h): &(u16, u16)) -> bool {
        if coord.0 > *screen_w || coord.1 > *screen_h || coord.0 < 1 || coord.1 < 1 {
            return true;
        }

        self.segments.contains(coord)
    }

    fn elongate(&mut self, screen_size: &(u16, u16)) {
        let last_segment = *self.segments.back().unwrap();

        let direction: &MoveDirection = if self.segments.len() >= 2 {
            let second_last = self.segments.iter().nth_back(1).unwrap();

            match (
                (last_segment.0 as i32 - second_last.0 as i32),
                (last_segment.1 as i32 - second_last.1 as i32),
            ) {
                (1, 0) => &MoveDirection::Right,
                (-1, 0) => &MoveDirection::Left,
                (0, 1) => &MoveDirection::Down,
                (0, -1) => &MoveDirection::Up,
                _ => panic!("This shouldn't happen. Nonexisting movement direction."),
            }
        } else {
            &self.move_direction
        };
        let new_segment = match direction {
            MoveDirection::Up => Coordinate(last_segment.0, last_segment.1 - 1),
            MoveDirection::Down => Coordinate(last_segment.0, last_segment.1 + 1),
            MoveDirection::Left => Coordinate(last_segment.0 - 1, last_segment.1),
            MoveDirection::Right => Coordinate(last_segment.0 + 1, last_segment.1),
        };

        if !self.check_collisions(&new_segment, screen_size) {
            self.segments.push_back(new_segment);
        }
    }

    fn update_pos(&mut self, screen_size: &(u16, u16)) {
        let head = &self.segments[0];

        let new_coord = match self.move_direction {
            MoveDirection::Up => Coordinate(head.0, head.1 - 1),
            MoveDirection::Down => Coordinate(head.0, head.1 + 1),
            MoveDirection::Left => Coordinate(head.0 - 1, head.1),
            MoveDirection::Right => Coordinate(head.0 + 1, head.1),
        };

        if self.check_collisions(&new_coord, screen_size) {
            return;
        }

        self.segments.push_front(new_coord);
        self.segments.pop_back();
    }
}

impl Render for Player {
    fn render(
        &self,
        screen: &mut termion::raw::RawTerminal<std::io::Stdout>,
    ) -> Result<(), std::io::Error> {
        for (index, Coordinate(x, y)) in self.segments.iter().enumerate() {
            let color = match index {
                0 => termion::color::Rgb(0, 255, 0),
                _ => termion::color::Rgb(255, 255, 255),
            };

            if let Err(e) = write!(
                screen,
                "{}{} {}",
                termion::cursor::Goto(*x, *y),
                termion::color::Bg(color),
                termion::color::Bg(termion::color::Reset),
            ) {
                return Err(e);
            }
        }
        Ok(())
    }
}

impl Food {
    fn new(screen_size: &(u16, u16), player: &Player) -> Self {
        Food {
            location: random_location(screen_size, player),
        }
    }

    fn check_eaten(&mut self, screen_size: &(u16, u16), player: &mut Player) {
        if *player.segments.front().unwrap() != self.location {
            return;
        };

        self.location = random_location(screen_size, player);
        player.elongate(screen_size);
    }
}
impl Render for Food {
    fn render(
        &self,
        screen: &mut termion::raw::RawTerminal<std::io::Stdout>,
    ) -> Result<(), std::io::Error> {
        write!(
            screen,
            "{}{}{}'{}",
            termion::cursor::Goto(self.location.0, self.location.1),
            termion::color::Bg(termion::color::Rgb(255, 0, 0)),
            termion::color::Fg(termion::color::Rgb(0, 0, 0)),
            termion::color::Bg(termion::color::Reset),
        )
    }
}

fn random_location(screen: &(u16, u16), player: &Player) -> Coordinate {
    let mut x = 0;
    let mut y = 0;

    while player.check_collisions(&Coordinate(x, y), screen) {
        x = rand::random_range(1..screen.0);
        y = rand::random_range(1..screen.1);
    }

    Coordinate(x, y)
}

fn clear(screen: &mut termion::raw::RawTerminal<std::io::Stdout>) -> Result<(), std::io::Error> {
    write!(
        screen,
        "{}{}",
        termion::cursor::Goto(1, 1),
        termion::clear::All
    )
}

fn main() {
    let mut screen = stdout().into_raw_mode().unwrap();
    let mut stdin = async_stdin().bytes();
    let mut screen_size = termion::terminal_size().unwrap();
    clear(&mut screen).unwrap();

    let mut player = Player::new();
    let mut food = vec![
        Food::new(&screen_size, &player),
        Food::new(&screen_size, &player),
        Food::new(&screen_size, &player),
        Food::new(&screen_size, &player),
    ];

    let mut prev_frame_time = std::time::Instant::now();
    let mut prev_move_update = std::time::Instant::now();
    'game: loop {
        screen_size = termion::terminal_size().unwrap();

        // Clear screen
        clear(&mut screen).unwrap();

        // Input handling
        while let Some(Ok(b)) = stdin.next() {
            write!(screen, "{}{}", termion::cursor::Goto(2, screen_size.1), b).unwrap();
            match b {
                113 => break 'game,
                97 => player.elongate(&screen_size),
                _ => {}
            };

            player.change_direction(match b {
                104 => MoveDirection::Left,
                107 => MoveDirection::Up,
                106 => MoveDirection::Down,
                108 => MoveDirection::Right,
                _ => continue,
            })
        }

        // Updating player position
        if prev_move_update.elapsed() > MOVE_DURATION {
            prev_move_update = std::time::Instant::now();
            player.update_pos(&screen_size)
        };

        for i in &mut food {
            // Checking if eaten
            i.check_eaten(&screen_size, &mut player);

            // Rendering
            i.render(&mut screen).unwrap();
        }
        player.render(&mut screen).unwrap();
        write!(screen, "{}", termion::cursor::Goto(1, screen_size.1)).unwrap();

        // Flushing to screen
        screen.flush().unwrap();

        // Limiting FPS
        {
            let frame_time = std::time::Instant::now() - prev_frame_time;
            if frame_time < FRAME_DURATION {
                std::thread::sleep(FRAME_DURATION - frame_time);
            }
            prev_frame_time = std::time::Instant::now();
        }
    }
}
