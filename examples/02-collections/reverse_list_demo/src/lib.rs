#[derive(Debug)]

struct Node {
    val: i32,
    next: Option<Box<Node>>,
}

impl Node {
    fn new(val: i32) -> Box<Node> {
        Box::new(Node { val, next: None })
    }

    fn link(&mut self, node: Box<Node>) {
        self.next = Some(node);
    }
}

#[derive(Debug)]
struct BoxedLinkedList {
    head: Option<Box<Node>>,
}

impl BoxedLinkedList {
    fn new() -> BoxedLinkedList {
        BoxedLinkedList { head: None }
    }

    fn display(&self) {
        println!("{:?}", self);
    }

    fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    fn push_front(&mut self, val: i32) {
        let mut new_node = Node::new(val);
        if let Some(node) = self.head.take() {
            new_node.link(node);
        }
        self.head = Some(new_node);
    }

    fn pop_front(&mut self) -> Option<i32> {
        match self.head.take() {
            None => None,
            Some(node) => {
                self.head = node.next;
                Some(node.val)
            }
        }
    }

    fn push_back(&mut self, value: i32) {
        let new_node = Node::new(value);
        match self.head.as_mut() {
            None => self.head = Some(new_node),
            Some(mut curr) => {
                while curr.next.is_some() {
                    curr = curr.next.as_mut().unwrap();
                }
                curr.link(new_node);
            }
        }
    }

    fn pop_back(&mut self) -> Option<i32> {
        match self.head.as_mut() {
            None => None,
            Some(mut curr) => {
                // as_ref将一个不可变的值的引用转换为另一个不可变的引用
                while curr.next.is_some() && curr.next.as_ref().unwrap().next.is_some() {
                    curr = curr.next.as_mut().unwrap();
                }
                match curr.next {
                    Some(_) => Some(curr.next.take().unwrap().val),
                    None => Some(self.head.take().unwrap().val),
                }
            }
        }
    }

    fn insert(&mut self, value: i32) {
        let mut new_node = Node::new(value);
        match self.head.as_mut() {
            None => self.head = Some(new_node),
            Some(mut curr) => {
                if value <= curr.val {
                    self.push_front(value);
                } else {
                    while curr.next.is_some() && curr.next.as_ref().unwrap().val < value {
                        curr = curr.next.as_mut().unwrap();
                    }

                    if let Some(node) = curr.next.take() {
                        new_node.link(node);
                    }
                    curr.next = Some(new_node);
                }
            }
        }
    }

    fn delete(&mut self, value: i32) -> bool {
        match self.head.as_mut() {
            None => false,
            Some(mut curr) => {
                if curr.val == value {
                    self.head = self.head.take().unwrap().next;
                    true
                } else {
                    while curr.next.is_some() && curr.next.as_ref().unwrap().val != value {
                        curr = curr.next.as_mut().unwrap();
                    }

                    match curr.next.take() {
                        None => false,
                        Some(node) => {
                            curr.next = node.next;
                            true
                        }
                    }
                }
            }
        }
    }

    fn traverse(&self) {
        let mut curr = self.head.as_ref();
        while curr.is_some() {
            print!("{:?}\t", curr.unwrap().val);
            curr = curr.unwrap().next.as_ref();
        }
    }

    fn reverse(&mut self) {
        if self.is_empty() || self.head.as_ref().unwrap().next.is_none() {
            return;
        }

        let mut left = self.head.as_mut().unwrap().next.take();
        while left.is_some() {
            let mut t = left.take().unwrap();
            left = t.next;
            t.next = self.head.take();
            self.head = Some(t);
        }
    }
}


