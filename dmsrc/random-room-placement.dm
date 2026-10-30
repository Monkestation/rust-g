/**
 * Places rooms of random size at random spots of an area, skipping any that would overlap one already placed.
 * Simpler than binary space partitioning and with worse coverage, but good for scattering rooms over it.
 * Room sizes are those of maint ruins (3x3, 3x5, 5x3, 5x4, 10x5 and 10x10, plus one tile of wall on each side),
 * so pre-made ruins can be dropped into them.
 *
 * Return:
 * * a json list of rooms, each one with id, x, y, x2, y2, width, height and center (with x and y)
 *
 * Arguments:
 * * width: width of the area
 * * height: height of the area
 * * desired_room_count: how many rooms to try to place
 * * hash: seed for the random generator, the same seed gives the same rooms
 */
#define rustg_random_room_generate(width, height, desired_room_count, hash) \
	RUSTG_CALL(RUST_G, "random_room_generate")(width, height, desired_room_count, hash)
