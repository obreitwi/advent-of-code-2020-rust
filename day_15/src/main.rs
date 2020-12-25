use std::collections::{HashMap, VecDeque};

fn main() {
    let input: Vec<usize> = vec![8, 13, 1, 0, 18, 9];

    assert_eq!(play(&[1, 3, 2], 2020), 1);
    assert_eq!(play(&[2, 1, 3], 2020), 10);
    assert_eq!(play(&[1, 2, 3], 2020), 27);
    assert_eq!(play(&[2, 3, 1], 2020), 78);
    assert_eq!(play(&[3, 2, 1], 2020), 438);
    assert_eq!(play(&[3, 1, 2], 2020), 1836);

    println!("(part1) The 2020th number is {}", play(&input[..], 2020));
}

fn play(input: &[usize], num_turns: usize) -> usize {
    let mut track: HashMap<usize, VecDeque<usize>> = HashMap::new();

    for (turn, num) in input.iter().enumerate() {
        let mut deque = VecDeque::new();
        deque.push_front(turn);
        track.insert(*num, deque);
    }

    let mut last_num: usize = input[input.len() - 1];

    for turn in input.len()..num_turns {
        let current_num = match track.get(&last_num) {
            None => 0,
            Some(vd) => match vd.len() {
                1 => 0,
                2 => vd[0] - vd[1],
                l => {
                    panic!("Invalid deque len: {}", l);
                }
            },
        };

        match track.get_mut(&current_num) {
            None => {
                let mut vd = VecDeque::new();
                vd.push_front(turn);
                track.insert(current_num, vd);
            }
            Some(vd) => {
                vd.push_front(turn);
                vd.truncate(2);
            }
        }

        last_num = current_num;
    }
    last_num
}
