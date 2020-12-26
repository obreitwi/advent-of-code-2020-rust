use std::collections::{BTreeMap, HashMap, VecDeque};

fn main() {
    let input: Vec<usize> = vec![8, 13, 1, 0, 18, 9];

    assert_eq!(play(&[0, 3, 6], 2020), 436);
    assert_eq!(play(&[1, 3, 2], 2020), 1);
    assert_eq!(play(&[2, 1, 3], 2020), 10);
    assert_eq!(play(&[1, 2, 3], 2020), 27);
    assert_eq!(play(&[2, 3, 1], 2020), 78);
    assert_eq!(play(&[3, 2, 1], 2020), 438);
    assert_eq!(play(&[3, 1, 2], 2020), 1836);

    println!("(part1) The 2020th number is {}", play(&input[..], 2020));

    assert_eq!(play(&[0, 3, 6], 30000000), 175594);
    assert_eq!(play(&[1, 3, 2], 30000000), 2578);
    assert_eq!(play(&[2, 1, 3], 30000000), 3544142);
    assert_eq!(play(&[1, 2, 3], 30000000), 261214);
    assert_eq!(play(&[2, 3, 1], 30000000), 6895259);
    assert_eq!(play(&[3, 2, 1], 30000000), 18);
    assert_eq!(play(&[3, 1, 2], 30000000), 362);

    println!(
        "(part2) The 2020th number is {}",
        play(&input[..], 30000000)
    );
}

fn play_v1(input: &[usize], num_turns: usize) -> usize {
    let mut track: HashMap<usize, VecDeque<usize>> = HashMap::new();

    for (turn, num) in input.iter().enumerate() {
        let mut deque = VecDeque::new();
        deque.push_front(turn);
        track.insert(*num, deque);
    }

    let mut last_num: usize = input[input.len() - 1];

    for turn in input.len()..num_turns {
        if (turn + 1) % 1000000 == 0 {
            println!("{}/{}", turn + 1, num_turns);
        }
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

fn advance(track: &mut BTreeMap<usize, usize>) {
    for v in track.values_mut() {
        *v += 1;
    }
}

fn play_v2(input: &[usize], num_turns: usize) -> usize {
    let mut track: BTreeMap<usize, usize> = BTreeMap::new();

    for num in input.iter() {
        advance(&mut track);
        track.insert(*num, 0);
    }
    let mut seen_at: HashMap<BTreeMap<usize, usize>, usize> = HashMap::new();

    let mut last: usize = input[input.len() - 1];
    let mut turn = input.len();

    let mut cycle_found = false;

    while turn < num_turns {
        let num = track.get(&last).cloned().unwrap_or(0);
        // println!("[{}/{}] {} -> {}", turn, num_turns, last, num);
        // println!("Pre-advance: {:#?}", track);
        advance(&mut track);
        track.insert(last, 1);
        // println!("Post-advance: {:#?}", track);
        last = num;

        if !cycle_found {
            match seen_at.get(&track) {
                None => {
                    seen_at.insert(track.clone(), turn);
                }
                Some(prev_turn) => {
                    // we have found a previously visited state -> shorten
                    let cycle_len = turn - prev_turn;

                    let remaining = num_turns - turn;

                    turn += (remaining / cycle_len) * cycle_len;
                    cycle_found = true;
                }
            }
        }
        turn += 1;
    }
    println!("{}", last);
    last
}

fn play(input: &[usize], num_turns: usize) -> usize {
    let mut track: HashMap<usize, usize> = HashMap::new();

    for (turn, num) in input.iter().enumerate() {
        track.insert(*num, turn);
    }

    let mut last_num: usize = input[input.len() - 1];

    for turn in input.len()..num_turns {
        if (turn + 1) % 10000000 == 0 {
            println!("{}/{}", turn + 1, num_turns);
        }
        let current_num = match track.get(&last_num) {
            None => 0,
            Some(last_turn) => turn - last_turn - 1,
        };
        // println!("{}", current_num);
        track.insert(last_num, turn - 1);
        // println!("After turn #{}: {:#?}", turn, track);
        last_num = current_num;
    }
    last_num
}
