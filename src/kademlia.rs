use std::collections::HashMap;
use std::net::Ipv4Addr;


#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Hash, Debug)]
pub struct NodeId {
    pub data: [u32; 5]
}

impl NodeId {
    fn new(a: u32, b: u32, c: u32, d: u32, e: u32) -> NodeId {
        NodeId {data: [a, b, c, d, e]}
    }

    fn midpoint(&self, other: NodeId) -> NodeId {
        let mut self_div_2 = self.clone();
        self_div_2.data[0] >>= 1;
        for i in 1..5 {
            if self_div_2.data[i] & 1 == 1 {
                self_div_2.data[i-1] |= 0x80000000;
            }
            self_div_2.data[i] >>= 1;
        }

        let mut other_div_2 = other.clone();
        other_div_2.data[0] >>= 1;
        for i in 1..5 {
            if other_div_2.data[i] & 1 == 1 {
                other_div_2.data[i-1] |= 0x80000000;
            }
            other_div_2.data[i] >>= 1;
        }

        let mut carry: u32 = self.data[0] & other.data[0] & 1;
        let mut final_node_id = NodeId {data: [0, 0, 0, 0, 0]};
        for i in 0..5 {
            let datum1 = self_div_2.data[i] as u64;
            let datum2 = other_div_2.data[i] as u64;
            let mut val = datum1 + datum2 + carry as u64;
            carry = (val > 0xffffffff) as u32;
            final_node_id.data[i] = val as u32;
        }

        return final_node_id;
    }

    fn plus_one(&self) -> NodeId {
        let mut new_node_id = self.clone();

        for i in 0..5 {
            if new_node_id.data[i] == 0xffffffff {
                new_node_id.data[i] = 0;
            } else {
                new_node_id.data[i] += 1;
                break;
            }
        }

        return new_node_id;
    }

    fn distance(&self, other: NodeId) -> u32 {
        (self.data[0] ^ other.data[0]).count_ones() +
        (self.data[1] ^ other.data[1]).count_ones() +
        (self.data[2] ^ other.data[2]).count_ones() +
        (self.data[3] ^ other.data[3]).count_ones() +
        (self.data[4] ^ other.data[4]).count_ones()
    }
}

#[cfg(test)]
mod tests {
    use kademlia::NodeId;

    #[test]
    fn test_plus_one_simple() {
        let node_id: NodeId = NodeId::new(1, 0, 0, 0, 0);
        let node_id_plus_one = node_id.plus_one();
        let expected = NodeId::new(2, 0, 0, 0, 0);
        assert_eq!(node_id_plus_one, expected);
    }

    #[test]
    fn test_plus_one_carry_over() {
        let node_id: NodeId = NodeId::new(0xffffffff, 0, 0, 0, 0);
        let node_id_plus_one = node_id.plus_one();
        let expected = NodeId::new(0, 1, 0, 0, 0);
        assert_eq!(node_id_plus_one, expected);
    }

    #[test]
    fn test_plus_one_carry_over_twice() {
        let node_id = NodeId::new(0xffffffff, 0xffffffff, 0, 0, 0);
        let node_id_plus_one = node_id.plus_one();
        let expected = NodeId::new(0, 0, 1, 0, 0);
        assert_eq!(node_id_plus_one, expected);
    }

    #[test]
    fn test_distance() {
        let id1 = NodeId::new(1, 0, 0, 0, 0);
        let id2 = NodeId::new(0, 0, 0xffffffff, 0, 1);
        assert_eq!(id1.distance(id2), 34);
    }

    #[test]
    fn test_midpoint_simple() {
        let id1 = NodeId::new(1, 0, 0, 0, 0);
        let id2 = NodeId::new(8, 0, 0, 0, 0);
        let id3 = NodeId::new(9, 0, 0, 0, 0);

        assert_eq!(id1.midpoint(id2), NodeId::new(4, 0, 0, 0, 0));
        assert_eq!(id1.midpoint(id3), NodeId::new(5, 0, 0, 0, 0));
    }

    #[test]
    fn test_midpoint_simple_2() {
        let id1 = NodeId::new(0, 0, 0, 0, 0);
        let id2 = NodeId::new(0, 1, 0, 0, 0);
        assert_eq!(id1.midpoint(id2), NodeId::new(2147483648, 0, 0, 0, 0));
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Node {
    pub ip_address: Ipv4Addr,
    pub port: u16,
    pub id: NodeId
}

impl Node {
    fn distance(&self, other: Node) -> u32 {
        self.id.distance(other.id)
    }
}

struct KBucket {
    k_size: u32,
    range: (NodeId, NodeId),
    nodes: HashMap<NodeId, Node>
}

impl KBucket {
    fn add(&mut self, node: Node) -> bool {
        true
    }

    fn split(&self) -> (KBucket, KBucket) {
        let midpoint = self.range.0.midpoint(self.range.1);
        let mut bucket1 = KBucket {
            k_size: self.k_size,
            range: (self.range.0, midpoint),
            nodes: HashMap::new()
        };

        let mut bucket2 = KBucket {
            k_size: self.k_size,
            range: (midpoint.plus_one(), self.range.1),
            nodes: HashMap::new()
        };

        for (node_id, node) in &self.nodes {
            if *node_id <= bucket1.range.1 {
                bucket1.nodes.insert(*node_id, *node);
            } else {
                bucket2.nodes.insert(*node_id, *node);
            }
        }

        return (bucket1, bucket2);
    }

    fn has_in_range(&self, node: Node) -> bool {
        node.id >= self.range.0 && node.id <= self.range.1
    }

    fn depth(&self) -> u32 {
        0
    }
}

pub struct RoutingTable {
    node: Node,
    buckets: Vec<KBucket>
}

impl RoutingTable {
    fn add(&mut self, node: Node) {
        let bucket_index = self.get_bucket_for(&node);

        if self.buckets[bucket_index].add(node.clone()) {
            return;
        }

        let should_split: bool = {
            let ref bucket = self.buckets[bucket_index];
            bucket.has_in_range(node) || bucket.depth() % 5 != 0
        };

        if should_split {
            self.split_bucket(bucket_index);
            self.add(node);
        } else {
            // TODO
        }
    }

    fn get_bucket_for(&self, node: &Node) -> usize {
        for (i, bucket) in self.buckets.iter().enumerate() {
            if bucket.range.1 > node.id {
                return i;
            }
        }
        return 0;
    }

    fn split_bucket(&mut self, index: usize) {
        let (bucket1, bucket2) = self.buckets[index].split();
        self.buckets[index] = bucket1;
        self.buckets.insert(index + 1, bucket2);
    }
}
