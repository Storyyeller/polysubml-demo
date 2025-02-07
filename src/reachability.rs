use std::collections::HashMap;

#[derive(Debug, Clone)]
struct OrderedMap<K, V> {
    keys: Vec<K>,
    m: HashMap<K, V>,
}
impl<K: Eq + std::hash::Hash + Clone, V> OrderedMap<K, V> {
    fn new() -> Self {
        Self {
            keys: Vec::new(),
            m: HashMap::new(),
        }
    }

    fn insert(&mut self, k: K, v: V) -> Option<V> {
        let old = self.m.insert(k.clone(), v);
        if old.is_none() {
            self.keys.push(k);
        }
        old
    }

    fn iter_keys(&self) -> std::slice::Iter<K> {
        self.keys.iter()
    }

    fn retain(&mut self, f: impl Fn(&K) -> bool) {
        self.m.retain(|k, _| f(k));
        // Also remove any extra keys left from unrelated deletions
        // since journal removal may have left extra keys in the list.
        self.keys.retain(|k| self.m.contains_key(k));
    }
}

pub trait ExtNodeDataTrait {
    fn truncate(&mut self, i: TypeNodeInd);
}

pub trait EdgeDataTrait<ExtNodeData>: Clone {
    fn update(&mut self, other: &Self) -> bool;
    fn expand(self, hole: &ExtNodeData, ind: TypeNodeInd) -> Self;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeNodeInd(pub usize);

struct ReachabilityNode<ExtNodeData, ExtEdgeData> {
    data: ExtNodeData,
    flows_from: OrderedMap<TypeNodeInd, ExtEdgeData>,
    flows_to: OrderedMap<TypeNodeInd, ExtEdgeData>,
}
impl<N: ExtNodeDataTrait, E> ReachabilityNode<N, E> {
    fn fix_and_truncate(&mut self, i: TypeNodeInd) {
        self.data.truncate(i);
        self.flows_from.retain(|&k| k < i);
        self.flows_to.retain(|&k| k < i);
    }
}

pub struct Reachability<ExtNodeData, ExtEdgeData> {
    nodes: Vec<ReachabilityNode<ExtNodeData, ExtEdgeData>>,

    // Nodes past this point may be reverted in case of a type error
    // Value of 0 indicates no mark is set (or if a mark is set, there's nothing to do anyway)
    rewind_mark: TypeNodeInd,
    journal: Vec<(TypeNodeInd, TypeNodeInd, Option<ExtEdgeData>)>,
}
impl<ExtNodeData: ExtNodeDataTrait, ExtEdgeData: EdgeDataTrait<ExtNodeData>> Reachability<ExtNodeData, ExtEdgeData> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            rewind_mark: TypeNodeInd(0),
            journal: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn get(&self, i: TypeNodeInd) -> Option<&ExtNodeData> {
        self.nodes.get(i.0).map(|rn| &rn.data)
    }
    pub fn get_mut(&mut self, i: TypeNodeInd) -> Option<&mut ExtNodeData> {
        self.nodes.get_mut(i.0).map(|rn| &mut rn.data)
    }
    pub fn get_edge(&self, lhs: TypeNodeInd, rhs: TypeNodeInd) -> Option<&ExtEdgeData> {
        self.nodes.get(lhs.0).and_then(|rn| rn.flows_to.m.get(&rhs))
    }

    pub fn add_node(&mut self, data: ExtNodeData) -> TypeNodeInd {
        let i = self.len();

        let n = ReachabilityNode {
            data,
            flows_from: OrderedMap::new(),
            flows_to: OrderedMap::new(),
        };
        self.nodes.push(n);
        TypeNodeInd(i)
    }

    fn update_edge_value(&mut self, lhs: TypeNodeInd, rhs: TypeNodeInd, val: ExtEdgeData) {
        let old = self.nodes[lhs.0].flows_to.insert(rhs, val.clone());
        self.nodes[rhs.0].flows_from.insert(lhs, val);

        // If the nodes are >= rewind_mark, they'll be removed during rewind anyway
        // so we only have to journal edge values when both are below the mark.
        if lhs < self.rewind_mark && rhs < self.rewind_mark {
            self.journal.push((lhs, rhs, old));
        }
    }

    pub fn add_edge(
        &mut self,
        lhs: TypeNodeInd,
        rhs: TypeNodeInd,
        edge_val: ExtEdgeData,
        out: &mut Vec<(TypeNodeInd, TypeNodeInd, ExtEdgeData)>,
    ) {
        // println!("add_edge {}->{}", lhs.0, rhs.0);
        let mut work = vec![(lhs, rhs, edge_val)];

        while let Some((lhs, rhs, mut edge_val)) = work.pop() {
            // println!("    add_edge_sub {}->{}", lhs.0, rhs.0);
            let old_edge = self.nodes[lhs.0].flows_to.m.get_mut(&rhs);
            match old_edge {
                Some(old) => {
                    let mut old = old.clone();
                    if old.update(&edge_val) {
                        // println!("reevaluating {} {}", lhs.0, rhs.0);
                        edge_val = old; // updated value will be inserted into map below
                    } else {
                        // New edge value did not cause an update compared to existing edge value.
                        continue;
                    }
                }
                None => {}
            };
            self.update_edge_value(lhs, rhs, edge_val.clone());

            let temp = edge_val.clone().expand(&self.nodes[lhs.0].data, lhs);
            for &lhs2 in self.nodes[lhs.0].flows_from.iter_keys() {
                work.push((lhs2, rhs, temp.clone()));
            }

            let temp = edge_val.clone().expand(&self.nodes[rhs.0].data, rhs);
            for &rhs2 in self.nodes[rhs.0].flows_to.iter_keys() {
                work.push((lhs, rhs2, temp.clone()));
            }

            // Inform the caller that a new edge was added
            out.push((lhs, rhs, edge_val));
        }
    }

    pub fn save(&mut self) {
        assert!(self.rewind_mark.0 == 0);
        self.rewind_mark = TypeNodeInd(self.nodes.len());
    }

    pub fn revert(&mut self) {
        let i = self.rewind_mark;
        self.rewind_mark = TypeNodeInd(0);
        self.nodes.truncate(i.0);

        while let Some((lhs, rhs, val)) = self.journal.pop() {
            if let Some(val) = val {
                *self.nodes[lhs.0].flows_to.m.get_mut(&rhs).unwrap() = val.clone();
                *self.nodes[rhs.0].flows_from.m.get_mut(&lhs).unwrap() = val;
            } else {
                self.nodes[lhs.0].flows_to.m.remove(&rhs);
                self.nodes[rhs.0].flows_from.m.remove(&lhs);
            }
        }

        // If we removed edges above, the edge maps will have extra keys
        // fix_and_truncate will fix that in addition to truncating edges >= i
        for n in self.nodes.iter_mut() {
            n.fix_and_truncate(i);
        }
    }

    pub fn make_permanent(&mut self) {
        self.rewind_mark = TypeNodeInd(0);
        self.journal.clear();
    }
}
