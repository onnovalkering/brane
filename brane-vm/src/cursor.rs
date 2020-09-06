use std::rc::Rc;
use std::cell::RefCell;
use redis::{Connection, Client};
use redis::Commands;
use specifications::instructions::Move;

pub trait Cursor {
    fn get_position(
        &self,
    ) -> usize;

    fn set_position(
        &self,
        value: usize,
    ) -> usize;

    fn get_depth(
        &self,
    ) -> usize;

    fn set_depth(
        &self,
        value: usize,
    ) -> usize;

    fn get_subposition(
        &self,
        depth: usize,
    ) -> usize;

    fn set_subposition(
        &self,
        depth: usize,
        value: usize,
    ) -> usize;

    fn go(
        &self,
        movement: Move,
    ) -> ();

    fn enter_sub(
        &self,
        max_subposition: usize,
    ) -> ();

    fn exit_sub(
        &self,
    ) -> ();
}

///
///
///
#[derive(Clone)]
pub struct RedisCursor {
    connection: Rc<RefCell<Connection>>,
    prefix: String,
}

impl RedisCursor {
    ///
    ///
    ///
    pub fn new(
        prefix: String,
        client: &Client,
    ) -> Self {
        let connection = client.get_connection().unwrap();
        RedisCursor { connection: Rc::new(RefCell::new(connection)), prefix }
    }

    ///
    ///
    ///
    pub fn get(
        &self,
        key: &str
    ) -> Option<usize> {
        self.connection.borrow_mut().get(format!("{}_{}", self.prefix, key)).ok()
    }

    ///
    ///
    ///
    pub fn set(
        &self,
        key: &str,
        value: usize
    ) -> () {
        let _: () = self.connection.borrow_mut().set(format!("{}_{}", self.prefix, key), value).unwrap();
    }
}

impl Cursor for RedisCursor {
    ///
    ///
    ///
    fn get_position(
        &self,
    ) -> usize {
        self.get("position").unwrap_or(0)
    }

    ///
    ///
    ///
    fn set_position(
        &self,
        value: usize,
    ) -> usize {
        self.set("position", value);
        self.get_position()
    }

    ///
    ///
    ///    
    fn get_depth(
        &self,
    ) -> usize {
        self.get("depth").unwrap_or(0)
    }

    ///
    ///
    ///    
    fn set_depth(
        &self,
        value: usize,
    ) -> usize {
        self.set("depth", value);
        self.get_depth()
    }

    ///
    ///
    ///    
    fn get_subposition(
        &self,
        depth: usize,
    ) -> usize {
        self.get(&format!("subposition_{}", depth)).unwrap_or(0)
    }

    ///
    ///
    ///    
    fn set_subposition(
        &self,
        depth: usize,
        value: usize,
    ) -> usize {
        self.set(&format!("subposition_{}", depth), value);
        self.get_subposition(depth)
    }

    ///
    ///
    ///
    fn go(
        &self,
        movement: Move,
    ) -> () {
        let depth = self.get_depth();
        let position = self.get_position();
        let subposition = self.get_subposition(depth);

        match movement {
            Move::Backward => {
                if depth == 0 {
                    self.set_position(position - 1);
                } else {
                    self.set_subposition(depth, subposition - 1);
                }
            },
            Move::Forward => {
                if depth == 0 {
                    self.set_position(position + 1);
                } else {
                    let max_subposition = self.get(&format!("subposition_{}_max", depth)).unwrap();
                    let new_subposition = subposition + 1;

                    if new_subposition <= max_subposition {
                        self.set_subposition(depth, new_subposition);
                    } else {
                        self.exit_sub();
                    }
                }              
            },
            Move::Skip => {
                if depth == 0 {
                    self.set_position(position + 2);
                } else {
                    let max_subposition = self.get(&format!("subposition_{}_max", depth)).unwrap();
                    let new_subposition = subposition + 2;

                    if new_subposition <= max_subposition {
                        self.set_subposition(depth, new_subposition);
                    } else {
                        self.exit_sub();
                    }
                }
            },
        }        
    }
    
    ///
    ///
    ///
    fn enter_sub(
        &self,
        max_subposition: usize,
    ) -> () {
        let depth = self.get_depth();
        let new_depth = depth + 1;

        debug!("Enter subroutine: {} -> {}", depth, new_depth);

        self.set_depth(new_depth);
        self.set(&format!("subposition_{}", new_depth), 0);
        self.set(&format!("subposition_{}_max", new_depth), max_subposition);
    }

    ///
    ///
    ///
    fn exit_sub(
        &self,
    ) -> () {
        let depth = self.get_depth();
        let new_depth = depth - 1;

        debug!("Exit subroutine: {} -> {}", depth, new_depth);

        self.set_depth(new_depth);
        self.go(Move::Forward)
    }
}