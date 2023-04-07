// Copyright 2023 Google LLC
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

// wumpus game to exercise some shared lib functions

use anyhow::Result;
use hovercraft::Pair;
use std::fmt;

// how about the map is always a 4x4 grid
pub const MAP_WIDTH: i64 = 4;
pub const MAP_HEIGHT: i64 = 4;

#[derive(Debug, Default, Clone)]
struct World {
    size: Pair<i64>,
    grid: Vec<Vec<i64>>,
}

impl World {
    fn new() -> World {
        let grid = vec![vec![0; MAP_WIDTH as usize]; MAP_HEIGHT as usize];
        let size = Pair::new(MAP_WIDTH, MAP_HEIGHT);
        World { size, grid }
    }
    fn update(&mut self, cell: Pair<usize>, value: i64) {
        self.grid[cell.y][cell.x] = value;
    }
    fn print(&self) {
        for row in self.grid.iter() {
            for cell in row.iter() {
                print!("{}", cell);
            }
            println!();
        }
    }
}

fn main() -> Result<(), String> {
    let mut world = World::new();
    world.update(Pair { x: 1, y: 1 }, 3);
    println!("it ran!");
    world.print();
    Ok(())
}
