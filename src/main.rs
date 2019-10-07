use std::collections::HashMap;
use std::fmt;
use std::io;

const WIDTH: usize = 30;
const HEIGHT: usize = 15;

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

/// Compute the “Manhattan distance” between two points.
fn manh_dist(a: [u32; 2], b: [u32; 2]) -> u32 {
  ((a[0] as f32 - b[0] as f32).abs() + (a[1] as f32 - b[1] as f32).abs()) as u32
}

trait TryFrom<T>: Sized {
  type Error;

  fn try_from(t: T) -> Result<Self, Self::Error>;
}

trait TryInto<T> {
  type Error;

  fn try_into(self) -> Result<T, Self::Error>;
}

impl<T, U> TryInto<T> for U where T: TryFrom<U> {
  type Error = T::Error;

  fn try_into(self) -> Result<T, Self::Error> {
    T::try_from(self)
  }
}

/// Entity types.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum EntityType {
  Miner,
  OpponentMiner,
  BurriedRadar,
  BurriedTrap,
}

impl TryFrom<u32> for EntityType {
  type Error = String;

  fn try_from(value: u32) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(EntityType::Miner),
      1 => Ok(EntityType::OpponentMiner),
      2 => Ok(EntityType::BurriedRadar),
      3 => Ok(EntityType::BurriedTrap),
      _ => Err(format!("unknown entity type: {}", value)),
    }
  }
}

/// Entity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Entity {
  Miner(usize),
  OpponentMiner(usize),
  BurriedRadar,
  BurriedTrap,
}

/// Possible items a miner can hold.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Item {
  Radar,
  Trap,
  Ore
}

impl TryFrom<i32> for Item {
  type Error = String;

  fn try_from(value: i32) -> Result<Self, Self::Error> {
    match value {
      2 => Ok(Item::Radar),
      3 => Ok(Item::Trap),
      4 => Ok(Item::Ore),
      _ => Err(format!("unknown item: {}", value)),
    }
  }
}

/// Possible items a miner can request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum RequestItem {
  Radar,
  Trap
}

impl fmt::Display for RequestItem {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      RequestItem::Radar => f.write_str("RADAR"),
      RequestItem::Trap => f.write_str("TRAP"),
    }
  }
}

impl RequestItem {
  fn into_item(self) -> Item {
    match self {
      RequestItem::Radar => Item::Radar,
      RequestItem::Trap => Item::Trap,
    }
  }
}

/// Possible request.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Request {
  Move(u32, u32),
  Wait,
  Dig(u32, u32),
  Item(RequestItem),
}

impl fmt::Display for Request {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Request::Move(x, y) => write!(f, "MOVE {} {}", x, y),
      Request::Wait => f.write_str("WAIT"),
      Request::Dig(x, y) => write!(f, "DIG {} {}", x, y),
      Request::Item(ref item) => write!(f, "REQUEST {}", item),
    }
  }
}

impl Request {
  fn submit(self) {
    println!("{}", self);
  }

  fn back_to_hq(position: [u32; 2]) -> Request {
    Request::Move(0, position[1])
  }
}

/// A request with a possible associated comment.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct OutputRequest {
  req: Request,
  comment: Option<String>
}

impl OutputRequest {
  fn new<C>(req: Request, comment: C) -> Self where C: Into<Option<String>> {
    OutputRequest {
      req,
      comment: comment.into()
    }
  }
}

impl From<Request> for OutputRequest {
  fn from(req: Request) -> Self {
    OutputRequest::new(req, None)
  }
}

impl fmt::Display for OutputRequest {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "{}", self.req)?;

    if let Some(ref comment) = self.comment {
      write!(f, " {}", comment)
    } else {
      Ok(())
    }
  }
}

/// Unique ID for all entities in the game.
type UID = u32;

#[derive(Debug)]
struct GameState {
  // informational
  my_score: u32,
  opponent_score: u32,
  cells: Vec<Cell>,
  miners: Vec<Miner>,
  opponent_miners: Vec<Miner>,
  entities: HashMap<UID, Entity>,
  burried_radars: HashMap<UID, [u32; 2]>,
  burried_traps: HashMap<UID, [u32; 2]>,
  radar_cooldown: u32,
  trap_cooldown: u32,

  // tactical
}

impl Default for GameState {
  fn default() -> Self {
    GameState {
      my_score: 0,
      opponent_score: 0,
      cells: vec![Cell::default(); WIDTH * HEIGHT],
      miners: Vec::new(),
      opponent_miners: Vec::new(),
      entities: HashMap::new(),
      burried_radars: HashMap::new(),
      burried_traps: HashMap::new(),
      radar_cooldown: 0,
      trap_cooldown: 0,
    }
  }
}

impl GameState {
  fn set_my_score(&mut self, score: u32) {
    self.my_score = score;
  }

  fn set_opponent_score(&mut self, score: u32) {
    self.opponent_score = score;
  }

  fn set_ore(&mut self, x: usize, y: usize, ore_amount: Option<usize>) {
    self.cells[y * WIDTH + x].ore_amount = ore_amount;
  }

  fn set_hole(&mut self, x: usize, y: usize, hole: bool) {
    self.cells[y * WIDTH + x].has_hole = hole;
  }

  fn set_radar_cooldown(&mut self, cooldown: u32) {
    self.radar_cooldown = cooldown;
  }

  fn set_trap_cooldown(&mut self, cooldown: u32) {
    self.trap_cooldown = cooldown;
  }

  fn add_entity(&mut self, uid: UID, entity: Entity) {
    self.entities.insert(uid, entity);
  }

  fn entity_exists(&self, uid: UID) -> bool {
    self.entities.contains_key(&uid)
  }

  fn add_miner(&mut self, miner: Miner) -> usize {
    let index = self.miners.len();
    self.miners.push(miner);

    index
  }

  fn add_opponent_miner(&mut self, miner: Miner) -> usize {
    let index = self.opponent_miners.len();
    self.opponent_miners.push(miner);

    index
  }

  fn update_position(&mut self, uid: UID, px: u32, py: u32) {
    match self.entities.get(&uid) {
      Some(Entity::Miner(index)) => {
        let miner = &mut self.miners[*index];
        miner.x = px;
        miner.y = py;
      }

      Some(Entity::OpponentMiner(index)) => {
        let miner = &mut self.opponent_miners[*index];
        miner.x = px;
        miner.y = py;
      }

      _ => eprintln!("trying to update miner {} position, but it’s not a miner", uid)
    }
  }

  fn update_item(&mut self, uid: UID, item: Option<Item>) {
    match self.entities.get(&uid) {
      Some(Entity::Miner(index)) => {
        self.miners[*index].item = item;
      }

      Some(Entity::OpponentMiner(index)) => {
        self.opponent_miners[*index].item = item;
      }

      _ => eprintln!("trying to update miner {} item, but it’s not a miner", uid)
    }
  }

  fn kill(&mut self, uid: UID) {
    match self.entities.get(&uid) {
      Some(Entity::Miner(index)) => {
        self.miners[*index].alive = false;
      }

      Some(Entity::OpponentMiner(index)) => {
        self.opponent_miners[*index].alive = false;
      }

      _ => eprintln!("trying to kill miner {}, but it’s not a miner", uid)
    }
  }

  fn burry_radar(&mut self, uid: UID, x: u32, y: u32) {
    self.burried_radars.insert(uid, [x, y]);
  }

  fn burry_trap(&mut self, uid: UID, x: u32, y: u32) {
    self.burried_traps.insert(uid, [x, y]);
  }

  fn update_radar_position(&mut self, uid: UID, x: u32, y: u32) {
    if let Some(ref mut p) = self.burried_radars.get_mut(&uid) {
      p[0] = x;
      p[1] = y;
    } else {
      eprintln!("trying to update burried radar {} position, but it’s not a radar", uid);
    }
  }

  fn update_trap_position(&mut self, uid: UID, x: u32, y: u32) {
    if let Some(ref mut p) = self.burried_traps.get_mut(&uid) {
      p[0] = x;
      p[1] = y;
    } else {
      eprintln!("trying to update burried trap {} position, but it’s not a trap", uid);
    }
  }

  fn miners(&self) -> impl Iterator<Item = &Miner> {
    self.miners.iter()
  }

  fn miner(&self, index: usize) -> Option<&Miner> {
    self.miners.get(index)
  }
}

/// Describe a single cell on the grid.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Cell {
  ore_amount: Option<usize>,
  has_hole: bool
}

impl Default for Cell {
  fn default() -> Self {
    Cell {
      ore_amount: None,
      has_hole: false,
    }
  }
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct Miner {
  x: u32,
  y: u32,
  item: Option<Item>,
  uid: UID,
  alive: bool,
}

fn main() {
  let mut input_line = String::new();
  io::stdin().read_line(&mut input_line).unwrap();
  let inputs = input_line.split(" ").collect::<Vec<_>>();

  let width = parse_input!(inputs[0], u32);
  let height = parse_input!(inputs[1], u32); // size of the map

  let mut game_state = GameState::default();

  // game loop
  loop {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(" ").collect::<Vec<_>>();

    let my_score = parse_input!(inputs[0], u32); // Amount of ore delivered
    let opponent_score = parse_input!(inputs[1], u32);

    game_state.set_my_score(my_score);
    game_state.set_opponent_score(opponent_score);

    for y in 0..height as usize {
      let mut input_line = String::new();
      io::stdin().read_line(&mut input_line).unwrap();
      let inputs = input_line.split_whitespace().collect::<Vec<_>>();

      // we skip x = 0 as it’s HQ
      for x in 0..width as usize {
        let ore: Option<usize> = inputs[2*y].trim().parse().ok(); // amount of ore or "?" if unknown
        let hole = parse_input!(inputs[2 * y + 1], u32) == 1; // 1 if cell has a hole

        game_state.set_ore(x, y, ore);
        game_state.set_hole(x, y, hole);
      }
    }

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let inputs = input_line.split(" ").collect::<Vec<_>>();

    let entity_count = parse_input!(inputs[0], u32); // number of entities visible to you
    let radar_cooldown = parse_input!(inputs[1], u32); // turns left until a new radar can be requested
    let trap_cooldown = parse_input!(inputs[2], u32); // turns left until a new trap can be requested

    game_state.set_radar_cooldown(radar_cooldown);
    game_state.set_trap_cooldown(trap_cooldown);

    for i in 0..entity_count as usize {
      let mut input_line = String::new();
      io::stdin().read_line(&mut input_line).unwrap();
      let inputs = input_line.split(" ").collect::<Vec<_>>();

      let uid = parse_input!(inputs[0], u32); // unique id of the entity
      let entity_type: EntityType = parse_input!(inputs[1], u32).try_into().unwrap();

      let x = parse_input!(inputs[2], i32);
      let y = parse_input!(inputs[3], i32); // position of the entity
      let item = parse_input!(inputs[4], i32).try_into().ok(); // if this entity is a robot, the item it is carrying (-1 for NONE, 2 for RADAR, 3 for TRAP, 4 for ORE)

      // check if we need to update our entities
      if !game_state.entity_exists(uid) {
        // if it’s a miner, add it to the list of miners
        match entity_type {
          EntityType::Miner => {
            let miner_index = game_state.add_miner(Miner {
              x: x as u32,
              y: y as u32,
              item,
              uid,
              alive: true,
            });

            game_state.add_entity(uid, Entity::Miner(miner_index));
          }

          EntityType::OpponentMiner => {
            let opponent_miner_index = game_state.add_opponent_miner(Miner {
              x: x as u32,
              y: y as u32,
              item,
              uid,
              alive: true,
            });

            game_state.add_entity(uid, Entity::OpponentMiner(opponent_miner_index));
          }

          EntityType::BurriedRadar => {
            game_state.add_entity(uid, Entity::BurriedRadar);
            game_state.burry_radar(uid, x as u32, y as u32);
          }

          EntityType::BurriedTrap => {
            game_state.add_entity(uid, Entity::BurriedTrap);
            game_state.burry_trap(uid, x as u32, y as u32);
          }
        }
      } else {
        match entity_type {
          EntityType::Miner | EntityType::OpponentMiner => {
            if x == -1 && y == -1 {
              // this miner is dead
              game_state.kill(uid);
            }

            // update position
            game_state.update_position(uid, x as u32, y as u32);

            // update item
            game_state.update_item(uid, item);
          }

          EntityType::BurriedRadar => {
            // position of this radar has changed
            game_state.update_radar_position(uid, x as u32, y as u32);
          }

          EntityType::BurriedTrap => {
            // position of this trap has changed
            game_state.update_trap_position(uid, x as u32, y as u32);
          }
        }
      }
    }

    for (miner_index, miner) in game_state.miners().enumerate() {
      Request::Wait.submit();
    }
  }
}
