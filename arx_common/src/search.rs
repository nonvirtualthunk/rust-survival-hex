use std::ops;
use std::hash::Hash;
use std::collections::HashMap;
use std::collections::BinaryHeap;
use std::cmp::Ordering;



struct NodeAndCost<T, C: PartialOrd + PartialEq>(T, C);

impl <T,C : PartialOrd> PartialEq<Self> for NodeAndCost<T,C> {
    fn eq(&self, other: &NodeAndCost<T, C>) -> bool {
        self.1.eq(&other.1)
    }
}
impl <T,C : PartialOrd> Eq for NodeAndCost<T,C> {}

impl <T,C : PartialOrd> PartialOrd<Self> for NodeAndCost<T,C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.1.partial_cmp(&other.1)
    }
}
impl <T,C : PartialOrd> Ord for NodeAndCost<T,C> {
    fn cmp(&self, other: &Self) -> Ordering {
        // invert to turn things into a min-heap instead of a max heap
        match self.1.partial_cmp(&other.1).expect("total ordering was a problem after all") {
            Ordering::Less => Ordering::Greater,
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Less
        }
    }
}

pub fn flood_search<
    T: PartialEq + Eq + Hash + Clone,
    C: PartialOrd + PartialEq + ops::Add<Output=C> + ops::Sub<Output=C> + Copy,
    CF: Fn(&T, &T) -> C,
    NF: Fn(&T) -> Vec<T>
>(start: T, limit: C, cost_func: CF, neighbor_func: NF) -> HashMap<T, C> {
//    let mut parent_by_node = HashMap::new();
    let mut cost_by_node : HashMap<T,C> = HashMap::new();
    let mut heap = BinaryHeap::new();


    let zero_hack = (cost_func)(&start, &start) - (cost_func)(&start, &start);
    heap.push(NodeAndCost(start.clone(), zero_hack));

    while !heap.is_empty() {
        match heap.pop() {
            Some(NodeAndCost(node, cost)) => {
                // if we haven't reached the limit yet
                if cost <= limit {
                    // if the existing cost is greater than the way we got here this time, or if we've never visited
                    let should_act = cost_by_node.get(&node).map(|c| c > &cost).unwrap_or(true);
                    if should_act {
                        // grab all the neighbors and enqueue them with the cost to get there from here
                        for neighbor in (neighbor_func)(&node) {
                            let next_cost = (cost_func)(&node, &neighbor);
                            heap.push(NodeAndCost(neighbor, cost + next_cost));
                        }
                        // insert the cost to get here
                        cost_by_node.insert(node, cost);
                    }
                }
            },
            None => error!("nothing on heap, but we checked that there was")
        }
    }

    cost_by_node
}