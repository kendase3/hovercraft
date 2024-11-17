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
        connected_rooms.push(self.rooms[0]);
        let mut distant_rooms = Vec::new();
        for i in 1..self.rooms.len() {
            distant_rooms.push(self.rooms[i]);
        }
        while distant_rooms.len() > 0 {
            let src = connected_rooms[0]; // can make random
            let dst = distant_rooms.pop().unwrap();
            // then we do a bunch of map stuff
            self.path(&src, &dst);
            // then we add it to the closed set
            // TODO(skend): make it return a bool for success?
            connected_rooms.push(dst);
        }
    }
    fn get_random_wall(&self, src: &Room) -> &Cell {
        let mut candidates = Vec::new();
        // TODO(skend): start with just the top wall and don't validate yet
        for i in 1..(src.width - 1) {
            candidates.push(&self.data[src.height as usize][i as usize]);
        }
        let mut rng = rand::thread_rng();
        let ret = rng.gen_range(0..candidates.len());
        candidates[ret]
    }

    fn path(&mut self, src: &Room, dst: &Room) {
        // the hairy part of pathing
        // we make a door in a valid-ish spot on the src
        // just needs to be a wall node without an adjacent
        // wall perpendicular to its wall type...does that
        // make sense?
        // we can just have the room mark for us its walls
        // we'll pick one at random and validate it
        // if it's bad, pick another one
        let start_cell = self.get_random_wall(&src);
        // we make a door in a valid-ish spot on the dst
        // in the same kind of way we chose a valid start

        // we construct an open set of nodes around the src node

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
}
