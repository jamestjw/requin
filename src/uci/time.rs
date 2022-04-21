// This file handles time management

/// # Arguments
///
/// * `remaining_time` - Remaining time for the current player (in milliseconds)
/// * `increment` - Increment for each move for the current player (in milliseconds)
pub fn max_search_time(remaining_time: u32, increment: u32) -> u32 {
    // Assume that there will be another 30 moves
    // TODO: Improve this
    let estimate_remaining_moves_count = 30;

    // Divide remaining time equally
    (remaining_time + estimate_remaining_moves_count * increment) / estimate_remaining_moves_count
}
