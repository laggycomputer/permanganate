use std::ops::Index;

use itertools::Itertools;
use varisat::{Lit, Var};

pub(crate) fn exactly_one(vars: Vec<Var>) -> Vec<Vec<Lit>> {
    let mut clauses = Vec::with_capacity(vars.len() * (vars.len() + 1) / 2 + 1);

    // no two are true; (!A + !B) * (!A + !C) * ...
    clauses.extend(vars.iter()
        .combinations(2)
        .map(|pair| vec![pair.index(0).negative(), pair.index(1).negative()])
    );
    // at least one var is true; A + B + C + ...
    clauses.push(vars.iter().map(|v| v.positive()).collect_vec());

    clauses
}