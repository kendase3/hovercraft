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

use std::fmt;

#[derive(Copy, Clone, Debug)]
enum Terrain {
    Wall,
    Room,
    Corridor
}

// a map unit, roughly a square meter i guess
#[derive(Copy, Clone, Debug)]
struct Cell {
    terrain: Terrain
}

impl Cell {
    fn new(terrain: Terrain) -> Self {
       Cell { terrain }
    }
}

// a 2d map displayable as characters
#[derive(Clone, Debug)]
struct Mapp {
    data: Vec<Vec<Cell>>
}

impl Mapp {
    fn new(height: i32, width: i32) -> Self {
        let mut rows = Vec::new();
        for _ in 0..height {
            let row = vec![Cell::new(Terrain::Wall); width as usize];
            rows.push(row);
        }
        Mapp { data: rows }
    }
}

impl fmt::Display for Mapp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO")
    }
}

fn main() {
    let mapp = Mapp::new(4, 4);
    println!("{}", mapp);
}
