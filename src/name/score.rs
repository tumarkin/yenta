use counter::Counter;

pub fn score_combination_queue(
    from_token_counter: &Counter<String>,
    from_norm: f64,
    to_token_counter: &Counter<String>,
    to_norm: f64,
    combination_queue: Vec<(f64, &String, &String)>,
) -> f64 {
    // A list of matches sort from worst to best. The algorithm will
    // pop off the last value, getting the best possible unused token match.
    let mut sorted_combination_queue = combination_queue;
    sorted_combination_queue
        .sort_unstable_by(|(val_a, _, _), (val_b, _, _)| val_a.partial_cmp(val_b).unwrap());

    let mut score_in_common = 0.0;
    let mut from_tokens_used: Counter<String> = Counter::new();
    let mut to_tokens_used: Counter<String> = Counter::new();

    while let Some((this_score, from_token, to_token)) = sorted_combination_queue.pop() {
        score_in_common += this_score;

        from_tokens_used[from_token] += 1;
        to_tokens_used[to_token] += 1;

        if from_tokens_used[from_token] == from_token_counter[from_token] {
            sorted_combination_queue.retain(|(_, ft, _)| ft != &from_token);
        }

        if to_tokens_used[to_token] == to_token_counter[to_token] {
            sorted_combination_queue.retain(|(_, _, tt)| tt != &to_token);
        }
    }

    score_in_common / (from_norm * to_norm)
}
