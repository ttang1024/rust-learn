struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
}

impl<T: std::fmt::Display> LinkedList<T> {
    fn push(&mut self, value: T) {
        let new_node = Box::new(Node {
            value: value,
            next: self.head.take(),
        });
        self.head = Some(new_node);
    }

    fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.value
        })
    }

    fn print(&self) {
        let mut current = &self.head;
        while let Some(node) = current {
            println!("{}", node.value);
            current = &node.next;
        }
    }
}

fn main() {
    let mut list = LinkedList::<i32> { head: None };
    list.push(1);
    list.push(2);
    list.push(3);
    // list.print();
    let v = list.pop();
    println!("{:?}", v); // Some(3) 
    list.print(); // 2  1
}
