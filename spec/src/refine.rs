#![allow(unused_imports)]
use builtin::*;
use builtin_macros::*;
use state_machines_macros::{case_on_init, case_on_next, state_machine};
use vstd::{map::*, pervasive::*, seq::*, set::*, *};

verus! {

pub mod r#spec {
    pub struct State {}
    pub spec fn inv(s: State) -> bool;
    
    pub spec fn init(s: State) -> bool;
    pub spec fn next(s: State, s_next: State) -> bool;
}

pub mod refinement {
    use super::r#spec;

    pub struct State {}
    pub spec fn abstraction(s: State) -> r#spec::State;
    pub spec fn inv(s: State) -> bool;

    pub spec fn init(s: State) -> bool;
    pub spec fn next(s: State, s_next: State) -> bool;

    pub proof fn refinement_init(s: State)
        requires init(s),
        ensures inv(s),
                r#spec::init(abstraction(s))
    ;
    pub proof fn refinement_next(s: State, s_next: State)
        requires inv(s),
                 next(s, s_next),
        ensures inv(s_next),
                r#spec::next(abstraction(s), abstraction(s_next))
             || abstraction(s) == abstraction(s_next)
    ;
}

fn main() { }

}