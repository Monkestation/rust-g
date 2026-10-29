//! Random room placement, ported from yogstation13/rust-g.
//! Places non overlapping rooms with the dimensions of maint ruins at random spots of an area.
use crate::error::Result;
use rand::{
    RngExt, SeedableRng,
    distr::{Distribution, weighted::WeightedIndex},
    rngs::StdRng,
};
use serde::Serialize;

byond_fn!(fn random_room_generate(width, height, desired_room_count, hash) {
    random_room_gen(width, height, desired_room_count, hash).ok()
});

fn random_room_gen(
    width_as_str: &str,
    height_as_str: &str,
    desired_room_count_as_str: &str,
    hash_as_str: &str,
) -> Result<String> {
    let width = width_as_str.parse::<usize>()?;
    let height = height_as_str.parse::<usize>()?;
    let desired_room_count = desired_room_count_as_str.parse::<usize>()?;
    let seed = hash_as_str.parse::<u64>()?;

    let mut rng = StdRng::seed_from_u64(seed);
    let rooms = place_rooms_random(width, height, desired_room_count, &mut rng);

    Ok(serde_json::to_string(&rooms)?)
}

/// Inside dimensions (width, height) of the maint ruins the rooms are made for, and how often each one is picked.
const ROOM_LAYOUTS: [(usize, usize); 6] = [(3, 3), (3, 5), (5, 3), (5, 4), (10, 5), (10, 10)];
const ROOM_WEIGHTS: [u32; 6] = [4, 3, 4, 3, 2, 1];
/// How many rooms in a row can fail to fit before giving up.
const MAX_ATTEMPTS: usize = 15;

fn place_rooms_random(
    width: usize,
    height: usize,
    desired_room_count: usize,
    rng: &mut StdRng,
) -> Vec<Room> {
    let mut rooms: Vec<Room> = Vec::new();
    let dist = WeightedIndex::new(ROOM_WEIGHTS).expect("the room weights are valid");
    let mut attempts = 0;
    while rooms.len() <= desired_room_count && attempts <= MAX_ATTEMPTS {
        attempts += 1;
        let (inner_width, inner_height) = ROOM_LAYOUTS[dist.sample(rng)];
        // the layouts are the inside of the room, the walls take one more tile on each side
        let room_width = inner_width + 2;
        let room_height = inner_height + 2;
        if room_width > width || room_height > height {
            continue;
        }

        let x = rng.random_range(0..width).min(width - room_width);
        let y = rng.random_range(0..height).min(height - room_height);

        let room = Room::new(
            format!("RRPS room: {}", rooms.len()),
            x,
            y,
            room_width,
            room_height,
        );

        if !rooms.iter().any(|other| room.intersects(other)) {
            rooms.push(room);
            attempts = 0;
        }
    }
    rooms
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

    pub fn intersects(&self, other: &Self) -> bool {
        self.x <= other.x2 && self.x2 >= other.x && self.y <= other.y2 && self.y2 >= other.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rooms_fit_and_do_not_overlap() {
        let mut rng = StdRng::seed_from_u64(42);
        let rooms = place_rooms_random(100, 80, 40, &mut rng);
        assert!(!rooms.is_empty());
        for (i, room) in rooms.iter().enumerate() {
            assert!(room.x2 <= 100 && room.y2 <= 80);
            for other in &rooms[i + 1..] {
                assert!(!room.intersects(other));
            }
        }
    }

    #[test]
    fn same_seed_same_rooms() {
        let a = random_room_gen("100", "80", "30", "1234").unwrap();
        let b = random_room_gen("100", "80", "30", "1234").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn area_too_small_gives_no_rooms() {
        let mut rng = StdRng::seed_from_u64(1);
        assert!(place_rooms_random(2, 2, 10, &mut rng).is_empty());
    }
}
