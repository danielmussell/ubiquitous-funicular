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
use smallvec::SmallVec;
use std::{collections::HashMap, convert::TryInto};

use crate::{Battlesnake, Board, Coord, Direction, Game};

const BOARD_SIZE: usize = 11;
const DENSE_BOARD_LENGTH: usize = BOARD_SIZE + 2;
const DENSE_BOARD_SIZE: usize = DENSE_BOARD_LENGTH * 2;
const PLAYER_COUNT: usize = 2;

const HAS_FRUIT: i32 = -16;

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

  fn get_xy_mut(&mut self, x: isize, y: isize) -> &mut T {
    &mut self.board[(y + 1) as usize * (BOARD_SIZE + 2) + (x + 1) as usize]
  }

  fn get_coord(&self, c: Coord) -> T {
    self.get_xy(c.x as isize, c.y as isize)
  }

  fn get_coord_mut(&mut self, c: Coord) -> &mut T {
    self.get_xy_mut(c.x as isize, c.y as isize)
  }
}

#[derive(Copy, Clone)]
struct Node {
  turn: i32,
  board: DenseBoard<i32>,
  heads: [Coord; PLAYER_COUNT],
  lengths: [i32; PLAYER_COUNT],
  our_health: i32,
}

impl Node {
  fn new(turn: i32, board: DenseBoard<i32>, heads: [Coord; PLAYER_COUNT], our_health: i32) -> Node {
    Node {
      turn,
      board,
      heads,
      lengths: [0; PLAYER_COUNT],
      our_health,
    }
  }

  /// True iff snake with give index can collide with a wall
  fn is_head_colliding_wall(&self, snake_idx: usize) -> bool {
    self.heads[snake_idx].x <= 0
      || self.heads[snake_idx].x >= (BOARD_SIZE - 1) as i32
      || self.heads[snake_idx].y <= 0
      || self.heads[snake_idx].y >= (BOARD_SIZE - 1) as i32
  }

  fn apply_move(&self, snake_idx: usize, direction: Direction) -> Node {
    let mut new_node = self.clone();
    if new_node.is_head_colliding_wall(snake_idx) {
      return self.clone();
    }
    match direction {
      Direction::Up => new_node.heads[snake_idx].y -= 1,
      Direction::Down => new_node.heads[snake_idx].y += 1,
      Direction::Left => new_node.heads[snake_idx].x -= 1,
      Direction::Right => new_node.heads[snake_idx].x += 1,
    }
    // println!("head {:?} {:?}", snake_idx, new_node.heads[snake_idx]);
    *new_node.board.get_coord_mut(self.heads[snake_idx]) = self.lengths[snake_idx] + self.turn;
    // if new_node.board.get_coord(new_node.heads[snake_idx]) == HAS_FRUIT {
    //   new_node.our_health = 100;
    // }
    new_node
  }

  fn is_head_colliding_snake(&self, snake_idx: usize) -> bool {
    if self.is_head_colliding_wall(snake_idx) {
      return true;
    }

    let Coord { x, y } = self.heads[snake_idx];
    self.board.get_xy((x - 1) as isize, y as isize) > self.turn
      || self.board.get_xy((x + 1) as isize, y as isize) > self.turn
      || self.board.get_xy(x as isize, (y - 1) as isize) > self.turn
      || self.board.get_xy(x as isize, (y + 1) as isize) > self.turn;
    self.board.get_xy(x as isize, y as isize) > self.turn
  }

  fn apply_move_array(&self, directions: &SmallVec<[Direction; (PLAYER_COUNT - 1)]>) -> Node {
    let mut node = self.apply_move(1, directions[0]);
    for i in 2..PLAYER_COUNT {
      node = self.apply_move(i, directions[i - 1]);
    }
    node.turn += 1;
    node.our_health -= 1;
    node
  }

  fn evaluate(&self) -> i32 {
    if self.is_head_colliding_wall(0) || self.is_head_colliding_snake(0) || self.our_health <= 2 {
      -1000000000 + self.turn
    } else {
      2 * voronoi(&self)[0] - voronoi(&self).iter().sum::<i32>()
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
      "head": "rudolph", // TODO: Choose head
      "tail": "mouse", // TODO: Choose tail
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
  let mut owned_by_new = DenseBoard::init(MAGIC_NOT_OWNED);
  let mut voronoi_scores = [0; PLAYER_COUNT];

  for (i, head) in node.heads.iter().enumerate() {
    *owned_by.get_coord_mut(*head) = i as i32;
    *owned_by_new.get_coord_mut(*head) = i as i32;
  }

  let mut not_done = true;

  while not_done {
    // print_board(&owned_by);
    not_done = false;
    for x in 0..(BOARD_SIZE as i32) {
      for y in 0..(BOARD_SIZE as i32) {
        let coord = Coord { x, y };

        let tests = [
          Coord { x: x - 1, y },
          Coord { x: x + 1, y },
          Coord { x, y: y - 1 },
          Coord { x, y: y + 1 },
        ];

        for test in tests.iter() {
          if node.board.get_coord(coord) - node.turn <= 0
            && owned_by.get_coord(*test) != MAGIC_NOT_OWNED
            && owned_by.get_coord(coord) == MAGIC_NOT_OWNED
          {
            *owned_by_new.get_coord_mut(coord) = owned_by.get_coord(*test);
            voronoi_scores[owned_by.get_coord(*test) as usize] += 1;
            not_done = true;
          }
        }
      }
    }
    owned_by = owned_by_new.clone();
  }

  return voronoi_scores;
}

const PLAYER_ID: i32 = 0;
fn alphabeta(
  node: Node,
  depth: i32,
  mut alpha: i32,
  mut beta: i32,
  maximising_player: bool,
) -> i32 {
  if depth == 0 {
    return node.evaluate();
  }
  if maximising_player {
    let mut value = -10000;
    for my_dir in [
      Direction::Up,
      Direction::Down,
      Direction::Left,
      Direction::Right,
    ]
    .iter()
    {
      let new_node = node.apply_move(0, *my_dir);
      value = std::cmp::max(value, alphabeta(new_node, depth - 1, alpha, beta, false));
      if value >= beta {
        break;
      }
      alpha = std::cmp::max(alpha, value);
    }
    return value;
  } else {
    let mut value = 10000;
    let mut enemy_turns = Vec::<SmallVec<[Direction; (PLAYER_COUNT - 1)]>>::new(); // [ [U, U], [U, D], ... ]
    enemy_turns.push(SmallVec::new());
    for i in 1..node.heads.len() {
      // for every snake, copy all the enemy moves and append the new move
      let mut new_enemy_turns = Vec::new();
      for enemy_turn in enemy_turns.iter() {
        for my_move in [
          Direction::Up,
          Direction::Down,
          Direction::Left,
          Direction::Right,
        ]
        .iter()
        {
          let mut new_turn = enemy_turn.clone();
          new_turn.push(*my_move);
          new_enemy_turns.push(new_turn);
        }
      }
      enemy_turns = new_enemy_turns;
    }
    for enemy_turn in enemy_turns.iter() {
      let new_node = node.apply_move_array(enemy_turn);
      value = std::cmp::min(value, alphabeta(new_node, depth - 1, alpha, beta, true));
      if value <= alpha {
        break;
      }
      beta = std::cmp::min(beta, value);
    }
    return value;
  }
}

fn print_board(board: &DenseBoard<i32>) {
  for x in -1..(BOARD_SIZE as i32 + 1) {
    for y in -1..(BOARD_SIZE as i32 + 1) {
      let coord = Coord { x, y };
      if board.get_coord(coord) == i32::MAX {
        print!("{:3}", -1);
      } else {
        print!("{:3}", board.get_coord(coord));
      }
    }
    println!();
  }
}

pub fn get_move(_game: &Game, turn: &i32, _board: &Board, you: &Battlesnake) -> Option<Direction> {
  // build board
  let mut board = DenseBoard::init(0);
  let mut heads = [Coord { x: 0, y: 0 }; PLAYER_COUNT];
  for (s, snake) in _board.snakes.iter().enumerate() {
    for (i, body) in snake.body.iter().enumerate() {
      *board.get_coord_mut(Coord {
        x: body.x as i32,
        y: body.y as i32,
      }) = turn + snake.body.len() as i32 - i as i32;
    }
    heads[s] = Coord {
      x: snake.body[0].x as i32,
      y: snake.body[0].y as i32,
    };
    *board.get_coord_mut(Coord {
      x: snake.body[0].x as i32,
      y: snake.body[0].y as i32,
    }) = 0;
  }
  // let board = Node::new(*turn, _board, you.health.try_into().unwrap());

  //
  // create walls
  for x in -1..(BOARD_SIZE as i32 + 1) {
    *board.get_xy_mut(x as isize, -1) = i32::MAX;
    *board.get_xy_mut(x as isize, BOARD_SIZE as isize) = i32::MAX;
  }
  for y in -1..(BOARD_SIZE as i32 + 1) {
    *board.get_xy_mut(-1, y as isize) = i32::MAX;
    *board.get_xy_mut(BOARD_SIZE as isize, y as isize) = i32::MAX;
  }

  let node = Node::new(*turn, board, heads, you.health.try_into().unwrap());

  let mut best_move = Direction::Up;
  let mut best_score = alphabeta(node.apply_move(0, best_move), 2, i32::MIN, i32::MAX, false);

  println!("start");
  for direction in [
    Direction::Down,
    Direction::Left,
    Direction::Right,
    Direction::Up,
  ] {
    let score = alphabeta(node.apply_move(0, direction), 2, i32::MIN, i32::MAX, false);
    println!("{:?} score: {}", direction, score);
    if score > best_score {
      best_score = score;
      best_move = direction;
    }
  }
  println!("end");

  // print_board(&node.board);
  println!("turn: {}", turn);
  println!("voronoi: {:?}", voronoi(&node));

  println!("best move: {:?}", best_move);

  Some(best_move)

  // 1. Don't run into the wall

  // We've included code to prevent your Battlesnake from moving backwards
  // let my_head = &you.body[0]; // Coordinates of your head
  // let my_neck = &you.body[1]; // Coordinates of your "neck"

  // let mut is_move_safe: HashMap<_, _> = vec![
  //   (Direction::Up, you.head.y == 0),
  //   (Direction::Down, you.head.y == BOARD_SIZE as i32 - 1),
  //   (Direction::Left, you.head.x == 0),
  //   (Direction::Right, you.head.y == BOARD_SIZE as i32 - 1),
  // ]
  // .into_iter()
  // .collect();

  // // 2. Don't run into other snakes (including ourselves)
  // if _board
  //   .snakes
  //   .iter()
  //   .any(|s| s.body.iter().any(|b| b.y == you.head.y - 1))
  // {
  //   *is_move_safe.get_mut(&Direction::Up).unwrap() = false;
  // }
  // if _board
  //   .snakes
  //   .iter()
  //   .any(|s| s.body.iter().any(|b| b.y == you.head.y + 1))
  // {
  //   *is_move_safe.get_mut(&Direction::Down).unwrap() = false;
  // }
  // if _board
  //   .snakes
  //   .iter()
  //   .any(|s| s.body.iter().any(|b| b.x == you.head.x - 1))
  // {
  //   *is_move_safe.get_mut(&Direction::Left).unwrap() = false;
  // }
  // if _board
  //   .snakes
  //   .iter()
  //   .any(|s| s.body.iter().any(|b| b.x == you.head.x + 1))
  // {
  //   *is_move_safe.get_mut(&Direction::Right).unwrap() = false;
  // }

  // // Are there any safe moves left?
  // let safe_moves = is_move_safe
  //   .into_iter()
  //   .filter(|&(_, v)| v)
  //   .map(|(k, _)| k)
  //   .collect::<Vec<_>>();

  // // Choose a random move from the safe ones
  // let chosen = safe_moves.choose(&mut rand::thread_rng()).unwrap().clone();

  // // TODO: Step 4 - Move towards food instead of random, to regain health and survive longer
  // let food = &_board.food;

  // info!("MOVE {}: {:?}", turn, chosen);
  // return Some(chosen);
}
