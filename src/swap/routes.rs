use crate::swap::PoolEdge;

pub struct SwapRoute {

}

pub struct  RouteCache  {
    pub edges: Vec<PoolEdge>,
    pub graph: u64,  // to implement
}

pub fn new() -> RouteCache {
    RouteCache {  edges: vec![], graph:0 } 
}


impl  RouteCache {
}