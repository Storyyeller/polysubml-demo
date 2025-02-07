use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use crate::core::VarSpec;
use crate::parse_types::SourceLoc;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct BoundPairsSetSub(HashMap<SourceLoc, SourceLoc>);
impl BoundPairsSetSub {
    fn filter_left(&mut self, mut f: impl FnMut(SourceLoc) -> bool) {
        self.0.retain(|k, v| f(*k));
    }

    fn filter_right(&mut self, mut f: impl FnMut(SourceLoc) -> bool) {
        self.0.retain(|k, v| f(*v));
    }

    fn push(&mut self, pair: (SourceLoc, SourceLoc)) {
        self.0.insert(pair.0, pair.1);
    }

    fn update_intersect(&mut self, other: &Self) {
        self.0.retain(|k, v| other.get(*k, *v));
    }

    fn update_intersect_flipped(&mut self, other: &Self) {
        self.0.retain(|k, v| other.get(*v, *k));
    }

    fn get(&self, loc1: SourceLoc, loc2: SourceLoc) -> bool {
        self.0.get(&loc1).copied() == Some(loc2)
    }
}

#[derive(Debug, Clone, Default)]
pub struct BoundPairsSet(Option<Rc<BoundPairsSetSub>>, bool);
impl BoundPairsSet {
    fn as_ptr(&self) -> (Option<*const BoundPairsSetSub>, bool) {
        (self.0.as_ref().map(Rc::as_ptr), self.1)
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    fn mutate(&mut self, f: impl FnOnce(&mut BoundPairsSetSub)) -> bool {
        let mut sub = self.0.as_ref().map(|r| BoundPairsSetSub::clone(r)).unwrap_or_default();
        f(&mut sub);
        let sub = if sub.0.is_empty() { None } else { Some(Rc::new(sub)) };

        // If there were no actual changes, reuse the existing value rather than the copied one
        // Also return if there was a change or not
        if sub != self.0 {
            self.0 = sub;
            true
        } else {
            false
        }
    }

    pub fn filter_left(&mut self, f: impl FnMut(SourceLoc) -> bool) {
        if self.1 {
            self.mutate(|sub| sub.filter_right(f));
        } else {
            self.mutate(|sub| sub.filter_left(f));
        }
    }

    pub fn filter_right(&mut self, f: impl FnMut(SourceLoc) -> bool) {
        if self.1 {
            self.mutate(|sub| sub.filter_left(f));
        } else {
            self.mutate(|sub| sub.filter_right(f));
        }
    }

    pub fn push(&mut self, mut pair: (SourceLoc, SourceLoc)) {
        if self.1 {
            pair = (pair.1, pair.0);
        }
        self.mutate(|sub| sub.push(pair));
    }

    pub fn flip(&self) -> Self {
        Self(self.0.clone(), !self.1)
    }

    pub fn update_intersect(&mut self, other: &Self) -> bool {
        // Skip intersection when self === other
        if self.as_ptr() == other.as_ptr() {
            return false;
        }

        let flip = self.1 != other.1;
        if let Some(other) = other.0.as_ref() {
            if flip {
                self.mutate(|sub| sub.update_intersect_flipped(other))
            } else {
                self.mutate(|sub| sub.update_intersect(other))
            }
        } else {
            self.clear();
            true
        }
    }

    pub fn get(&self, loc1: SourceLoc, loc2: SourceLoc) -> bool {
        if let Some(sub) = self.0.as_ref() {
            if self.1 { sub.get(loc2, loc1) } else { sub.get(loc1, loc2) }
        } else {
            // Empty set
            false
        }
    }

    // Return true if there's any (loc, loc2) in self such that (loc, name) is in lhs and (loc2, name) is in rhs
    pub fn disjoint_union_vars_have_match<'a>(&self, mut lhs: &'a HashSet<VarSpec>, mut rhs: &'a HashSet<VarSpec>) -> bool {
        if self.1 {
            (lhs, rhs) = (rhs, lhs);
        }

        if let Some(sub) = self.0.as_ref() {
            for spec in lhs.iter().copied() {
                if let Some(loc2) = sub.0.get(&spec.loc).copied() {
                    let expect = VarSpec {
                        loc: loc2,
                        name: spec.name,
                    };
                    if rhs.contains(&expect) {
                        return true;
                    }
                } else {
                    continue;
                }
            }
        } else {
            // Empty set
        }

        false
    }
}
