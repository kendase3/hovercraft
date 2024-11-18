// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// a star implementation
use rand::Rng;
use std::cmp::min;
use std::fmt;

const MAX_ADDROOM_FAILS: u32 = 3;

#[derive(Copy, Clone, Debug, PartialEq)]
enum Terrain {
    Innards,
    Corridor,
    TallWall,
    WideWall,
    Full,
    Debug,
}

impl Terrain {
    fn diggable(self) -> bool {
        if self == Terrain::Corridor || self == Terrain::Full {
            true
        } else {
            false
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Pair {
    x: u32,
    y: u32,
}

// a map unit, roughly a square meter i guess
#[derive(Copy, Clone, Debug)]
struct Cell {
    terrain: Terrain,
}

impl Cell {
    fn new(terrain: Terrain) -> Self {
        Cell { terrain }
    }
    fn as_char(&self) -> char {
        match self.terrain {
            Terrain::TallWall => '|',
            Terrain::WideWall => '-',
            // FIXME(skend): for testing
            Terrain::Full => '~',
            Terrain::Innards => '.',
            Terrain::Corridor => '#',
            Terrain::Debug => 'C',
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

#[derive(Copy, Clone, Debug)]
struct Room {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Room {
    fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Room {
            x,
            y,
            width,
            height,
        }
    }
}

impl fmt::Display for Room {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "room y = {}, x = {}, height = {}, width = {}",
            self.y, self.x, self.height, self.width
        )
    }
}

// a 2d map displayable as characters
#[derive(Clone, Debug)]
struct Mapp {
    data: Vec<Vec<Cell>>,
    rooms: Vec<Room>,
}

impl Mapp {
    fn new(height: u32, width: u32) -> Self {
        let mut rows = Vec::new();
        for _ in 0..height {
            let row = vec![Cell::new(Terrain::Full); width as usize];
            rows.push(row);
        }
        Mapp {
            data: rows,
            rooms: Vec::new(),
        }
    }
    fn default() -> Self {
        Mapp::new(20, 80)
    }
    fn height(&self) -> u32 {
        self.data.len() as u32
    }
    fn width(&self) -> u32 {
        self.data[0].len() as u32
    }
    fn add_room(&mut self) -> bool {
        let mut rng = rand::thread_rng();
        let start_x = rng.gen_range(0..75);
        let start_y = rng.gen_range(0..15);
        let max_width = min(10, 79 - start_x);
        let max_height = min(10, 19 - start_y);
        let size_x = rng.gen_range(4..=max_width);
        let size_y = rng.gen_range(4..=max_height);
        // i think we want the end to be inclusive
        let end_x = start_x + size_x;
        let end_y = start_y + size_y;
        // make sure the four corners are empty
        if self.data[start_y][start_x].terrain != Terrain::Full
            || self.data[end_y][end_x].terrain != Terrain::Full
            || self.data[start_y][end_x].terrain != Terrain::Full
            || self.data[end_y][start_x].terrain != Terrain::Full
        {
            return false;
        }
        // we can place this room!
        // add the room entity to our list of rooms
        self.rooms.push(Room::new(
            start_x as u32,
            start_y as u32,
            size_x as u32,
            size_y as u32,
        ));

        // make the top row wide walls
        // and the bottom row too
        for i in start_x..end_x {
            self.data[start_y][i].terrain = Terrain::WideWall;
            self.data[end_y - 1][i].terrain = Terrain::WideWall;
        }
        // start and end each inner row with tallwalls
        // weirdly we end up at just end_y because we -1 and +1 i think
        // notably this is running up to but not including the end
        for i in start_y + 1..end_y - 1 {
            self.data[i][start_x].terrain = Terrain::TallWall;
            for j in start_x + 1..end_x - 1 {
                self.data[i][j].terrain = Terrain::Innards;
            }
            self.data[i][end_x - 1].terrain = Terrain::TallWall;
        }
        // then say we did it
        true
    }
    fn add_rooms(&mut self) {
        // add rooms until MAX_ADDROOM_FAILS in a row
        let mut addroom_fails = 0;
        while addroom_fails < MAX_ADDROOM_FAILS {
            let ret = self.add_room();
            if !ret {
                addroom_fails += 1;
            }
        }
    }
    fn add_paths(&mut self) {
        if self.rooms.len() > 2 {
            return;
        }
        // the a-star business
        let mut connected_rooms = Vec::new();
        connected_rooms.push(0);
        let mut distant_rooms = Vec::new();
        for i in 1..self.rooms.len() {
            distant_rooms.push(i as u32);
        }
        while distant_rooms.len() > 0 {
            let src = connected_rooms[0]; // can make random
            let dst = distant_rooms.pop().unwrap();
            // then we do a bunch of map stuff
            self.path(src, dst);
            // then we add it to the closed set
            // TODO(skend): make it return a bool for success?
            connected_rooms.push(dst as u32);
        }
    }

    // TODO(skend): rethink this func; work primarily in pairs
    // with only actual data access using the mapp struct
    fn get_random_wall(&mut self, src_index: u32) -> (Pair, Pair) {
        let mut data = &mut self.data;
        let src = self.rooms[src_index as usize];
        let mut candidates = Vec::new();
        // add the top and bottom walls' cells
        for i in 1..(src.width - 1) {
            candidates.push(Pair {
                y: src.y,
                x: src.x + i,
            });
            candidates.push(Pair {
                y: src.y + src.height - 1,
                x: src.x + i,
            });
        }
        for i in 1..(src.height - 1) {
            candidates.push(Pair {
                y: src.y + i,
                x: src.x,
            });
            candidates.push(Pair {
                y: src.y + i,
                x: src.x + src.width - 1,
            });
        }
        let mut rng = rand::thread_rng();
        let mut wall_validated = false;
        // FIXME(skend): temp mut
        //let maybe_ret = &self.data[0][0]; // temporary sane value
        let mut maybe_ret = &mut data[0][0]; // temporary sane value
        let mut x = 0;
        let mut y = 0;
        let mut coord2 = Pair { y: 0, x: 0 };
        while !wall_validated {
            let rand_pair = rng.gen_range(0..candidates.len());
            x = candidates[rand_pair].x;
            y = candidates[rand_pair].y;
            maybe_ret = &mut data[y as usize][x as usize];
            // determine which face of the wall it's on
            // check if it's on the north wall
            let mut terr = Terrain::Full; // stub value
            if y == src.y {
                // check the cell outside it
                if y == 0 {
                    continue;
                }
                coord2 = Pair { y: y - 1, x };
            } else if y == src.y + src.height - 1 {
                // TODO(skend): check off-by-one on this
                if src.y + src.height >= data.len() as u32 {
                    continue;
                }
                coord2 = Pair { y: y + 1, x };
            } else if x == src.x {
                if x == 0 {
                    continue;
                }
                coord2 = Pair { y, x: x - 1 };
            } else if x == src.x + src.width - 1 {
                if src.x + src.width >= data[0].len() as u32 {
                    continue;
                }
                coord2 = Pair { y, x: x + 1 };
            } else {
                println!("uh oh...");
            }
            let terr = data[coord2.y as usize][coord2.x as usize].terrain;
            if terr.diggable() {
                wall_validated = true;
            }
        }
        // FIXME(skend): why was i returning Cell? cells don't even know
        // their own coords
        (Pair { y, x }, coord2)
    }

    fn path(&mut self, src: u32, dst: u32) {
        // the hairy part of pathing
        // we make a door in a valid-ish spot on the src
        // just needs to be a wall node without an adjacent
        // wall perpendicular to its wall type...does that
        // make sense?
        // we can just have the room mark for us its walls
        // we'll pick one at random and validate it
        // if it's bad, pick another one
        let (start_cell, first_node) = self.get_random_wall(src);
        // we make a door in a valid-ish spot on the dst
        let (end_cell, _) = self.get_random_wall(dst);

        // we construct an open set of nodes around the src node
        // TODO(skend): we could save time by not simming around
        // inside the room, always start path on valid outer node
        // which we did discover in get_random_wall, maybe could be
        // part of return? or maybe that should just be the start
        // of the path? but we want to turn the wall into a door
        // so that could be annoying to go find again

        // and then we just execute the algorithm, take the most
        // promising one and dig in

        // we retrieve the heuristic, manhattan distance in our case
    }
}

impl fmt::Display for Mapp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret = "".to_owned();
        for room in self.rooms.iter() {
            ret += &format!("{}\n", room);
        }
        for i in 0..self.height() as usize {
            for j in 0..self.width() as usize {
                ret += &format!("{}", self.data[i][j]);
            }
            ret += "\n"
        }
        write!(f, "{}", ret)
    }
}

fn main() {
    let mut mapp = Mapp::default();
    mapp.add_rooms();
    println!("{}", mapp);
    mapp.get_random_wall(0);
    println!("{}", mapp);
}
