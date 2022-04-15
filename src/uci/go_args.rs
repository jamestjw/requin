use crate::board::{Coordinate, PieceType};

#[allow(dead_code)]
pub struct GoArgs {
    pub search_moves: Option<Vec<(Coordinate, Coordinate, Option<PieceType>)>>,
    pub ponder: bool,
    pub wtime: Option<u32>,
    pub btime: Option<u32>,
    pub winc: Option<u32>,
    pub binc: Option<u32>,
    pub movestogo: Option<u32>,
    pub depth: u8,
    pub nodes: u32,
    pub mate: Option<u32>,
    pub movetime: u32,
    pub infinite: bool,
}

impl GoArgs {
    fn new() -> Self {
        Self {
            search_moves: None,
            ponder: false,
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            movestogo: None,
            depth: 5,
            nodes: 10,
            mate: None,
            movetime: 1000,
            infinite: true,
        }
    }

    // This function assumes that the args_str is valid,
    // i.e. it conforms to the regex of the Go command
    pub fn new_from_args_str(args_str: String) -> Self {
        let mut arg_vec = args_str.split_whitespace().peekable();
        let mut res = GoArgs::new();

        while let Some(arg_key) = arg_vec.next() {
            match arg_key {
                "wtime" => {
                    // Assume that format of args are correct, i.e.
                    // the below should never fail

                    res.wtime = Some(arg_vec.next().unwrap().parse::<u32>().unwrap());
                }
                "btime" => {
                    res.btime = Some(arg_vec.next().unwrap().parse::<u32>().unwrap());
                }
                "winc" => {
                    res.winc = Some(arg_vec.next().unwrap().parse::<u32>().unwrap());
                }
                "binc" => {
                    res.binc = Some(arg_vec.next().unwrap().parse::<u32>().unwrap());
                }
                "movestogo" => {
                    res.movestogo = Some(arg_vec.next().unwrap().parse::<u32>().unwrap());
                }
                "depth" => {
                    res.depth = arg_vec.next().unwrap().parse::<u8>().unwrap();
                }
                "nodes" => {
                    res.nodes = arg_vec.next().unwrap().parse::<u32>().unwrap();
                }
                "mate" => {
                    res.mate = Some(arg_vec.next().unwrap().parse::<u32>().unwrap());
                }
                "movetime" => {
                    res.movetime = arg_vec.next().unwrap().parse::<u32>().unwrap();
                }
                "infinite" => {
                    res.infinite = true;
                }
                "ponder" => {
                    res.ponder = true;
                }
                "searchmoves" => {
                    let mut moves = vec![];
                    while let Some(arg_value) = arg_vec.next_if(|x| {
                        ![
                            "searchmoves",
                            "ponder",
                            "wtime",
                            "btime",
                            "winc",
                            "binc",
                            "movestogo",
                            "depth",
                            "nodes",
                            "mate",
                            "movetime",
                            "infinite",
                        ]
                        .contains(x)
                    }) {
                        moves.push(Coordinate::new_from_long_algebraic_notation(arg_value));
                    }

                    res.search_moves = Some(moves);
                }
                // If an unknown arg is encountered, we pretend that
                // nothing happened and fall back to default values
                _ => break,
            }
        }

        res
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_args_parser_with_one_kv_pair() {
        let args = GoArgs::new_from_args_str(" depth 47 ".into());
        assert_eq!(args.depth, 47);
    }

    #[test]
    fn test_args_parser_with_multiple_kv_pairs() {
        let args = GoArgs::new_from_args_str(" depth 47 winc 3000 binc 1000 nodes 24 ".into());
        assert_eq!(args.depth, 47);
        assert_eq!(args.nodes, 24);
        assert_eq!(args.winc, Some(3000));
        assert_eq!(args.binc, Some(1000));
    }

    #[test]
    fn test_args_parser_with_searchmoves() {
        let args = GoArgs::new_from_args_str(" searchmoves e2e4 d2d4 ".into());
        assert_eq!(
            args.search_moves,
            Some(vec![
                (Coordinate::E2, Coordinate::E4, None),
                (Coordinate::D2, Coordinate::D4, None)
            ])
        );
    }

    #[test]
    fn test_args_parser_with_searchmoves_and_other_args() {
        let args = GoArgs::new_from_args_str(" depth 250 searchmoves e2e4 d2d4 nodes 14".into());
        assert_eq!(
            args.search_moves,
            Some(vec![
                (Coordinate::E2, Coordinate::E4, None),
                (Coordinate::D2, Coordinate::D4, None)
            ])
        );
        assert_eq!(args.depth, 250);
        assert_eq!(args.nodes, 14);
    }
}
