pub fn to_multimap<K, V, I>(input: I) -> Vec<(K, Vec<V>)>
where
    K: PartialEq,
    I: IntoIterator<Item = (K, V)>,
{
    let mut output: Vec<(K, Vec<V>)> = vec![];
    for (k, v) in input {
        let i = output.iter().position(|(x, _)| x == &k);

        if let Some(p) = i {
            output[p].1.push(v);
        } else {
            output.push((k, vec![v]));
        }
    }

    output
}
