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

use rand::Rng;
/// a star implementation
use std::fmt;

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
            //Terrain::Wall => ' ',
            // FIXME(skend): for testing
            Terrain::TallWall => '|',
            Terrain::WideWall => '-',
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

// a 2d map displayable as characters
#[derive(Clone, Debug)]
struct Mapp {
    data: Vec<Vec<Cell>>,
}

impl Mapp {
    fn new(height: i32, width: i32) -> Self {
        let mut rows = Vec::new();
        for _ in 0..height {
            let row = vec![Cell::new(Terrain::Full); width as usize];
            rows.push(row);
        }
        Mapp { data: rows }
    }
    fn default() -> Self {
        Mapp::new(20, 80)
    }
    fn height(&self) -> i32 {
        self.data.len() as i32
    }
    fn width(&self) -> i32 {
        self.data[0].len() as i32
    }
    fn add_room(&mut self) -> bool {
        let mut rng = rand::thread_rng();
        let start_x = rng.gen_range(0..=10);
        let start_y = rng.gen_range(0..=10);
        let size_x = rng.gen_range(4..=10);
        let size_y = rng.gen_range(4..=9);
        // i think we want the end to be inclusive
        let end_x = start_x + size_x;
        let end_y = start_y + size_y;
        // make sure the two diagonal corners are empty
        if self.data[start_y][start_x].terrain != Terrain::Full
            || self.data[end_y][end_x].terrain != Terrain::Full
        {
            return false;
        }
        // we can place this room!
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
}

impl fmt::Display for Mapp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ret = "".to_owned();
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
    println!("{}", mapp);
    mapp.add_room();
    println!("{}", mapp);
}
