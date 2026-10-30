//! Binary space partitioning, ported from yogstation13/rust-g.
//! Cuts an area in half again and again until the pieces would get too small, then puts one room in each piece.
use crate::error::Result;
use rand::{RngExt, SeedableRng, rngs::StdRng};
use serde::Serialize;

byond_fn!(fn bsp_generate(width, height, hash, map_subsection_min_size, map_subsection_min_room_width, map_subsection_min_room_height) {
    bsp_gen(width, height, hash, map_subsection_min_size, map_subsection_min_room_width, map_subsection_min_room_height).ok()
});

fn bsp_gen(
    width_as_str: &str,
    height_as_str: &str,
    hash_as_str: &str,
    map_subsection_min_size_as_str: &str,
    map_subsection_min_room_width_as_str: &str,
    map_subsection_min_room_height_as_str: &str,
) -> Result<String> {
    let width = width_as_str.parse::<usize>()?;
    let height = height_as_str.parse::<usize>()?;
    let min_room_width = map_subsection_min_room_width_as_str.parse::<usize>()?;
    let min_room_height = map_subsection_min_room_height_as_str.parse::<usize>()?;
    // a subsection smaller than the rooms it has to hold would leave no valid room size, so it is never allowed
    let min_size = map_subsection_min_size_as_str
        .parse::<usize>()?
        .max(min_room_width.max(min_room_height) + 1);
    let seed = hash_as_str.parse::<u64>()?;

    let mut rng = StdRng::seed_from_u64(seed);
    let rooms = generate_rooms(
        width,
        height,
        min_size,
        min_room_width,
        min_room_height,
        &mut rng,
    );

    Ok(serde_json::to_string(&rooms)?)
}

fn generate_rooms(
    width: usize,
    height: usize,
    min_size: usize,
    min_room_width: usize,
    min_room_height: usize,
    rng: &mut StdRng,
) -> Vec<Room> {
    let mut root = Leaf::new(0, 0, width, height);
    root.split_down(min_size, rng);
    root.create_rooms(min_room_width, min_room_height, rng);

    let mut rooms = Vec::new();
    root.collect_rooms(&mut rooms);
    rooms
}

struct Leaf {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    children: Option<Box<(Leaf, Leaf)>>,
    room: Option<Room>,
}

impl Leaf {
    fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Leaf {
            x,
            y,
            width,
            height,
            children: None,
            room: None,
        }
    }

    /// Keeps splitting this piece and its halves until they are as small as they can get.
    fn split_down(&mut self, min_size: usize, rng: &mut StdRng) {
        if self.split(min_size, rng)
            && let Some(children) = self.children.as_mut()
        {
            children.0.split_down(min_size, rng);
            children.1.split_down(min_size, rng);
        }
    }

    fn split(&mut self, min_size: usize, rng: &mut StdRng) -> bool {
        // wide pieces get cut vertically and tall ones horizontally; when it's close, it's a coin toss
        let mut split_horz = rng.random_range(0..2) == 1;
        if self.width > self.height && (self.width as f32 / self.height as f32) >= 1.25 {
            split_horz = false;
        } else if self.height > self.width && (self.height as f32 / self.width as f32) >= 1.25 {
            split_horz = true;
        }

        let length = if split_horz { self.height } else { self.width };
        // the piece is already small enough, so stop splitting
        if length <= min_size * 2 {
            return false;
        }
        let max = length - min_size;

        let split_pos = rng.random_range(min_size..max);
        let (first, second) = if split_horz {
            (
                Leaf::new(self.x, self.y, self.width, split_pos),
                Leaf::new(
                    self.x,
                    self.y + split_pos,
                    self.width,
                    self.height - split_pos,
                ),
            )
        } else {
            (
                Leaf::new(self.x, self.y, split_pos, self.height),
                Leaf::new(
                    self.x + split_pos,
                    self.y,
                    self.width - split_pos,
                    self.height,
                ),
            )
        };
        self.children = Some(Box::new((first, second)));
        true
    }

    /// Puts a room of random size and position inside every piece that wasn't split.
    fn create_rooms(&mut self, min_room_width: usize, min_room_height: usize, rng: &mut StdRng) {
        if let Some(children) = self.children.as_mut() {
            children
                .0
                .create_rooms(min_room_width, min_room_height, rng);
            children
                .1
                .create_rooms(min_room_width, min_room_height, rng);
            return;
        }
        if self.width < min_room_width || self.height < min_room_height {
            return;
        }
        let width = rng.random_range(min_room_width..=self.width);
        let height = rng.random_range(min_room_height..=self.height);
        let x = rng.random_range(0..=self.width - width);
        let y = rng.random_range(0..=self.height - height);
        self.room = Some(Room::new(
            "bsp room".to_string(),
            x + self.x,
            y + self.y,
            width,
            height,
        ));
    }

    fn collect_rooms(&self, rooms: &mut Vec<Room>) {
        if let Some(room) = &self.room {
            rooms.push(room.clone());
        }
        if let Some(children) = &self.children {
            children.0.collect_rooms(rooms);
            children.1.collect_rooms(rooms);
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub struct Point {
    x: usize,
    y: usize,
}

/// A room as BYOND gets it: bottom left corner, the opposite corner, its size and its center.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Room {
    id: String,
    x: usize,
    y: usize,
    x2: usize,
    y2: usize,
    width: usize,
    height: usize,
    center: Point,
}

impl Room {
    pub fn new(id: String, x: usize, y: usize, width: usize, height: usize) -> Self {
        Room {
            id,
            x,
            y,
            x2: x + width,
            y2: y + height,
            width,
            height,
            center: Point {
                x: x + (width / 2),
                y: y + (height / 2),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overlaps(a: &Room, b: &Room) -> bool {
        a.x < b.x2 && a.x2 > b.x && a.y < b.y2 && a.y2 > b.y
    }

    #[test]
    fn rooms_cover_the_area_without_overlapping() {
        let mut rng = StdRng::seed_from_u64(7);
        let rooms = generate_rooms(200, 150, 10, 5, 5, &mut rng);
        assert!(rooms.len() > 20, "only {} rooms", rooms.len());
        for (i, room) in rooms.iter().enumerate() {
            assert!(room.x2 <= 200 && room.y2 <= 150);
            assert!(room.width >= 5 && room.height >= 5);
            for other in &rooms[i + 1..] {
                assert!(!overlaps(room, other));
            }
        }
    }

    #[test]
    fn same_seed_same_rooms() {
        let a = bsp_gen("120", "90", "99", "10", "4", "4").unwrap();
        let b = bsp_gen("120", "90", "99", "10", "4", "4").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn min_size_is_raised_above_the_room_size() {
        // asking for subsections smaller than the rooms used to crash the server in the original
        let json = bsp_gen("60", "60", "5", "2", "8", "8").unwrap();
        let rooms: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert!(!rooms.is_empty());
    }

    #[test]
    fn tiny_area_gives_one_room_or_none() {
        let mut rng = StdRng::seed_from_u64(3);
        assert!(generate_rooms(4, 4, 10, 5, 5, &mut rng).is_empty());
        let mut rng = StdRng::seed_from_u64(3);
        assert_eq!(generate_rooms(12, 12, 10, 5, 5, &mut rng).len(), 1);
    }
}
