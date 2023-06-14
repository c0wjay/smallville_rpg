pub const ASPECT_RATIO: f32 = 16. / 9.;
pub const UNIT_SIZE: f32 = 8.;
pub const GRID_SIZE: f32 = 16.;
// Shifts Coordinate upside&rightside to the center of the grid.
// For example, for `GRID_SIZE == 16. && GRID_OFFSET ==8.`, if the entity's Coordinate is (0, 0), the actual position (GlobalTransform.translation) is (8., 8.).
pub const GRID_OFFSET: f32 = 8.;
