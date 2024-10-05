pub fn calculate_player_score(chosen_card: u8, card_list: &Vec<u8>, current_score: u8) -> u8 {
    let mut is_ace_card = false;
    if chosen_card == 1 || chosen_card == 14 || chosen_card == 27 || chosen_card == 40 {
        is_ace_card = true;
    }

    let card_score: u8 = get_card_score(chosen_card);

    let mut ace_number_in_list = card_list.iter().filter(|&n| *n == 1 || *n == 14 || *n == 27 || *n == 40).count();

    // if chosen_card is an ace, then reduce the ace_number_in_list by 1
    // this is because that chosen_card is already included in card_list
    if is_ace_card {
        ace_number_in_list -= 1;
    }

    // Condition 1
    // -------------------------------------------------------------------------
    if ace_number_in_list == 0 && !is_ace_card {
        return current_score + card_score;
    }

    // Condition 2
    // -------------------------------------------------------------------------
    if ace_number_in_list == 0 && is_ace_card {
        let s = current_score + 11;

        return if s > 21 {
            current_score + 1
        } else {
            s
        };
    }

    // Set temporary variable
    // -------------------------------------------------------------------------
    let mut temp_score: u8 = 0;

    // create copy of card list
    let mut temp_card_list = card_list.to_owned();

    // remove 1, 14, 27, and 40 value (ace card) in temporary card list
    temp_card_list.retain(|&x| x != 1 && x != 14 && x != 27 && x != 40);

    // calculate new score without any ace card
    for card in temp_card_list.into_iter() {
        temp_score = temp_score.saturating_add(get_card_score(card));
    }

    // Condition 3
    // -------------------------------------------------------------------------
    if ace_number_in_list > 0 && !is_ace_card {
        // calculate new score
        return calculate_new_score_with_ace_card(ace_number_in_list as u8, temp_score);
    }

    // Condition 4
    // -------------------------------------------------------------------------
    if ace_number_in_list > 0 && is_ace_card {
        let new_ace_number = ace_number_in_list as u8 + 1;

        // calculate new score
        return calculate_new_score_with_ace_card(new_ace_number, temp_score);
    }

    0
}

pub fn get_card_score(chosen_card: u8) -> u8 {
    match chosen_card {
        1 | 14 | 27 | 40 => {
            // Ace
            0
        }
        10 | 11 | 12 | 13 | 23 | 24 | 25 | 26 | 36 | 37 | 38 | 39 | 49 | 50 | 51 | 52 => {
            // 10, Jack, Queen, King
            10
        }
        2 | 15 | 28 | 41 => {
            2
        }
        3 | 16 | 29 | 42 => {
            3
        }
        4 | 17 | 30 | 43 => {
            4
        }
        5 | 18 | 31 | 44 => {
            5
        }
        6 | 19 | 32 | 45 => {
            6
        }
        7 | 20 | 33 | 46 => {
            7
        }
        8 | 21 | 34 | 47 => {
            8
        }
        9 | 22 | 35 | 48 => {
            9
        }
        _ => {
            0
        }
    }
}

pub fn calculate_new_score_with_ace_card(ace_card_number: u8, new_score: u8) -> u8 {
    // Ace card value can be 1 or 11
    // if by adding Ace card the score became more than 21, the Ace card value is 1
    // else Ace card value is 11
    //
    // if we have 2 Ace card, the value is those card is 1 and 11
    // we can't have both value as 11, because 11 + 11 > 21
    // the same applied if we have 3 Ace card, the value should be 1, 1, and 11
    // it also could be 1, 1, and 1, depending on the new score
    //
    match ace_card_number {
        1 => {
            if new_score + 11 > 21 {
                new_score + 1
            } else {
                new_score + 11
            }
        }
        2 => {
            if new_score + 12 > 21 {
                new_score + 2
            } else {
                new_score + 12
            }
        }
        3 => {
            if new_score + 13 > 21 {
                new_score + 3
            } else {
                new_score + 13
            }
        }
        4 => {
            if new_score + 14 > 21 {
                new_score + 4
            } else {
                new_score + 14
            }
        }
        _ => {
            new_score
        }
    }
}