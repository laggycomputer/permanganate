use std::ops::Index;

use itertools::Itertools;
use varisat::Lit;

fn invert(lit: Lit) -> Lit {
    match lit.is_negative() {
        true => lit.var().positive(),
        false => lit.var().negative(),
    }
}

pub(crate) fn exactly_one(vars: Vec<Lit>) -> Vec<Vec<Lit>> {
    let mut clauses = Vec::with_capacity(vars.len() * (vars.len() + 1) / 2 + 1);

    // no two are true; (!A + !B) * (!A + !C) * ...
    clauses.extend(vars.iter()
        .combinations(2)
        .map(|pair| vec![invert(**pair.index(0)), invert(**pair.index(1))])
    );
    // at least one var is true; A + B + C + ...
    clauses.push(vars);

    clauses
}