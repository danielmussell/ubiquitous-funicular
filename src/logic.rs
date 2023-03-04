// Welcome to
// __________         __    __  .__                               __
// \______   \_____ _/  |__/  |_|  |   ____   ______ ____ _____  |  | __ ____
//  |    |  _/\__  \\   __\   __\  | _/ __ \ /  ___//    \\__  \ |  |/ // __ \
//  |    |   \ / __ \|  |  |  | |  |_\  ___/ \___ \|   |  \/ __ \|    <\  ___/
//  |________/(______/__|  |__| |____/\_____>______>___|__(______/__|__\\_____>
//
// This file can be a nice home for your Battlesnake logic and helper functions.
//
// To get you started we've included code to prevent your Battlesnake from moving backwards.
// For more info see docs.battlesnake.com

use log::info;
use rand::seq::SliceRandom;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{Battlesnake, Board, Coord, Direction, Game};

const BOARD_SIZE: usize = 11;
const DENSE_BOARD_LENGTH: usize = BOARD_SIZE + 2;
const DENSE_BOARD_SIZE: usize = DENSE_BOARD_LENGTH * 2;
const PLAYER_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DenseBoard<T>
where
  T: Clone + Copy,
{
  board: [T; (BOARD_SIZE + 2) * (BOARD_SIZE + 2)],
}

impl<T> DenseBoard<T>
where
  T: Clone + Copy,
{
  fn init(default: T) -> DenseBoard<T> {
    DenseBoard {
      board: [default; (BOARD_SIZE + 2) * (BOARD_SIZE + 2)],
    }
  }

  fn get_xy(&self, x: isize, y: isize) -> T {
    self.board[(y + 1) as usize * (BOARD_SIZE + 2) + (x + 1) as usize]
  }

  fn get_xy_mut(&mut self, x: usize, y: usize) -> &mut T {
    &mut self.board[(y + 1) * (BOARD_SIZE + 2) + (x + 1)]
  }

  fn get_coord(&self, c: Coord) -> T {
    self.get_xy(c.x as isize, c.y as isize)
  }

  fn get_coord_mut(&mut self, c: Coord) -> &mut T {
    self.get_xy_mut(c.x as usize, c.y as usize)
  }
}

struct Node {
  turn: i32,
  area: DenseBoard<i32>,
  heads: [Coord; PLAYER_COUNT],
}

impl Node {
  fn new(turn: i32, board: &Board) -> Node {
    let mut area = DenseBoard::init(0);
    Node {
      turn: turn as i32,
      area,
      heads: [Coord { x: 0, y: 0 }; PLAYER_COUNT],
    }
  }
}

// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
pub fn info() -> Value {
  info!("INFO");

  return json!({
      "apiversion": "1",
      "author": "", // TODO: Your Battlesnake Username
      "color": "#e9ecef", // TODO: Choose color
      "head": "default", // TODO: Choose head
      "tail": "default", // TODO: Choose tail
  });
}

// start is called when your Battlesnake begins a game
pub fn start(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
  info!("GAME START");
}

// end is called when your Battlesnake finishes a game
pub fn end(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
  info!("GAME OVER");
}

fn voronoi(node: &Node) -> [i32; PLAYER_COUNT] {
  const MAGIC_NOT_OWNED: i32 = -1;
  let mut owned_by = DenseBoard::init(MAGIC_NOT_OWNED);
  let mut voronoi_scores = [0; PLAYER_COUNT];

  for (i, head) in node.heads.iter().enumerate() {
    *owned_by.get_coord_mut(*head) = i as i32;
  }

  let mut not_done = true;

  while not_done {
    for x in 0..(BOARD_SIZE as i32) {
      for y in 0..(BOARD_SIZE as i32) {
        not_done = false;

        let coord = Coord { x, y };
        if node.area.get_coord(coord) - node.turn > 0 {
          continue;
        }

        let tests = [
          Coord { x: x - 1, y },
          Coord { x: x + 1, y },
          Coord { x, y: y - 1 },
          Coord { x, y: y + 1 },
        ];

        for test in tests.iter() {
          if node.area.get_coord(*test) - node.turn < 0
            && owned_by.get_coord(*test) != MAGIC_NOT_OWNED
          {
            *owned_by.get_coord_mut(coord) = owned_by.get_coord(*test);
            voronoi_scores[node.area.get_coord(*test) as usize] += 1;
          }
          not_done = true;
        }
      }
    }
  }

  return voronoi_scores;
}

// move is called on every turn and returns your next move
// Valid moves are "up", "down", "left", or "right"
// See https://docs.battlesnake.com/api/example-move for available data
pub fn get_move(_game: &Game, turn: &i32, _board: &Board, you: &Battlesnake) -> Option<Direction> {
  // We've included code to prevent your Battlesnake from moving backwards
  let my_head = &you.body[0]; // Coordinates of your head
  let my_neck = &you.body[1]; // Coordinates of your "neck"

  let mut is_move_safe: HashMap<_, _> = vec![
    (Direction::Up, my_neck.y <= my_head.y),
    (Direction::Down, my_neck.y >= my_head.y),
    (Direction::Left, my_neck.x >= my_head.x),
    (Direction::Right, my_neck.x <= my_head.x),
  ]
  .into_iter()
  .collect();

  // TODO: Step 1 - Prevent your Battlesnake from moving out of bounds
  // let board_width = &board.width;
  // let board_height = &board.height;

  // TODO: Step 2 - Prevent your Battlesnake from colliding with itself
  // let my_body = &you.body;

  // TODO: Step 3 - Prevent your Battlesnake from colliding with other Battlesnakes
  // let opponents = &board.snakes;

  // Are there any safe moves left?
  let safe_moves = is_move_safe
    .into_iter()
    .filter(|&(_, v)| v)
    .map(|(k, _)| k)
    .collect::<Vec<_>>();

  // Choose a random move from the safe ones
  let chosen = safe_moves.choose(&mut rand::thread_rng()).unwrap().clone();

  // TODO: Step 4 - Move towards food instead of random, to regain health and survive longer
  // let food = &board.food;

  info!("MOVE {}: {:?}", turn, chosen);
  return Some(chosen);
}
