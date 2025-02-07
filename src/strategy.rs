/// The `Strategy` trait defines how to choose a slot given a valid gap.
/// The parameters:
///   - `lower`: the number in the slot immediately to the left (or a lower bound).
///   - `upper`: the number in the slot immediately to the right (or an upper bound).
///   - `first_slot`: the index of the first available slot in the gap.
///   - `last_slot`: the index of the last available slot in the gap.
///   - `number`: the drawn number to place.
///
/// The function returns one of the indices in `available_slots`.
pub trait Strategy {
    fn choose_slot(
        &self,
        lower: i32,
        upper: i32,
        first_slot: usize,
		last_slot: usize,
        number: i32,
    ) -> usize;
}
