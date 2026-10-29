/**
 * Splits an area into rooms with binary space partitioning: the area is cut in half (with a little randomness),
 * then each half again, until a cut would leave a piece smaller than the minimum size. One room of random size
 * and position is then placed inside every piece. Rooms never overlap and cover the area evenly.
 *
 * Return:
 * * a json list of rooms, each one with id, x, y, x2, y2, width, height and center (with x and y)
 *
 * Arguments:
 * * width: width of the area
 * * height: height of the area
 * * hash: seed for the random generator, the same seed gives the same rooms
 * * map_subsection_min_size: smallest size a piece can be cut to. It is raised above the minimum room size if needed
 * * map_subsection_min_room_width: smallest width a room can have
 * * map_subsection_min_room_height: smallest height a room can have
 */
#define rustg_bsp_generate(width, height, hash, map_subsection_min_size, map_subsection_min_room_width, map_subsection_min_room_height) \
	RUSTG_CALL(RUST_G, "bsp_generate")(width, height, hash, map_subsection_min_size, map_subsection_min_room_width, map_subsection_min_room_height)
